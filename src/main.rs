//! stash
//!
//! Command-line utility for managing a stash of encrypted files.
//!
//! Usage: stash <command> [<args>]
//!
//! Available commands:
//!   - add [-c] <file>: Encrypt a file and add it to the stash (optionally copy it).
//!   - grab [-c] <file>: Decrypt a file from the stash and drop it in the current directory (optionally copy it).
//!   - delete <file>: Delete a stashed file.
//!   - list: List the contents of the stash.
//!   - archive: Create a compressed tarball from stash contents.
//!   - unpack: Unpack archive of stash contents.
//!
//! Example usage:
//! ```shell
//! $ stash add secret_file.txt
//! $ stash add -c secret_file.txt
//! $ stash grab secret_file.txt
//! $ stash grab -c secret_file.txt
//! $ stash delete secret_file.txt
//! $ stash list
//! $ stash archive
//! $ stash unpack
//! ```
//!
//! For more information, refer to the documentation of each command and its respective functions.
//!
//! Authors: Jacob Bentley,
//!          Richard Duffy

use stash::*;
use std::{
    env,
    io::{self, Error, ErrorKind},
    path::Path,
    process::{exit, Command, Stdio},
};

const USAGE: &str = "usage: stash <command> [<args>]";
const ERR: &str = "stash: error:";

fn main() {
    //
    //  Parse command line arguments
    //
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    if cli_args.is_empty() {
        eprintln!("{}", USAGE);
        exit(1);
    }

    //  Authenticate as `stash` user
    //
    let stash_user = "stash";
    let current_user = match env::var("USER") {
        Ok(user) => user,
        Err(_) => {
            eprintln!("{} Failed to detect current user", ERR);
            exit(1);
        }
    };

    //  Create `stash` user if it doesn't exist
    //
    if !user_exists(stash_user) {
        match create_user(&current_user, stash_user) {
            Ok(_) => (),
            Err(msg) => {
                eprintln!("{} Failed to create `stash` user: {}", ERR, msg);
                exit(1);
            }
        }
    }

    //  Only execute `stash` operations as `stash` user
    //
    if current_user != stash_user {
        match run_as_stash(stash_user, cli_args) {
            Ok(_) => exit(0),
            Err(msg) => {
                eprintln!("Failed to run program as `stash` user: {}", msg);
                exit(1);
            }
        }
    }

    //  Execute main program
    //
    let mut stash = match Stash::new() {
        Ok(stash) => stash,
        Err(msg) => {
            eprintln!("Failed to initialize `stash` object: {}", msg);
            exit(1);
        }
    };

    let cmd = &cli_args[0];
    let args = &cli_args[1..];

    //  Handle different commands and arguments from CLI
    //
    match cmd.as_str() {
        //
        "add" => {
            if args.len() != 1 && args.len() != 2 {
                eprintln!("usage: stash add [-c] <file>");
                exit(1);
            }
            let (file, option) = match args.len() {
                1 => (&args[0], false),
                2 => {
                    let flag = args[0] == "-c";
                    (&args[1], flag)
                }
                _ => {
                    eprintln!("{} Unable to parse arguments", ERR);
                    exit(1);
                }
            };
            //  Encrypt file and add it to stash
            //
            match stash.add(file, option) {
                Ok(_) => {}
                Err(msg) => eprintln!("{} {}", ERR, msg),
            }
        }
        "grab" => {
            if args.len() != 1 && args.len() != 2 {
                eprintln!("usage: stash grab [-c] <file>");
                return;
            }
            let (file, option) = match args.len() {
                1 => (&args[0], false),
                2 => {
                    let flag = args[0] == "-c";
                    (&args[1], flag)
                }
                _ => {
                    eprintln!("{} Unable to parse arguments", ERR);
                    exit(1);
                }
            };
            //  Decrypt file and drop in current directory
            //
            match stash.grab(file, option) {
                Ok(_) => {}
                Err(msg) => eprintln!("{} {}", ERR, msg),
            }
        }
        "delete" => {
            if args.len() != 1 {
                eprintln!("usage: stash delete <file>");
                exit(1);
            }
            let file = &args[0];

            //  Delete a file in the stash
            //
            match stash.delete(file) {
                Ok(_) => {}
                Err(msg) => {
                    eprintln!("{} {}", ERR, msg);
                    exit(1);
                }
            }
        }
        "list" => {
            if !args.is_empty() {
                eprintln!("usage: stash list");
                exit(1);
            }
            //  Display contents of stash
            //
            match stash.list() {
                Ok(contents) => println!("{}", contents),
                Err(msg) => {
                    eprintln!("{} {}", ERR, msg);
                    exit(1);
                }
            }
        }
        "archive" => {
            if !args.is_empty() {
                eprintln!("usage: stash archive");
                exit(1);
            }
            //  Create `.tar.gz` of stash contents
            //
            match stash.archive() {
                Ok(_) => {}
                Err(msg) => {
                    eprintln!("{} {}", ERR, msg);
                    exit(1);
                }
            }
        }
        "unpack" => {
            if !args.is_empty() {
                eprintln!("usage: stash unpack");
                exit(1);
            }
            //  Unpack `.tar.gz` of stash contents
            //
            match stash.unpack() {
                Ok(_) => {}
                Err(msg) => {
                    eprintln!("{} {}", ERR, msg);
                    exit(1);
                }
            }
        }
        _ => {
            eprintln!("{}", USAGE);
            eprintln!("Unknown command: {}", cmd);
            exit(1);
        }
    }
}

/// Check if a user (ie. stash) exists on the local system.
///
fn user_exists(user: &str) -> bool {
    //
    //  Use Linux `id` command to check for user
    //
    let id = Command::new("id")
        .arg(user)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match id {
        Ok(status) => status.success(),
        Err(err) => {
            eprintln!("Failed to execute `id` command: {}", err);
            false
        }
    }
}

/// Create `stash` user with home directory at `/home/$USER/.stash`.
///
fn create_user(existing_user: &str, stash_user: &str) -> Result<(), Error> {
    //
    let user_home = env::var("HOME").map_err(|err| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to retrieve home directory: {}", err),
        )
    })?;
    let stash_path = Path::new(&user_home).join(".stash");

    //  Create `stash` user: same group as `$USER`
    //
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
        .map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to execute `useradd` command: {}", err),
            )
        })?;

    if !useradd.status.success() {
        let err = String::from_utf8_lossy(&useradd.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Error creating user: {}", err),
        ));
    }

    //  Set password for `stash` user (prompt)
    //
    let passwd = Command::new("sudo")
        .args(["passwd", stash_user])
        .status()
        .map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to execute 'passwd': {}", err),
            )
        })?;

    if !passwd.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Error setting password for user {}", stash_user),
        ));
    }

    Ok(())
}

/// Re-execute as `stash` with the same args passed in originally.
///
fn run_as_stash(stash_user: &str, args: Vec<String>) -> Result<(), io::Error> {
    //
    //  Find source of the binary currently running
    //
    let current_exe = env::current_exe().map_err(|err| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get current executable path: {}", err),
        )
    })?;

    //  Build `sudo` command to execute as `stash`
    //
    let mut command = Command::new("sudo");
    command.arg("-u").arg(stash_user).arg(current_exe);
    for arg in args {
        command.arg(arg);
    }

    let status = command.status().map_err(|err| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to execute `sudo` command: {}", err),
        )
    })?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Failed to execute as `stash` user (exit code: {})", status),
        ));
    }

    Ok(())
}
