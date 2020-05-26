/*!
# Add 2 Integer

Input two integers and add them together.
*/

extern crate scanner_rust;

use std::io::{self, Write};

use scanner_rust::ScannerAscii;

fn main() {
    print!("Please input two integers, a and b: ");
    io::stdout().flush().unwrap();

    let mut sc = ScannerAscii::new(io::stdin());

    let a = {
        loop {
            match sc.next_isize() {
                Ok(i) => break i.unwrap_or(0),
                Err(_) => {
                    print!("Re-input a and b: ");
                    io::stdout().flush().unwrap();
                }
            }
        }
    };

    let b = {
        loop {
            match sc.next_isize() {
                Ok(i) => break i.unwrap_or(0),
                Err(_) => {
                    print!("Re-input b: ");
                    io::stdout().flush().unwrap();
                }
            }
        }
    };

    println!("{} + {} = {}", a, b, a + b);
}
