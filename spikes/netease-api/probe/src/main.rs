//! 摸底实验 T-2：ncm-api-rs 能不能走通 扫码登录 → 喜欢列表 → 播放地址 → 逐字歌词。
//!
//! 用法：
//!   probe login            终端里显示二维码，扫码后把 Cookie 存到 .home/cookie
//!   probe run [歌曲数]     用保存的 Cookie 依次试各接口，默认试 3 首
//!   probe url <序号> [音质] 打印喜欢列表里第几首的播放地址，给 T-1 的 play 用
//!
//! 凭据规矩：不打印 Cookie、原始响应、用户 ID、昵称、歌曲 ID、地址的查询串和歌词原文，
//! 只打印状态码、条数和耗时。

mod qr;
mod xeapi;

use std::future::Future;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use ncm_api_rs::{create_client, ApiClient, ApiResponse, NcmError, Query};
use serde_json::Value;

const LEVELS: [&str; 4] = ["standard", "exhigh", "lossless", "hires"];
/// 请求之间停一下，免得触发风控。
const PAUSE: Duration = Duration::from_millis(300);
const POLL: Duration = Duration::from_secs(2);
/// 最多往后翻多少首，找带逐字歌词的歌。
const LYRIC_SEARCH_LIMIT: usize = 15;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("login") => login().await,
        Some("run") => run(parse_or(args.get(1), 3)?).await,
        Some("compare") => compare(parse_or(args.get(1), 6)?).await,
        Some("lyrics") => lyric_survey(parse_or(args.get(1), 100)?).await,
        Some("url") => {
            let index = args.get(1).context("用法：probe url <序号> [音质]")?;
            let level = args.get(2).map_or("exhigh", String::as_str);
            print_url(index.parse()?, level).await
        }
        _ => bail!("用法：probe login | probe run [歌曲数] | probe url <序号> [音质]"),
    }
}

fn parse_or(arg: Option<&String>, default: usize) -> Result<usize> {
    Ok(arg.map(|s| s.parse()).transpose()?.unwrap_or(default))
}

fn cookie_path() -> PathBuf {
    let home = std::env::var_os("SPLAYER_TUI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.home"));
    home.join("cookie")
}

fn logged_in_client() -> Result<ApiClient> {
    let path = cookie_path();
    let cookie = std::fs::read_to_string(&path).context("没有保存的 Cookie，先运行 probe login")?;
    Ok(create_client(Some(cookie)))
}

