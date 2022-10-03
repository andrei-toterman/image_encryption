use std::{error::Error, fs::File, io::BufWriter, path::Path};

use image::{
    codecs::jpeg,
    error::{ImageFormatHint, UnsupportedError, UnsupportedErrorKind},
    io::Reader,
    ColorType, ImageEncoder, ImageFormat, ImageResult,
};
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};

pub struct Image {
    format: ImageFormat,
    pixels: Vec<u8>,
    color: ColorType,
    width: u32,
    height: u32,
}

pub fn load_image(path: impl AsRef<Path>) -> Result<Image, Box<dyn Error>> {
    let reader = Reader::open(path)?.with_guessed_format()?;
    let format = reader.format().ok_or_else(|| {
        UnsupportedError::from_format_and_kind(
            ImageFormatHint::Unknown,
            UnsupportedErrorKind::Format(ImageFormatHint::Unknown),
        )
    })?;

    let image = reader.decode()?;
    Ok(Image {
        format,
        height: image.height(),
        width: image.width(),
        color: image.color(),
        pixels: image.into_bytes(),
    })
}

pub fn write_image(path: impl AsRef<Path>, img: Image) -> ImageResult<()> {
    // must handle Jpeg case on its own because the default quality is too low
    if img.format == ImageFormat::Jpeg {
        let writer = &mut BufWriter::new(File::create(path)?);
        jpeg::JpegEncoder::new_with_quality(writer, 100).write_image(
            &img.pixels,
            img.width,
            img.height,
            img.color,
        )
    } else {
        image::save_buffer_with_format(
            path,
            &img.pixels,
            img.width,
            img.height,
            img.color,
            img.format,
        )
    }
}

// get the byte of rank i from a u32
fn byte(num: u32, i: usize) -> u8 {
    num.to_le_bytes()[i]
}

pub fn encrypt_image(img: &mut Image, key: u64) {
    let mut rng = SmallRng::seed_from_u64(key);
    // this value is used in the first step of encrypting the pixels, so it must be obtained before other RNG calls
    let start = rng.gen::<u32>();

    let dim = (img.width * img.height) as usize;
    let channels = img.color.channel_count() as usize;

    let mut rand_nums = Vec::<u32>::with_capacity(dim);
    for _ in 0..rand_nums.capacity() {
        rand_nums.push(rng.gen());
    }

    let mut permutation = (0..dim as u32).collect::<Vec<u32>>();
    permutation.shuffle(&mut rng);

    // permute the pixels of the buffer based on the above permutation
    let mut pixels_perm = Vec::with_capacity(channels * dim);
    for perm in permutation {
        for c in 0..channels {
            pixels_perm.push(img.pixels[channels * perm as usize + c]);
        }
    }

    // encrypt the first set of bytes by doing some XORs
    let mut enc_pixels = Vec::<u8>::with_capacity(channels * dim);
    for c in 0..channels {
        enc_pixels.push(byte(start, c) ^ pixels_perm[c] ^ byte(rand_nums[0], c));
    }

    // encrypt each pixel based on the previous one
    for i in 1..dim {
        for c in 0..channels {
            enc_pixels.push(
                enc_pixels[channels * (i - 1) + c]
                    ^ pixels_perm[channels * i + c]
                    ^ byte(rand_nums[i], c),
            );
        }
    }

    img.pixels = enc_pixels;
}

pub fn decrypt_image(img: &mut Image, key: u64) {
    let mut rng = SmallRng::seed_from_u64(key);
    // get the same initial value used for encrypting
    let start = rng.gen::<u32>();

    let dim = (img.width * img.height) as usize;
    let channels = img.color.channel_count() as usize;

    let mut rand_nums = Vec::<u32>::with_capacity(dim);
    for _ in 0..rand_nums.capacity() {
        rand_nums.push(rng.gen());
    }

    let mut permutation = (0..dim as u32).collect::<Vec<u32>>();
    permutation.shuffle(&mut rng);

    // compute the inverse of the above permutation
    let mut inv_permutation = vec![0u32; dim];
    for i in 0..permutation.len() {
        inv_permutation[permutation[i] as usize] = i as u32;
    }

    // compute the first set of unencrypted, but permuted pixels from the encrypted ones
    let mut pixels_perm = Vec::<u8>::with_capacity(channels * dim);
    for c in 0..channels {
        pixels_perm.push(byte(start, c) ^ img.pixels[c] ^ byte(rand_nums[0], c));
    }

    // decrypt each pixel based on the previous one
    for i in 1..dim {
        for c in 0..channels {
            pixels_perm.push(
                img.pixels[channels * (i - 1) + c]
                    ^ img.pixels[channels * i + c]
                    ^ byte(rand_nums[i], c),
            )
        }
    }

    let mut dec_pixels = Vec::with_capacity(channels * dim);
    // put the permuted pixels into the right order by using the inverse of the permutation
    for perm in inv_permutation {
        for c in 0..channels {
            dec_pixels.push(pixels_perm[channels * perm as usize + c]);
        }
    }

    img.pixels = dec_pixels;
}
