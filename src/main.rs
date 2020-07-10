use clap::{App, Arg, SubCommand};
use lazy_static::lazy_static;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::{collections::HashMap, path::PathBuf};

mod exiftool;

lazy_static! {
  static ref CONFIG_FOLDER: String = format!(
    "{}/{}",
    dirs::home_dir().unwrap().as_path().display().to_string(),
    ".elodie_rs"
  );
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct FileSha256 {
  path: PathBuf,
  sha256: String,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
  println!("{}", *CONFIG_FOLDER);

  let matches = App::new("elodie_rs")
    .version("0.1.0")
    .author("Patrick Wozniak <email@patwoz.de>")
    .about("A rewrite of elodie.")
    .arg(
      Arg::with_name("hashfile")
        .help("Path to hash.json database which contains all of the sha256 (default is hash.json)")
        .default_value("hash.json")
        .takes_value(true)
    )
    .subcommand(
      SubCommand::with_name("import")
        .about("Import files or directories by reading their EXIF and organizing them accordingly.")
        .arg(
          Arg::with_name("destination")
            .help("Copy imported files into this directory.")
            .long("destination")
            .takes_value(true)
            .required(true),
        )
        .arg(
          Arg::with_name("SOURCE")
            .help("Import files from this directory, if specified.")
            .requires("destination")
            .takes_value(true)
            .required(true),
        ),
    )
    .subcommand(
      SubCommand::with_name("generate-db")
        .about(&format!("Regenerate the hash.json database which contains all of the sha256 signatures of media files. The hash.json file is located at {}.", *CONFIG_FOLDER)[..])
        .arg(
          Arg::with_name("SOURCE")
            .help("Import files from this directory, if specified.")
            .takes_value(true)
            .required(true),
        )
    )
    .get_matches();

  fs::create_dir_all(format!("{}", *CONFIG_FOLDER))?;

  if let Some(matches) = matches.subcommand_matches("import") {
    if matches.is_present("SOURCE") {
      import(matches.value_of("SOURCE").unwrap())
    } else {
      println!("No source given");
    }
  }

  if let Some(matches) = matches.subcommand_matches("generate-db") {
    if matches.is_present("SOURCE") {
      let destination = format!(
        "{}/{}",
        *CONFIG_FOLDER,
        matches.value_of("hashfile").unwrap_or("hash.json")
      );
      generate_db(matches.value_of("SOURCE").unwrap(), &destination)
    } else {
      println!("No source given");
    }
  }

  Ok(())
}

fn import(source: &str) {
  let files = read_source_dir(&source).expect("Failed to get sha256 for dir.");

  for (index, image_file) in files.iter().enumerate() {
    println!(
      "{}. {}: {}",
      index + 1,
      image_file.path.display(),
      image_file.sha256,
    );
  }
}

fn generate_db(source: &str, destination: &str) {
  let files = read_source_dir(&source).expect("Failed to get sha256 for dir.");
  let mut hashes = HashMap::new();
  for image_file in files.iter() {
    hashes.insert(&image_file.sha256, &image_file.path);
  }
  let hashes_json = serde_json::to_value(&hashes).unwrap();
  let file = fs::File::create(destination).expect(&format!("Failed creating {}.", destination));
  serde_json::to_writer(&file, &hashes_json).expect(&format!("Failed to write {}.", destination));
  println!("{}", hashes_json);
}

fn is_file(entry: &DirEntry) -> Option<PathBuf> {
  let path = entry.path();
  if path.is_file() {
    Some(path)
  } else {
    None
  }
}

fn read_source_dir(source: &str) -> Result<Vec<FileSha256>, Box<dyn Error>> {
  let mut files_path: Vec<_> = fs::read_dir(source)?
    .into_iter()
    .filter_map(|direntry| direntry.ok())
    .filter_map(|direntry| is_file(&direntry))
    .collect();

  let files: Vec<FileSha256> = files_path
    .par_iter_mut()
    .filter_map(|path| {
      hash_file(&mut Sha256::new(), &path)
        .map(|sha256| FileSha256 {
          path: path.to_owned(),
          sha256,
        })
        .ok()
    })
    .collect();

  Ok(files)
}

fn hash_file(hasher: &mut Sha256, path: &PathBuf) -> Result<String, Box<dyn Error>> {
  let mut file = fs::File::open(path)?;
  io::copy(&mut file, hasher)?;
  Ok(format!("{:x}", &hasher.finalize_reset()))
}
