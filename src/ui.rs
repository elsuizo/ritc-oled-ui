/// User interface primitives
use embedded_graphics::{
    image::{Image, ImageRawLE},
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
    let im: ImageRawLE<BinaryColor> = ImageRawLE::new(include_bytes!("../Images/rust.raw"), 64);
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

    let result: TextOrImage = match state {
        MenuState::Row1(true) => Text::new("--- Menu 1 ---", Point::new(0, 13), normal),
        MenuState::Row1(false) => Text::new("--- Menu 1 ---", Point::new(0, 13), background),
        MenuState::Row2(true) => Text::new("--- Menu 2 ---", Point::new(0, 33), normal),
        MenuState::Row2(false) => Text::new("--- Menu 2 ---", Point::new(0, 33), background),
        MenuState::Row3(true) => Text::new("--- Menu 3 ---", Point::new(0, 53), normal),
        MenuState::Row3(false) => Text::new("--- Menu 3 ---", Point::new(0, 53), background),
        MenuState::Image => Image::new(&im, Point::new(0, 13)),
    };

    // MonoTextStyle::new(&FONT_9X15, BinaryColor::On),
    // Draw the text after the background is drawn.
    result.draw(target)?;

    Ok(())
}

enum TextOrImage {
    Text,
    Image,
}

//-------------------------------------------------------------------------
//                        finite state machine for the menu
//-------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum Msg {
    Up,    // Up button
    Down,  // Down button
    Enter, // Enter button
}

#[derive(Copy, Clone)]
pub enum Items {
    Item1,
    Item2,
    Item3,
}

type BackgroundFlag = bool;

#[derive(Copy, Clone)]
pub enum MenuState {
    Row1(BackgroundFlag),
    Row2(BackgroundFlag),
    Row3(BackgroundFlag),
    Image,
}

impl MenuState {
    fn is_row(&self) -> bool {
        matches!(self, Self::Row1(_) | Self::Row2(_) | Self::Row3(_))
    }
}

#[derive(Copy, Clone)]
pub struct MenuFSM {
    pub state: MenuState,
}

impl MenuFSM {
    pub fn init(state: MenuState) -> Self {
        Self { state }
    }

    pub fn next_state(&mut self, msg: Msg) {
        use MenuState::*;
        use Msg::*;

        self.state = match (self.state, msg) {
            (Row1(_), Up) => Row3(false),
            (Row1(_), Down) => Row2(false),
            (Row2(_), Up) => Row1(false),
            (Row2(_), Down) => Row3(false),
            (Row3(_), Up) => Row2(false),
            (Row3(_), Down) => Row1(false),
            (_, Enter) => Image,
            (Image, _) => Row1(false),
        }
    }
}
