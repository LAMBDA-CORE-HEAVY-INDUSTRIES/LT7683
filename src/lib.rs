#![no_std]
#![deny(unsafe_code)]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[repr(u8)]
pub enum Register {
    // Configuration registers:
    /// Software reset register.
    Srr = 0x00,
    /// Chip configuration register.
    Ccr = 0x01,
    /// Memory access control register.
    Macr = 0x02,
    /// Input Control Register.
    Icr = 0x03,
    /// Memory Data Read/Write Port.
    Mrwdp = 0x04,

    // Pll setting registers:
    /// PCLK PLL Control Register 1.
    Ppllc1 = 0x05,
    /// PCLK PLL Control Register 2.
    Ppllc2 = 0x06,
    /// MCLK PLL Control Register 1.
    Mpllc1 = 0x07,
    /// MCLK PLL Control Register 2.
    Mpllc2 = 0x08,
    /// CCLK PLL Control Register 1.
    Cpllc1 = 0x09,
    /// CCLK PLL Control Register 2.
    Cpllc2 = 0x0A,

    // Interrupt control registers:
    // TODO:

    // LCD display control registers:
    /// Main/PIP Window Control Register.
    Mpwctr = 0x10,
    /// PIP Window Color Depth Setting.
    Pipcdep = 0x11,
    /// Display Configuration Register.
    Dpcr = 0x12,
    /// Panel Scan Clock and Data Setting Register.
    Pcsr = 0x13,
    /// Horizontal Display Width Register.
    Hdwr = 0x14,
    /// Horizontal Display Width Fine Tune Register.
    Hdwftr = 0x15,
    /// Horizontal Non-Display Period Register.
    Hndr = 0x16,
    /// Horizontal Non-Display Period Fine Tune Register.
    Hndftr = 0x17,
    /// HSYNC Start Position Register.
    Hstr = 0x18,
    /// HSYNC Pulse Width Register.
    Hpwr = 0x19,
    /// Vertical Display Height Register 1.
    Vdhr1 = 0x1A,
    /// Vertical Display Height Register 2.
    Vdhr2 = 0x1B,
    /// Vertical Non-Display Period Register 1.
    Vndr1 = 0x1C,
    /// Vertical Non-Display Period Register 2.
    Vndr2 = 0x1D,
    /// VSYNC Start Position Register.
    Vstr = 0x1E,
    /// VSYNC Pulse Width Register.
    Vpwr = 0x1F,
    /// Main Image Start Address 1
    Misa1 = 0x20,
    /// Main Image Start Address 2
    Misa2 = 0x21,
    /// Main Image Start Address 3
    Misa3 = 0x22,
    /// Main Image Start Address 4
    Misa4 = 0x23,
    /// Main Image Width 1
    Miw1 = 0x24,
    /// Main Image Width 2
    Miw2 = 0x25,
    /// Main Window Upper-Left Corner X-Coordinates 1.
    Mwulx1 = 0x26,
    /// Main Window Upper-Left Corner X-Coordinates 2.
    Mwulx2 = 0x27,
    /// Main Window Upper-Left corner Y-Coordinates 1.
    Mwuly1 = 0x28,
    /// Main Window Upper-Left corner Y-Coordinates 2.
    Mwuly2 = 0x29,
    /// PIP Window 1 or 2 Display Upper-Left Corner X-Coordinates 1.
    Pwdulx1 = 0x2A, 
    /// PIP Window 1 or 2 Display Upper-Left Corner X-Coordinates 2.
    Pwdulx2 = 0x2B, 
    /// PIP Window 1 or 2 Display Upper-Left corner Y-Coordinates 1.
    Pwduly1 = 0x2C,
    /// PIP Window 1 or 2 Display Upper-Left corner Y-Coordinates 2.
    Pwduly2 = 0x2D,
    /// PIP Image 1 or 2 Start Address 1.
    Pisa1 = 0x2E,
    /// PIP Image 1 or 2 Start Address 2.
    Pisa2 = 0x2F,
    /// PIP Image 1 or 2 Start Address 3.
    Pisa3 = 0x30,
    /// PIP Image 1 or 2 Start Address 4.
    Pisa4 = 0x31,
    /// PIP Image 1 or 2 Width 1.
    Piw1 = 0x32,
    /// PIP Image 1 or 2 Width 2.
    Piw2 = 0x33,
    /// PIP Window Image 1 or 2 Upper-Left Corner X-Coordinates 1.
    Pwiulx1 = 0x34,
    /// PIP Window Image 1 or 2 Upper-Left Corner X-Coordinates 2.
    Pwiulx2 = 0x35,
    /// PIP Window Image 1 or 2 Upper-Left Corner Y-Coordinates 1.
    Pwiuly1 = 0x36,
    /// PIP Window Image 1 or 2 Upper-Left Corner Y-Coordinates 2.
    Pwiuly2 = 0x37,
    /// PIP Window 1 or 2 Width 1.
    Pww1 = 0x38,
    /// PIP Window 1 or 2 Width 2.
    Pww2 = 0x39,
    /// PIP Window 1 or 2 Height 1.
    Pwh1 = 0x3A,
    /// PIP Window 1 or 2 Height 2.
    Pwh2 = 0x3B,
    /// Graphic / Text Cursor Control Register.
    Gtccr = 0x3C,
    /// Blink Time Control Register.
    Btcr = 0x3D,
    /// Text Cursor Horizontal Size Register.
    Curhs = 0x3E,
    /// Text Cursor Vertical Size Register.
    Curvs = 0x3F, 
    /// Graphic Cursor Horizontal Position Register 1.
    Gchp1 = 0x40,
    /// Graphic Cursor Horizontal Position Register 2.
    Gchp2 = 0x41,
    /// Graphic Cursor Vertical Position Register 1.
    Gcvp1 = 0x42,
    /// Graphic Cursor Vertical Position Register 2.
    Gcvp2 = 0x43,
    /// Graphic Cursor Color 0.
    Gcc0 = 0x44,
    /// Graphic Cursor Color 1.
    Gcc1 = 0x45,

