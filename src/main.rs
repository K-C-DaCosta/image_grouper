use clap::{arg, command, Command};
use image_grouper::{filesysutils::*, graph::HammingMST, perceptual, *};
use rayon::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct ProgramOutput {
    pub group_table: HashMap<GroupID, GroupInfo>,
    pub image_info_list: Vec<ImageEntry>,
}

fn main() {
    let matches = command!()
        .about("A program the programatically groups similar images into folders")
        .arg(
            arg!([directory] "will recursively traverse from here to collect group images")
                .min_values(1),
        )
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

    if let Some(directories) = matches.values_of("directory") {
        //setup an iterator to bfs the filesystem for image files
        let file_iterator = directories
            .map(|dir| {
                let dir: &Path = dir.as_ref();
                dir
            })
            .filter(|dir| dir.is_dir())
            .flat_map(|path| {
                FileSystemIterator::new(path)
                    .filter(|path| path.is_file())
                    .filter(|path| path.extension().is_some())
                    .filter(|image_file| {
                        let ext = image_file.extension().unwrap().to_str().unwrap_or_default();
                        VALID_IMAGE_EXTS.contains(&ext)
                    })
                    .filter_map(|file| image::open(&file).ok().zip(Some(file)))
            });

        //execute iterator here
        let image_info_list = match hash_method {
            HashType::AHASH => {
                // println!("picked ahash");
                file_iterator
                    .par_bridge()
                    .map(|(img, path)| ImageEntry {
                        hash: {
                            let h = perceptual::ahash(&img);
                            println!("{:?} hashed...", path);
                            h
                        },
                        path,
                    })
                    .collect::<Vec<_>>()
            }
            HashType::DHASH => {
                // println!("picked dhash");
                file_iterator
                    .par_bridge()
                    .map(|(img, path)| ImageEntry {
                        hash: {
                            let h = perceptual::dhash(&img);
                            println!("{:?} hashed...", path);
                            h
                        },
                        path,
                    })
                    .collect::<Vec<_>>()
            }
            _ => {
                eprintln!("{:?} not implemented", hash_method);
                vec![]
            }
        };

        println!("creating minimum spanning tree...");
        let mimimum_spanning_tree = HammingMST::new(&image_info_list).unwrap();

        println!("{:?}", image_info_list);
        println!("{:?}", mimimum_spanning_tree);

        let mut file_name = 0;
        let mut sym_link_path = PathBuf::new();
        std::fs::create_dir("./sorted");
        mimimum_spanning_tree.dfs_preorder_iterative(|_, sf| {
            // println!("{}", sf.idx);
            let image = &image_info_list[sf.idx];
            if let Some(ext) = image.path.extension() {
                sym_link_path.clear();
                sym_link_path.push("./sorted/");
                sym_link_path.push(format!("{}", file_name));
                sym_link_path.set_extension(ext);
                println!("{:?} -> {:?}", image.path, sym_link_path);
                std::os::unix::fs::symlink(&image.path, &sym_link_path);
                file_name += 1;
            }
        });

        //below is the algorithm where I group images based on similarity score
        // let mut group_counter = 0;
        // let mut group_table: HashMap<GroupID, GroupInfo> = HashMap::new();
        // const EPSILON: u64 = 80;

        // for i in 0..image_info_list.len() {
        //     let ImageEntry {
        //         hash: hash_a,
        //         path: _path_a,
        //     } = &image_info_list[i];

        //     let mut belongs_to_group = false;
        //     //check if the image is already in a bucket
        //     for (_, groups) in group_table.iter_mut() {
        //         let group_hash = groups.hash;
        //         let score = perceptual::similarity_score(*hash_a, group_hash);
        //         if score > EPSILON {
        //             groups.similar_images.push(ImageInfo {
        //                 hash: *hash_a,
        //                 image_idx: i,
        //             });

        //             belongs_to_group = true;
        //             break;
        //         }
        //     }
        //     if belongs_to_group {
        //         continue;
        //     }

        //     for j in i + 1..image_info_list.len() {
        //         let ImageEntry {
        //             hash: hash_b,
        //             path: _path_b,
        //         } = &image_info_list[j];

        //         //score is between 0-100
        //         let score = perceptual::similarity_score(*hash_a, *hash_b);

        //         if score > EPSILON {
        //             //the two images are similar
        //             //create a group with two of the images inside
        //             group_table.insert(
        //                 group_counter,
        //                 GroupInfo {
        //                     hash: *hash_a,
        //                     similar_images: vec![
        //                         ImageInfo {
        //                             hash: *hash_a,
        //                             image_idx: i,
        //                         },
        //                         ImageInfo {
        //                             hash: *hash_b,
        //                             image_idx: j,
        //                         },
        //                     ],
        //                 },
        //             );

        //             //increment group counter
        //             group_counter += 1;
        //         }
        //     }
        // }

        // let output = ProgramOutput {
        //     image_info_list,
        //     group_table,
        // };
        // if let Ok(json) = serde_json::to_string(&output) {
        //     println!("{}", json);
        // } else {
        //     eprintln!("Error: failed to serialize grouping data");
        // }
    }
}
