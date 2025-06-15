#![no_std]
#![deny(unsafe_code)]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

pub struct LT7683<DATA, RS, WR, RD, CS, RES, DELAY> {
    /// DB0 to DB7.
    data: DATA,
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
