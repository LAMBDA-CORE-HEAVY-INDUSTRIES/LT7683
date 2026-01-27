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
    /// BTE Control Register 0.
    BteCtrl0 = 0x90,
    /// BTE Control Register 1.
    BteCtrl1 = 0x91,
    /// BTE Color Depth Register.
    BteColr = 0x92,
    /// Source 0 Memory Start Address 0.
    S0Str0 = 0x93,
    /// Source 0 Memory Start Address 1.
    S0Str1 = 0x94,
    /// Source 0 Memory Start Address 2.
    S0Str2 = 0x95,
    /// Source 0 Memory Start Address 3.
    S0Str3 = 0x96,
    /// Source 0 Image Width 0.
    S0Wth0 = 0x97,
    /// Source 0 Image Width 1.
    S0Wth1 = 0x98,
    /// Source 0 X-Coordinate 0.
    S0X0 = 0x99,
    /// Source 0 X-Coordinate 1.
    S0X1 = 0x9A,
    /// Source 0 Y-Coordinate 0.
    S0Y0 = 0x9B,
    /// Source 0 Y-Coordinate 1.
    S0Y1 = 0x9C,
    /// Source 1 Memory Start Address 0.
    S1Str0 = 0x9D,
    /// Source 1 Memory Start Address 1.
    S1Str1 = 0x9E,
    /// Source 1 Memory Start Address 2.
    S1Str2 = 0x9F,
    /// Source 1 Memory Start Address 3.
    S1Str3 = 0xA0,
    /// Source 1 Image Width 0.
    S1Wth0 = 0xA1,
    /// Source 1 Image Width 1.
    S1Wth1 = 0xA2,
    /// Source 1 X-Coordinate 0.
    S1X0 = 0xA3,
    /// Source 1 X-Coordinate 1.
    S1X1 = 0xA4,
    /// Source 1 Y-Coordinate 0.
    S1Y0 = 0xA5,
    /// Source 1 Y-Coordinate 1.
    S1Y1 = 0xA6,
    /// Destination Memory Start Address 0.
    DtStr0 = 0xA7,
    /// Destination Memory Start Address 1.
    DtStr1 = 0xA8,
    /// Destination Memory Start Address 2.
    DtStr2 = 0xA9,
    /// Destination Memory Start Address 3.
    DtStr3 = 0xAA,
    /// Destination Image Width 0.
    DtWth0 = 0xAB,
    /// Destination Image Width 1.
    DtWth1 = 0xAC,
    /// Destination X-Coordinate 0.
    DtX0 = 0xAD,
    /// Destination X-Coordinate 1.
    DtX1 = 0xAE,
    /// Destination Y-Coordinate 0.
    DtY0 = 0xAF,
    /// Destination Y-Coordinate 1.
    DtY1 = 0xB0,
    /// BTE Window Width 0.
    BteWth0 = 0xB1,
    /// BTE Window Width 1.
    BteWth1 = 0xB2,
    /// BTE Window Height 0.
    BteHig0 = 0xB3,
    /// BTE Window Height 1.
    BteHig1 = 0xB4,

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
