#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

#![allow(unused_variables)]
#![allow(dead_code)]

use panic_rtt_target as _;

//use stm32f3_discovery::{
    //button,
    //button::interrupt::TriggerMode,
    //leds::Leds,
    //stm32f3xx_hal::{prelude::*, pac},
    //switch_hal::ToggleableOutputSwitch,
//};
use stm32f3xx_hal::{self as hal, pac, prelude::*, delay::Delay, adc, adc::Adc};

use core::fmt::Write;
use cortex_m_rt::entry;
use micromath::F32Ext;
use reed_solomon::Decoder;
use reed_solomon::Encoder;
use rtt_target::rtt_init;
use tinyvec::ArrayVec;

const SAMPLE_COUNT: usize = 1000;
const REPEAT: usize = 1000;
const MESSAGE: [u8; 11] = *b"Hello, led!";
const INTERVALS: [u32; 16] = [
    1, 10, 100, 200, 300, 400, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000,
];
const ECC_LENGTHS: [usize; 6] = [4, 8, 16, 32, 64, 128];

#[entry]
fn main() -> ! {
    let channels = rtt_init! {
        up: {
            0: {
                size: 1024
                    name: "Terminal"
            }
        }
    };
    let mut channel = channels.up.0;
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();

    // Set up the system clock. We want to run at 48MHz for this one.
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze(&mut flash.acr);

    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);

    let mut led = gpioe
            .pe13
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);


    // Create a delay abstraction based on SysTick
    let mut delay = Delay::new(cp.SYST, clocks);

    let adc = p.ADC1;
    let common_adc = adc::CommonAdc::new(p.ADC1_2, &clocks, &mut rcc.ahb);

    let mut adc = Adc::new(adc, adc::config::Config::default(), &clocks, &common_adc);

    // Set up pin PA0 as analog pin.
    // This pin is connected to the user button on the stm32f3discovery board.
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut analog_pin = gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);


    write!(
        &mut channel,
        "Hello!\n",
    )
    .unwrap();

    loop {
        led.toggle();
        delay.delay_ms(1000u32);
        let adc_data: u16 = adc.read(&mut analog_pin).unwrap();
    write!(
        &mut channel,
        "ADC reads {}\n",
        adc_data,
    )
    .unwrap();


    }
}
