#![no_std]
#![deny(unsafe_code)]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

pub trait LT7683Interface {
    type Error;
    fn write_command(&mut self, register: Register) -> Result<(), Self::Error>;
    fn write_data(&mut self, data: u8) -> Result<(), Self::Error>;
    fn read_data(&mut self) -> Result<u8, Self::Error>;
    fn read_status(&mut self) -> Result<u8, Self::Error>;
}

pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub h_back_porch: u16,
    pub h_front_porch: u16,
    pub h_sync_width: u16,
    pub v_back_porch: u16,
    pub v_front_porch: u16,
    pub v_sync_width: u16,
    pub color_depth: ColorDepth,
}

impl Default for DisplayConfig {
    /// Default timing for 7" 1024x600 display (ER-TFT070A2-6-5633).
    fn default() -> Self {
        Self {
            width: 1024,
            height: 600,
            h_back_porch: 160,
            h_front_porch: 160,
            h_sync_width: 70,
            v_back_porch: 23,
            v_front_porch: 12,
            v_sync_width: 10,
            color_depth: ColorDepth::Bpp16,
        }
    }
}

impl DisplayConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct LT7683<I: LT7683Interface, RESET> {
    pub spi_interface: I,
    pub res: RESET,
    pub config: DisplayConfig,
}

impl<I: LT7683Interface, RESET: OutputPin> LT7683<I, RESET> {
    pub fn new(spi_interface: I, res: RESET, display_config: DisplayConfig) -> Self {
        Self { spi_interface, res, config: display_config}
    }

    pub fn write_register(&mut self, register: Register, data: u8) -> Result<(), I::Error> {
        self.spi_interface.write_command(register)?;
        self.spi_interface.write_data(data)?;
        Ok(())
    }

    pub fn read_register(&mut self, register: Register) -> Result<u8, I::Error> {
        self.spi_interface.write_command(register)?;
        self.spi_interface.read_data()
    }

    /// Read the status register.
    /// Bit 7: Memory read/write busy
    /// Bit 6: BTE busy
    /// Bit 5: DMA busy
    /// Bit 4: Text engine busy
    /// Bit 3: Core busy
    /// Bit 2: SDRAM ready (1 = ready)
    /// Bit 1: reserved
    /// Bit 0: Operation mode status
    pub fn read_status(&mut self) -> Result<u8, I::Error> {
        self.spi_interface.read_status()
    }

    pub fn is_sdram_ready(&mut self) -> Result<bool, I::Error> {
        let status = self.read_status()?;
        Ok((status & 0x04) != 0) // Bit 2 = SDRAM ready
    }

    pub fn wait_sdram_ready<D: DelayNs>(&mut self, delay: &mut D) -> Result<bool, I::Error> {
        for _ in 0..100 {
            if self.is_sdram_ready()? {
                return Ok(true);
            }
            delay.delay_ms(10);
        }
        Ok(false) // Timeout
    }

