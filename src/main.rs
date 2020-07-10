use clap::{App, Arg, SubCommand};
use lazy_static::lazy_static;
use std::error::Error;
use std::fs;

mod elodie;
mod exiftool;

lazy_static! {
  static ref CONFIG_FOLDER: String = format!(
    "{}/{}",
    dirs::home_dir().unwrap().as_path().display().to_string(),
    ".elodie_rs"
  );
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
      elodie::import(matches.value_of("SOURCE").unwrap())
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
      elodie::generate_db(matches.value_of("SOURCE").unwrap(), &destination)
    } else {
      println!("No source given");
    }
  }

  Ok(())
}
