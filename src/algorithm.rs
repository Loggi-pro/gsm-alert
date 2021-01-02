use crate::indication::{Indication, IndicationState};
use crate::sim900::Sim900;
use crate::timer::{CounterTypeExt, MilliSeconds, TimeType, Timer};

pub struct MainLogic {
    sim900: Sim900,
    indication: Indication,
    current_state: IndicationState,
    timer: Timer,
}

impl MainLogic {
    pub fn new(sim900: Sim900, indication: Indication) -> MainLogic {
        MainLogic {
            sim900: sim900,
            indication: indication,
            current_state: IndicationState::Idle,
            timer: Timer::new(),
        }
    }
    fn check_gsm(&mut self) -> bool {
        if self.sim900.power_on().is_err() {
            return false;
        }
        if self.sim900.setup().is_err() {
            return false;
        }
        self.sim900.power_off();
        true
    }

    pub fn init(&mut self) {
        self.indication.set_state(IndicationState::Error);
        if !self.check_gsm() {
            return;
        }
        self.current_state = IndicationState::Idle;
        self.indication.set_state(self.current_state);
    }
    pub fn poll(&mut self) {
        self.indication.poll();
        if self.timer.every(10.sec()) {
            if !self.check_gsm() {
                self.indication.set_state(IndicationState::Error);
            } else {
                self.indication.set_state(self.current_state);
            }
        }
    }
}
