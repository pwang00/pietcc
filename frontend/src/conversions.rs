use UnknownPixelSettings::*;
use image::Rgb;
use piet_core::color::Lightness;
use piet_core::color::{Hue::*, Lightness::*};
use piet_core::instruction::Instruction;
use std::panic;

#[derive(Copy, Clone)]
pub enum UnknownPixelSettings {
    TreatAsError,
    TreatAsWhite,
    TreatAsBlack,
}

pub fn rgb_to_lightness(pixel: &Rgb<u8>, settings: UnknownPixelSettings) -> Lightness {
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
        _ => match settings {
            TreatAsError => panic!("Invalid pixel encountered! {:?}", pixel.0),
            TreatAsWhite => White,
            TreatAsBlack => Black,
        },
    }
}

pub(crate) fn decode_instr(left: Lightness, right: Lightness) -> Option<Instruction> {
    match left - right {
        (0, 1) => Some(Instruction::Add),
        (0, 2) => Some(Instruction::Div),
        (0, 3) => Some(Instruction::Gt),
        (0, 4) => Some(Instruction::Dup),
        (0, 5) => Some(Instruction::CharIn),

        (1, 0) => Some(Instruction::Push),
        (1, 1) => Some(Instruction::Sub),
        (1, 2) => Some(Instruction::Mod),
        (1, 3) => Some(Instruction::Ptr),
        (1, 4) => Some(Instruction::Roll),
        (1, 5) => Some(Instruction::IntOut),

        (2, 0) => Some(Instruction::Pop),
        (2, 1) => Some(Instruction::Mul),
        (2, 2) => Some(Instruction::Not),
        (2, 3) => Some(Instruction::Swi),
        (2, 4) => Some(Instruction::IntIn),
        (2, 5) => Some(Instruction::CharOut),

        _ => None,
    }
}

#[cfg(test)]
mod test_parse {
    use super::{UnknownPixelSettings, decode_instr, rgb_to_lightness};
    use image::Rgb;
    use piet_core::instruction::Instruction;

    const SETTINGS: UnknownPixelSettings = UnknownPixelSettings::TreatAsError;
    #[test]
    fn test_convert_hue_change() {
        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0xC0, 0xFF]);
        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        // Hue steps run forward from the block being left to the block being entered, so blue
        // to magenta is the single step that decodes to Add
        assert_eq!(decode_instr(l2, l1), Some(Instruction::Add));

        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0xFF, 0xFF]);
        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        // Likewise cyan to magenta is two steps
        assert_eq!(decode_instr(l2, l1), Some(Instruction::Div))
    }

    #[test]
    fn test_convert_lightness_change() {
        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xC0]);
        let pix2 = Rgb::<u8>([0xFF, 0x00, 0x00]);
        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(decode_instr(l1, l2), Some(Instruction::Push));

        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xC0]);
        let pix2 = Rgb::<u8>([0xC0, 0x00, 0x00]);
        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(decode_instr(l1, l2), Some(Instruction::Pop))
    }

    #[test]
    fn test_convert_hue_lightness_change() {
        // Light magenta to dark blue is five hue steps and two lightness steps, the corner of
        // the instruction table holding out(char)
        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xFF]);
        let pix2 = Rgb::<u8>([0x00, 0x00, 0xC0]);
        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(decode_instr(l1, l2), Some(Instruction::CharOut))
    }

    #[test]
    fn test_rgb_convert_white_none() {
        let pix1 = Rgb::<u8>([0xFF, 0xFF, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0xC0, 0xFF]);

        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(decode_instr(l1, l2), None);
    }

    #[test]
    fn test_rgb_convert_black_none() {
        let pix1 = Rgb::<u8>([0x00, 0x00, 0x00]);
        let pix2 = Rgb::<u8>([0xC0, 0xC0, 0xFF]);

        let l1 = rgb_to_lightness(&pix1, SETTINGS);
        let l2 = rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(decode_instr(l1, l2), None);
    }
}
