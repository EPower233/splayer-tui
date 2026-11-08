//! 摸底实验 T-2：把 SPlayer-Next 的 xeapi 移植成 Rust，看它能不能稳定拿到播放地址。
//!
//! 对应 SPlayer-Next：electron/main/apis/netease/core/crypto.ts、xeapi.ts、request.ts @ a8716df
//!
//! 流程：先拉服务端的 X25519 公钥；请求体用随机密钥加密，随机密钥再用公钥封装；
//! 第一次响应的响应头会下发会话密钥，之后的请求复用它。

use std::collections::HashMap;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use aes::{Aes128, Aes256};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes128Gcm, Nonce};
use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use ecb::cipher::block_padding::Pkcs7;
use ecb::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit};
use hmac::{Hmac, Mac};
use ncm_api_rs::util::cookie::{cookie_obj_to_string, cookie_to_json};
use ncm_api_rs::util::device::{generate_wnmcid, random_hex};
use rand::rngs::OsRng;
use rand::{Rng, RngCore};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, COOKIE, USER_AGENT};
use serde_json::{json, Value};
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey};

const API_DOMAIN: &str = "https://interface.music.163.com";
const XEAPI_DOMAIN: &str = "https://interface3.music.163.com";
const STATIC_KEY_HEX: &str = "ab1d5a430f6bb04a3f01e81ddd72bd916d5ce591248ac128714806d7f8fb1b84";
const SIGN_KEY: &[u8] =
    b"mUHCwVNWJbunMqAHf5MImuirT6plvs6VSFW62MGHstFQxhBGdEoIhLItH3djc4+FB/OKty3+lL2rGeoFBpVe5g==";
const EAPI_KEY: &[u8] = b"e82ckenh8dichen8";
const ANDROID_UA: &str =
    "NeteaseMusic/9.5.61.260802021928(9005061);Dalvik/2.1.0 (Linux; U; Android 12; HBN-AL00 Build/cd737a2.0)";
const OS: &str = "android";
const OSVER: &str = "16";
const APPVER: &str = "9.1.65";
const FORM: &str = "application/x-www-form-urlencoded";

type HmacSha256 = Hmac<Sha256>;

pub struct Xeapi {
    http: reqwest::Client,
    cookie: HashMap<String, String>,
    device_id: String,
    static_key: Vec<u8>,
    key_version: String,
    public_key: [u8; 32],
    sk: String,
    /// (会话 ID, 会话密钥)，由第一次响应的响应头下发。
    session: Option<(String, String)>,
}

fn aes_ecb_encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let invalid = |_| anyhow!("密钥长度不对");
    Ok(match key.len() {
        16 => ecb::Encryptor::<Aes128>::new_from_slice(key)
            .map_err(invalid)?
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext),
        32 => ecb::Encryptor::<Aes256>::new_from_slice(key)
            .map_err(invalid)?
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext),
        other => bail!("不支持 {other} 字节的密钥"),
    })
}

fn aes_ecb_decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let invalid = |_| anyhow!("密钥长度不对");
    let failed = |_| anyhow!("解密失败");
    match key.len() {
        16 => ecb::Decryptor::<Aes128>::new_from_slice(key)
            .map_err(invalid)?
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(failed),
        32 => ecb::Decryptor::<Aes256>::new_from_slice(key)
            .map_err(invalid)?
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(failed),
        other => bail!("不支持 {other} 字节的密钥"),
    }
}

fn hmac_sha256(key: &[u8], message: &[&[u8]]) -> Result<Vec<u8>> {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(key).map_err(|_| anyhow!("HMAC 密钥长度不对"))?;
    for part in message {
        mac.update(part);
    }
    Ok(mac.finalize().into_bytes().to_vec())
}

fn sign(timestamp: &str, nonce: &str) -> Result<String> {
    let digest = hmac_sha256(SIGN_KEY, &[timestamp.as_bytes(), nonce.as_bytes()])?;
    Ok(STANDARD.encode(digest))
}

fn now_millis() -> Result<String> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis().to_string())
}

/// 随机 XOR 后按字节旋转，随机串放在最前面。
fn mid_transform(ciphertext: &[u8]) -> Vec<u8> {
    let mut random = [0u8; 16];
    OsRng.fill_bytes(&mut random);
    let xored: Vec<u8> = ciphertext
        .iter()
        .enumerate()
        .map(|(i, byte)| byte ^ random[i & 0x0f])
        .collect();
    let rotate = if xored.is_empty() {
        0
    } else {
        usize::from(random[0] & 0x0f) % xored.len()
    };
    [&random[..], &xored[rotate..], &xored[..rotate]].concat()
}

