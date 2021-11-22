/// User interface primitives
use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

pub const DISPLAY_WIDTH: i32 = 128;
pub const DISPLAY_HEIGHT: i32 = DISPLAY_WIDTH / 2;
const X_PAD: i32 = 1;
const Y_PAD: i32 = 2;
const CHAR_HEIGHT: i32 = 14;
const CHAR_WIDTH: i32 = 6;
// const BAR_WIDTH: u32 = (DISP_WIDTH - X_PAD * 2) as u32;

pub fn draw_text<D>(target: &mut D, text: &str, x: i32, y: i32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    // building the text
    let normal = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(BinaryColor::On)
        .build();

    let background = MonoTextStyleBuilder::from(&normal)
        .background_color(BinaryColor::On)
        .text_color(BinaryColor::Off)
        .build();
    // generate a new text type with a background
    let text = Text::new(text, Point::new(x, y), background);
    // MonoTextStyle::new(&FONT_9X15, BinaryColor::On),
    // Draw the text after the background is drawn.
    text.draw(target)?;

    Ok(())
}
