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
pub const ROWS_HEIGT: i32 = DISPLAY_WIDTH / 3;
const X_PAD: i32 = 1;
const Y_PAD: i32 = 2;
const CHAR_HEIGHT: i32 = 14;
const CHAR_WIDTH: i32 = 6;
// const BAR_WIDTH: u32 = (DISP_WIDTH - X_PAD * 2) as u32;

pub fn draw_text<D>(
    target: &mut D,
    message: &str,
    x: i32,
    y: i32,
    selected: bool,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    // normal text
    let normal = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(BinaryColor::On)
        .build();

    // text with background
    let background = MonoTextStyleBuilder::from(&normal)
        .background_color(BinaryColor::On)
        .text_color(BinaryColor::Off)
        .build();

    let text = match selected {
        true => Text::new(message, Point::new(x, y), normal),
        false => Text::new(message, Point::new(x, y), background),
    };
    // MonoTextStyle::new(&FONT_9X15, BinaryColor::On),
    // Draw the text after the background is drawn.
    text.draw(target)?;

    Ok(())
}

pub fn draw_menu<D>(target: &mut D, state: MenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    // normal text
    let normal = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(BinaryColor::On)
        .build();
    // text with background
    let background = MonoTextStyleBuilder::from(&normal)
        .background_color(BinaryColor::On)
        .text_color(BinaryColor::Off)
        .build();

    let text = match state {
        MenuState::Row1 => Text::new("--- Menu 1 ---", Point::new(0, 13), normal),
        MenuState::Row2 => Text::new("--- Menu 2 ---", Point::new(0, 33), normal),
        MenuState::Row3 => Text::new("--- Menu 3 ---", Point::new(0, 53), normal),
    };

    // MonoTextStyle::new(&FONT_9X15, BinaryColor::On),
    // Draw the text after the background is drawn.
    text.draw(target)?;

    Ok(())
}

//-------------------------------------------------------------------------
//                        finite state machine for the menu
//-------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum Msg {
    Button0, // Up
    Button1, // Down
}

#[derive(Copy, Clone)]
pub enum MenuState {
    Row1,
    Row2,
    Row3,
}

#[derive(Copy, Clone)]
pub struct MenuFSM {
    pub state: MenuState,
}

impl MenuFSM {
    pub fn init(state: MenuState) -> Self {
        Self { state }
    }

    pub fn advance(&mut self, msg: Msg) {
        use MenuState::*;
        use Msg::*;

        self.state = match (self.state, msg) {
            (Row1, Button0) => Row3,
            (Row1, Button1) => Row2,
            (Row2, Button0) => Row1,
            (Row2, Button1) => Row3,
            (Row3, Button0) => Row2,
            (Row3, Button1) => Row1,
        }
    }
}
