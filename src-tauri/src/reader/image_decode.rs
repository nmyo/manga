use super::types::{ReaderManifest, ReaderPage};
use crate::api::{ApiError, ApiErrorKind, ApiResult};
use image::{DynamicImage, RgbImage};

const JM_SCRAMBLE_ID: u32 = 220_980;
const SCRAMBLED_WEBP_QUALITY: f32 = 75.0;

pub(crate) fn encode_scrambled_webp_cache(decoded: &RgbImage) -> Vec<u8> {
    let (width, height) = decoded.dimensions();
    let encoder = webp::Encoder::from_rgb(decoded, width, height);

    encoder.encode(SCRAMBLED_WEBP_QUALITY).to_vec()
}

pub(crate) fn decode_scrambled_image(
    original: DynamicImage,
    read_id: u32,
    page_name: &str,
) -> ApiResult<RgbImage> {
    let rgb = original.to_rgb8();
    let seed = segmentation_count(read_id, page_name);

    Ok(reorder_scrambled_rgb_rows(&rgb, seed))
}

fn reorder_scrambled_rgb_rows(source: &RgbImage, seed: u32) -> RgbImage {
    let (natural_width, natural_height) = source.dimensions();
    let row_bytes = natural_width as usize * 3;
    let source_bytes = source.as_raw();
    let mut decoded = RgbImage::new(natural_width, natural_height);
    let decoded_bytes = decoded.as_mut();
    let remainder = natural_height % seed;

    for index in 0..seed {
        let mut height = natural_height / seed;
        let mut dy = height * index;
        let sy = natural_height - height * (index + 1) - remainder;

        if index == 0 {
            height += remainder;
        } else {
            dy += remainder;
        }

        for row in 0..height {
            let source_offset = (sy + row) as usize * row_bytes;
            let target_offset = (dy + row) as usize * row_bytes;
            let source_row = &source_bytes[source_offset..source_offset + row_bytes];
            let target_row = &mut decoded_bytes[target_offset..target_offset + row_bytes];

            target_row.copy_from_slice(source_row);
        }
    }

    decoded
}

pub(crate) fn should_decode_image(manifest: &ReaderManifest, page: &ReaderPage) -> bool {
    !is_gif_source(&page.source_url)
        && segmentation_count(manifest.read_id_number, &page.page_name) > 1
}

fn segmentation_count(read_id: u32, page_name: &str) -> u32 {
    if read_id < JM_SCRAMBLE_ID {
        return 0;
    }

    if read_id < 268_850 {
        return 10;
    }

    let key = format!("{read_id}{page_name}");
    let key_md5 = format!("{:x}", md5::compute(key));
    let last_char = key_md5
        .as_bytes()
        .last()
        .copied()
        .map(u32::from)
        .unwrap_or_default();

    if read_id > 421_926 {
        return (last_char % 8) * 2 + 2;
    }

    (last_char % 10) * 2 + 2
}

pub(crate) fn source_extension(source_url: &str) -> &'static str {
    let source = source_url.split('?').next().unwrap_or(source_url);
    let extension = source
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "gif" => "gif",
        "png" => "png",
        "webp" => "webp",
        "jpeg" => "jpg",
        _ => "jpg",
    }
}

fn is_gif_source(source_url: &str) -> bool {
    source_extension(source_url) == "gif"
}

pub(crate) fn map_image_error(error: image::ImageError) -> ApiError {
    ApiError::new(ApiErrorKind::Decode, error.to_string())
}
