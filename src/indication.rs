use crate::hal::gpio::{Output, PushPull, Pxx};
use crate::timer::{CounterTypeExt, Timer};
use embedded_hal::digital::v2::OutputPin;
struct Led {
    pin: LedPin,
    is_on: bool,
}

impl Led {
    fn new(pin: LedPin, is_on: bool) -> Self {
        let mut res = Led { pin, is_on };
        if is_on {
            res.pin.set_high().unwrap();
        } else {
            res.pin.set_low().unwrap();
        }
        res
    }
    fn toggle(&mut self) {
        if self.is_on {
            self.pin.set_low().unwrap();
        } else {
            self.pin.set_high().unwrap();
        }
        self.is_on = !self.is_on;
    }
    fn set_high(&mut self) {
        self.pin.set_high().unwrap();
        self.is_on = true;
    }
    fn set_low(&mut self) {
        self.pin.set_low().unwrap();
        self.is_on = false;
    }
}
type LedPin = Pxx<Output<PushPull>>;

#[derive(Copy, Clone, PartialEq)]
pub enum IndicationState {
    Nothing,
    Idle,
    Error,
    ReadyToArm,
    Armed,
}
pub struct Indication {
    led_red: Led,
    led_green: Led,
    state: IndicationState,
    state_changed: bool,
    timer: Timer,
}

impl Indication {
    pub fn new(pin_red: LedPin, pin_green: LedPin) -> Self {
        Indication {
            led_red: Led::new(pin_red, false),
            led_green: Led::new(pin_green, false),
            state: IndicationState::Nothing,
            state_changed: true,
            timer: Timer::new(),
        }
    }
    pub fn set_state(&mut self, state: IndicationState) {
        self.state_changed = self.state != state;
        self.state = state;
    }

    pub fn poll(&mut self) {
        if !self.timer.every(1.sec()) {
            return;
        };
        match self.state {
            IndicationState::Nothing => {
                if self.state_changed {
                    self.led_red.set_low();
                    self.led_green.set_low();
                }
            }
            IndicationState::Idle => {
                if self.state_changed {
                    self.led_red.set_low();
                    self.led_green.set_high();
                }
            }
            IndicationState::Error => {
                if self.state_changed {
                    self.led_red.set_low();
                    self.led_green.set_low();
                } else {
                    self.led_red.toggle();
                    self.led_green.toggle();
                }
            }
            IndicationState::ReadyToArm => {
                if self.state_changed {
                    self.led_red.set_low();
                    self.led_green.set_high();
                } else {
                    self.led_red.toggle();
                }
            }
            IndicationState::Armed => {
                if self.state_changed {
                    self.led_red.set_high();
                    self.led_green.set_high();
                }
            }
        }
        self.state_changed = false;
    }
}
