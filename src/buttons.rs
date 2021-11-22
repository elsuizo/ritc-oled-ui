/// Button primitives and implementations
use core::convert::Infallible;
use embedded_hal::digital::v2::InputPin;

pub enum PinState {
    PinUp,
    PinDown,
    Nothing,
}

type Counter = u8;

#[derive(Copy, Clone)]
enum ButtonState {
    High(Counter),
    Low(Counter),
}

pub struct Button<P> {
    typ: P,
    state: ButtonState,
}

impl<P: InputPin<Error = Infallible>> Button<P> {
    pub fn new(typ: P) -> Self {
        Self {
            typ,
            state: ButtonState::High(0u8),
        }
    }

    pub fn polling(&mut self) -> PinState {
        use self::ButtonState::*;
        let value = self.typ.is_high().unwrap();
        match (&mut self.state, value) {
            (High(counter), true) => *counter = 0,
            (High(counter), false) => *counter += 1,
            (Low(counter), true) => *counter += 1,
            (Low(counter), false) => *counter = 0,
        }
        match self.state {
            High(cnt) if cnt >= 30 => {
                self.state = Low(0);
                PinState::PinUp
            }
            Low(cnt) if cnt >= 30 => {
                self.state = High(0);
                PinState::PinDown
            }
            _ => PinState::Nothing,
        }
    }
}
