use crate::button::Button;
use crate::door_sensor::{DoorSensor, DoorState};
use crate::indication::{Indication, IndicationState};
use crate::sim900::Sim900;
use crate::timer::{CounterTypeExt, TimeType, Timer};
pub struct MainLogic {
    sim900: Sim900,
    indication: Indication,
    power_button: Button,
    door_sensor: DoorSensor,
    current_state: IndicationState,
    is_error: bool,
    timer: Timer,
    timer2: Timer,
    check_state: u8,
    flag_go_check: bool,
}

fn synchronize<T: Sized>(mut f: impl FnMut() -> Option<T>) -> T {
    loop {
        if let Some(x) = f() {
            return x;
        }
    }
}

impl MainLogic {
    pub fn new(
        sim900: Sim900,
        indication: Indication,
        power_button: Button,
        door_sensor: DoorSensor,
    ) -> MainLogic {
        MainLogic {
            sim900,
            indication,
            power_button,
            door_sensor,
            current_state: IndicationState::Idle,
            is_error: false,
            timer: Timer::new(),
            timer2: Timer::new(),
            check_state: 0,
            flag_go_check: false,
        }
    }
    fn delay(&mut self) -> Option<bool> {
        if self.timer2.every(3.sec()) {
            return Some(true);
        }
        return None;
    }
    fn check_gsm(&mut self) -> Option<bool> {
        match self.check_state {
            0 => {
                if let Some(res) = self.sim900.power_on() {
                    if res.is_err() {
                        self.check_state = 0;
                        Some(false)
                    } else {
                        //good go next state
                        self.check_state = 1;
                        None
                    }
                } else {
                    None
                }
            }
            1 => {
                if self.sim900.setup().is_err() {
                    self.check_state = 0;
                    Some(false)
                } else {
                    self.check_state = 2;
                    None //go next
                }
            }
            2 => {
                if self.sim900.power_off().is_some() {
                    self.check_state = 0;
                    Some(true)
                } else {
                    None
                }
            }
            _ => {
                self.check_state = 0;
                None
            }
        }
    }
    fn set_error(&mut self) {
        self.is_error = true;
        self.indication.set_state(IndicationState::Error);
    }
    fn clear_error(&mut self) {
        self.is_error = false;
        self.indication.set_state(self.current_state);
    }
    pub fn init(&mut self) {
        if !synchronize(|| self.check_gsm()) {
            self.set_error();
        } else {
            self.current_state = match self.door_sensor.is_closed() {
                true => IndicationState::Armed,
                false => IndicationState::Idle,
            };
            self.clear_error();
        }
    }
    fn button_poll(&mut self) {
        if self.is_error {
            return;
        }
        if let Some(true) = self.power_button.is_pressed() {
            self.current_state = match self.current_state {
                IndicationState::Armed
                | IndicationState::ReadyToArm
                | IndicationState::CheckBeforeArm => IndicationState::Idle,
                IndicationState::Idle => IndicationState::CheckBeforeArm,
                _ => self.current_state,
            };
            self.indication.set_state(self.current_state);
        }
    }
    fn state_arm(&mut self) {
        self.current_state = IndicationState::Armed;
        self.indication.set_state(self.current_state);
    }
    fn state_disarm(&mut self) {
        self.current_state = IndicationState::Idle;
        self.indication.set_state(self.current_state);
    }
    fn door_poll(&mut self) {
        match self.current_state {
            IndicationState::ReadyToArm => {
                if let Some(DoorState::Closed) = self.door_sensor.state() {
                    self.state_arm();
                }
            }
            IndicationState::Armed => {
                if let Some(DoorState::Opened) = self.door_sensor.state() {
                    self.state_disarm();
                }
            }
            _ => (),
        }
    }

    fn gsm_poll<T: TimeType>(&mut self, period: T) {
        //check gsm every 30 sec on error, or only before arm
        match self.current_state {
            IndicationState::CheckBeforeArm => self.flag_go_check = true,
            IndicationState::Error => {
                if self.timer.every(period) {
                    self.flag_go_check = true
                }
            }
            _ => (),
        }
        if !self.flag_go_check {
            return;
        }
        if let Some(gsm_good) = self.check_gsm() {
            //on check result
            self.flag_go_check = false;
            //if was check mode before arm then set ready
            if self.current_state == IndicationState::CheckBeforeArm && gsm_good {
                self.current_state = IndicationState::ReadyToArm;
            }
            //from other mode=> go to error or previous mode
            match gsm_good {
                true => self.clear_error(),
                false => self.set_error(),
            };
        }
    }

    pub fn poll(&mut self) {
        self.indication.poll();
        self.button_poll();
        self.door_poll();
        self.gsm_poll(30.sec());
    }
}
