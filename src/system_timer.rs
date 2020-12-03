use core::sync::atomic::Ordering;
extern crate hal;
use crate::hal::{
    pac::{interrupt, Interrupt, TIM2},
    prelude::*,
    timer::{CountDownTimer, Event, Timer},
};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

pub type CounterType = u16;

#[path = "atomic_types.rs"]
mod atomic_types;
use atomic_types::HasAtomic;

type TimeType = <CounterType as HasAtomic>::Atomic;

static COUNTER_MS: TimeType = TimeType::new(0);
static _TIM: Mutex<RefCell<Option<CountDownTimer<TIM2>>>> = Mutex::new(RefCell::new(None));
pub struct SystemTimer {}
#[allow(dead_code)]
impl SystemTimer {
    pub fn now() -> CounterType {
        COUNTER_MS.load(Ordering::Relaxed)
    }
    pub fn inc(cnt: CounterType) {
        COUNTER_MS.fetch_add(cnt, Ordering::Relaxed);
    }
    pub fn init(tim: TIM2, clocks: &hal::rcc::Clocks, mut apb1: &mut hal::rcc::APB1) {
        let mut timer = Timer::tim2(tim, &clocks, &mut apb1).start_count_down(1.khz());
        timer.listen(Event::Update);
        cortex_m::interrupt::free(|cs| *_TIM.borrow(cs).borrow_mut() = Some(timer));
        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
        }
        cortex_m::peripheral::NVIC::unpend(Interrupt::TIM2);
    }
}

#[interrupt]
fn TIM2() {
    static mut TIM: Option<CountDownTimer<TIM2>> = None;
    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| _TIM.borrow(cs).replace(None).unwrap())
    });
    SystemTimer::inc(1);
    tim.clear_update_interrupt_flag();
}