/// 响应里的 Set-Cookie 只留 `名字=值`，拼成请求用的 Cookie 字符串。
fn cookie_header(set_cookies: &[String]) -> String {
    set_cookies
        .iter()
        .filter_map(|cookie| cookie.split(';').next())
        .map(str::trim)
        .filter(|pair| !pair.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn has_cookie(cookie: &str, name: &str) -> bool {
    cookie.split("; ").any(|pair| pair.split('=').next() == Some(name))
}

async fn login() -> Result<()> {
    let client = create_client(None);
    let key_response = client.login_qr_key(&Query::new()).await.map_err(describe)?;
    let body = &key_response.body;
    let key = body["unikey"]
        .as_str()
        .or_else(|| body["data"]["unikey"].as_str())
        .ok_or_else(|| anyhow!("拿不到二维码 key，code={}", body["code"]))?
        .to_string();

    let url = format!("https://music.163.com/login?codekey={key}");
    qr::show(&url)?;
    println!("请用网易云音乐 App 扫这个二维码，然后在手机上确认。");

    let mut last_code = 0;
    loop {
        tokio::time::sleep(POLL).await;
        let query = Query::new().param("key", &key);
        let response = client.login_qr_check(&query).await.map_err(describe)?;
        let code = response.body["code"].as_i64().unwrap_or(-1);
        if code != last_code {
            println!("二维码状态：{code}");
            last_code = code;
        }
        match code {
            801 => {}
            802 => {}
            803 => return save_login(&response),
            800 => bail!("二维码过期了，重新运行 probe login"),
            other => bail!("意外的状态码 {other}"),
        }
    }
}

fn save_login(response: &ApiResponse) -> Result<()> {
    let cookie = cookie_header(&response.cookie);
    if !has_cookie(&cookie, "MUSIC_U") {
        bail!("响应里没有登录 Cookie");
    }
    let path = cookie_path();
    std::fs::create_dir_all(path.parent().context("Cookie 路径没有父目录")?)?;
    std::fs::write(&path, cookie)?;
    println!("登录成功，Cookie 已保存到被 git 忽略的本地文件。可以关掉这个窗口了。");
    Ok(())
}

/// 把接口错误变成不含响应内容的简短描述。
fn describe(error: NcmError) -> anyhow::Error {
    anyhow!("接口失败：{error}")
}

async fn timed<F>(name: &str, call: F) -> Result<Value>
where
    F: Future<Output = Result<ApiResponse, NcmError>>,
{
    let started = Instant::now();
    let response = call.await.map_err(describe)?;
    println!(
        "{name:<20} {:>6} ms  code={}",
        started.elapsed().as_millis(),
        response.body["code"]
    );
    tokio::time::sleep(PAUSE).await;
    Ok(response.body)
}

async fn liked_ids(client: &ApiClient) -> Result<Vec<String>> {
    let status = timed("login_status", client.login_status(&Query::new())).await?;
    let account = if status["account"].is_object() {
        &status["account"]
    } else {
        &status["data"]["account"]
    };
    let uid = account["id"].as_i64().context("Cookie 无效，先运行 probe login")?;
    println!("已登录，vipType={}", account["vipType"]);

    let query = Query::new().param("uid", &uid.to_string());
    let likes = timed("likelist", client.likelist(&query)).await?;
    let ids: Vec<String> = likes["ids"]
        .as_array()
        .context("喜欢列表里没有 ids")?
        .iter()
        .map(|id| id.to_string())
        .collect();
    println!("喜欢列表：{} 首", ids.len());
    Ok(ids)
}

fn host_of(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or("?")
}

/// 试着从 CDN 取前 1 KB，确认地址真的能下载。
async fn check_cdn(url: &str) -> Result<()> {
    let started = Instant::now();
    let response = reqwest::Client::new()
        .get(url)
        .header("Range", "bytes=0-1023")
        .send()
        .await
        .map_err(|e| anyhow!("CDN 请求失败：{}", e.without_url()))?;
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("-")
            .to_string()
    };
    println!(
        "  CDN {} {:>6} ms  status={} type={} range={}",
        host_of(url),
        started.elapsed().as_millis(),
        response.status().as_u16(),
        header("content-type"),
        header("content-range"),
    );
    Ok(())
}

async fn probe_urls(client: &ApiClient, index: usize, id: &str) -> Result<()> {
    println!("歌曲 #{index}");
    let mut best_url = None;
    for level in LEVELS {
        let query = Query::new().param("id", id).param("level", level);
        let body = match timed(&format!("  song_url_v1 {level}"), client.song_url_v1(&query)).await {
            Ok(body) => body,
            Err(error) => {
                println!("    失败：{error}");
                continue;
            }
        };
        let item = &body["data"][0];
        let url = item["url"].as_str();
        println!(
            "    地址={} 实际音质={} br={} 格式={} 大小={:.1}MB fee={} 试听={}",
            if url.is_some() { "有" } else { "无" },
            item["level"],
            item["br"],
            item["type"],
            item["size"].as_f64().unwrap_or(0.0) / 1_048_576.0,
            item["fee"],
            if item["freeTrialInfo"].is_null() { "否" } else { "是" },
        );
        if let Some(url) = url {
            best_url = Some(url.to_string());
        }
    }
    match best_url {
        Some(url) => check_cdn(&url).await,
        None => {
            println!("  所有音质都拿不到地址");
            Ok(())
        }
    }
}

/// 打印一种歌词的有无和字节数，返回歌词原文给解析用（原文本身不打印）。
fn lyric_field<'a>(body: &'a Value, field: &str) -> (&'a str, String) {
    let text = body[field]["lyric"].as_str().unwrap_or("");
    let summary = if text.is_empty() {
        format!("{field}=无")
    } else {
        format!("{field}={}B", text.len())
    };
    (text, summary)
}

