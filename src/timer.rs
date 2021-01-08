use crate::hal::pac::TIM2;
use crate::hardware::system_timer::{CounterType, SystemTimer};
use core::sync::atomic::{self, Ordering};
pub struct Timer {
    time: CounterType,
}
#[allow(dead_code)]
impl Timer {
    pub fn init_system(tim: TIM2, clocks: &hal::rcc::Clocks, apb1: &mut hal::rcc::APB1) {
        SystemTimer::init(tim, clocks, apb1);
    }

    pub fn new() -> Timer {
        Timer {
            time: SystemTimer::now(),
        }
    }
    pub fn elapsed(&self) -> CounterType {
        return SystemTimer::now() - self.time;
    }

    pub fn reset(&mut self) {
        self.time = SystemTimer::now();
    }
    pub fn wait<T: TimeType>(&mut self, time: T) {
        self.reset();
        while self.waiting(&time) {
            atomic::compiler_fence(Ordering::SeqCst);
            continue;
        }
    }

    pub fn waiting<T: TimeType>(&mut self, time: &T) -> bool {
        let diff: CounterType = CounterType::wrapping_sub(SystemTimer::now(), self.time);
        return if diff < time.value() { true } else { false };
    }

    pub fn every<T: TimeType>(&mut self, time: T) -> bool {
        let now = SystemTimer::now();

        let diff: CounterType = CounterType::wrapping_sub(now, self.time);
        if diff >= time.value() {
            self.time = now;
            return true;
        }
        false
    }
}
/// Time unit
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct MilliSeconds(pub CounterType);

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Seconds(pub CounterType);

pub trait CounterTypeExt {
    fn mil(self) -> MilliSeconds;
    fn sec(self) -> Seconds;
}

impl CounterTypeExt for CounterType {
    fn mil(self) -> MilliSeconds {
        MilliSeconds(self)
    }
    fn sec(self) -> Seconds {
        Seconds(self)
    }
}

pub trait TimeType {
    fn value(&self) -> CounterType;
}

impl TimeType for MilliSeconds {
    fn value(&self) -> CounterType {
        self.0
    }
}

impl TimeType for Seconds {
    fn value(&self) -> CounterType {
        let ms = MilliSeconds::from(*self);
        ms.0
    }
}

impl From<Seconds> for MilliSeconds {
    fn from(val: Seconds) -> Self {
        Self(val.0 * 1_000)
    }
}

impl From<MilliSeconds> for Seconds {
    fn from(val: MilliSeconds) -> Self {
        Self(val.0 / 1_000)
    }
}

use core::ops::Add;
impl Add for Seconds {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Add for MilliSeconds {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
