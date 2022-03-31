use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub mod filesysutils;
pub mod graph;
pub mod perceptual;


pub const VALID_IMAGE_EXTS: &[&str] = &["bmp", "png", "jpg", "jpeg", "gif", "tga", "tiff", "ppm"];
pub type GroupID = usize;

#[derive(Copy, Clone, Debug)]
pub enum HashType {
    AHASH = 0,
    DHASH = 1,
    PHASH = 2,
}
impl Default for HashType {
    fn default() -> Self {
        Self::AHASH
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct ImageInfo {
    pub hash: u64,
    pub image_idx: usize,
}

#[derive(Clone, Serialize, Debug)]
pub struct GroupInfo {
    pub hash: u64,
    pub similar_images: Vec<ImageInfo>,
}

#[derive(Serialize, Debug)]
pub struct ImageEntry {
    pub hash: u64,
    pub path: PathBuf,
}
