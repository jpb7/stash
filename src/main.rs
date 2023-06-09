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

use stash::*;
use std::{
    env, io,
    path::Path,
    process::{exit, Command, Stdio},
};

const USAGE: &str = "usage: stash <command> [<args>]";

fn main() {
    //  Parse command line arguments
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("{}", USAGE);
        return;
    }
    //  Authenticate as `stash` user
    let stash_user = "stash";
    let current_user = env::var("USER").expect("Failed to retrieve current user");

    if !user_exists(stash_user) {
        create_user(&current_user, stash_user);
    }
    //  TODO: use timeout to prevent re-entering password again
    if current_user != stash_user {
        run_as_stash(stash_user, args).expect("Failed to execute as stash user");
        exit(0);
    }
    //  Execute main program
    let mut stash = Stash::new();

    let command = &args[0];
    let arguments = &args[1..];

    //  Handle different commands and arguments
    match command.as_str() {
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
        "add" => {
            if arguments.len() != 1 {
                eprintln!("usage: stash add <file>");
                return;
            }
            let file = &arguments[0];

            //  Encrypt file and add it to stash
            match stash.add(file) {
                Ok(_) => println!("File added successfully"),
                Err(err) => println!("{}", err),
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
                Ok(_) => println!("File copied successfully"),
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
                Ok(_) => println!("File grabbed successfully"),
                Err(err) => eprintln!("{}", err),
            }
        }
        "use" => {
            if arguments.len() != 1 {
                eprintln!("usage: stash use <file>");
                return;
            }
            let file = &arguments[0];

            //  Decrypt a file and copy it to current directory
            match stash.r#use(file) {
                Ok(_) => println!("File copied successfully"),
                Err(err) => eprintln!("{}", err),
            }
        }
        "delete" => {
            if arguments.len() != 1 {
                eprintln!("usage: stash delete <file>");
                return;
            }
            let file = &arguments[0];

            //  Delete a file in the stash
            match stash.delete(file) {
                Ok(_) => println!("File deleted"),
                Err(err) => eprintln!("{}", err),
            }
        }
        "archive" => {
            if !arguments.is_empty() {
                eprintln!("usage: stash archive");
                return;
            }
            //  Create `.tar.gz` of stash contents
            match stash.archive() {
                Ok(_) => println!("Stash contents archived"),
                Err(err) => eprintln!("{}", err),
            }
        }
        "unpack" => {
            if !arguments.is_empty() {
                eprintln!("usage: stash unpack");
                return;
            }
            //  Create `.tar.gz` of stash contents
            match stash.unpack() {
                Ok(_) => println!("Stash archive unpacked"),
                Err(err) => eprintln!("{}", err),
            }
        }
        _ => {
            eprintln!("{}", USAGE);
            eprintln!("Unknown command: {}", command);
            exit(1);
        }
    }
}

//  Return `true` if `user` exists on the local system
fn user_exists(user: &str) -> bool {
    Command::new("id")
        .arg(user)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

//  TODO: add Result return type
//  Create `stash` user with home directory at `/home/$USER/.stash`
fn create_user(existing_user: &str, stash_user: &str) {
    let user_home = env::var("HOME").expect("Failed to retrieve home directory");
    let stash_path = Path::new(&user_home).join(".stash");

    //  Create `stash` user
    let useradd = Command::new("sudo")
        .args([
            "useradd",
            "-m",
            "-G",
            existing_user,
            "-d",
            &stash_path.to_string_lossy(),
            stash_user,
        ])
        .output()
        .expect("Failed to create user");

    if !useradd.status.success() {
        let err_msg = String::from_utf8_lossy(&useradd.stderr);
        eprintln!("Error creating user: {}", err_msg);
        exit(1);
    }
    //  Set password for `stash` user
    let passwd = Command::new("sudo")
        .args(["passwd", stash_user])
        .status()
        .expect("Failed to execute sudo");

    if !passwd.success() {
        eprintln!("Error setting password for user {}", stash_user);
        exit(1);
    }
    //  End `sudo` session
    Command::new("sudo")
        .arg("-k")
        .status()
        .expect("Failed to manually terminate sudo session");
}

//  Log in as `stash` user and re-execute the program with same `args`.
fn run_as_stash(stash_user: &str, args: Vec<String>) -> Result<(), io::Error> {
    let current_exe = env::current_exe().expect("Failed to get current executable path");

    Command::new("su")
        .arg(stash_user)
        .arg("-c")
        .arg(format!("{} {}", current_exe.display(), args.join(" ")))
        .status()
        .expect("Failed to execute program as stash user");

    Ok(())
}