/// 响应：AES-ECB 解密，必要时 gunzip，再解析 JSON。
fn decrypt_response(body: &[u8]) -> Result<Value> {
    let decrypted = aes_ecb_decrypt(EAPI_KEY, body)?;
    let plaintext = if decrypted.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(decrypted.as_slice()).read_to_end(&mut out)?;
        out
    } else {
        decrypted
    };
    Ok(serde_json::from_slice(&plaintext)?)
}

impl Xeapi {
    /// 拉取服务端公钥，准备好发请求。
    pub async fn connect(cookie: &str, device_id: &str) -> Result<Self> {
        let http = reqwest::Client::new();
        let static_key = hex::decode(STATIC_KEY_HEX)?;
        let key = fetch_public_key(&http, &static_key, device_id).await?;
        let public_key: [u8; 32] = STANDARD
            .decode(key["publicKey"].as_str().context("公钥包里没有 publicKey")?)?
            .try_into()
            .map_err(|_| anyhow!("公钥不是 32 字节"))?;
        Ok(Self {
            http,
            cookie: cookie_to_json(cookie),
            device_id: device_id.to_string(),
            static_key,
            key_version: key["version"].as_str().unwrap_or("").to_string(),
            public_key,
            sk: key["sk"].as_str().context("公钥包里没有 sk")?.to_string(),
            session: None,
        })
    }

    /// 用 X25519 + AES-GCM 把动态密钥封装起来（请求里的 S 字段）。
    fn encrypt_s(&self, dynamic_key: &[u8]) -> Result<Vec<u8>> {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let ephemeral = PublicKey::from(&secret);
        let shared = secret.diffie_hellman(&PublicKey::from(self.public_key));

        let prk = hmac_sha256(&[0u8; 32], &[shared.as_bytes()])?;
        let okm = hmac_sha256(&prk, &[ephemeral.as_bytes(), &[1]])?;
        let cipher =
            Aes128Gcm::new_from_slice(&okm[..16]).map_err(|_| anyhow!("AES-GCM 密钥长度不对"))?;

        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);
        let plaintext = format!("{}|{OS}|{}", STANDARD.encode(dynamic_key), self.sk);
        let encrypted = cipher
            .encrypt(Nonce::from_slice(&iv), plaintext.as_bytes())
            .map_err(|_| anyhow!("AES-GCM 加密失败"))?;
        Ok([ephemeral.as_bytes().as_slice(), &iv, &encrypted].concat())
    }

    fn cookie_header(&self, buildver: &str, millis: &str) -> String {
        let mut cookie = self.cookie.clone();
        let nuid = random_hex(16);
        let defaults = [
            ("__remember_me", "true".to_string()),
            ("ntes_kaola_ad", "1".to_string()),
            ("_ntes_nnid", format!("{nuid},{millis}")),
            ("_ntes_nuid", nuid),
            ("WNMCID", generate_wnmcid()),
            ("WEVNSM", "1.0.0".to_string()),
            ("channel", "netease".to_string()),
            ("NMTID", format!("00O{}", &random_hex(19)[..38])),
        ];
        for (name, value) in defaults {
            cookie.entry(name.to_string()).or_insert(value);
        }
        let fixed = [
            ("deviceId", self.device_id.as_str()),
            ("sDeviceId", self.device_id.as_str()),
            ("os", OS),
            ("osver", OSVER),
            ("appver", APPVER),
            ("buildver", buildver),
        ];
        for (name, value) in fixed {
            cookie.insert(name.to_string(), value.to_string());
        }
        cookie_obj_to_string(&cookie)
    }

    fn headers(&self, buildver: &str, millis: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let plain = [
            ("x-client-enc-state", "ENCRYPTED"),
            ("x-aeapi", "true"),
            ("x-deviceid", self.device_id.as_str()),
            ("x-sdeviceid", self.device_id.as_str()),
            ("x-os", OS),
            ("x-osver", OSVER),
            ("x-appver", APPVER),
            ("x-buildver", buildver),
        ];
        for (name, value) in plain {
            headers.insert(HeaderName::from_static(name), HeaderValue::from_str(value)?);
        }
        if let Some(music_u) = self.cookie.get("MUSIC_U") {
            headers.insert(HeaderName::from_static("x-music-u"), HeaderValue::from_str(music_u)?);
        }
        headers.insert(USER_AGENT, HeaderValue::from_static(ANDROID_UA));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(FORM));
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookie_header(buildver, millis))?);
        Ok(headers)
    }

    /// `uri` 形如 `/api/song/enhance/player/url/v1`。
    pub async fn request(&mut self, uri: &str, data: &[(&str, &str)]) -> Result<Value> {
        let millis = now_millis()?;
        let buildver = millis[..10].to_string();

        let dynamic_key = match &self.session {
            Some((_, key)) => key.as_bytes().to_vec(),
            None => {
                let mut key = vec![0u8; 16];
                OsRng.fill_bytes(&mut key);
                key
            }
        };
        let session_id = self.session.as_ref().map_or("", |(id, _)| id.as_str());

        let content = serde_urlencoded::to_string(data)?;
        let plaintext = json!({ "content": content, "queryString": "e_r=true" }).to_string();
        let inner = aes_ecb_encrypt(&self.static_key, plaintext.as_bytes())?;
        let c = aes_ecb_encrypt(&dynamic_key, &mid_transform(&inner))?;
        let s = self.encrypt_s(&dynamic_key)?;
        let r = format!("{}|{session_id}", self.key_version);
        let r = aes_ecb_encrypt(&self.static_key, r.as_bytes())?;
        let body = serde_urlencoded::to_string([
            ("C", URL_SAFE_NO_PAD.encode(c)),
            ("S", STANDARD.encode(s)),
            ("R", STANDARD.encode(r)),
        ])?;

        let path = uri.strip_prefix("/api/").context("uri 要以 /api/ 开头")?;
        let response = self
            .http
            .post(format!("{XEAPI_DOMAIN}/xeapi/{path}"))
            .headers(self.headers(&buildver, &millis)?)
            .body(body)
            .send()
            .await
            .map_err(|e| anyhow!("xeapi 请求失败：{}", e.without_url()))?;

        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        };
        if let (Some(id), Some(key)) = (header("x-encr-ssid"), header("x-encr-sskey")) {
            self.session = Some((id, key));
        }

        let status = response.status().as_u16();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| anyhow!("读 xeapi 响应失败：{}", e.without_url()))?;
        decrypt_response(&bytes).map_err(|e| anyhow!("HTTP {status}，{e}"))
    }
}

