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
            4.MHz(),
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
        let _ = display.init_color_bar_test(&mut delay);
        delay.delay_ms(1000);
        display.init(&mut delay).unwrap();
        display.clear_screen(0x00).unwrap();

        display.draw_circle(100, 100, 50, 0xFF0000, true).unwrap();
        display.draw_circle(100, 250, 50, 0xFF0000, false).unwrap();
        display.write_text("Circles", 50, 320, None, 0xFFFFFF).unwrap();

        display.draw_ellipse(300, 100, 80, 40, 0x00FF00, true).unwrap();
        display.draw_ellipse(300, 250, 80, 40, 0x00FF00, false).unwrap();
        display.write_text("Ellipses", 260, 320, None, 0xFFFFFF).unwrap();

        display.draw_rounded_rectangle(420, 50, 580, 150, 20, 0x0000FF, true).unwrap();
        display.draw_rounded_rectangle(420, 200, 580, 300, 20, 0x0000FF, false).unwrap();
        display.write_text("Rounded Rects", 440, 320, None, 0xFFFFFF).unwrap();

        display.draw_triangle(700, 50, 650, 150, 750, 150, 0xFFFF00, true).unwrap();
        display.draw_triangle(700, 200, 650, 300, 750, 300, 0xFFFF00, false).unwrap();
        display.write_text("Triangles", 660, 320, None, 0xFFFFFF).unwrap();

        display.draw_circle(900, 150, 60, 0x333333, true).unwrap();
        display.draw_triangle(880, 110, 880, 190, 940, 150, 0x00FF00, true).unwrap();

        for i in 0..8 {
            let x = 50 + i * 60;
            let active = i == 2 || i == 5;
            let color = if active { 0xFF6600 } else { 0x333333 };
            display.draw_circle(x as u16, 460, 20, color, true).unwrap();
            display.draw_circle(x as u16, 460, 20, 0x666666, false).unwrap();
        }

        display.write_text("Buttons:", 550, 400, None, 0xFFFFFF).unwrap();
        display.draw_rounded_rectangle(550, 430, 680, 480, 10, 0x0066FF, true).unwrap();
        display.write_text("SAVE", 590, 448, None, 0xFFFFFF).unwrap();
        display.draw_rounded_rectangle(700, 430, 830, 480, 10, 0x333333, true).unwrap();
        display.draw_rounded_rectangle(700, 430, 830, 480, 10, 0x666666, false).unwrap();
        display.write_text("LOAD", 740, 448, None, 0xFFFFFF).unwrap();
        loop {}
    }
    loop {}
}
