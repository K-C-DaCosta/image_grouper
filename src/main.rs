use clap::{arg, command, Command};
use image_grouper::{filesysutils::*, perceptual};
use serde::Serialize;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

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

const VALID_IMAGE_EXTS: &[&str] = &["bmp", "png", "jpg", "jpeg", "gif", "tga", "tiff", "ppm"];

type GroupID = usize;

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

#[derive(Serialize)]
pub struct ImageEntry {
    hash: u64,
    path: PathBuf,
}

#[derive(Serialize)]
pub struct ProgramOutput {
    pub image_info_list: Vec<ImageEntry>,
    pub group_table: HashMap<GroupID, GroupInfo>,
}

fn main() {
    let matches = command!()
        .about("A program the programatically groups similar images into folders")
        .arg(arg!([directory] "will recursively traverse from here to collect group images"))
        .arg(
            arg!(
                -i --images <IMAGE_FILES> "expects specific paths of images to group"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            // Support non-UTF8 paths
            .allow_invalid_utf8(true)
            .min_values(1),
        )
        .arg(
            arg!(
                -f --func <TYPE> ... "hash function. TYPE can be: 'phash' 'ahash' or 'dhash'"
            )
            .required(false)
            .default_value("ahash"),
        )
        .subcommand(
            Command::new("test")
                .about("does testing things")
                .arg(arg!(-l --list "lists test values")),
        )
        .get_matches();

    let hash_method = match matches.value_of("func").unwrap_or("ahash") {
        "ahash" => HashType::AHASH,
        "dhash" => HashType::DHASH,
        "phash" => HashType::PHASH,
        _ => HashType::default(),
    };

    if let Some(directory) = matches.value_of("directory") {
        let path: &Path = directory.as_ref();

        if path.is_dir() == false {
            eprintln!("'{:?}' is not a directory", path);
            return;
        }

        //setup an iterator to bfs the filesystem for image files
        let file_iterator = FileSystemIterator::new(path)
            .filter(|path| path.is_file())
            .filter(|image_file| {
                let ext = image_file.as_os_str().to_str().unwrap_or_default();
                VALID_IMAGE_EXTS.contains(&ext)
            })
            .filter_map(|file| image::open(&file).ok().zip(Some(file)));

        //execute iterator here
        let image_info_list = match hash_method {
            HashType::AHASH => {
                // println!("picked ahash");
                file_iterator
                    .map(|(img, path)| ImageEntry {
                        hash: perceptual::ahash(&img),
                        path,
                    })
                    .collect::<Vec<_>>()
            }
            HashType::DHASH => {
                // println!("picked dhash");
                file_iterator
                    .map(|(img, path)| ImageEntry {
                        hash: perceptual::dhash(&img),
                        path,
                    })
                    .collect::<Vec<_>>()
            }
            _ => {
                eprintln!("{:?} not implemented", hash_method);
                vec![]
            }
        };

        //below is the algorithm where I group images based on similarity score
        let mut group_counter = 0;
        let mut group_table: HashMap<GroupID, GroupInfo> = HashMap::new();
        const EPSILON: u64 = 90;

        for i in 0..image_info_list.len() {
            let ImageEntry {
                hash: hash_a,
                path: _path_a,
            } = &image_info_list[i];

            //check if the image is already in a bucket
            for (_, groups) in group_table.iter_mut() {
                let group_hash = groups.hash;
                let score = perceptual::similarity_score(*hash_a, group_hash);
                if score > EPSILON {
                    groups.similar_images.push(ImageInfo {
                        hash: *hash_a,
                        image_idx: i,
                    })
                }
            }

            for j in i + 1..image_info_list.len() {
                let ImageEntry {
                    hash: hash_b,
                    path: _path_b,
                } = &image_info_list[j];

                //score is between 0-100
                let score = perceptual::similarity_score(*hash_a, *hash_b);

                if score > EPSILON {
                    //the two images are similar
                    //create a group with two of the images inside
                    group_table.insert(
                        group_counter,
                        GroupInfo {
                            hash: *hash_a,
                            similar_images: vec![
                                ImageInfo {
                                    hash: *hash_a,
                                    image_idx: i,
                                },
                                ImageInfo {
                                    hash: *hash_b,
                                    image_idx: j,
                                },
                            ],
                        },
                    );

                    //increment group counter
                    group_counter += 1;
                }
            }
        }

        let output = ProgramOutput{
            image_info_list,
            group_table
        };
        if let Ok(json) = serde_json::to_string(&output){
            println!("{}",json);
        }else{
            eprintln!("Error: failed to serialize grouping data");
        }
    }
}
