/// User interface primitives
use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

pub fn draw_text<D>(target: &mut D, text: &str, x: i32, y: i32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    // Create a styled text object for the time text.
    let text = Text::new(
        text,
        Point::new(x, y),
        MonoTextStyle::new(&FONT_9X15, BinaryColor::On),
    );
    // Draw the text after the background is drawn.
    text.draw(target)?;

    Ok(())
}