    pub fn hardware_reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        let _ = self.res.set_low();
        delay.delay_ms(10);
        let _ = self.res.set_high();
        delay.delay_ms(100); // Wait for chip to boot
        Ok(())
    }

    /// Software reset and wait for completion.
    pub fn software_reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.write_register(Register::Srr, 0x01)?;
        // Wait for reset to complete (bit 0 clears when done)
        for _ in 0..100 {
            let val = self.read_register(Register::Srr)?;
            if (val & 0x01) == 0 {
                return Ok(());
            }
            delay.delay_ms(1);
        }
        Ok(())
    }

    pub fn init_color_bar_test<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.hardware_reset(delay)?;
        self.software_reset(delay)?;
        self.configure_pll(delay)?;
        // TFT 24-bit output, SPI flash enabled, 8-bit host bus
        self.write_register(Register::Ccr, 0x80)?;
        self.configure_display_timing()?;
        // HSYNC high active, VSYNC high active, DE high active
        self.write_register(Register::Pcsr, 0xC0)?;
        // Display on with color bar
        self.write_register(Register::Dpcr, 0xE0)?;
        Ok(())
    }

    pub fn init<D: DelayNs>(&mut self, delay: &mut D, timing: &DisplayConfig) -> Result<(), I::Error> {

        self.hardware_reset(delay)?;
        self.software_reset(delay)?;
        self.configure_pll(delay)?;
        self.configure_sdram(delay)?;
        self.wait_sdram_ready(delay)?;
        // TFT 24-bit output, SPI flash enabled, 8-bit host bus
        self.write_register(Register::Ccr, 0x80)?;
        // 8-bit host, 16bpp
        self.write_register(Register::Macr, 0x40)?;
        // Graphic mode, SDRAM memory
        self.write_register(Register::Icr, 0x00)?;
        self.configure_display_timing()?;
        // HSYNC high active, VSYNC high active, DE high active
        self.write_register(Register::Pcsr, 0xC0)?;
        self.configure_main_window(&timing)?;
        // Display on
        self.write_register(Register::Dpcr, 0xC0)?;
        Ok(())

    }

    fn configure_pll<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.write_register(Register::Ppllc1, 0x06)?;
        self.write_register(Register::Ppllc2, 39)?;

        self.write_register(Register::Mpllc1, 0x04)?;
        self.write_register(Register::Mpllc2, 39)?;

        self.write_register(Register::Cpllc1, 0x04)?;
        self.write_register(Register::Cpllc2, 39)?;

        // Activate PLLs
        self.write_register(Register::Ccr, 0x00)?;
        delay.delay_us(10);
        self.write_register(Register::Ccr, 0x80)?;
        delay.delay_ms(1);
        Ok(())
    }

    fn configure_sdram<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.write_register(Register::Sdrar, 0x29)?;
        // CAS latency 3
        self.write_register(Register::Sdrmd, 0x03)?;

        // Refresh interval for 100MHz SDRAM clock
        // Formula: ((64000000 / 8192) / (1000 / DRAM_FREQ_MHZ)) - 2
        // For 100MHz: (64000000 / 8192) / 10 - 2 = 779 = 0x030B
        // TODO: check if this makes sense
        let sdram_itv: u16 = 779;
        self.write_register(Register::SdrRef1, sdram_itv as u8)?;
        self.write_register(Register::SdrRef2, (sdram_itv >> 8) as u8)?;
        // Start SDRAM initialization
        self.write_register(Register::Sdrcr, 0x01)?;
        delay.delay_ms(10);
        Ok(())
    }

    fn configure_display_timing(&mut self) -> Result<(), I::Error> {
        // Horizontal display width
        self.write_register(Register::Hdwr, ((self.config.width / 8) - 1) as u8)?;
        self.write_register(Register::Hdwftr, (self.config.width % 8) as u8)?;
        // Horizontal non-display period (back porch)
        self.write_register(Register::Hndr, ((self.config.h_back_porch / 8) - 1) as u8)?;
        self.write_register(Register::Hndftr, (self.config.h_back_porch % 8) as u8)?;
        // HSYNC start position (front porch - from end of display to start of HSYNC)
        self.write_register(Register::Hstr, ((self.config.h_front_porch / 8) - 1) as u8)?;
        // HSYNC pulse width
        self.write_register(Register::Hpwr, ((self.config.h_sync_width / 8) - 1) as u8)?;
        // Vertical display height
        let height_minus_1 = self.config.height - 1;
        self.write_register(Register::Vdhr1, (height_minus_1 & 0xFF) as u8)?;
        self.write_register(Register::Vdhr2, ((height_minus_1 >> 8) & 0xFF) as u8)?;
        // Vertical non-display period (back porch)
        self.write_register(Register::Vndr1, (self.config.v_back_porch - 1) as u8)?;
        self.write_register(Register::Vndr2, 0x00)?;
        // VSYNC start position (front porch - from end of display to start of VSYNC)
        self.write_register(Register::Vstr, (self.config.v_front_porch - 1) as u8)?;
        // VSYNC pulse width
        self.write_register(Register::Vpwr, (self.config.v_sync_width - 1) as u8)?;
        Ok(())
    }

    fn configure_main_window(&mut self, timing: &DisplayConfig) -> Result<(), I::Error> {
        // Main/PIP window control: 16bpp color depth
        self.write_register(Register::Mpwctr, 0x04)?;
        // Main image start address = 0
        self.write_register(Register::Misa1, 0x00)?;
        self.write_register(Register::Misa2, 0x00)?;
        self.write_register(Register::Misa3, 0x00)?;
        self.write_register(Register::Misa4, 0x00)?;
        // Main image width in bytes (width * 2 for 16bpp)
        let width_bytes = timing.width * 2;
        self.write_register(Register::Miw1, (width_bytes & 0xFF) as u8)?;
        self.write_register(Register::Miw2, ((width_bytes >> 8) & 0xFF) as u8)?;
        // Main window upper-left corner at (0, 0)
        self.write_register(Register::Mwulx1, 0x00)?;
        self.write_register(Register::Mwulx2, 0x00)?;
        self.write_register(Register::Mwuly1, 0x00)?;
        self.write_register(Register::Mwuly2, 0x00)?;
        // Canvas start address
        self.write_register(Register::Cvssa1, 0x00)?;
        self.write_register(Register::Cvssa2, 0x00)?;
        self.write_register(Register::Cvssa3, 0x00)?;
        self.write_register(Register::Cvssa4, 0x00)?;
        // Canvas image width
        self.write_register(Register::CvsImwth1, (width_bytes & 0xFF) as u8)?;
        self.write_register(Register::CvsImwth2, ((width_bytes >> 8) & 0xFF) as u8)?;
        // Active window
        self.set_active_window(0, 0, timing.width, timing.height)?;
        // Active window color depth: 16bpp
        self.write_register(Register::AwColor, 0x01)?;
        Ok(())
    }

   pub fn set_active_window(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<(), I::Error> {
       self.write_register(Register::AwulX1, x as u8)?;
       self.write_register(Register::AwulX2, (x >> 8) as u8)?;
       self.write_register(Register::AwulY1, y as u8)?;
       self.write_register(Register::AwulY2, (y >> 8) as u8)?;

       self.write_register(Register::AwWth1, width as u8)?;
       self.write_register(Register::AwWth2, (width >> 8) as u8)?;
       self.write_register(Register::AwHt1, height as u8)?;
       self.write_register(Register::AwHt2, (height >> 8) as u8)?;
       Ok(())
   }

   pub fn set_foreground_color(&mut self, color: u16) -> Result<(), I::Error> {
        match self.config.color_depth {
            ColorDepth::Bpp16 => {
                // RGB565.
                let r = ((color >> 11) & 0x1F) << 3;
                let g = ((color >> 5) & 0x3F) << 2;
                let b = ((color >> 0) & 0x1F) << 3;
                self.write_register(Register::Fgcr, r as u8)?;
                self.write_register(Register::Fgcg, g as u8)?;
                self.write_register(Register::Fgcb, b as u8)?;

            }
            _ => {
                self.write_register(Register::Fgcr, (color >> 8) as u8)?;
                self.write_register(Register::Fgcg, color as u8)?;
                self.write_register(Register::Fgcb, 0)?;
            }
        }
       Ok(())
   }
       Ok(())
   }

   pub fn draw_filled_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) -> Result<(), I::Error> {
       self.set_foreground_color(color)?;
       self.write_register(Register::Dlhsr1, x1 as u8)?;
       self.write_register(Register::Dlhsr2, (x1 >> 8) as u8)?;
       self.write_register(Register::Dlvsr1, y1 as u8)?;
       self.write_register(Register::Dlvsr2, (y1 >> 8) as u8)?;
       self.write_register(Register::Dlher1, x2 as u8)?;
       self.write_register(Register::Dlher2, (x2 >> 8) as u8)?;
       self.write_register(Register::Dlver1, y2 as u8)?;
       self.write_register(Register::Dlver2, (y2 >> 8) as u8)?;
       self.write_register(Register::Dcr0, 0xB0)?;
       // self.delay.delay_ms(1);
       Ok(())
   }

   pub fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) -> Result<(), I::Error> {
       self.set_foreground_color(color)?;
       self.write_register(Register::Dlhsr1, x1 as u8)?;
       self.write_register(Register::Dlhsr2, (x1 >> 8) as u8)?;
       self.write_register(Register::Dlvsr1, y1 as u8)?;
       self.write_register(Register::Dlvsr2, (y1 >> 8) as u8)?;
       self.write_register(Register::Dlher1, x2 as u8)?;
       self.write_register(Register::Dlher2, (x2 >> 8) as u8)?;
       self.write_register(Register::Dlver1, y2 as u8)?;
       self.write_register(Register::Dlver2, (y2 >> 8) as u8)?;
       self.write_register(Register::Dcr0, 0x80)?;
       // self.delay.delay_ms(1);
       Ok(())
   }

   /// Clear entire screen with color.
   pub fn clear_screen(&mut self, color: u16) -> Result<(), I::Error> {
       self.draw_filled_rectangle(0, 0, self.config.width - 1, self.config.height - 1, color)
   }
}

