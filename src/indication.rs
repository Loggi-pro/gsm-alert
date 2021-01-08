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
    IdleDoorClosed,
    Error,
    CheckBeforeArm,
    ReadyToArm,
    Armed,
}
pub struct Indication {
    led_red: Led,
    led_green: Led,
    state: IndicationState,
    timer: Timer,
}

impl Indication {
    pub fn new(pin_red: LedPin, pin_green: LedPin) -> Self {
        Indication {
            led_red: Led::new(pin_red, false),
            led_green: Led::new(pin_green, false),
            state: IndicationState::Nothing,
            timer: Timer::new(),
        }
    }
    pub fn set_state(&mut self, state: IndicationState) {
        //initial state
        if self.state != state {
            match state {
                IndicationState::Nothing => {
                    self.led_red.set_low();
                    self.led_green.set_low();
                }
                IndicationState::Idle => {
                    self.led_red.set_low();
                    self.led_green.set_high();
                }
                IndicationState::IdleDoorClosed => {
                    self.led_red.set_low();
                    self.led_green.set_low();
                }
                IndicationState::Error => {
                    self.led_red.set_low();
                    self.led_green.set_low();
                }
                IndicationState::ReadyToArm | IndicationState::CheckBeforeArm => {
                    self.led_red.set_low();
                    self.led_green.set_high();
                }
                IndicationState::Armed => {
                    self.led_red.set_high();
                    self.led_green.set_high();
                }
            }
        }
        self.state = state;
    }

    pub fn poll(&mut self) {
        match self.state {
            IndicationState::Nothing => {}
            IndicationState::Idle => {}
            IndicationState::IdleDoorClosed => {
                if self.timer.every(500.mil()) {
                    self.led_green.toggle();
                }
            }
            IndicationState::Error => {
                if self.timer.every(1.sec()) {
                    self.led_red.toggle();
                    self.led_green.toggle();
                }
            }
            IndicationState::CheckBeforeArm => {
                if self.timer.every(250.mil()) {
                    self.led_red.toggle();
                }
            }
            IndicationState::ReadyToArm => {
                if self.timer.every(1.sec()) {
                    self.led_red.toggle();
                }
            }
            IndicationState::Armed => {}
        }
    }
}
