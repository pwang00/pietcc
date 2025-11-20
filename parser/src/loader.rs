use crate::convert::{ConvertToLightness, UnknownPixelSettings};
use image::ImageError;
use piet_core::program::PietSource;
pub struct Loader;

impl ConvertToLightness for Loader {}

impl Loader {
    pub fn convert<'a>(
        filename: &str,
        settings: UnknownPixelSettings,
    ) -> Result<PietSource<'a>, ImageError> {
        let img = image::open(filename)?.into_rgb8();
        let (w, h) = img.dimensions();

        let leaked = Box::leak(Box::new(
            img.pixels()
                .map(|pix| <Self as ConvertToLightness>::rgb_to_lightness(pix, settings))
                .collect::<Vec<_>>(),
        ));

        Ok(PietSource::new(leaked, h, w))
    }
}
