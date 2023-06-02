//! stash
//!
//! Command-line utility for managing a stash of encrypted files.
//!
//! Usage: stash <command> [<args>]
//!
//! Available commands:
//!   - init: Create a new stash at `~/.stash`.
//!   - list: List the contents of the stash.
//!   - add <file>: Encrypt a file and add it to the stash.
//!   - copy <file>: Encrypt a file and copy it into the stash.
//!   - grab <file>: Decrypt a file from the stash and drop it in the current directory.
//!
//! Note: This utility assumes that the stash has been previously initialized.
//! If not, use the `init` command to create a new stash before using other commands.
//!
//! Example usage:
//! ```shell
//! $ stash init
//! $ stash list
//! $ stash add secret_file.txt
//! $ stash copy secret_file.txt
//! $ stash grab secret_file.txt
//! ```
//!
//! For more information, refer to the documentation of each command and its respective functions.
//!
//! Authors: Jacob Bentley,
//!          Richard Duffy

#![allow(unused_variables)]

use stash::*;

const USAGE: &str = "usage: stash <command> [<args>]";

fn main() {
    //  Parse command line arguments
    let mut stash = Stash::new();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("{}", USAGE);
        return;
    }

    //  Extract the command and its arguments
    let command = &args[0];
    let arguments = &args[1..];

    //  Handle different commands and arguments
    match command.as_str() {
        "init" => {
            if !arguments.is_empty() {
                eprintln!("usage: stash init");
                return;
            }

            //  Create new stash at `~/.stash`
            match stash.init() {
                Ok(result) => println!("New stash initialized"),
                Err(err) => eprintln!("{}", err),
            }
        }
        "add" => {
            if arguments.len() != 1 {
                eprintln!("usage: stash add <file>");
                return;
            }

            let file = &arguments[0];

            //  Encrypt file and add it to stash
            match stash.add(file) {
                Ok(result) => println!("File added successfully"),
                Err(err) => println!("{}", err),
            }
        }
        "list" => {
            if !arguments.is_empty() {
                eprintln!("usage: stash list");
                return;
            }

            //  Display contents of stash
            match stash.list() {
                Ok(contents) => println!("{}", contents),
                Err(err) => eprintln!("{}", err),
            }
        }
        "copy" => {
            if arguments.len() != 1 {
                eprintln!("usage: stash copy <file>");
                return;
            }

            let file = &arguments[0];

            //  Encrypt file and copy it to stash
            match stash.copy(file) {
                Ok(result) => println!("File copied successfully"),
                Err(err) => eprintln!("{}", err),
            }
        }
        "grab" => {
            if arguments.len() != 1 {
                eprintln!("usage: stash grab <file>");
                return;
            }

            let file = &arguments[0];

            //  Decrypt a file and move it to current directory
            match stash.grab(file) {
                Ok(result) => println!("File grabbed successfully"),
                Err(err) => eprintln!("{}", err),
            }
        }
        _ => {
            eprintln!("{}", USAGE);
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}
