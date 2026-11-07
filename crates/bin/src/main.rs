//! 可执行文件 `splayer-tui`：读配置，把适配器接进播放器大脑，再接上终端界面。
//!
//! 现在还只是骨架，只打印版本号。

fn main() {
    println!("splayer-tui {}", env!("CARGO_PKG_VERSION"));
}