/// Parallel 8-bit interface
pub struct ParallelInterface<DATA, RS, WR, RD, CS> {
    pub data: DATA,
    pub rs: RS,
    pub wr: WR,
    pub rd: RD,
    pub cs: CS,
}

impl<DATA, RS, WR, RD, CS, E> LT7683Interface for ParallelInterface<DATA, RS, WR, RD, CS>
where
    DATA: ParallelBus<Error = E>,
    RS: OutputPin,
    WR: OutputPin,
    RD: OutputPin,
    CS: OutputPin,
{
    type Error = E;

    fn write_command(&mut self, register: Register) -> Result<(), Self::Error> {
        let _ = self.rs.set_low();
        let _ = self.rd.set_high();
        self.data.write(register as u8);
        let _ = self.wr.set_low();
        let _ = self.wr.set_high();
        Ok(())
    }

    fn write_data(&mut self, data: u8) -> Result<(), Self::Error> {
        let _ = self.rs.set_high();
        let _ = self.rd.set_high();
        self.data.write(data);
        let _ = self.wr.set_low();
        let _ = self.wr.set_high();
        Ok(())
    }

    fn read_data(&mut self) -> Result<u8, E> {
        let _ = self.rs.set_high();
        let _ = self.wr.set_high();
        self.data.set_input();
        let _ = self.rd.set_low();
        let result = self.data.read()?;
        let _ = self.rd.set_high();
        self.data.set_output();
        Ok(result)
    }

    fn read_status(&mut self) -> Result<u8, E> {
        let _ = self.rs.set_low();
        let _ = self.wr.set_high();
        self.data.set_input();
        let _ = self.rd.set_low();
        let result = self.data.read()?;
        let _ = self.rd.set_high();
        self.data.set_output();
        Ok(result)
    }
}