async fn fetch_public_key(
    http: &reqwest::Client,
    static_key: &[u8],
    device_id: &str,
) -> Result<Value> {
    let nonce: String = {
        let mut rng = rand::thread_rng();
        (0..16).map(|_| char::from(b'0' + rng.gen_range(0..10u8))).collect()
    };
    let timestamp = now_millis()?;
    let signature = sign(&timestamp, &nonce)?;
    let form = serde_urlencoded::to_string([
        ("appVersion", "9.5.61"),
        ("currentKeyVersion", ""),
        ("deviceId", device_id),
        ("nonce", nonce.as_str()),
        ("os", OS),
        ("requestType", "active"),
        ("signature", signature.as_str()),
        ("t1", ""),
        ("t2", ""),
        ("timestamp", timestamp.as_str()),
        ("uid", ""),
    ])?;

    let json: Value = http
        .post(format!("{API_DOMAIN}/api/bsr/sk/get"))
        .header(USER_AGENT, ANDROID_UA)
        .header(CONTENT_TYPE, FORM)
        .header(COOKIE, format!("deviceId={device_id}"))
        .body(form)
        .send()
        .await
        .map_err(|e| anyhow!("拉取公钥失败：{}", e.without_url()))?
        .json()
        .await
        .map_err(|e| anyhow!("公钥响应不是 JSON：{}", e.without_url()))?;

    let payload = &json["data"];
    let encrypted = payload["encryptedData"]
        .as_str()
        .ok_or_else(|| anyhow!("拉取公钥失败，code={}", json["code"]))?;
    // 服务端的 timestamp 可能是数字，也可能是字符串，SPlayer-Next 那边靠 String() 统一。
    let server_time = match &payload["timestamp"] {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    let expected = sign(&server_time, &nonce)?;
    if payload["signature"].as_str() != Some(expected.as_str()) {
        bail!("公钥响应的签名对不上");
    }
    let decrypted = aes_ecb_decrypt(static_key, &STANDARD.decode(encrypted)?)?;
    Ok(serde_json::from_slice(&decrypted)?)
}
