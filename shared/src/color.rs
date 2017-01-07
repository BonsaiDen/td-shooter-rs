// Color Name Mapping ---------------------------------------------------------
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ColorName {
    Grey,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
    Black,
    White,
    Custom
}

impl ColorName {

    pub fn to_u8(&self) -> u8 {
        match *self {
            ColorName::Red => 1,
            ColorName::Orange => 2,
            ColorName::Yellow => 3,
            ColorName::Green => 4,
            ColorName::Teal => 5,
            ColorName::Blue => 6,
            ColorName::Purple => 7,
            ColorName::Pink => 8,
            _ => 0
        }
    }

    pub fn from_u8(value: u8) -> ColorName {
        match value {
            0 => ColorName::Grey,
            1 => ColorName::Red,
            2 => ColorName::Orange,
            3 => ColorName::Yellow,
            4 => ColorName::Green,
            5 => ColorName::Teal,
            6 => ColorName::Blue,
            7 => ColorName::Purple,
            8 => ColorName::Pink,
            _ => ColorName::Custom
        }
    }

    pub fn all_colored() -> Vec<ColorName> {
        vec![
            ColorName::Red,
            ColorName::Orange,
            ColorName::Yellow,
            ColorName::Green,
            ColorName::Teal,
            ColorName::Blue,
            ColorName::Purple,
            ColorName::Pink
        ]
    }

}


// RGB Color ------------------------------------------------------------------
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color {

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a
        }
    }

    pub fn from_name(name: ColorName) -> Color {
        match name  {
            ColorName::Red => Color::new(0xf2, 0x00, 0x00, 0xff),
            ColorName::Orange => Color::new(0xfd, 0x83, 0x1c, 0xff),
            ColorName::Yellow => Color::new(0xfd, 0xda, 0x31, 0xff),
            ColorName::Green => Color::new(0x3c, 0xdc, 0x00, 0xff),
            ColorName::Teal => Color::new(0x33, 0xd0, 0xd1, 0xff),
            ColorName::Blue => Color::new(0x0f, 0x5c, 0xf9, 0xff),
            ColorName::Purple => Color::new(0x82, 0x0c, 0xe6, 0xff),
            ColorName::Pink => Color::new(0xec, 0x34, 0xa7, 0xff),
            ColorName::Black => Color::new(0x00, 0x00, 0x00, 0xff),
            ColorName::White => Color::new(0xff, 0xff, 0xff, 0xff),
            _ => Color::new(0x80, 0x80, 0x80, 0xff)
        }
    }

    #[allow(float_cmp)]
    pub fn to_hsl(&self) -> HSLColor {

        let (r, g, b) = (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0
        );

        let (min, max) = (r.min(g).min(b), r.max(g).max(b));
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            // Achromatic Case
            (0.0, 0.0)

        } else {
            let d = max - min;
            let h = if max == r {
                (g - b) / d + if g < b {
                    6.0

                } else {
                    0.0
                }

            } else if max == g {
                (b - r) / d + 2.0

            } else {
                (r - g) / d + 4.0
            };

            let s = if l > 0.5 {
                d / (2.0 - max - min)

            } else {
                d / (max + min)
            };

            (h / 6.0, s)
        };

        HSLColor::new(h, s, l, self.a)

    }

    pub fn darken(&self, by: f32) -> Color {
        self.to_hsl().darken(by).to_rgb()
    }

    pub fn lighten(&self, by: f32) -> Color {
        self.to_hsl().lighten(by).to_rgb()
    }

    pub fn into_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

}


// HSL Color ------------------------------------------------------------------
#[derive(Debug)]
pub struct HSLColor {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: u8
}

impl HSLColor {

    pub fn new(h: f32, s: f32, l: f32, a: u8) -> HSLColor {
        HSLColor {
            h: h,
            s: s,
            l: l,
            a: a
        }
    }

    pub fn into_f32(self) -> [f32; 4] {
        [self.h, self.s, self.l, self.a as f32 / 255.0]
    }

    pub fn to_rgb(&self) -> Color {

        let (r, g, b) = if self.s == 0.0 {
            // Achromatic
            (self.l, self.l, self.l)

        } else {
            let q = if self.l < 0.5  {
                self.l * (1.0 + self.s)

            } else {
                self.l + self.s - self.l * self.s
            };

            let p = 2.0 * self.l - q;

            (
                hue_to_rgb(p, q, self.h + 1.0 / 3.0),
                hue_to_rgb(p, q, self.h),
                hue_to_rgb(p, q, self.h - 1.0 / 3.0)
            )

        };

        Color::new(
            (r * 255.0).ceil() as u8,
            (g * 255.0).ceil() as u8,
            (b * 255.0).ceil() as u8,
            self.a
        )

    }

    pub fn darken(&self, by: f32) -> HSLColor {
        HSLColor::new(self.h, self.s, (self.l * (1.0 - by)).max(0.0), self.a)
    }

    pub fn lighten(&self, by: f32) -> HSLColor {
        HSLColor::new(self.h, self.s, (self.l * (1.0 + by)).min(1.0), self.a)
    }

}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {

    let t = if t < 0.0 {
        t + 1.0

    } else if t > 1.0 {
        t - 1.0

    } else {
        t
    };

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t

    } else if t < 1.0 / 2.0 {
        q

    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0

    } else {
        p
    }

}

