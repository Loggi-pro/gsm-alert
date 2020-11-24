#![deny(unsafe_code)]
#![no_std]
#![no_main]

//#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
//extern crate board_support;

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic hnadler
use cortex_m_rt::entry;

extern crate hal;
use hal::{pac}; //mcu select

use hal::{prelude::*,timer::Timer};
//use pac::{TIM6};
use nb::block;
#[allow(unused_imports)]
use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};
use embedded_hal::digital::v2::OutputPin;



#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpio = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpio.pc13.into_push_pull_output(&mut gpio.crh);
    //let mut timer = Timer::syst(cp.SYST,&clocks).start_count_down(1.hz());
    let mut timer = Timer::tim1(dp.TIM1,&clocks,&mut rcc.apb2).start_count_down(1.hz());
    let mut itm = cp.ITM;
    
    iprintln!(&mut itm.stim[0], "hello wordl!");


    // Ждём пока таймер запустит обновление
    // и изменит состояние светодиода.
    loop {
        //timer.start(1.hz());
        block!(timer.wait()).unwrap();
        led.set_high().unwrap();
        //timer.start(1.hz());
        block!(timer.wait()).unwrap();
        led.set_low().unwrap();
    }


}