pub struct SpiInterface<SPI> {
    pub spi: SPI,
}

impl<SPI, E> LT7683Interface for SpiInterface<SPI>
where
    SPI: SpiDevice<Error = E>,
{
    type Error = E;

    fn write_command(&mut self, register: Register) -> Result<(), Self::Error> {
        let cmd_bytes = [0x00, register as u8];
        self.spi.write(&cmd_bytes)?;
        Ok(())
    }

    fn write_data(&mut self, data: u8) -> Result<(), Self::Error> {
        let data_bytes = [0x80, data];
        self.spi.write(&data_bytes)?;
        Ok(())
    }

    fn read_data(&mut self) -> Result<u8, E> {
        let mut buf = [0xC0, 0x00];
        self.spi.transfer_in_place(&mut buf)?;
        Ok(buf[1])
    }

    fn read_status(&mut self) -> Result<u8, E> {
        let mut buf = [0x40, 0x00];
        self.spi.transfer_in_place(&mut buf)?;
        Ok(buf[1])
    }
}

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
    /// Canvas Start Address 1.
    Cvssa1 = 0x50,
    /// Canvas Start Address 2.
    Cvssa2 = 0x51,
    /// Canvas Start Address 3.
    Cvssa3 = 0x52,
    /// Canvas Start Address 4.
    Cvssa4 = 0x53,
    /// Canvas Image Width 1.
    CvsImwth1 = 0x54,
    /// Canvas Image Width 2.
    CvsImwth2 = 0x55,
    /// Active Window Upper-Left Corner X-Coordinates 1.
    AwulX1 = 0x56,
    /// Active Window Upper-Left Corner X-Coordinates 2.
    AwulX2 = 0x57,
    /// Active Window Upper-Left Corner Y-Coordinates 1.
    AwulY1 = 0x58,
    /// Active Window Upper-Left Corner Y-Coordinates 2.
    AwulY2 = 0x59,
    /// Active Window Width 1.
    AwWth1 = 0x5A,
    /// Active Window Width 2.
    AwWth2 = 0x5B,
    /// Active Window Height 1.
    AwHt1 = 0x5C,
    /// Active Window Height 2.
    AwHt2 = 0x5D,
    /// Color Depth of Canvas & Active Window.
    AwColor = 0x5E,
    /// Graphic Read/Write X-Coordinate Register 1.
    Curh1 = 0x5F,
    /// Graphic Read/Write X-Coordinate Register 2.
    Curh2 = 0x60,
    /// Graphic Read/Write Y-Coordinate Register 1.
    Curv1 = 0x61,
    /// Graphic Read/Write Y-Coordinate Register 2.
    Curv2 = 0x62,
    /// Text Write X-Coordinates Register 1.
    FCurx1 = 0x63,
    /// Text Write X-Coordinates Register 2.
    FCurx2 = 0x64,
    /// Text Write Y-Coordinates Register 1.
    FCury1 = 0x65,
    /// Text Write Y-Coordinates Register 2.
    FCury2 = 0x66,
    /// Draw Line/Triangle Control Register 0
    Dcr0 = 0x67,
    /// Draw Line/Rectangle/Triangle Point 1 X-Coordinates Register 1.
    Dlhsr1 = 0x68,
    /// Draw Line/Rectangle/Triangle Point 1 X-Coordinates Register 2.
    Dlhsr2 = 0x69,
    /// Draw Line/Rectangle/Triangle Point 1 Y-Coordinates Register 1.
    Dlvsr1 = 0x6A,
    /// Draw Line/Rectangle/Triangle Point 1 Y-Coordinates Register 2.
    Dlvsr2 = 0x6B,
    /// Draw Line/Rectangle/Triangle Point 2 X-Coordinates Register 1.
    Dlher1 = 0x6C,
    /// Draw Line/Rectangle/Triangle Point 2 X-Coordinates Register 2.
    Dlher2 = 0x6D,
    /// Draw Line/Rectangle/Triangle Point 2 Y-Coordinates Register 1.
    Dlver1 = 0x6E,
    /// Draw Line/Rectangle/Triangle Point 2 Y-Coordinates Register 2.
    Dlver2 = 0x6F,
    /// Draw Triangle Point 3 X-Coordinates Register 1.
    Dtph1 = 0x70,
    /// Draw Triangle Point 3 X-Coordinates Register 2.
    Dtph2 = 0x71,
    /// Draw Triangle Point 3 Y-Coordinates Register 1.
    Dtpv1 = 0x72,
    /// Draw Triangle Point 3 Y-Coordinates Register 2.
    Dtpv2 = 0x73,
    /// Draw Circle/Ellipse/Ellipse Curve/Circle Square Control Register 1.
    Dcr1 = 0x76,
    ///  Draw Circle/Ellipse/Rounded-Rectangle Major-Radius Register 1.
    EllA1 = 0x77,
    ///  Draw Circle/Ellipse/Rounded-Rectangle Major-Radius Register 2.
    EllA2 = 0x78,
    /// Draw Circle/Ellipse/Rounded-rectangle Minor-Radius Register 1.
    EllB1 = 0x79,
    /// Draw Circle/Ellipse/Rounded-rectangle Minor-Radius Register 2.
    EllB2 = 0x7A,
    /// Draw Circle/Ellipse/Rounded-Rectangle Center X-Coordinates Register 1.
    Dehr1 = 0x7B,
    /// Draw Circle/Ellipse/Rounded-Rectangle Center X-Coordinates Register 2.
    Dehr2 = 0x7C,
    /// Draw Circle/Ellipse/Rounded-Rectangle Center Y-Coordinates Register 1.
    Devr1 = 0x7D,
    /// Draw Circle/Ellipse/Rounded-Rectangle Center Y-Coordinates Register 2.
    Devr2 = 0x7E,
    /// Foreground Color Register - Red.
    Fgcr = 0xD2,
    /// Foreground Color Register - Green.
    Fgcg = 0xD3,
    /// Foreground Color Register - Blue.
    Fgcb = 0xD4,

    // PWM control registers:
    // TODO: page 165 https://www.buydisplay.com/download/ic/LT7683.pdf

    // Bit block transfer engine (BTE) control registers:
    // TODO: page 168 https://www.buydisplay.com/download/ic/LT7683.pdf

    // Serial Flash & SPI Master Control Registers:
    // TODO: page 176 https://www.buydisplay.com/download/ic/LT7683.pdf

    // Text engine registers:
    /// Character Control Register 0.
    Ccr0 = 0xCC,
    /// Character Control Register 1.
    Ccr1 = 0xCD,
    /// Character Line gap Setting Register.
    Fldr = 0xD0,
    /// Character to Character Space Setting Register.
    F2fssr = 0xD1,
    /// Background Color Register - Red.
    Bgcr = 0xD5,
    /// Background Color Register - Green.
    Bgcg = 0xD6,
    /// Background Color Register - Blue.
    Bgcb = 0xD7,
    /// CGRAM Start Address 0.
    CgramStr0 = 0xDB,

    // Power Management Control Register:
    /// Power Management Register
    Pmu = 0xDF,

    // Display RAM Control Register:
    /// SDRAM Attribute Register.
    Sdrar = 0xE0,
    /// SDRAM Mode Register & Extended Mode Register.
    Sdrmd = 0xE1,
    /// SDRAM Auto Refresh Interval 1.
    SdrRef1 = 0xE2,
    /// SDRAM Auto Refresh Interval 2.
    SdrRef2 = 0xE3,
    /// SDRAM Control Register.
    Sdrcr = 0xE4,

    // GPIO Register:
    /// GPIO-A Direction.
    Gpioad = 0xF0,
    /// GPIO-A.
    Gpioa = 0xF1,
    /// GPIO-B.
    Gpiob = 0xF2,
    /// GPIO-C Direction.
    Gpiocd = 0xF3,
    /// GPIO-C.
    Gpioc = 0xF4,
    /// GPIO-D Direction.
    Gpiodd = 0xF5,
    /// GPIO-D.
    Gpiod = 0xF6,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorDepth {
    /// 8-bit color (RGB 3:3:2).
    Bpp8 = 0x00,
    /// 16-bit color (RGB 5:6:5).
    Bpp16 = 0x01,
    /// 24-bit color (RGB 8:8:8).
    Bpp24 = 0x02,
}