    // Geometric engine control registers:
    // TODO: page 156  https://www.buydisplay.com/download/ic/LT7683.pdf
}

pub struct LT7683<DATA, RS, WR, RD, CS, RES, DELAY> {
    /// DB0 to DB7.
    data: DATA,
    /// aka A0
    /// 1 = data read/write mode.
    /// 0 = status read/command write mode. 
    rs: RS,
    /// active-LOW data write.
    wr: WR,
    /// active-LOW data read.
    rd: RD,
    /// Active-LOW to enable LT7683.
    cs: CS,
    /// Active-LOW hardware reset.
    res: RES,
    delay: DELAY,
}

impl<DATA, RS, WR, RD, CS, RES, DELAY, E> LT7683<DATA, RS, WR, RD, CS, RES, DELAY>
where
    DATA: ParallelBus<Error = E>,
    RS: OutputPin,
    WR: OutputPin,
    RD: OutputPin,
    CS: OutputPin,
    RES: OutputPin,
    DELAY: DelayNs,
{
    pub fn new(data: DATA, rs: RS, wr: WR, rd: RD, cs: CS, res: RES, delay: DELAY) -> Result<Self, E> {
        let mut display = Self {
            data,
            rs,
            wr,
            rd,
            cs,
            res,
            delay,
        };
        display.cs.set_low();
        display.hardware_reset()?;
        Ok(display)
    }

    /// Sets register address before reading/writing data.
    pub fn write_command(&mut self, reg_addr: Register) -> Result<(), E> {
        self.rs.set_low();
        self.rd.set_high();
        self.data.write(reg_addr as u8);
        self.delay.delay_ns(10);
        self.wr.set_low();
        self.delay.delay_ns(150);
        self.wr.set_high();
        Ok(())
    }

    pub fn write_data(&mut self, data: u8) -> Result<(), E> {
        self.rs.set_high();
        self.rd.set_high();
        self.data.write(data);
        self.delay.delay_ns(10);
        self.wr.set_low();
        self.delay.delay_ns(150);
        self.wr.set_high();
        Ok(())
    }

    pub fn read_data(&mut self) -> Result<u8, E> {
        todo!();
    }

    pub fn read_status(&mut self) {
        //TODO: page 131 https://www.buydisplay.com/download/ic/LT7683.pdf
        todo!();
    }

    fn hardware_reset(&mut self) -> Result<(), E> {
        self.res.set_low();
        self.delay.delay_ms(10);
        self.res.set_high();
        self.delay.delay_ms(3);
        Ok(())
    }
}


pub trait ParallelBus {
    type Error;

    fn write(&mut self, value: u8) -> ();
    fn read(&mut self) -> Result<u8, Self::Error>;
    fn set_input(&mut self) -> ();
    fn set_output(&mut self) -> ();
}
