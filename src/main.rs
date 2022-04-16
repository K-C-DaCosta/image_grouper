use clap::{arg, command, Command};
use image_grouper::{filesysutils::*, graph::HammingMST, perceptual, *};
use path_absolutize::*;
use rayon::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    time::{Duration, Instant},
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
        .about("specify output directory")
        .arg(arg!( -o --output [OUTPUT_DIRECTORY] "directory of sorted files").max_values(1))
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

    let output_directory: &Path = matches.value_of("output").unwrap_or("./sorted").as_ref();

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
                eprintln!("[{:?}] not implemented", hash_method);
                vec![]
            }
        };

        println!("creating minimum spanning tree...");
        let mimimum_spanning_tree = HammingMST::new(&image_info_list).unwrap();

        // println!("{:?}", image_info_list);
        // println!("{:?}", mimimum_spanning_tree);

        let mut file_name = 0;
        let mut sym_link_path = PathBuf::new();

        std::fs::create_dir(output_directory);

        let mut circuit = mimimum_spanning_tree
            .iter()
            .filter_map(|a| a)
            .collect::<Vec<_>>();

        // spend extactly 10 seconds iteratively improving the tour
        graph::iteratively_improve_tour(30_000_000, 10_000, &mut circuit, &image_info_list);

        circuit.iter().for_each(|&idx| {
            // println!("{}", sf.idx);
            let image = &image_info_list[idx];
            let absolute_path = image.path.absolutize().unwrap();
            if let Some(ext) = image.path.extension() {
                sym_link_path.clear();
                sym_link_path.push(output_directory);
                sym_link_path.push(format!("{}", file_name));
                sym_link_path.set_extension(ext);
                // println!("{:?} -> {:?}", absolute_path, sym_link_path);
                std::os::unix::fs::symlink(&absolute_path, &sym_link_path);
                file_name += 1;
            }
        });
    }
}
