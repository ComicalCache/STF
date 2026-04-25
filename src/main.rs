#![feature(str_lines_remainder, str_as_str)]
#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]

mod frontend;
mod stf;
mod util;

use std::{io::Write, path::PathBuf};

use frontend::{html::Html, txt::Txt};

fn main() -> Result<(), usize> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filepath> [frontend width height]...", args[0]);
        return Err(1);
    }

    // Parse filepath and turn it to directory path.
    let mut path = PathBuf::from(&args[1]);
    let file = match std::fs::read_to_string(&path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to read file: {err}");
            return Err(1);
        }
    };
    let Some(filename) = path.file_prefix().map(|f| f.as_os_str().to_os_string()) else {
        eprintln!("Expected a filepath with file as first argument");
        return Err(1);
    };
    path.pop();

    let tags = stf::parse(&file);
    let title = format!("#include <cmath/{}>", filename.to_string_lossy());

    let triple = &args[2..];
    if triple.len() % 3 != 0 {
        eprintln!("Expected optional arguments to be in triples of: <frontend> <width> <height>");
        return Err(1);
    }
    if triple.is_empty() {
        return Ok(());
    }

    for triple in triple.chunks(3) {
        let frontend = &triple[0];
        let width = &triple[1];
        let height = &triple[2];

        let width: usize = match width.parse() {
            Ok(width) => width,
            Err(_) => {
                eprintln!("Invalid width '{}', expected an integer", width);
                return Err(1);
            }
        };
        let height: usize = match height.parse() {
            Ok(height) => height,
            Err(_) => {
                eprintln!("Invalid height '{}', expected an integer", height);
                return Err(1);
            }
        };

        let mut out_path = path.clone();
        out_path.push(&filename);

        let (extension, content) = match frontend.to_lowercase().as_str() {
            "html" => ("html", Html::generate(&title, tags.clone(), width, height)),
            "txt" => ("txt", Txt::generate(tags.clone(), width, height)),
            _ => {
                eprintln!("Unknown frontend: '{}', expected 'html' or 'txt'", frontend);
                return Err(1);
            }
        };
        out_path.add_extension(extension);

        match std::fs::File::create(&out_path) {
            Ok(mut file) => match write!(file, "{content}") {
                Ok(()) => println!("Wrote {}", out_path.file_name().unwrap().to_string_lossy()),
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
    }

    Ok(())
}
