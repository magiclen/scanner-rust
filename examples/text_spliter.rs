/*!
# Text Spliter

Input an existing directory and a text file, and this tool can help you split that text file by empty lines into small text files named `%d.txt`.
*/

use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use scanner_rust::{Scanner, ScannerError};

fn main() -> Result<(), ScannerError> {
    let mut sc = Scanner::new(io::stdin());

    let directory = loop {
        print!("Input an existing directory: ");

        io::stdout().flush()?;

        let line = sc.next_line()?;

        match line {
            Some(line) => {
                let path = PathBuf::from(line);

                if !path.exists() || !path.is_dir() {
                    continue;
                }

                break path;
            }
            None => return Ok(()),
        }
    };

    let file_path = loop {
        print!("Input a text file: ");

        io::stdout().flush()?;

        let line = sc.next_line()?;

        match line {
            Some(line) => {
                let path = PathBuf::from(line);

                if !path.exists() || !path.is_file() {
                    continue;
                }

                break path;
            }
            None => return Ok(()),
        }
    };

    let mut sc = Scanner::scan_path(file_path)?;

    let mut counter = 1;
    let mut s = String::new();

    loop {
        let line = sc.next_line()?;

        match line {
            Some(line) => {
                if line.is_empty() {
                    let file_name = format!("{}.txt", counter);
                    let file_path = Path::join(&directory, file_name);

                    fs::write(file_path, &s[..(s.len() - 1)])?;

                    s.clear();

                    counter += 1;
                } else {
                    s.push_str(&line);
                    s.push('\n');
                }
            }
            None => {
                if !s.is_empty() {
                    let file_name = format!("{}.txt", counter);
                    let file_path = Path::join(&directory, file_name);

                    fs::write(file_path, &s[..(s.len() - 1)])?;
                }

                break;
            }
        }
    }

    Ok(())
}
