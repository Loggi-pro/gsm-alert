use crate::hardware::system_timer::{SystemTimer,CounterType};
use crate::hal::{
    pac::{TIM2},
};
pub struct Timer {
    time:CounterType
}
#[allow(dead_code)]
impl Timer {

    pub fn init_system(tim: TIM2, clocks: &hal::rcc::Clocks, apb1: &mut hal::rcc::APB1){
        SystemTimer::init(tim,clocks,apb1);
    }

    pub fn new()->Timer {
        Timer {time:SystemTimer::now()}
    }
    pub fn elapsed(&self)->CounterType {
        return SystemTimer::now()-self.time;
    }
    pub fn every<T:TimeType>(&mut self,time:T)->bool {
        let now = SystemTimer::now();
        
        let diff:CounterType = CounterType::wrapping_sub(now,self.time);
        if diff >=time.value() {
            self.time = now;
            return true
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
    fn sec(self)->Seconds {
        Seconds(self)
    }
}


pub trait TimeType {
    fn value(&self)->CounterType;
}

impl TimeType for MilliSeconds {
    fn value(&self)->CounterType {
        self.0
    }
}

impl TimeType for Seconds {
    fn value(&self)->CounterType {
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
        Self(val.0  / 1_000)
    }
}