//pub struct LT7683<DATA, RS, WR, RD, CS, RES, DELAY> {
//    /// DB0 to DB7.
//    data: DATA,
//    /// aka A0
//    /// 1 = data read/write mode.
//    /// 0 = status read/command write mode. 
//    rs: RS,
//    /// active-LOW data write.
//    wr: WR,
//    /// active-LOW data read.
//    rd: RD,
//    /// Active-LOW to enable LT7683.
//    cs: CS,
//    /// Active-LOW hardware reset.
//    res: RES,
//    delay: DELAY,
//    config: DisplayConfig,
//}

//impl<DATA, RS, WR, RD, CS, RES, DELAY, E> LT7683<DATA, RS, WR, RD, CS, RES, DELAY>
//where
//    DATA: ParallelBus<Error = E>,
//    RS: OutputPin,
//    WR: OutputPin,
//    RD: OutputPin,
//    CS: OutputPin,
//    RES: OutputPin,
//    DELAY: DelayNs,
//{
//    pub fn new(data: DATA, rs: RS, wr: WR, rd: RD, cs: CS, res: RES, delay: DELAY, config: DisplayConfig) -> Result<Self, E> {
//        let mut display = Self {
//            data, rs, wr, rd, cs, res, delay, config,
//        };
//        display.cs.set_low();
//        display.hardware_reset()?;
//        display.init_display()?;
//        Ok(display)
//    }

