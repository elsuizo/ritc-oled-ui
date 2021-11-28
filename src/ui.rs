/// User interface primitives
use embedded_graphics::{
    image::{Image, ImageRawLE},
    mono_font::{ascii::FONT_9X15, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

// TODO(elsuizo:2021-11-28): use this constants for a better text positions
pub const DISPLAY_WIDTH: i32 = 128;
pub const DISPLAY_HEIGHT: i32 = DISPLAY_WIDTH / 2;
pub const ROWS_HEIGT: i32 = DISPLAY_WIDTH / 3;
const CHAR_HEIGHT: i32 = 14;
const CHAR_WIDTH: i32 = 6;

/// This is the principal function that renders all the menu states
pub fn draw_menu<D>(target: &mut D, state: MenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let logo_image = ImageRawLE::new(include_bytes!("../Images/very_logo.raw"), 64);
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

    // TODO(elsuizo:2021-11-28): could be better some sort of calculation for the place of the
    // menus positions and not hardcoded values...
    match state {
        MenuState::Row1(true) => {
            Text::new("--- Menu 1 ---", Point::new(0, 13), background).draw(target)?;
            Text::new("--- Menu 2 ---", Point::new(0, 33), normal).draw(target)?;
            Text::new("--- Menu 3 ---", Point::new(0, 53), normal).draw(target)?;
        }
        MenuState::Row2(true) => {
            Text::new("--- Menu 1 ---", Point::new(0, 13), normal).draw(target)?;
            Text::new("--- Menu 2 ---", Point::new(0, 33), background).draw(target)?;
            Text::new("--- Menu 3 ---", Point::new(0, 53), normal).draw(target)?;
        }
        MenuState::Row3(true) => {
            Text::new("--- Menu 1 ---", Point::new(0, 13), normal).draw(target)?;
            Text::new("--- Menu 2 ---", Point::new(0, 33), normal).draw(target)?;
            Text::new("--- Menu 3 ---", Point::new(0, 53), background).draw(target)?;
        }
        MenuState::Row1(false) | MenuState::Row2(false) | MenuState::Row3(false) => {
            Text::new("--- Menu 1 ---", Point::new(0, 13), normal).draw(target)?;
            Text::new("--- Menu 2 ---", Point::new(0, 33), normal).draw(target)?;
            Text::new("--- Menu 3 ---", Point::new(0, 53), normal).draw(target)?;
        }
        MenuState::Image => {
            Image::new(&logo_image, Point::new(32, 0)).draw(target)?;
        }
    }
    Ok(())
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

type BackgroundFlag = bool;

#[derive(Copy, Clone)]
pub enum MenuState {
    Row1(BackgroundFlag),
    Row2(BackgroundFlag),
    Row3(BackgroundFlag),
    Image,
}

impl MenuState {
    /// check if the state is a Rown
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
            (Row1(false), Up) => Row1(true),
            (Row1(false), Down) => Row1(true),
            (Row1(_), Up) => Row3(true),
            (Row1(_), Down) => Row2(true),
            (Row2(_), Up) => Row1(true),
            (Row2(_), Down) => Row3(true),
            (Row3(_), Up) => Row2(true),
            (Row3(_), Down) => Row1(true),
            (_, Enter) => Image,
            (Image, _) => Row1(false),
        }
    }
}
