#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use embedded_hal_bus::spi::ExclusiveDevice;
use lt7683::DisplayConfig;
use panic_halt as _;
use cortex_m_rt::entry;
use stm32f4xx_hal::{self as hal, spi::Spi};
use crate::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.MHz()).freeze();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();

        let sck = gpioa.pa5.into_alternate::<5>();  // SPI1_SCK
        let mosi = gpioa.pa7.into_alternate::<5>(); // SPI1_MOSI / SDO
        let miso = gpioa.pa6.into_alternate::<5>(); // SPI1_MISO / SDI
        let cs = gpioa.pa4.into_push_pull_output(); // SCS
        let res = gpiob.pb0.into_push_pull_output_in_state(hal::gpio::PinState::High);

        let spi_bus = Spi::new(
            dp.SPI1,
            (sck, miso, mosi),
            embedded_hal::spi::MODE_0,
            1.MHz(),
            &clocks,
        );
        let spi_delay = cp.SYST.delay(&clocks);
        let spi_device = ExclusiveDevice::new(spi_bus, cs, spi_delay).unwrap();

        let pb10_pwm = gpiob.pb10.into_alternate::<1>();
        let (_, (_, _, pwm_ch3, _)) = dp.TIM2.pwm_hz(1.kHz(), &clocks);
        let mut pwm_ch3 = pwm_ch3.with(pb10_pwm);
        pwm_ch3.enable();
        let max_duty = pwm_ch3.get_max_duty();
        pwm_ch3.set_duty(max_duty / 2);

        let mut delay = dp.TIM5.delay_us(&clocks);
        let spi_interface = lt7683::SpiInterface { spi: spi_device };
        let display_config = DisplayConfig::new();
        let mut display = lt7683::LT7683::new(spi_interface, res, display_config);
        display.init(&mut delay).unwrap();
        display.clear_screen(0x0000).unwrap();
        display.draw_line(400, 100, 420, 300, 0xAAAA);
        display.draw_line(900, 100, 1000, 200, 0x2222);
        display.draw_line(600, 500, 600, 300, 0xBBBB);
        display.draw_filled_rectangle(500, 500, 800, 300, 0xB2B2);
        display.draw_filled_rectangle(200, 400, 300, 200, 0xFFFF);
        display.draw_filled_rectangle(800, 100, 700, 150, 0x2222);
        loop {}
    }
    loop {}
}
