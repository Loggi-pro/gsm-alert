extern crate hal;
use crate::hal::{
    gpio::{gpioa::PA10, gpioa::PA9},
    gpio::{Alternate, Floating, Input, PushPull},
    pac::{usart1, Interrupt, USART1},
    serial::{Rx, Serial, Tx},
};

use core::ptr;
use core::sync::atomic::{self, Ordering};
const MAX_SIZE: usize = 50;

type Usart1 = Serial<USART1, (PA9<Alternate<PushPull>>, PA10<Input<Floating>>)>;
pub struct UsartAdapter {
    flag_ready: core::sync::atomic::AtomicBool,
    tx: Tx<USART1>,
    rx: Rx<USART1>,
    tx_channel: hal::dma::dma1::C4,
    rx_channel: hal::dma::dma1::C5,
    rx_buf: [u8; MAX_SIZE],
}
#[allow(dead_code)]
impl UsartAdapter {
    fn get_hw() -> &'static mut usart1::RegisterBlock {
        unsafe { &mut *(USART1::ptr() as *mut _) }
    }
    pub fn new(serial: Usart1, channels: hal::dma::dma1::Channels) -> Self {
        unsafe { cortex_m::peripheral::NVIC::unmask(Interrupt::USART1) };
        cortex_m::peripheral::NVIC::unpend(Interrupt::USART1);
        let uart = UsartAdapter::get_hw();
        //enable interrupt on line idle
        uart.cr1.modify(|_, w| w.idleie().set_bit());
        //create dma channels
        let (tx, rx) = serial.split();
        let (tx_channel, rx_channel) = (channels.4, channels.5);
        // DMA channel selection depends on the peripheral:
        // - USART1: TX = 4, RX = 5
        // - USART2: TX = 6, RX = 7
        // - USART3: TX = 3, RX = 2

        UsartAdapter {
            flag_ready: core::sync::atomic::AtomicBool::new(false),
            tx: tx,
            rx: rx,
            tx_channel: tx_channel,
            rx_channel: rx_channel,
            rx_buf: [0; MAX_SIZE],
        }
    }

    pub fn write_data(&mut self, arr: &[u8]) {
        //start separate DMAs for sending and receiving the data
        self.tx_channel
            // .set_peripheral_address(unsafe { &(*USART1::ptr()).dr as *const _ as u32 }, false);
            .set_peripheral_address(&(UsartAdapter::get_hw().dr) as *const _ as u32, false);
        let (ptr, len) = (arr.as_ptr(), arr.len());
        self.tx_channel.set_memory_address(ptr as u32, true);
        self.tx_channel.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);

        self.tx_channel.ch().cr.modify(|_, w| {
            w.mem2mem()
                .clear_bit()
                .pl()
                .medium()
                .msize()
                .bits8()
                .psize()
                .bits8()
                .circ()
                .clear_bit()
                .dir()
                .set_bit()
        });
        self.tx_channel.start();
        //block until all data was transmitted and received
        while self.tx_channel.in_progress() {}
        //stop
        atomic::compiler_fence(Ordering::Acquire);
        self.tx_channel.ifcr().write(|w| w.cgif4().set_bit()); // C4 channel
        self.tx_channel.ch().cr.modify(|_, w| w.en().clear_bit());

        unsafe {
            ptr::read_volatile(&0);
        }
        atomic::compiler_fence(Ordering::Acquire);
    }

    pub fn isr_handler(&mut self) {
        let uart = UsartAdapter::get_hw();
        //sequence to clear flag
        let _a = uart.sr.read();
        let _data = uart.dr.read();
        self.flag_ready.store(true, Ordering::Relaxed)
    }
}
