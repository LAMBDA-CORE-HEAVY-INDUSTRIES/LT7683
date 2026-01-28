#![no_std]
#![deny(unsafe_code)]

mod registers;

use crate::registers::Register;
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
    /// Default config for 7" 1024x600 display (ER-TFT070A2-6-5633).
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
        delay.delay_ms(100);
        Ok(())
    }

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

    pub fn init<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
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
        self.configure_main_window()?;
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

        // TODO: Verify that this is correct. Do we want to configure this by user?
        let sdram_itv: u16 = 0x030D;
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

    fn configure_main_window(&mut self) -> Result<(), I::Error> {
        // Main/PIP window control: 16bpp color depth
        self.write_register(Register::Mpwctr, 0x04)?;
        // Main image start address
        self.write_register(Register::Misa1, 0x00)?;
        self.write_register(Register::Misa2, 0x00)?;
        self.write_register(Register::Misa3, 0x00)?;
        self.write_register(Register::Misa4, 0x00)?;
        let width = self.config.width;
        self.write_register(Register::Miw1, (width & 0xFF) as u8)?;
        self.write_register(Register::Miw2, ((width >> 8) & 0xFF) as u8)?;
        // Main window upper-left corner
        self.write_register(Register::Mwulx1, 0x00)?;
        self.write_register(Register::Mwulx2, 0x00)?;
        self.write_register(Register::Mwuly1, 0x00)?;
        self.write_register(Register::Mwuly2, 0x00)?;
        // Canvas start address
        self.write_register(Register::Cvssa1, 0x00)?;
        self.write_register(Register::Cvssa2, 0x00)?;
        self.write_register(Register::Cvssa3, 0x00)?;
        self.write_register(Register::Cvssa4, 0x00)?;
        // Canvas image width in pixels
        self.write_register(Register::CvsImwth1, (width & 0xFF) as u8)?;
        self.write_register(Register::CvsImwth2, ((width >> 8) & 0xFF) as u8)?;
        // Active window
        self.set_active_window(0, 0, self.config.width, self.config.height)?;
        self.write_register(Register::AwColor, self.config.color_depth as u8)?;
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

    pub fn set_foreground_color(&mut self, color: u32) -> Result<(), I::Error> {
        self.set_color_registers(color, Register::Fgcr, Register::Fgcg, Register::Fgcb)
    }

    pub fn set_background_color(&mut self, color: u32) -> Result<(), I::Error> {
        self.set_color_registers(color, Register::Bgcr, Register::Bgcg, Register::Bgcb)
    }

    /// Sets the blink period of text cursor from range of 1 (0x00) to 256 (0xFF), expressed in
    /// number of display frames. For example if the panel runs at 60hz, blink period of 0x00 would
    /// mean the text cursor blinks every 16.7ms.
    pub fn set_text_cursor_blink_period(&mut self, blink_period: u8) -> Result<(), I::Error> {
        self.write_register(Register::Btcr, blink_period)?;
        Ok(())
    }

    pub fn enable_text_cursor(&mut self) -> Result<(), I::Error> {
        self.write_register(Register::Gtccr, 0x03)?;
        Ok(())
    }

    fn set_color_registers(&mut self, color: u32, reg_r: Register, reg_g: Register, reg_b: Register) -> Result<(), I::Error> {
        // Input: 0x00RRGGBB (8 bits per channel)
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        match self.config.color_depth {
            ColorDepth::Bpp8 => {
                self.write_register(reg_r, r & 0xE0)?;
                self.write_register(reg_g, g & 0xE0)?;
                self.write_register(reg_b, b & 0xC0)?;
            }
            ColorDepth::Bpp16 => {
                self.write_register(reg_r, r & 0xF8)?;
                self.write_register(reg_g, g & 0xFC)?;
                self.write_register(reg_b, b & 0xF8)?;
            }
            ColorDepth::Bpp24 => {
                self.write_register(reg_r, r)?;
                self.write_register(reg_g, g)?;
                self.write_register(reg_b, b)?;
            }
        }
        Ok(())
    }

    /// Wait for drawing engine to complete (check status bit 3 = core busy).
    pub fn wait_busy_draw(&mut self) -> Result<(), I::Error> {
        loop {
            let status = self.read_status()?;
            if (status & 0x08) == 0 {
                break;
            }
        }
        Ok(())
    }

    pub fn draw_filled_rectangle(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u32) -> Result<(), I::Error> {
        self.set_foreground_color(color)?;
        // Set start point
        self.write_register(Register::Dlhsr1, x1 as u8)?;
        self.write_register(Register::Dlhsr2, (x1 >> 8) as u8)?;
        self.write_register(Register::Dlvsr1, y1 as u8)?;
        self.write_register(Register::Dlvsr2, (y1 >> 8) as u8)?;
        // Set end point
        self.write_register(Register::Dlher1, x2 as u8)?;
        self.write_register(Register::Dlher2, (x2 >> 8) as u8)?;
        self.write_register(Register::Dlver1, y2 as u8)?;
        self.write_register(Register::Dlver2, (y2 >> 8) as u8)?;

        self.write_register(Register::Dcr1, 0xE0)?;
        self.wait_busy_draw()?;
        Ok(())
    }

    pub fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u32) -> Result<(), I::Error> {
        self.set_foreground_color(color)?;
        // Set start point
        self.write_register(Register::Dlhsr1, x1 as u8)?;
        self.write_register(Register::Dlhsr2, (x1 >> 8) as u8)?;
        self.write_register(Register::Dlvsr1, y1 as u8)?;
        self.write_register(Register::Dlvsr2, (y1 >> 8) as u8)?;
        // Set end point
        self.write_register(Register::Dlher1, x2 as u8)?;
        self.write_register(Register::Dlher2, (x2 >> 8) as u8)?;
        self.write_register(Register::Dlver1, y2 as u8)?;
        self.write_register(Register::Dlver2, (y2 >> 8) as u8)?;

        self.write_register(Register::Dcr0, 0x80)?;
        self.wait_busy_draw()?;
        Ok(())
    }

    /// When bg_color is not provided, characters background will be the canvas background.
    pub fn write_text(&mut self, text: &str, x: u16, y: u16, bg_color: Option<u32>, fg_color: u32) -> Result<(), I::Error> {
        self.write_text_scaled(text, x, y, bg_color, fg_color, 1, 1)
    }

    /// When bg_color is not provided, characters background will be the canvas background.
    /// scale_x and scale_y: 1-4 (1 = normal size, 2 = 2x, 3 = 3x, 4 = 4x).
    pub fn write_text_scaled(
        &mut self, text: &str, x: u16, y: u16,
        bg_color: Option<u32>, fg_color: u32, scale_x: u8, scale_y: u8
    ) -> Result<(), I::Error> {
        // TODO: This is now hardcoded as Internal CGROM character.
        // Make it user configurable, in case external one is desired.
        // This also controls the size (8x16, 12x24, 16x32)
        self.write_register(Register::Ccr0, 0x00)?;

        let scale_x_bits = (scale_x.saturating_sub(1).min(3)) & 0x03;
        let scale_y_bits = (scale_y.saturating_sub(1).min(3)) & 0x03;
        let scale_bits = (scale_y_bits << 2) | scale_x_bits;
        match bg_color {
            Some(bg_color) => {
                self.write_register(Register::Ccr1, scale_bits)?;
                self.set_background_color(bg_color)?
            },
            None => {
                self.write_register(Register::Ccr1, 0x40 | scale_bits)?;
            }
        }
        self.set_foreground_color(fg_color)?;
        self.write_register(Register::Icr, 0x04)?;
        self.write_register(Register::FCurx1, x as u8)?;
        self.write_register(Register::FCurx2, (x >> 8) as u8)?;
        self.write_register(Register::FCury1, y as u8)?;
        self.write_register(Register::FCury2, (y >> 8) as u8)?;
        for &char in text.as_bytes() {
            self.write_register(Register::Mrwdp, char)?;
        }
        Ok(())
    }

    /// Clear entire screen with color.
    pub fn clear_screen(&mut self, color: u32) -> Result<(), I::Error> {
        self.draw_filled_rectangle(0, 0, self.config.width - 1, self.config.height - 1, color)
    }

    pub fn wait_bte_complete(&mut self) -> Result<(), I::Error> {
        // TODO: timeout?
        loop {
            let ctrl = self.read_register(Register::BteCtrl0)?;
            // Bit 4 = BTE busy when read
            if (ctrl & 0x10) == 0 {
                break;
            }
        }
        // Disable BTE
        self.write_register(Register::BteCtrl0, 0x00)?;
        Ok(())
    }

    /// Fill a rectangular area with a solid color using BTE.
    /// This is faster than draw_filled_rectangle for large areas.
    pub fn bte_solid_fill(&mut self, x: u16, y: u16, width: u16, height: u16, color: u32) -> Result<(), I::Error> {
        self.set_foreground_color(color)?;
        // Set destination memory start address (canvas address = 0)
        self.write_register(Register::DtStr0, 0x00)?;
        self.write_register(Register::DtStr1, 0x00)?;
        self.write_register(Register::DtStr2, 0x00)?;
        self.write_register(Register::DtStr3, 0x00)?;
        // Set destination X/Y coordinates
        self.write_register(Register::DtX0, x as u8)?;
        self.write_register(Register::DtX1, (x >> 8) as u8)?;
        self.write_register(Register::DtY0, y as u8)?;
        self.write_register(Register::DtY1, (y >> 8) as u8)?;
        // NOTE: when doing constant color/alpha blending, bits 4-2 should be different.
        let depth = self.config.color_depth as u8;
        self.write_register(Register::BteColr, depth | (depth << 5) | (depth << 2))?;
        // Set destination image width
        let canvas_width = self.config.width;
        self.write_register(Register::DtWth0, canvas_width as u8)?;
        self.write_register(Register::DtWth1, (canvas_width >> 8) as u8)?;
        // Set BTE window width and height
        self.write_register(Register::BteWth0, width as u8)?;
        self.write_register(Register::BteWth1, (width >> 8) as u8)?;
        self.write_register(Register::BteHig0, height as u8)?;
        self.write_register(Register::BteHig1, (height >> 8) as u8)?;

        // BTE Solid fill
        self.write_register(Register::BteCtrl1, 0x0C)?;
        // Enable BTE write
        self.write_register(Register::BteCtrl0, 0x10)?;
        self.wait_bte_complete()?;
        Ok(())
    }

    pub fn bte_memory_copy(
        &mut self,
        src_x: u16, src_y: u16,
        dst_x: u16, dst_y: u16,
        width: u16, height: u16
    ) -> Result<(), I::Error> {
        let canvas_width = self.config.width;
        // Set source 0 memory start address (canvas address = 0)
        self.write_register(Register::S0Str0, 0x00)?;
        self.write_register(Register::S0Str1, 0x00)?;
        self.write_register(Register::S0Str2, 0x00)?;
        self.write_register(Register::S0Str3, 0x00)?;
        // Set source 0 image width
        self.write_register(Register::S0Wth0, canvas_width as u8)?;
        self.write_register(Register::S0Wth1, (canvas_width >> 8) as u8)?;
        // Set source 0 X/Y coordinates
        self.write_register(Register::S0X0, src_x as u8)?;
        self.write_register(Register::S0X1, (src_x >> 8) as u8)?;
        self.write_register(Register::S0Y0, src_y as u8)?;
        self.write_register(Register::S0Y1, (src_y >> 8) as u8)?;
        // Set destination memory start address
        self.write_register(Register::DtStr0, 0x00)?;
        self.write_register(Register::DtStr1, 0x00)?;
        self.write_register(Register::DtStr2, 0x00)?;
        self.write_register(Register::DtStr3, 0x00)?;
        // Set destination image width
        self.write_register(Register::DtWth0, canvas_width as u8)?;
        self.write_register(Register::DtWth1, (canvas_width >> 8) as u8)?;
        // Set destination X/Y coordinates
        self.write_register(Register::DtX0, dst_x as u8)?;
        self.write_register(Register::DtX1, (dst_x >> 8) as u8)?;
        self.write_register(Register::DtY0, dst_y as u8)?;
        self.write_register(Register::DtY1, (dst_y >> 8) as u8)?;
        // Set BTE window width and height
        self.write_register(Register::BteWth0, width as u8)?;
        self.write_register(Register::BteWth1, (width >> 8) as u8)?;
        self.write_register(Register::BteHig0, height as u8)?;
        self.write_register(Register::BteHig1, (height >> 8) as u8)?;
        // NOTE: when doing constant color/alpha blending, bits should be 4-2 different.
        let depth = self.config.color_depth as u8;
        self.write_register(Register::BteColr, depth | (depth << 5) | (depth << 2))?;
        // BTE memory copy: ROP=0xC (S0), operation=0x2
        self.write_register(Register::BteCtrl1, 0xC2)?;
        // Enable BTE write
        self.write_register(Register::BteCtrl0, 0x10)?;
        self.wait_bte_complete()?;
        Ok(())
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

#[derive(Debug, Clone, Copy)]
pub enum ColorDepth {
    /// 8-bit color (RGB 3:3:2).
    Bpp8 = 0x00,
    /// 16-bit color (RGB 5:6:5).
    Bpp16 = 0x01,
    /// 24-bit color (RGB 8:8:8).
    Bpp24 = 0x02,
}