/// 返回这首歌有没有逐字歌词。
async fn probe_lyric(client: &ApiClient, index: usize, id: &str) -> Result<bool> {
    let query = Query::new().param("id", id);
    let body = timed(&format!("lyric_new #{index}"), client.lyric_new(&query)).await?;

    let (lrc, lrc_summary) = lyric_field(&body, "lrc");
    let (_, tlyric_summary) = lyric_field(&body, "tlyric");
    let (_, roma_summary) = lyric_field(&body, "romalrc");
    let (yrc, yrc_summary) = lyric_field(&body, "yrc");
    let (ytlrc, ytlrc_summary) = lyric_field(&body, "ytlrc");
    let (_, yroma_summary) = lyric_field(&body, "yromalrc");
    println!(
        "    {lrc_summary} {tlyric_summary} {roma_summary} {yrc_summary} {ytlrc_summary} {yroma_summary}"
    );

    if !lrc.is_empty() {
        println!("    lrc 解析：{} 行", amll_lyric::lrc::parse_lrc(lrc).len());
    }
    if yrc.is_empty() {
        return Ok(false);
    }

    let started = Instant::now();
    let lines = amll_lyric::yrc::parse_yrc(yrc);
    let elapsed = started.elapsed();
    let raw_lines = yrc.lines().filter(|line| line.starts_with('[')).count();
    let words: usize = lines.iter().map(|line| line.words.len()).sum();
    let first = lines.first().and_then(|line| line.words.first());
    println!(
        "    yrc 解析：原文 {raw_lines} 行 → {} 行 {words} 字，{} µs，首字 {:?}，翻译 {} 行",
        lines.len(),
        elapsed.as_micros(),
        first.map(|word| (word.start_time, word.end_time)),
        ytlrc.lines().filter(|line| line.starts_with('[')).count(),
    );
    Ok(true)
}

async fn run(samples: usize) -> Result<()> {
    let client = logged_in_client()?;
    let ids = liked_ids(&client).await?;

    for (index, id) in ids.iter().take(samples).enumerate() {
        probe_urls(&client, index + 1, id).await?;
    }

    let mut with_yrc = 0;
    let searched = ids.len().min(LYRIC_SEARCH_LIMIT);
    for (index, id) in ids.iter().take(searched).enumerate() {
        if probe_lyric(&client, index + 1, id).await? {
            with_yrc += 1;
        }
    }
    println!("逐字歌词：前 {searched} 首里有 {with_yrc} 首");
    Ok(())
}

/// 设备 ID 存下来，免得每次运行都像一台新设备。
fn device_id() -> Result<String> {
    let path = cookie_path().with_file_name("device_id");
    if let Ok(id) = std::fs::read_to_string(&path) {
        return Ok(id);
    }
    let id = ncm_api_rs::util::device::generate_device_id();
    std::fs::write(&path, &id)?;
    Ok(id)
}

/// 一种取地址方式的统计。
#[derive(Default)]
struct Tally {
    urls: usize,
    no_url: usize,
    errors: usize,
    total_ms: u128,
    first_error: Option<String>,
}

impl Tally {
    fn record(&mut self, result: Result<Value>, elapsed: Duration) {
        self.total_ms += elapsed.as_millis();
        match result {
            Ok(body) if body["data"][0]["url"].is_string() => self.urls += 1,
            Ok(body) if body["code"] == 200 => self.no_url += 1,
            Ok(body) => self.fail(format!("code={}", body["code"])),
            Err(error) => self.fail(error.to_string()),
        }
    }

    fn fail(&mut self, message: String) {
        self.errors += 1;
        self.first_error.get_or_insert(message);
    }

    fn summary(&self) -> String {
        let total = self.urls + self.no_url + self.errors;
        format!(
            "有地址 {}/{total}，无地址 {}，报错 {}，平均 {} ms{}",
            self.urls,
            self.no_url,
            self.errors,
            self.total_ms / total.max(1) as u128,
            self.first_error.as_ref().map_or(String::new(), |e| format!("（{e}）")),
        )
    }
}

