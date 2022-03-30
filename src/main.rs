use clap::{arg, command, Command};
use image_grouper::{filesysutils::*, perceptual};
use std::{env, path::Path};

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
                -f --func <TYPE> ... "hash function"
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
        match hash_method {
            HashType::AHASH => {
                // println!("picked ahash");
                file_iterator
                    .map(|(img, path)| (perceptual::ahash(&img), path))
                    .for_each(|(hash, path)| {

                    })
            }
            HashType::DHASH => {
                // println!("picked dhash");
                file_iterator
                    .map(|(img, path)| (perceptual::dhash(&img), path))
                    .for_each(|(hash, path)| {

                    })
            }
            _ => eprintln!("{:?} not implemented", hash_method),
        }
    }

    // // You can check the value provided by positional arguments, or option arguments
    // if let Some(name) = matches.value_of("name") {
    //     println!("Value for name: {}", name);
    // }

    // if let Some(raw_config) = matches.value_of_os("config") {
    //     let config_path = Path::new(raw_config);
    //     println!("Value for config: {}", config_path.display());
    // }

    // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // match matches.occurrences_of("debug") {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // // You can check for the existence of subcommands, and if found use their
    // // matches just as you would the top level cmd
    // if let Some(matches) = matches.subcommand_matches("test") {
    //     // "$ myapp test" was run
    //     if matches.is_present("list") {
    //         // "$ myapp test -l" was run
    //         println!("Printing testing lists...");
    //     } else {
    //         println!("Not printing testing lists...");
    //     }
    // }
}
