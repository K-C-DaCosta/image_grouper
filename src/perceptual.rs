use image::{imageops::FilterType, GenericImageView};
/// average hashing
pub fn ahash(image: &image::DynamicImage) -> u64 {
    let downscaled = image.resize_exact(8, 8, FilterType::Gaussian).grayscale();

    let average = downscaled
        .pixels()
        .map(|(_, _, e)| e[0] as u64)
        .sum::<u64>()
        / 64;

    let mut hash = 0;
    for i in 0..8 {
        for j in 0..8 {
            let a = downscaled.get_pixel(j, i)[0] as u64;
            hash |= ((a > average) as u64) << (j + i * 8);
        }
    }
    hash
}
/// difference hashing
/// source: https://web.archive.org/web/20210806051726/https://people.cs.umass.edu/~liberato/courses/2020-spring-compsci590k/lectures/09-perceptual-hashing/
pub fn dhash(image: &image::DynamicImage) -> u64 {
    let downscaled = image.resize_exact(9, 8, FilterType::Gaussian).grayscale();
    let mut hash = 0;
    for i in 0..8 {
        for j in 0..8 {
            let a = downscaled.get_pixel(j, i)[0];
            let b = downscaled.get_pixel(j, i + 1)[0];
            hash |= ((a < b) as u64) << (j + i * 8);
        }
    }
    hash
}

pub fn phash(image: &image::DynamicImage) -> u64 {
    unimplemented!("p-hash currently not implemented")
}
/// # Description
/// computes the similarity score
/// ## returns
/// a value of 0-100. where 0 meaning no similarity and 100 meaning very similar
pub fn similarity_score(hash_a: u64, hash_b: u64) -> u64 {
    ((hash_a ^ hash_b).count_zeros() as u64 * 100) /  64
}

#[test]
fn ahash_sanity() {
    let i1 = image::open("/home/narco/Pictures/memes/pepe/1566100937558.png").unwrap();
    let i2 = image::open("/home/narco/Pictures/memes/pepe/1566100937558_c.png").unwrap();
    let a = ahash(&i1);
    let b = ahash(&i2);
    let score = similarity_score(a, b);
    println!("a = {:08x}",a);
    println!("b = {:08x}",b);
    println!("score = {}",score);
}