/// 同样的歌、同样的音质，eapi 和 xeapi 轮流取地址，比成功率。
async fn compare(rounds: usize) -> Result<()> {
    const URI: &str = "/api/song/enhance/player/url/v1";
    const SONGS: usize = 3;

    let client = logged_in_client()?;
    let ids = liked_ids(&client).await?;
    let cookie = std::fs::read_to_string(cookie_path())?;
    let mut xeapi = xeapi::Xeapi::connect(&cookie, &device_id()?).await?;
    println!("xeapi 公钥已拿到");

    for level in LEVELS {
        let mut eapi_tally = Tally::default();
        let mut xeapi_tally = Tally::default();
        for round in 0..rounds {
            let id = &ids[round % SONGS.min(ids.len())];

            let started = Instant::now();
            let query = Query::new().param("id", id).param("level", level);
            let result = client.song_url_v1(&query).await.map(|r| r.body).map_err(describe);
            eapi_tally.record(result, started.elapsed());
            tokio::time::sleep(PAUSE).await;

            let started = Instant::now();
            let ids_param = format!("[{id}]");
            let data = [("ids", ids_param.as_str()), ("level", level), ("encodeType", "flac")];
            let result = xeapi.request(URI, &data).await;
            xeapi_tally.record(result, started.elapsed());
            tokio::time::sleep(PAUSE).await;
        }
        println!("{level:<9} eapi：{}", eapi_tally.summary());
        println!("{level:<9} xeapi：{}", xeapi_tally.summary());
    }

    let ids_param = format!("[{}]", ids[0]);
    let data = [("ids", ids_param.as_str()), ("level", "exhigh"), ("encodeType", "flac")];
    let body = xeapi.request(URI, &data).await?;
    let url = body["data"][0]["url"].as_str().context("xeapi 没拿到地址")?;
    check_cdn(url).await
}

/// 统计前几首的歌词：只打印总数，看逐字歌词有多常见、解析会不会丢行。
async fn lyric_survey(count: usize) -> Result<()> {
    let client = logged_in_client()?;
    let ids = liked_ids(&client).await?;

    let (mut errors, mut with_lrc, mut with_tlyric, mut with_yrc, mut with_ytlrc) = (0, 0, 0, 0, 0);
    let (mut raw_lines, mut parsed_lines, mut words, mut parse_micros) = (0, 0, 0, 0u128);
    let mut lossy_songs = 0;
    let sampled = ids.len().min(count);

    for id in ids.iter().take(sampled) {
        let query = Query::new().param("id", id);
        let body = match client.lyric_new(&query).await {
            Ok(response) => response.body,
            Err(_) => {
                errors += 1;
                tokio::time::sleep(PAUSE).await;
                continue;
            }
        };
        let has = |field: &str| !lyric_field(&body, field).0.is_empty();
        with_lrc += usize::from(has("lrc"));
        with_tlyric += usize::from(has("tlyric"));
        with_ytlrc += usize::from(has("ytlrc"));

        let (yrc, _) = lyric_field(&body, "yrc");
        if !yrc.is_empty() {
            with_yrc += 1;
            let started = Instant::now();
            let lines = amll_lyric::yrc::parse_yrc(yrc);
            parse_micros += started.elapsed().as_micros();
            let raw = yrc.lines().filter(|line| line.starts_with('[')).count();
            raw_lines += raw;
            parsed_lines += lines.len();
            words += lines.iter().map(|line| line.words.len()).sum::<usize>();
            lossy_songs += usize::from(lines.len() != raw);
        }
        tokio::time::sleep(PAUSE).await;
    }

    println!("抽样 {sampled} 首，接口报错 {errors}");
    println!("lrc {with_lrc} 首，翻译 {with_tlyric} 首，逐字 {with_yrc} 首，逐字翻译 {with_ytlrc} 首");
    println!(
        "逐字解析：原文 {raw_lines} 行 → {parsed_lines} 行，{words} 字，行数对不上的 {lossy_songs} 首，共 {parse_micros} µs"
    );
    Ok(())
}

/// 只把地址打到标准输出，方便 `play "$(probe url 1)"`。
async fn print_url(index: usize, level: &str) -> Result<()> {
    let client = logged_in_client()?;
    let status = client.login_status(&Query::new()).await.map_err(describe)?.body;
    let uid = status["account"]["id"]
        .as_i64()
        .or_else(|| status["data"]["account"]["id"].as_i64())
        .context("Cookie 无效，先运行 probe login")?;
    let query = Query::new().param("uid", &uid.to_string());
    let likes = client.likelist(&query).await.map_err(describe)?;
    let id = likes.body["ids"][index.saturating_sub(1)].to_string();

    let cookie = std::fs::read_to_string(cookie_path())?;
    let mut xeapi = xeapi::Xeapi::connect(&cookie, &device_id()?).await?;
    let ids_param = format!("[{id}]");
    let data = [("ids", ids_param.as_str()), ("level", level), ("encodeType", "flac")];
    let body = xeapi.request("/api/song/enhance/player/url/v1", &data).await?;
    let url = body["data"][0]["url"].as_str().context("这首歌拿不到地址")?;
    println!("{url}");
    Ok(())
}
