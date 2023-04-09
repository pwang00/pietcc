use image::Rgb;
use types::color::{Hue::*, Lightness, Lightness::*};

// I feel like converting pixels to lightness would make the code more maintainable
pub trait ConvertToLightness {
    fn rgb_to_lightness(pixel: &Rgb<u8>) -> Lightness {
        match pixel.0 {
            [0x00, 0x00, 0x00] => Black,
            [0xFF, 0xFF, 0xFF] => White,
            [0xFF, 0xC0, 0xC0] => Light(Red),
            [0xFF, 0xFF, 0xC0] => Light(Yellow),
            [0xC0, 0xFF, 0xC0] => Light(Green),
            [0xC0, 0xFF, 0xFF] => Light(Cyan),
            [0xC0, 0xC0, 0xFF] => Light(Blue),
            [0xFF, 0xC0, 0xFF] => Light(Magenta),
            [0xFF, 0x00, 0x00] => Reg(Red),
            [0xFF, 0xFF, 0x00] => Reg(Yellow),
            [0x00, 0xFF, 0x00] => Reg(Green),
            [0x00, 0xFF, 0xFF] => Reg(Cyan),
            [0x00, 0x00, 0xFF] => Reg(Blue),
            [0xFF, 0x00, 0xFF] => Reg(Magenta),
            [0xC0, 0x00, 0x00] => Dark(Red),
            [0xC0, 0xC0, 0x00] => Dark(Yellow),
            [0x00, 0xC0, 0x00] => Dark(Green),
            [0x00, 0xC0, 0xC0] => Dark(Cyan),
            [0x00, 0x00, 0xC0] => Dark(Blue),
            [0xC0, 0x00, 0xC0] => Dark(Magenta),
            _ => panic!("Invalid pixel encountered! {:?}", pixel.0),
        }
    }
}