//    pub fn init_display(&mut self) -> Result<(), E> {
//        // self.write_register(Register::Dpcr, 0x05)?; // test color bar
//        self.write_register(Register::Srr, 0x01)?;
//        self.delay.delay_ms(10);

//        // 8-bit parallel interface mode.
//        // self.write_register(Register::Ccr, 0x00)?;

//        // self.configure_pll()?;
//        // self.configure_display_timing()?;
//        // self.configure_memory()?;
//        // self.set_active_window(0, 0, self.config.width, self.config.height)?;
//        // self.clear_screen(0x0000)?;
//        Ok(())
//    }

//    fn configure_pll(&mut self) -> Result<(), E> {
//        // Pixel clock.
//        self.write_register(Register::Ppllc1, 0x07)?;
//        self.write_register(Register::Ppllc2, 0x03)?;
//        // Memory clock.
//        self.write_register(Register::Mpllc1, 0x05)?;
//        self.write_register(Register::Mpllc2, 0x03)?;
//        // Core clock.
//        self.write_register(Register::Cpllc1, 0x03)?;
//        self.write_register(Register::Cpllc2, 0x02)?;
//        self.delay.delay_ms(10);
//        Ok(())
//    }

//    fn configure_display_timing(&mut self) -> Result<(), E> {
//        let width = self.config.width;
//        let height = self.config.height;
//        self.write_register(Register::Hdwr, ((width / 8) - 1) as u8)?;
//        self.write_register(Register::Hdwftr, (width % 8) as u8)?;
//        self.write_register(Register::Hndr, 0x05)?;
//        self.write_register(Register::Hndftr, 0x00)?;
//        self.write_register(Register::Hstr, 0x01)?;
//        self.write_register(Register::Hpwr, 0x03)?;

