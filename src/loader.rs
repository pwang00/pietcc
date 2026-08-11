use frontend::conversions::{UnknownPixelSettings, rgb_to_lightness};
use image::ImageError;
use piet_core::program::PietSource;

pub fn to_lightness_raster(
    filename: &str,
    settings: UnknownPixelSettings,
) -> Result<PietSource, ImageError> {
    let img = image::open(filename)?.into_rgb8();
    let (w, h) = img.dimensions();
    Ok(PietSource::new(
        img.pixels()
            .map(|pix| rgb_to_lightness(pix, settings))
            .collect::<Vec<_>>(),
        h as usize,
        w as usize,
    ))
}
