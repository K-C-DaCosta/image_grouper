use image::{imageops::FilterType, GenericImageView};

pub fn ahash(image: &image::DynamicImage) -> u64 {
    let downscaled = image.resize(8, 8, FilterType::Gaussian).grayscale();

    let average = downscaled
        .pixels()
        .map(|(_, _, e)| e[0] as u64)
        .sum::<u64>()
        / 64;

    let mut hash = 0;
    for i in 0..8 {
        for j in 0..8 {
            let a = downscaled.get_pixel(i, j)[0] as u64;
            hash |= ((a > average) as u64) << (j + i);
        }
    }
    hash
}
/// difference hashing
/// source: https://web.archive.org/web/20210806051726/https://people.cs.umass.edu/~liberato/courses/2020-spring-compsci590k/lectures/09-perceptual-hashing/
pub fn dhash(image: &image::DynamicImage) -> u64 {
    let downscaled = image.resize(9, 8, FilterType::Gaussian).grayscale();
    let mut hash = 0;
    for i in 0..8 {
        for j in 0..8 {
            let a = downscaled.get_pixel(i, j)[0];
            let b = downscaled.get_pixel(i, j + 1)[0];
            hash |= ((a < b) as u64) << (j + i);
        }
    }
    hash
}

pub fn phash(image: &image::DynamicImage) -> u64 {
    unimplemented!("p-hash currently not implemented")
}
