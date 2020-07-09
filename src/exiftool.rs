use serde_json::Value;
use std::process::Command;

pub fn parse_file(path: &str) -> Option<Value> {
  let output = Command::new("exiftool")
    .arg("-charset")
    .arg("UTF8")
    .arg("-EXIF:DateTimeOriginal")
    .arg("-G")
    .arg("-n")
    .arg("-j")
    .arg(&path)
    .output()
    .expect("Failed to execute 'exiftool'.");
  let j = String::from_utf8(output.stdout).expect("Failed parsing utf8");
  let metadatas: Vec<Value> = serde_json::from_str(&j).expect("Failed to parse json");
  metadatas.into_iter().next()
}
