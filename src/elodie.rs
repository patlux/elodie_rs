use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::io;
use std::{collections::HashMap, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Hash, Eq, PartialEq, Debug)]
struct FileSha256 {
  path: PathBuf,
  sha256: String,
}

pub fn import(source: &str) {
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

pub fn generate_db(source: &str, destination: &str) {
  let files = read_source_dir(&source).expect("Failed to get sha256 for dir.");
  let mut hashes = HashMap::with_capacity(files.len());
  for image_file in files.iter() {
    hashes.insert(&image_file.sha256, &image_file.path);
  }
  let hashes_json = serde_json::to_value(&hashes).unwrap();
  let file = fs::File::create(destination).expect(&format!("Failed creating {}.", destination));
  serde_json::to_writer(&file, &hashes_json).expect(&format!("Failed to write {}.", destination));
  // println!("{}", hashes_json);
}

fn read_source_dir(source: &str) -> Result<Vec<FileSha256>, Box<dyn Error>> {
  let mut files_path: Vec<_> = WalkDir::new(source)
    .into_iter()
    .filter_map(|direntry| direntry.ok())
    .filter(|direntry| is_image(&direntry).unwrap_or(false))
    .map(|direntry| direntry.path().into())
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

fn is_image(entry: &DirEntry) -> Option<bool> {
  entry
    .file_name()
    .to_str()
    .map(|s| s.ends_with("jpg") || s.ends_with("png"))
}

fn hash_file(hasher: &mut Sha256, path: &PathBuf) -> Result<String, Box<dyn Error>> {
  let mut file = fs::File::open(path)?;
  io::copy(&mut file, hasher)?;
  Ok(format!("{:x}", &hasher.finalize_reset()))
}
