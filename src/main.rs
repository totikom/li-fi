#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use stm32f7xx_hal as hal;

use crate::hal::{pac, prelude::*, delay::Delay};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let gpiob = p.GPIOB.split();

    let mut red = gpiob.pb14.into_push_pull_output();
    let mut green = gpiob.pb0.into_push_pull_output();
    let mut blue = gpiob.pb7.into_push_pull_output();

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    // Create a delay abstraction based on SysTick
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut interval: u32 = 500;
    loop {
            green.set_high();
            delay.delay_ms(interval);

            blue.set_high();
            delay.delay_ms(interval);

            red.set_high();
            delay.delay_ms(interval);
            
            green.set_low();
            delay.delay_ms(interval);

            blue.set_low();
            delay.delay_ms(interval);

            red.set_low();
            delay.delay_ms(interval);

            if interval > 1 {
                interval /= 3;
                interval *= 2;
            } else {
                interval = 500;
            }
    }

}
