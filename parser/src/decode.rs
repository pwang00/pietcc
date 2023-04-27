use types::color::Lightness;
use types::instruction::Instruction;

pub trait DecodeInstruction {
    fn decode_instr(left: Lightness, right: Lightness) -> Option<Instruction> {
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
}

#[cfg(test)]
mod test_parse {
    use super::DecodeInstruction;
    use crate::convert::ConvertToLightness;
    use image::Rgb;
    use types::instruction::Instruction;

    struct Test;
    impl DecodeInstruction for Test {}
    impl ConvertToLightness for Test {}
    const SETTINGS: UnknownPixelSettings = UnknownPixelSettings::TreatAsError;
    #[test]
    fn test_convert_hue_change() {
        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0xC0, 0xFF]);
        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(Test::decode_instr(l1, l2), Some(Instruction::Add));

        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0xFF, 0xFF]);
        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(Test::decode_instr(l1, l2), Some(Instruction::Div))
    }

    #[test]
    fn test_convert_lightness_change() {
        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xC0]);
        let pix2 = Rgb::<u8>([0xFF, 0x00, 0x00]);
        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(Test::decode_instr(l1, l2), Some(Instruction::Push));

        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xC0]);
        let pix2 = Rgb::<u8>([0xC0, 0x00, 0x00]);
        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(Test::decode_instr(l1, l2), Some(Instruction::Pop))
    }

    #[test]
    fn test_convert_hue_lightness_change() {
        let pix1 = Rgb::<u8>([0xFF, 0xC0, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0x00, 0x00]);
        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        println!("{:?}, {:?}", l1, l2);
        println!("{:?}", l1 - l2);

        assert_eq!(Test::decode_instr(l1, l2), Some(Instruction::CharOut))
    }

    #[test]
    fn test_rgb_convert_white_none() {
        let pix1 = Rgb::<u8>([0xFF, 0xFF, 0xFF]);
        let pix2 = Rgb::<u8>([0xC0, 0xC0, 0xFF]);

        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(Test::decode_instr(l1, l2), None);
    }

    #[test]
    fn test_rgb_convert_black_none() {
        let pix1 = Rgb::<u8>([0x00, 0x00, 0x00]);
        let pix2 = Rgb::<u8>([0xC0, 0xC0, 0xFF]);

        let l1 = Test::rgb_to_lightness(&pix1, SETTINGS);
        let l2 = Test::rgb_to_lightness(&pix2, SETTINGS);

        assert_eq!(Test::decode_instr(l1, l2), None);
    }
}
