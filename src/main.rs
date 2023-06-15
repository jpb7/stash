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

/// The main entry point of the stash program.
///
/// This function parses command line arguments, authenticates as the `stash` user,
/// creates the `stash` user if it doesn't exist, and executes stash operations based
/// on the specified command and arguments.
///
/// # Notes
///
/// - This function assumes that the `stash` user already exists on the system.
///   If the `stash` user doesn't exist, it will attempt to create the user using
///   the `create_user` function. Please make sure that the necessary permissions
///   are in place to create a new user.
/// - The execution of stash operations is performed based on the user's privilege.
///   If the current user is the `stash` user, the operations will be executed directly.
///   If the current user is different, the operations will be executed as the `stash` user
///   using the `run_as_stash` function.
/// - The function assumes that the `Stash` struct is properly initialized and can be used
///   to perform the stash operations. Please ensure that the `Stash` struct is correctly
///   implemented and initialized before invoking the main function.
///
fn main() {
    //  Parse command line arguments
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    if cli_args.is_empty() {
        eprintln!("{}", USAGE);
        exit(1);
    }

    //  Authenticate as `stash` user
    let stash_user = "stash";
    let current_user = match env::var("USER") {
        Ok(user) => user,
        Err(_) => {
            eprintln!("{} Failed to detect current user", ERR);
            exit(1);
        }
    };

    //  Create `stash` user if it doesn't exist
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
    match cmd.as_str() {
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

/// Check if a user exists on the local system. Typically just used for `stash` user.
///
/// This function checks whether the specified `user` exists on the local system by executing
/// the `id` command with the user's name as an argument. If the `id` command succeeds and
/// returns a successful status code, it means that the user exists. Otherwise, it is assumed
/// that the user does not exist.
///
/// # Arguments
///
/// * `user` - The username to check for existence.
///
/// # Returns
///
/// Returns `true` if the user exists on the local system, `false` otherwise.
///
/// # Examples
///
/// ```rust
///     let user = "john";
///
///     if user_exists(user) {
///         println!("The user {} exists.", user);
///     } else {
///         println!("The user {} does not exist.", user);
///     }
/// }
/// ```
///
/// # Errors
///
/// This function does not return any errors. If there is a problem executing the `id` command,
/// an error message will be printed to the standard error stream, but the function will still
/// return `false`.
///
/// # Notes
///
/// This function relies on the availability of the `id` command and assumes that the execution
/// environment has the necessary privileges to run the command. If these assumptions are not valid
/// in your specific environment, you may need to modify the implementation accordingly.
///
fn user_exists(user: &str) -> bool {
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

/// Create the `stash` user with the home directory located at `/home/$USER/.stash`.
///
/// This function creates the `stash` user with the specified username and sets its home directory
/// to `/home/$USER/.stash`, where `$USER` is the username of the existing user. The `stash` user
/// is typically used by the stash application to store encrypted files and perform secure operations.
///
/// # Arguments
///
/// * `existing_user` - The username of an existing user that will be added to the `stash` user's group.
/// * `stash_user` - The desired username for the `stash` user.
///
/// # Returns
///
/// Returns `Ok(())` if the user creation is successful. Otherwise, returns an `Error` indicating the failure.
///
/// # Examples
///
/// ```rust
/// fn main() -> Result<(), std::io::Error> {
///     let existing_user = "admin";
///     let stash_user = "stash";
///
///     // Create the `stash` user
///     create_user(existing_user, stash_user)?;
///
///     // The rest of the program logic goes here...
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// This function can return an `Error` if there is a problem executing the necessary commands to create
/// the `stash` user or set its password. The specific error details will be provided in the `Error` value.
///
/// # Security Considerations
///
/// Creating a user and setting its password require elevated privileges. Ensure that proper security measures
/// are in place and validate user input to prevent unauthorized access and potential security vulnerabilities.
///
/// # Notes
///
/// This function assumes that the execution environment has the necessary privileges and commands (`useradd` and `passwd`)
/// to create the `stash` user and set its password. It also assumes that the home directory of the existing user
/// can be obtained using `env::var("HOME")`. If these assumptions are not valid in your specific environment,
/// you may need to modify the implementation accordingly.
///
fn create_user(existing_user: &str, stash_user: &str) -> Result<(), Error> {
    let user_home = env::var("HOME").map_err(|err| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to retrieve home directory: {}", err),
        )
    })?;
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

    //  Set password for `stash` user
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

/// Log in as the `stash` user and re-execute the program with the same arguments originally
/// passed from the command line.
///
/// This function allows you to run the program as the `stash` user by using `sudo` to execute
/// the current executable with the specified arguments.
///
/// # Arguments
///
/// * `stash_user` - The username of the `stash` user.
/// * `args` - A vector of `String` arguments to be passed to the re-executed program.
///
/// # Returns
///
/// Returns `Ok(())` if the re-execution is successful. Otherwise, returns an `io::Error`
/// indicating the failure.
///
/// # Errors
///
/// This function can return an `io::Error` if there is a problem executing the `sudo` command
/// or if the re-execution as the `stash` user fails.
///
/// # Security Considerations
///
/// Running the program as the `stash` user using `sudo` grants elevated privileges. Ensure
/// that proper security measures are in place and validate user input to prevent unauthorized
/// access and potential security vulnerabilities.
///
/// # Notes
///
/// This function assumes that the current executable path can be obtained using
/// `env::current_exe()`. If this assumption is not valid in your specific environment,
/// you may need to modify the implementation accordingly.
///
/// This function requires the execution environment to have `sudo` installed and properly
/// configured to allow execution as the `stash` user.
///
fn run_as_stash(stash_user: &str, args: Vec<String>) -> Result<(), io::Error> {
    let current_exe = env::current_exe().map_err(|err| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to get current executable path: {}", err),
        )
    })?;

    //  Build `sudo` command to re-execute, and pass it CLI args
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
