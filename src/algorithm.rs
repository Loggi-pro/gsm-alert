use crate::button::Button;
use crate::door_sensor::{DoorSensor, DoorState};
use crate::indication::{Indication, IndicationState};
use crate::sim900::Sim900;
use crate::timer::{CounterTypeExt, Timer};

struct Resources {
    sim900: Sim900,
    indication: Indication,
    power_button: Button,
    door_sensor: DoorSensor,
    check_state: u8,
}

impl Resources {
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
}
pub struct MainLogic {
    resources: Resources,
    current_state: AlgorithmState,
}

fn synchronize<T: Sized>(mut f: impl FnMut() -> Option<T>) -> T {
    loop {
        if let Some(x) = f() {
            return x;
        }
    }
}

enum AlgorithmState {
    IdleState(Idle),
    CheckState(Check),
    ReadyToArmState(ReadyToArm),
    ArmedState(Armed),
    ErrorState(Error),
}

impl MainLogic {
    pub fn new(
        sim900: Sim900,
        indication: Indication,
        power_button: Button,
        door_sensor: DoorSensor,
    ) -> MainLogic {
        MainLogic {
            resources: Resources {
                sim900,
                indication,
                power_button,
                door_sensor,
                check_state: 0,
            },
            current_state: AlgorithmState::IdleState(Idle {}),
        }
    }

    fn update_view(&mut self) {
        let new_view_state = match self.current_state {
            AlgorithmState::IdleState(_) => IndicationState::Idle,
            AlgorithmState::CheckState(_) => IndicationState::CheckBeforeArm,
            AlgorithmState::ReadyToArmState(_) => IndicationState::ReadyToArm,
            AlgorithmState::ArmedState(_) => IndicationState::Armed,
            AlgorithmState::ErrorState(_) => IndicationState::Error,
        };
        self.resources.indication.set_state(new_view_state);
    }

    pub fn init(&mut self) {
        if let AlgorithmState::IdleState(x) = self.current_state {
            self.current_state = x.init(&mut self.resources);
            self.update_view();
        }
    }
    pub fn poll(&mut self) {
        self.resources.indication.poll();
        if let Some(x) = self.current_state.poll(&mut self.resources) {
            self.current_state = x;
            self.update_view();
        }
    }
}

struct Check {}
struct Armed {}
struct ReadyToArm {}
struct Error {
    timer: Timer,
    from_state: IndicationState,
    flag_go_check: bool,
}
#[derive(Copy, Clone)]
struct Idle {}
impl Idle {
    pub fn init(self, resources: &mut Resources) -> AlgorithmState {
        if !synchronize(|| resources.check_gsm()) {
            return AlgorithmState::ErrorState(Error::new(IndicationState::Idle));
        }
        match resources.door_sensor.is_closed() {
            true => AlgorithmState::ArmedState(Armed {}),
            false => AlgorithmState::IdleState(self),
        }
    }
    fn button_poll(&self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(true) = resources.power_button.is_pressed() {
            Some(AlgorithmState::CheckState(Check {}))
        } else {
            None
        }
    }
    fn poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        self.button_poll(resources)
    }
}

impl Check {
    fn button_poll(&self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(true) = resources.power_button.is_pressed() {
            Some(AlgorithmState::IdleState(Idle {}))
        } else {
            None
        }
    }
    fn gsm_poll(&self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(gsm_good) = resources.check_gsm() {
            return match gsm_good {
                true => Some(AlgorithmState::ReadyToArmState(ReadyToArm {})),
                false => Some(AlgorithmState::ErrorState(Error::new(
                    IndicationState::ReadyToArm,
                ))),
            };
        }
        None
    }
    fn poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(x) = self.button_poll(resources) {
            return Some(x);
        }
        if let Some(x) = self.gsm_poll(resources) {
            return Some(x);
        }
        return None;
    }
}
impl Error {
    pub fn new(from_state: IndicationState) -> Self {
        Self {
            timer: Timer::new(),
            from_state,
            flag_go_check: false,
        }
    }
    fn gsm_poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        //check gsm every 30 sec on error
        if self.timer.every(30.sec()) {
            self.flag_go_check = true
        }

        if !self.flag_go_check {
            return None;
        }
        if let Some(gsm_good) = resources.check_gsm() {
            if !gsm_good {
                return None;
            }
            return match self.from_state {
                IndicationState::CheckBeforeArm => Some(AlgorithmState::CheckState(Check {})),
                IndicationState::ReadyToArm => Some(AlgorithmState::ReadyToArmState(ReadyToArm {})),
                IndicationState::Armed => Some(AlgorithmState::ArmedState(Armed {})),
                _ => Some(AlgorithmState::IdleState(Idle {})),
            };
        }
        return None;
    }

    pub fn poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        self.gsm_poll(resources)
    }
}

impl ReadyToArm {
    fn button_poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(true) = resources.power_button.is_pressed() {
            return Some(AlgorithmState::IdleState(Idle {}));
        }
        None
    }
    fn door_poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(DoorState::Closed) = resources.door_sensor.state() {
            //do some actions
            return Some(AlgorithmState::ArmedState(Armed {}));
        }
        None
    }
    fn poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(x) = self.button_poll(resources) {
            return Some(x);
        }
        if let Some(x) = self.door_poll(resources) {
            return Some(x);
        }
        return None;
    }
}

impl Armed {
    fn button_poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(true) = resources.power_button.is_pressed() {
            return Some(AlgorithmState::IdleState(Idle {}));
        }
        None
    }
    fn door_poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(DoorState::Opened) = resources.door_sensor.state() {
            //do some actions
            //smsing
            return Some(AlgorithmState::IdleState(Idle {}));
        }
        None
    }
    fn poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        if let Some(x) = self.button_poll(resources) {
            return Some(x);
        }
        if let Some(x) = self.door_poll(resources) {
            return Some(x);
        }
        return None;
    }
}
impl AlgorithmState {
    fn poll(&mut self, resources: &mut Resources) -> Option<AlgorithmState> {
        match self {
            AlgorithmState::IdleState(x) => x.poll(resources),
            AlgorithmState::CheckState(x) => x.poll(resources),
            AlgorithmState::ReadyToArmState(x) => x.poll(resources),
            AlgorithmState::ArmedState(x) => x.poll(resources),
            AlgorithmState::ErrorState(x) => x.poll(resources),
        }
    }
}
