use dirs::home_dir;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::PathBuf;

mod exiftool;

struct ImageFile {
  path_src: String,
  sha256: String,
  created_at: Option<String>,
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
  let home_folder = home_dir().expect("Failed to get home folder.");
  let path_dir = format!("{}/Pictures/Camera/", home_folder.display());
  println!("Get files in {}", &path_dir);

  let image_files = read_sha256_dir(&path_dir).expect("Failed to get sha256 for dir.");

  for (index, image_file) in image_files.iter().enumerate() {
    println!(
      "{}. {}: {}, {:?}",
      index + 1,
      image_file.path_src,
      image_file.sha256,
      image_file.created_at
    );
  }

  Ok(())
}

fn is_file(entry: &DirEntry) -> Option<PathBuf> {
  let path = entry.path();
  if path.is_file() {
    Some(path)
  } else {
    None
  }
}

fn read_sha256_dir(path_dir: &str) -> Result<Vec<ImageFile>, Box<dyn Error>> {
  let mut files_path: Vec<String> = fs::read_dir(path_dir)?
    .filter_map(|direntry| direntry.ok())
    .filter_map(|direntry| is_file(&direntry))
    .filter_map(|path| path.into_os_string().into_string().ok())
    .collect();

  let image_files: Vec<_> = files_path
    .par_iter_mut()
    .filter_map(|path_str| {
      let mut hasher: Sha256 = Sha256::new();
      let metadata = exiftool::parse_file(path_str).expect("Failed to get metadata");
      let created_at = match metadata["EXIF:DateTimeOriginal"].as_str() {
        Some(v) => Some(v.to_owned()),
        None => None,
      };

      match hash_file(&mut hasher, &path_str) {
        Ok(sha256) => Some(ImageFile {
          path_src: path_str.to_owned(),
          created_at,
          sha256,
        }),
        Err(_) => None,
      }
    })
    .collect();

  Ok(image_files)
}

fn hash_file(hasher: &mut Sha256, path_to_file: &str) -> Result<String, Box<dyn Error>> {
  let mut file = fs::File::open(path_to_file)?;
  io::copy(&mut file, hasher)?;
  Ok(format!("{:x}", &hasher.finalize_reset()))
}
