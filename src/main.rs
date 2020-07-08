use dirs::home_dir;
use sha2::{Digest, Sha256};
use std::error;
use std::fmt::Write;
use std::fs;
use std::io;

struct ImageFile {
  path_src: String,
  sha256: String,
}

fn main() {
  let home_folder = home_dir().expect("Failed to get home folder.");
  let dir_path = format!("{}/Pictures/Camera/", home_folder.display());
  println!("Get files in {}", &dir_path);

  read_sha256_dir(&dir_path).expect("Failed to get sha256 for dir.");
}

fn read_sha256_dir(path_dir: &str) -> Result<(), Box<dyn error::Error>> {
  let mut hasher: Sha256 = Sha256::new();

  let re = fs::read_dir(path_dir)?;
  let image_files: Vec<_> = re
    .map(|res_dir_entry| res_dir_entry.unwrap().path())
    .filter(|path_buf| path_buf.is_file())
    .map(|path_buf| format!("{}", path_buf.display()))
    .filter_map(|path_str| {
      let mut sha256 = String::new();
      match hash_file(&mut hasher, &path_str, &mut sha256) {
        Ok(_) => Some(ImageFile {
          path_src: path_str,
          sha256,
        }),
        Err(_) => return None,
      }
    })
    .collect();

  for (index, image_file) in image_files.iter().enumerate() {
    println!(
      "{}. {}: {}",
      index + 1,
      image_file.path_src,
      image_file.sha256
    );
  }
  Ok(())
}

fn hash_file(
  hasher: &mut Sha256,
  path_to_file: &str,
  output: &mut String,
) -> Result<(), Box<dyn error::Error>> {
  let mut file = fs::File::open(path_to_file)?;
  io::copy(&mut file, hasher)?;
  let result = hasher.finalize_reset();
  output.write_fmt(format_args!("{:x}", &result))?;
  Ok(())
}
