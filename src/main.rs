use dirs::home_dir;
use sha2::{Digest, Sha256};
use std::error;
use std::fmt::Write;
use std::fs;
use std::io;

fn main() {
  let home_folder = home_dir().expect("Failed to get home folder.");
  let dir_path = format!("{}/Pictures/Camera/", home_folder.display());
  println!("Get files in {}.", &dir_path);

  read_sha256_dir(&dir_path).expect("Failed to get sha256 for dir.");
  println!("{}", dir_path);
}

fn read_sha256_dir(path_dir: &str) -> io::Result<()> {
  for entry in fs::read_dir(&path_dir)? {
    let path = entry?.path();
    if !path.is_file() {
      continue;
    }
    let mut file_sha256sum = String::new();
    let file_path = format!("{}", path.display());
    let mut hasher: Sha256 = Sha256::new();
    read_sha256(&mut hasher, &file_path, &mut file_sha256sum).expect(&format!(
      "Failed to read sha256sum for file '{}'",
      file_path
    ));
    println!("{}: {}", file_path, file_sha256sum);
  }
  Ok(())
}

fn read_sha256(
  hasher: &mut Sha256,
  path: &str,
  output: &mut String,
) -> Result<(), Box<dyn error::Error>> {
  let mut file = fs::File::open(path)?;
  io::copy(&mut file, hasher)?;
  let result = hasher.finalize_reset();
  output.write_fmt(format_args!("{:x}", &result))?;
  Ok(())
}