//        self.write_register(Register::Vdhr1, (height - 1) as u8)?;
//        self.write_register(Register::Vdhr2, ((height - 1) >> 8) as u8)?;
//        self.write_register(Register::Vndr1, 0x12)?;
//        self.write_register(Register::Vndr2, 0x00)?;
//        self.write_register(Register::Vstr, 0x01)?;
//        self.write_register(Register::Vpwr, 0x05)?;

//        self.write_register(Register::Dpcr, 0x00)?; // Display on
//        Ok(())
//    }

//    fn configure_memory(&mut self) -> Result<(), E> {
//        self.write_register(Register::AwColor, self.config.color_depth as u8)?;
//        self.write_register(Register::Misa1, 0x00)?;
//        self.write_register(Register::Misa2, 0x00)?;
//        self.write_register(Register::Misa3, 0x00)?;
//        self.write_register(Register::Misa4, 0x00)?;
//        let width_bytes = self.config.width * (self.config.color_depth as u16 + 1);
//        self.write_register(Register::Miw1, width_bytes as u8)?;
//        self.write_register(Register::Miw2, (width_bytes >> 8) as u8)?;
//        Ok(())
//    }

//    pub fn set_active_window(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<(), E> {
//        self.write_register(Register::AwulX1, x as u8)?;
//        self.write_register(Register::AwulX2, (x >> 8) as u8)?;
//        self.write_register(Register::AwulY1, y as u8)?;
//        self.write_register(Register::AwulY2, (y >> 8) as u8)?;

//        self.write_register(Register::AwWth1, width as u8)?;
//        self.write_register(Register::AwWth2, (width >> 8) as u8)?;
//        self.write_register(Register::AwHt1, height as u8)?;
//        self.write_register(Register::AwHt2, (height >> 8) as u8)?;
//        Ok(())
//    }

