#![feature(str_lines_remainder)]
#![feature(str_as_str)]

mod html;
mod stf;
mod txt;
mod util;

use std::{io::Write, path::PathBuf};

use html::Html;
use txt::Txt;

fn main() -> Result<(), usize> {
    let (mut path, file) = if let Some(path) = std::env::args().nth(1) {
        match std::fs::read_to_string(&path) {
            Ok(file) => (PathBuf::from(path), file),
            Err(err) => {
                eprintln!("Failed to read file: {err}");
                return Err(1);
            }
        }
    } else {
        eprintln!("Expected filepath as first argument");
        return Err(1);
    };

    let Some(filename) = path.file_prefix().map(|f| f.as_os_str().to_os_string()) else {
        eprintln!("Expected a filepath with file as first argument");
        return Err(1);
    };
    path.pop();

    // Paths.
    let mut html_path = path.clone();
    html_path.push(&filename);
    html_path.add_extension("html");
    let mut txt_path = path.clone();
    txt_path.push(&filename);
    txt_path.add_extension("txt");

    // Data.
    let tags = stf::parse(&file);
    let title = format!("#include <cmath/{}>", filename.to_string_lossy());

    // Generate HTML.
    let html = Html::generate(&title, tags.clone(), 80, 40);
    match std::fs::File::create(&html_path) {
        Ok(mut file) => match write!(file, "{html}") {
            Ok(()) => println!("Wrote {}", html_path.file_name().unwrap().to_string_lossy()),
            Err(err) => {
                eprintln!("Failed to write file: {err}");
                return Err(1);
            }
        },
        Err(err) => {
            eprintln!("Failed to create file: {err}");
            return Err(1);
        }
    }

    // Generate TXT.
    let txt = Txt::generate(tags, 80, 40);
    match std::fs::File::create(&txt_path) {
        Ok(mut file) => match write!(file, "{txt}") {
            Ok(()) => println!("Wrote {}", txt_path.file_name().unwrap().to_string_lossy()),
            Err(err) => {
                eprintln!("Failed to write file: {err}");
                return Err(1);
            }
        },
        Err(err) => {
            eprintln!("Failed to create file: {err}");
            return Err(1);
        }
    }

    Ok(())
}
