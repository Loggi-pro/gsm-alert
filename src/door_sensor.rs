use crate::hal::gpio::{Input, PullUp, Pxx};
use crate::timer::{MilliSeconds, Timer};
use embedded_hal::digital::v2::InputPin;
pub struct DoorSensor {
    pin: Pxx<Input<PullUp>>,
    counter: u8,
    timer: Timer,
    is_opened_last_state: bool,
}
pub enum DoorState {
    Opened,
    Closed,
}

impl DoorSensor {
    const MAX_COUNT: u8 = 3;
    const TIMEOUT: MilliSeconds = MilliSeconds(1000_u16);
    pub fn new(pin: Pxx<Input<PullUp>>) -> Self {
        let mut res = DoorSensor {
            pin,
            counter: Self::MAX_COUNT,
            timer: Timer::new(),
            is_opened_last_state: true,
        };
        res.is_opened_last_state = res.is_open();
        res
    }
    pub fn is_open(&self) -> bool {
        match self.pin.is_high() {
            Ok(x) => x,
            _ => true,
        }
    }
    pub fn is_closed(&self) -> bool {
        !self.is_open()
    }
    pub fn state(&mut self) -> Option<DoorState> {
        if !self.timer.every(Self::TIMEOUT) {
            return None;
        }
        let is_open = self.is_open();
        if is_open != self.is_opened_last_state {
            if self.counter > 0 {
                self.counter = self.counter - 1;
                if self.counter == 0 {
                    self.is_opened_last_state = is_open;

                    return match is_open {
                        true => Some(DoorState::Opened),
                        false => Some(DoorState::Closed),
                    };
                }
            }
        } else {
            self.counter = Self::MAX_COUNT;
        }
        return None;
    }
}
