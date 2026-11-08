//! 摸底实验 T-1：用和 ffmpeg-audio 相同的 reqwest 配置连一个地址，打印完整错误链。

use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let url = std::env::args().nth(1).expect("用法：diag <url>");
    for (name, use_proxy) in [("no_proxy（引擎默认）", false), ("系统代理", true)] {
        let builder = reqwest::Client::builder().connect_timeout(Duration::from_secs(10));
        let builder = if use_proxy { builder } else { builder.no_proxy() };
        let client = builder.build().expect("build client");
        match client.get(&url).header("Range", "bytes=0-1023").send().await {
            Ok(resp) => println!("{name}: HTTP {}", resp.status()),
            Err(err) => {
                println!("{name}: 失败");
                let mut source: Option<&dyn Error> = Some(&err);
                let mut depth = 0;
                while let Some(e) = source {
                    println!("  {depth}: {e}");
                    source = e.source();
                    depth += 1;
                }
            }
        }
    }
}
