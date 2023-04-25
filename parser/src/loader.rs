use crate::convert::ConvertToLightness;
use image::ImageError;
use types::program::Program;
pub struct Loader;

impl ConvertToLightness for Loader {}

impl Loader {
    pub fn convert<'a>(filename: &str) -> Result<Program<'a>, ImageError> {
        let img = image::open(filename)?.into_rgb8();
        let (w, h) = img.dimensions();

        let leaked = Box::leak(Box::new(
            img.pixels()
                .map(<Self as ConvertToLightness>::rgb_to_lightness)
                .collect::<Vec<_>>(),
        ));

        Ok(Program::new(leaked, h, w))
    }
}
