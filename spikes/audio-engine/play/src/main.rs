//! 摸底实验 T-1：用普通 Rust 程序调 SPlayer-Next 的音频引擎。
//!
//! 用法：play <本地文件或网络地址> [每段播放秒数，默认 10] [seek 到第几秒，默认 30]
//! 先播一段，再 seek，再播一段，期间打印引擎发来的事件。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use audio_engine::{AudioPlayer, JsPlayerEvent};
use napi::bindgen_prelude::Function;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let source = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("用法：play <文件或地址> [每段播放秒数] [seek 秒数]"))?;
    let play_secs: u64 = args.next().map(|s| s.parse()).transpose()?.unwrap_or(10);
    let seek_to: f64 = args.next().map(|s| s.parse()).transpose()?.unwrap_or(30.0);

    let player = Arc::new(AudioPlayer::new()?);
    // 自动验证时用 PLAY_VOLUME=0，整条输出链路照常走，只是不出声。
    if let Ok(volume) = std::env::var("PLAY_VOLUME") {
        player.set_volume(volume.parse()?);
    }
    let position_events = Arc::new(AtomicU64::new(0));
    let position_events_in_callback = Arc::clone(&position_events);
    let started = Instant::now();
    player.on_event(Function::new(move |event: JsPlayerEvent| {
        if event.event_type == "position" {
            position_events_in_callback.fetch_add(1, Ordering::Relaxed);
        }
        if event.event_type != "position" && event.event_type != "fftData" {
            println!(
                "[{:>6.2}s] 事件 {} {}",
                started.elapsed().as_secs_f64(),
                event.event_type,
                event.state.unwrap_or_default()
            );
        }
    }))?;

    let t = Instant::now();
    let meta = player.load(source, Some(true), None).await?;
    println!(
        "加载完成 {:.0} ms：{} / {} / {}，{:.1} s，{} Hz（原始 {} Hz，{} bit），{} 声道，{}",
        t.elapsed().as_secs_f64() * 1000.0,
        meta.title.as_deref().unwrap_or("?"),
        meta.artist.as_deref().unwrap_or("?"),
        meta.album.as_deref().unwrap_or("?"),
        meta.duration,
        meta.sample_rate,
        meta.original_sample_rate,
        meta.bits_per_sample,
        meta.channels,
        meta.codec
    );

    tokio::time::sleep(Duration::from_secs(play_secs)).await;
    println!("播了 {play_secs} s，引擎报告位置 {:.2} s", player.get_position());

    let t = Instant::now();
    player.seek(seek_to).await?;
    println!(
        "seek 到 {seek_to} s 用了 {:.0} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    tokio::time::sleep(Duration::from_secs(play_secs)).await;
    println!("又播了 {play_secs} s，引擎报告位置 {:.2} s", player.get_position());

    // stop 会销毁 HTTP 流内部的 tokio 运行时，不能在异步上下文里做，必须放到阻塞线程上。
    // Node 里同步调用走 JS 主线程，所以上游不会遇到；普通 Rust 程序要自己注意。
    let stopping = Arc::clone(&player);
    tokio::task::spawn_blocking(move || stopping.stop()).await?;
    println!(
        "停止，共收到 {} 个进度事件",
        position_events.load(Ordering::Relaxed)
    );
    Ok(())
}
