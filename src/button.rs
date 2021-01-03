use crate::hal::gpio::{Input, PullUp, Pxx};
use crate::timer::{MilliSeconds, Timer};
use embedded_hal::digital::v2::InputPin;
pub struct Button {
    pin: Pxx<Input<PullUp>>,
    counter: u8,
    is_default_high: bool,
    timer: Timer,
}

impl Button {
    const MAX_COUNT: u8 = 3;
    const TIMEOUT: MilliSeconds = MilliSeconds(25_u16);
    pub fn new(pin: Pxx<Input<PullUp>>, is_default_high: bool) -> Self {
        Button {
            pin,
            counter: Self::MAX_COUNT,
            is_default_high,
            timer: Timer::new(),
        }
    }
    pub fn is_pressed(&mut self) -> Option<bool> {
        if !self.timer.every(Self::TIMEOUT) {
            return None;
        }
        match self.pin.is_high() {
            //unpressed
            Ok(is_high) if is_high == self.is_default_high => {
                self.counter = Self::MAX_COUNT;
                None
            }
            //pressed
            Ok(_) => {
                if self.counter > 0 {
                    self.counter = self.counter - 1;
                    if self.counter == 0 {
                        return Some(true);
                    }
                }
                None
            }
            //other
            _ => None,
        }
    }
}
