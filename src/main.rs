//! Stash
//!
//! Command-line utility for managing a stash of encrypted files.
//!
//! Usage: stash <command> [<args>]
//!
//! Available commands:
//!   - init <path> <label>: Create a new stash with the given label at the specified path.
//!   - list <label>: List the contents of the stash with the given label.
//!   - move <file> <label>: Encrypt the file and move it to the stash with the given label.
//!   - copy <file> <label>: Encrypt the file and copy it to the stash with the given label.
//!   - grab <file> <label>: Decrypt a file from the stash with the given label and move it to the current directory.
//!
//! Note: This utility assumes that the stash has been previously initialized.
//! If not, use the `init` command to create a new stash before using other commands.
//!
//! Example usage:
//! ```shell
//! $ stash init ~/stash my_stash
//! $ stash list my_stash
//! $ stash move secret_file.txt my_stash
//! $ stash grab secret_file.txt my_stash
//! ```
//!
//! For more information, refer to the documentation of each command and its respective functions.
//!
//! Authors: Jacob Bentley,
//!          Richard Duffy
//! Date:    05/24/2023

#![allow(unused_variables)]

mod stash_lib;
use stash_lib::*;

const USAGE: &str = "\nUsage: stash <command> [<args>]";

fn main() {
    //  Parse command line arguments
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
            if arguments.len() != 2 {
                println!("\nUsage: stash init <path> <label>");
                return;
            }
            let path = &arguments[0];
            let label = &arguments[1];

            //  Create new stash called `label`
            match init_stash(path, label) {
                Ok(result) => println!("\nDirectory created successfully"),
                Err(err) => println!("\nFailed to create directory"),
            }
        }
        "list" => {
            if arguments.len() != 1 {
                println!("\nUsage: stash list <label>");
                return;
            }
            let label = &arguments[0];

            //  Display contents of specified stash
            match list_stash(label) {
                Ok(result) => println!("\nFiles listed"),
                Err(err) => println!("\nFailed to list"),
            }
        }
        "move" => {
            if arguments.len() != 2 {
                println!("\nUsage: stash move <file> <label>");
                return;
            }
            let file = &arguments[0];
            let label = &arguments[1];

            //  Encrypt file and move it to stash
            match move_file(file, label) {
                Ok(result) => println!("\nFile moved successfully"),
                Err(err) => println!("\nFailed to move file"),
            }
        }
        "copy" => {
            if arguments.len() != 2 {
                println!("\nUsage: stash copy <file> <label>");
                return;
            }
            let file = &arguments[0];
            let label = &arguments[1];

            //  Encrypt file and copy it to stash
            match copy_file(file, label) {
                Ok(result) => println!("\nFile copied successfully"),
                Err(err) => println!("\nFailed to copy file"),
            }
        }
        "grab" => {
            if arguments.len() != 2 {
                println!("\nUsage: stash grab <file> <label>");
                return;
            }
            let file = &arguments[0];
            let label = &arguments[1];

            //  Decrypt a file and move it to current directory
            match grab_file(file, label) {
                Ok(result) => println!("\nFile grabbed successfully"),
                Err(err) => println!("\nFailed to grab file"),
            }
        }
        _ => {
            println!("{}", USAGE);
            println!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}