//    pub fn set_foreground_color(&mut self, color: u16) -> Result<(), E> {
//        match self.config.color_depth {
//            ColorDepth::Bpp16 => {
//                // RGB565.
//                let r = ((color >> 11) & 0x1F) << 3; // Scale 5-bit to 8-bit.
//                let g = ((color >> 5) & 0x3F) << 2;  // Scale 6-bit to 8-bit.
//                let b = (color & 0x1F) << 3;         // Scale 5-bit to 8-bit.
//                self.write_register(Register::Fgcr, r as u8)?;
//                self.write_register(Register::Fgcg, g as u8)?;
//                self.write_register(Register::Fgcb, b as u8)?;
//            }
//            _ => {
//                self.write_register(Register::Fgcr, (color >> 8) as u8)?;
//                self.write_register(Register::Fgcg, color as u8)?;
//                self.write_register(Register::Fgcb, 0)?;
//            }
//        }
//        Ok(())
//    }

//    pub fn draw_filled_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) -> Result<(), E> {
//        self.set_foreground_color(color)?;
//        self.write_register(Register::Dlhsr1, x1 as u8)?;
//        self.write_register(Register::Dlhsr2, (x1 >> 8) as u8)?;
//        self.write_register(Register::Dlvsr1, y1 as u8)?;
//        self.write_register(Register::Dlvsr2, (y1 >> 8) as u8)?;
//        self.write_register(Register::Dlher1, x2 as u8)?;
//        self.write_register(Register::Dlher2, (x2 >> 8) as u8)?;
//        self.write_register(Register::Dlver1, y2 as u8)?;
//        self.write_register(Register::Dlver2, (y2 >> 8) as u8)?;
//        self.write_register(Register::Dcr0, 0xB0)?;
//        // self.delay.delay_ms(1);
//        Ok(())
//    }

//    pub fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u16) -> Result<(), E> {
//        self.set_foreground_color(color)?;
//        self.write_register(Register::Dlhsr1, x1 as u8)?;
//        self.write_register(Register::Dlhsr2, (x1 >> 8) as u8)?;
//        self.write_register(Register::Dlvsr1, y1 as u8)?;
//        self.write_register(Register::Dlvsr2, (y1 >> 8) as u8)?;
//        self.write_register(Register::Dlher1, x2 as u8)?;
//        self.write_register(Register::Dlher2, (x2 >> 8) as u8)?;
//        self.write_register(Register::Dlver1, y2 as u8)?;
//        self.write_register(Register::Dlver2, (y2 >> 8) as u8)?;
//        self.write_register(Register::Dcr0, 0x80)?;
//        // self.delay.delay_ms(1);
//        Ok(())
//    }

//    /// Clear entire screen with color.
//    pub fn clear_screen(&mut self, color: u16) -> Result<(), E> {
//        self.draw_filled_rectangle(0, 0, self.config.width - 1, self.config.height - 1, color)
//    }

//    pub fn write_register(&mut self, register: Register, data: u8) -> Result<(), E> {
//        self.write_command(register)?;
//        self.write_data(data)?;
//        Ok(())
//    }

//    pub fn write_command(&mut self, reg_addr: Register) -> Result<(), E> {
//        self.rd.set_high();
//        self.rs.set_low();
//        self.data.write(reg_addr as u8);
//        self.delay.delay_ns(10);
//        self.wr.set_low();
//        self.delay.delay_ns(150);
//        self.wr.set_high();
//        Ok(())
//    }

//    pub fn write_data(&mut self, data: u8) -> Result<(), E> {
//        self.rd.set_high();
//        self.rs.set_high();
//        self.data.write(data);
//        self.delay.delay_ns(10);
//        self.wr.set_low();
//        self.delay.delay_ns(150);
//        self.wr.set_high();
//        Ok(())
//    }

//    pub fn read_data(&mut self) -> Result<u8, E> {
//        todo!();
//    }

//    pub fn read_status(&mut self) {
//        //TODO: page 131 https://www.buydisplay.com/download/ic/LT7683.pdf
//        todo!();
//    }

//    fn hardware_reset(&mut self) -> Result<(), E> {
//        self.res.set_low();
//        self.delay.delay_ms(10);
//        self.res.set_high();
//        self.delay.delay_ms(3);
//        Ok(())
//    }
//}


pub trait ParallelBus {
    type Error;

    fn write(&mut self, value: u8) -> ();
    fn read(&mut self) -> Result<u8, Self::Error>;
    fn set_input(&mut self) -> ();
    fn set_output(&mut self) -> ();
}
