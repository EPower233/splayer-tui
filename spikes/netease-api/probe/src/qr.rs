//! 在终端里画二维码：固定黑字白底，不受终端配色影响；上下两个模块合成一个字符。

use std::io::stdout;

use crossterm::execute;
use crossterm::style::{Color, Colors, Print, ResetColor, SetColors};
use qrcode::{Color as Module, QrCode};

/// 二维码规范要求四周留 4 个模块的空白，否则手机不好识别。
const QUIET_ZONE: i32 = 4;

pub fn show(text: &str) -> anyhow::Result<()> {
    let code = QrCode::new(text.as_bytes())?;
    let width = code.width() as i32;
    let modules = code.to_colors();
    let is_dark = |x: i32, y: i32| {
        (0..width).contains(&x)
            && (0..width).contains(&y)
            && modules[(y * width + x) as usize] == Module::Dark
    };

    let mut out = stdout();
    for y in (-QUIET_ZONE..width + QUIET_ZONE).step_by(2) {
        let line: String = (-QUIET_ZONE..width + QUIET_ZONE)
            .map(|x| match (is_dark(x, y), is_dark(x, y + 1)) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            })
            .collect();
        execute!(
            out,
            SetColors(Colors::new(Color::Black, Color::White)),
            Print(line),
            ResetColor,
            Print("\n")
        )?;
    }
    Ok(())
}
