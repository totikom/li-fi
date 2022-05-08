#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use stm32f7xx_hal as hal;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let gpiob = p.GPIOB.split();
    let mut red = gpiob.pb14.into_push_pull_output();
    let mut green = gpiob.pb0.into_push_pull_output();
    let mut blue = gpiob.pb7.into_push_pull_output();

    loop {
        for _ in 0..10_000 {
            green.set_high().expect("gpio can never fail");
        }
        for _ in 0..10_000 {
            blue.set_high().expect("gpio can never fail");
        }
        for _ in 0..10_000 {
            red.set_high().expect("gpio can never fail");
        }
        for _ in 0..10_000 {
            green.set_low().expect("gpio can never fail");
        }
        for _ in 0..10_000 {
            blue.set_low().expect("gpio can never fail");
        }
        for _ in 0..10_000 {
            red.set_low().expect("gpio can never fail");
        }
    }

}
