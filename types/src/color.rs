use std::{hash::Hash, ops::Sub};
use Hue::*;
use Lightness::*;

#[repr(i8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Lightness {
    Light(Hue) = 2,
    Reg(Hue) = 1,
    Dark(Hue) = 0,
    Black = 3,
    White = 4,
}

#[repr(i8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Hue {
    Red = 0,
    Yellow = 1,
    Green = 2,
    Cyan = 3,
    Blue = 4,
    Magenta = 5,
}

impl Lightness {
    fn discriminant(&self) -> i8 {
        // SAFETY: Because `Self` is marked `repr(i8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `i8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<i8>() }
    }

    fn hue(&self) -> i8 {
        match self {
            Light(x) | Reg(x) | Dark(x) => *x as i8,
            _ => i8::MAX,
        }
    }

    pub fn components(&self) -> (i8, i8) {
        (self.discriminant(), self.hue())
    }
}

impl ToString for Hue {
    fn to_string(&self) -> String {
        String::from(match self {
            Red => "Red",
            Yellow => "Yellow",
            Green => "Green",
            Cyan => "Cyan",
            Blue => "Blue",
            Magenta => "Magenta",
        })
    }
}

impl ToString for Lightness {
    fn to_string(&self) -> String {
        match self {
            Light(hue) => format!("Light{}", hue.to_string()),
            Reg(hue) => format!("Reg{}", hue.to_string()),
            Dark(hue) => format!("Dark{}", hue.to_string()),
            White => "White".to_string(),
            Black => "Black".to_string(),
        }
    }
}

impl Sub for Lightness {
    type Output = (i8, i8);

    fn sub(self, rhs: Self) -> Self::Output {
        let lightness_self = self.discriminant();
        let lightness_rhs = rhs.discriminant();

        let hue_self = self.hue();
        let hue_rhs = rhs.hue();

        match (lightness_self, lightness_rhs) {
            (0..=2, 0..=2) => (
                (lightness_self - lightness_rhs).rem_euclid(3),
                (hue_rhs - hue_self).rem_euclid(6),
            ),
            _ => (i8::MAX, i8::MAX),
        }
    }
}

impl Hash for Lightness {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
