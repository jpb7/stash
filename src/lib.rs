//! # Stash Crate
//!
//! The `stash` crate provides functionality to manage an encrypted stash of files. It allows you to
//! create, add, grab, archive, and list files within the stash directory. The stash utilizes the
//! AES-256-GCM encryption algorithm for secure file encryption.
//!
//! ## Usage
//!
//! The crate exposes a `Stash` struct that represents the stash directory. You can create a new
//! stash using the `Stash::new` function, add files to the stash using the `add` function, grab a
//! copy of a file from the stash using the `grab` function, archive the stash contents using the
//! `archive` function, and list the contents of the stash using the `list` function.
//!
//! Here's an example demonstrating the basic usage of the `stash` crate:
//!
//! ```rust
//! use stash::Stash;
//!
//! fn main() -> Result<(), stash::Error> {
//!     // Create a new stash
//!     let stash = Stash::new()?;
//!
//!     // Add a file to the stash
//!     stash.add("file.txt", false)?;
//!
//!     // Grab a copy of a file from the stash
//!     stash.grab("file.txt", true)?;
//!
//!     // Archive the stash contents
//!     stash.archive()?;
//!
//!     // List the contents of the stash
//!     stash.list()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! For more detailed usage instructions and API documentation, please refer to the individual
//! module documentation and function comments.
//!
//! ## Features
//!
//! - File encryption and decryption using AES-256-GCM algorithm.
//! - Archive creation and extraction of stash contents.
//! - Database storage for encrypted secrets.
//!
//! ## Dependencies
//!
//! The `stash` crate relies on the following external dependencies:
//!
//! - `aes_gcm` (version 0.10.0) for AES-256-GCM encryption support.
//! - `linux_keyutils` (version 0.6.2) for keyring management on Linux systems.
//! - `sled` (version 0.34.1) for database storage.
//!
//! Please refer to the individual module documentation for more information on each dependency.
//!
//! ## License
//!
//! This crate is distributed under the terms of the MIT license. See the [LICENSE](./LICENSE) file
//! for details.
//!
//! ---
//!
//! This documentation was generated with the assistance of [ChatGPT](https://github.com/openai/gpt-3.5-turbo)
//! and should be reviewed and updated as necessary.
//!
//! Last update: June 14, 2023
//!

use aes_gcm::{
    aead::{generic_array::GenericArray, AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm,
};
use linux_keyutils::{KeyRing, KeyRingIdentifier};
use serde_derive::{self, Deserialize, Serialize};
use sled::{self, Config, Db};
use std::{
    env, fs,
    io::{self, Error, ErrorKind, Read, Seek, Write},
    path::{Path, PathBuf},
    process::Command,
};
#[cfg(test)]
use tempfile::TempDir;
//use zeroize::Zeroize;

//  TODO: find a way to test this
#[allow(unused_macros)]
macro_rules! zeroize_all {
    ($($arg:expr),*) => {
        $(
            $arg.zeroize();
        )*
    };
}

//  TODO: zeroize on drop
/// Represents a secret consisting of a key and a nonce.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Secret {
    key: Vec<u8>,
    nonce: Vec<u8>,
}
impl Secret {
    /// Creates a new `Secret` with randomly generated key and nonce.
    fn new() -> Self {
        Secret {
            key: Aes256Gcm::generate_key(OsRng).to_vec(),
            nonce: Aes256Gcm::generate_nonce(OsRng).to_vec(),
        }
    }

    /// Creates a `Secret` from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `secret` - A byte slice representing the secret. It is expected to be
    ///              of length 64, where the first 32 bytes are the key and the
    ///              remaining 32 bytes are the nonce.
    ///
    /// # Panics
    ///
    /// This function will panic if the `secret` slice does not have a length of 64.
    ///
    fn from(secret: &[u8]) -> Self {
        Secret {
            key: secret[..32].to_vec(),
            nonce: secret[32..].to_vec(),
        }
    }

    /// Joins the key and nonce of the `Secret` into a single byte vector.
    ///
    /// The resulting byte vector contains the key followed by the nonce.
    ///
    /// # Returns
    ///
    /// A byte vector containing the key and nonce joined together.
    ///
    fn join(&self) -> Vec<u8> {
        let mut secret = Vec::with_capacity(self.key.len() + self.nonce.len());
        secret.extend_from_slice(&self.key);
        secret.extend_from_slice(&self.nonce);

        secret
    }

    /// Splits the `Secret` into its key and nonce components.
    ///
    /// # Returns
    ///
    /// A tuple containing the key and nonce as separate byte vectors.
    ///
    fn split(&self) -> (Vec<u8>, Vec<u8>) {
        (self.key.clone(), self.nonce.clone())
    }
}

//  TODO: zeroize on drop
/// Represents a stash that holds encrypted files.
#[derive(Debug, Clone)]
pub struct Stash {
    path: PathBuf,
    contents: PathBuf,
    is_archived: bool,
    keyring: KeyRing,
    //secret: Secret,
    db: Db,
}

impl Default for Stash {
    /// Creates a new `Stash` with default configuration.
    ///
    /// This is the default implementation of the `Default` trait for `Stash`. It simply
    /// calls the `new()` function to create a new `Stash` instance with the default
    /// configuration.
    ///
    /// # Returns
    ///
    /// A `Stash` instance with default configuration.
    ///
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Stash {
    /// Creates a new instance of the `Stash` struct.
    ///
    /// This function initializes a new `Stash` object with default settings and returns it wrapped in a
    /// `Result`. The `Stash` represents a stash directory and provides methods to manage files within
    /// the stash.
    ///
    /// # Returns
    ///
    /// A `Result` containing the newly created `Stash` object if the initialization is successful
    /// (`Ok`), or an error (`Err`) if any of the initialization steps fail.
    ///
    /// # Errors
    ///
    /// This function can return an error if there is a failure in retrieving the `HOME` environment
    /// variable, constructing the stash directory path, checking the existence of the `contents` file,
    /// creating the path to the `.db` file, creating the session-based `KeyRing`, or loading the stash
    /// database from the `.db` file. The error messages provide details about the specific failures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::stash::Stash;
    ///
    /// // Create a new stash object
    /// let stash = Stash::new()?;
    /// ```
    ///
    pub fn new() -> Result<Self, Error> {
        let home = env::var("HOME").map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to get `HOME` environment variable: {}", err),
            )
        })?;

        let path = PathBuf::from(&home);
        let contents = path.join("contents");
        let mut is_archived = false;

        if contents.exists() {
            is_archived = true;
        }

        //let secret_path = path.join(".secret");
        let db_path = path.join(".db");
        //  TODO: set up session-based encryption/decryption
        let keyring = KeyRing::from_special_id(KeyRingIdentifier::Session, false).unwrap();
        //let secret = Self::get_secret(&secret_path);
        let db = Self::get_db(&db_path)?;

        Ok(Stash {
            path,
            contents,
            is_archived,
            keyring,
            //secret,
            db,
        })
    }

    /// Retrieves or creates a `sled` database at the specified path.
    ///
    /// This function takes a path to the database file and returns a `Db` instance. If the
    /// database file exists at the specified path, it opens the existing database. If the
    /// file does not exist, it creates a new database at the specified path.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The path to the database file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Db` instance if the database retrieval or creation was
    /// successful. If an error occurs, it returns an `Err` with the corresponding error message.
    ///
    /// # Errors
    ///
    /// This function can return various errors, including:
    /// - If the provided `db_path` is invalid or cannot be converted to a valid string.
    /// - If there is an error opening an existing database.
    /// - If there is an error creating a new database.
    ///
    fn get_db(db_path: &Path) -> Result<Db, Error> {
        let db_str = db_path
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Invalid database path"))?;

        if db_path.exists() {
            sled::open(db_str).map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to open database: {}", err),
                )
            })
        } else {
            let config = Config::new().path(db_str);
            config.open().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to create database: {}", err),
                )
            })
        }
    }

    /*  TODO: set up session-based encryption/decryption
    // Read stash-level secrets from hidden file
    fn get_secret(secrets: &Path) -> Secret {
        let mut secrets_file;
        let mut secrets_raw = Vec::new();

        if !secrets.exists() {
            let secret = Secret::new();
            secrets_file = fs::File::create(secrets).expect("Failed to create secrets file");
            secrets_file
                .write_all(&secret.join())
                .expect("Failed to write stash key");
            secret
        } else {
            secrets_file = fs::File::open(secrets).expect("Failed to open secrets file");
            secrets_file
                .read_to_end(&mut secrets_raw)
                .expect("Failed to retrieve stash secrets");
            Secret::from(&secrets_raw)
        }
    }
    */

    /// Add a file to the stash, optionally as a copy.
    ///
    /// This method adds the specified `file` to the stash. If the stash does not exist, it returns
    /// an error. If the stash is in archive mode, it also returns an error, and you need to call
    /// `stash unpack` before adding more files. The `file` MUST be a regular file, not a directory.
    ///
    /// If `copy` is set to `true`, the file is copied to the stash. Otherwise, it is moved to the
    /// stash by renaming it. After adding the file to the stash, it is encrypted using a new secret
    /// (key and nonce), which is then stored in the database. The file's description is its filename,
    /// and that description is used for retrieval both from the session keyring and the database.
    ///
    /// Note that encryption secrets are preferentially used in cache via Linux keyrings. The main
    /// purpose of the `sled` database is for persistent storage.
    ///
    /// # Arguments
    ///
    /// * `file` - The path to the file to be added to the stash.
    /// * `copy` - A flag indicating whether to copy the file (`true`) or move it (`false`).
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the addition was successful (`Ok`) or an error occurred (`Err`).
    ///
    /// # Errors
    ///
    /// This method can return various errors, including:
    /// - If the stash does not exist.
    /// - If the stash is in archive mode.
    /// - If the source file is a directory.
    /// - If the destination file already exists in the stash.
    /// - If there is an error encrypting the file.
    /// - If there is an error inserting the encryption key into the database.
    /// - If there is an error caching the encryption key in the keyring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use std::io::Error;
    ///
    /// let stash = Stash::new()?;
    /// stash.add("/path/to/file.txt", false)?;
    /// ```
    ///
    pub fn add(&mut self, file: &str, copy: bool) -> Result<(), Error> {
        if !self.path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "No stash found"));
        } else if self.is_archived {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Stash is in archive mode. Call `stash unpack` before adding more files",
            ));
        }

        let src_path = Path::new(file);
        if src_path.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Source file is a directory",
            ));
        }

        let dst_path = self.path.join(src_path.file_name().ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput, "Failed to resolve new file path")
        })?);

        if dst_path.exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "File already in stash",
            ));
        }

        let secret = Secret::new();
        let description = src_path.to_string_lossy().to_string();

        if copy {
            fs::copy(src_path, &dst_path)?;
        } else {
            fs::rename(src_path, &dst_path)?;
        }

        Self::encrypt(&dst_path, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to encrypt file: {}", err))
        })?;

        self.db
            .insert(description.as_bytes(), secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to add encryption key to database: {}", err),
                )
            })?;

        self.keyring
            .add_key(&description, &secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to cache encryption key: {}", err),
                )
            })?;

        //zeroize_all!(src_path, dst_path, secret, description, key);

        Ok(())
    }

    /// Move a file from the stash into the current directory.
    ///
    /// This method moves the specified `file` from the stash into the current directory. If the stash does not exist,
    /// it returns an error. If the stash is in archive mode and `copy` is set to `false`, it also returns an error,
    /// and you need to call `stash unpack` to unpack the stash before moving files. The `file` parameter specifies
    /// the name of the file to be grabbed from the stash.
    ///
    /// The file is decrypted using the secret key associated with the file's description. The secret key is retrieved
    /// either from the system key if it exists or from the database. If `copy` is set to `true`, the file is copied
    /// to the current directory. Otherwise, it is moved to the current directory. If the destination file already exists
    /// in the current directory, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `file` - The name of the file to be grabbed from the stash.
    /// * `copy` - A flag indicating whether to copy the file (`true`) or move it (`false`).
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the grab operation was successful (`Ok`) or an error occurred (`Err`).
    ///
    /// # Errors
    ///
    /// This method can return various errors, including:
    /// - If the stash does not exist.
    /// - If the stash is in archive mode and `copy` is set to `false`.
    /// - If the file or secret key is not found in the stash.
    /// - If there is an error decrypting the file.
    /// - If there is an error moving or copying the file.
    /// - If there is an error removing the file from the database.
    /// - If there is an error removing the cached key from the keyring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    ///
    /// let mut stash = Stash::new()?;
    /// stash.grab("file.txt", true)?;
    /// ```
    ///
    pub fn grab(&mut self, file: &str, copy: bool) -> Result<(), Error> {
        if !self.path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "No stash found"));
        } else if self.is_archived && !copy && file != "contents" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Stash is in archive mode. Use `stash unpack` to unpack",
            ));
        }

        let src_path = self.path.join(file);
        let dst_path = env::current_dir()?.join(file);
        let secret;

        if dst_path.exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "File already exists in current directory",
            ));
        }

        //  Get secret from sys key if it exists; otherwise, use db
        if let Ok(key) = self.keyring.search(file) {
            secret = Secret::from(&key.read_to_vec().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to read encryption key: {}", err),
                )
            })?);
            if !copy {
                key.invalidate().map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to remove cached key: {}", err),
                    )
                })?;
            }
            //key.zeroize();
        } else if let Some(value) = self.db.get(file)? {
            secret = Secret::from(&value);
        } else {
            return Err(Error::new(ErrorKind::NotFound, "Secret not found"));
        }

        Self::decrypt(&src_path, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to decrypt file: {}", err))
        })?;

        if copy {
            fs::copy(src_path, dst_path).map_err(|err| {
                Error::new(ErrorKind::Other, format!("Failed to copy file: {}", err))
            })?;
        } else {
            fs::rename(src_path, dst_path).map_err(|err| {
                Error::new(ErrorKind::Other, format!("Failed to move file: {}", err))
            })?;
            self.db.remove(file).map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to remove file from database: {}", err),
                )
            })?;
        }

        if !copy && file == "contents" {
            self.is_archived = false;
        }
        //zeroize_all!(src_path, dst_path, secret);

        Ok(())
    }

    /// Delete a file in the stash.
    ///
    /// This method deletes the specified `file` from the stash. If the stash does not exist, it returns an error.
    /// If the stash is in archive mode and the `file` is not "contents", it also returns an error, and you need
    /// to call `stash unpack` to unpack the stash before deleting files. The `file` parameter specifies the name
    /// of the file to be deleted from the stash.
    ///
    /// The file is permanently removed from the stash's directory, and its corresponding entry is removed from
    /// the database. If the file is cached in the keyring, the cached key is invalidated and removed.
    ///
    /// # Arguments
    ///
    /// * `file` - The name of the file to be deleted from the stash.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the deletion was successful (`Ok`) or an error occurred (`Err`).
    ///
    /// # Errors
    ///
    /// This method can return various errors, including:
    /// - If the stash does not exist.
    /// - If the stash is in archive mode and the file is not "contents".
    /// - If the file is not found in the stash.
    /// - If there is an error deleting the file.
    /// - If there is an error removing the file's entry from the database.
    /// - If there is an error removing the cached key from the keyring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    ///
    /// let mut stash = Stash::new()?;
    /// stash.delete("file.txt")?;
    /// ```
    ///
    pub fn delete(&mut self, file: &str) -> Result<(), Error> {
        if !self.path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "No stash found"));
        } else if self.is_archived && file != "contents" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Stash is in archive mode. Use `stash unpack` to unpack",
            ));
        } else if file == ".db" || file == ".secret" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Cannot delete program file {}", file),
            ));
        }

        let target_path = self.path.join(file);
        if !target_path.exists() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "File not found in stash",
            ));
        }

        fs::remove_file(
            target_path
                .to_str()
                .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Failed to parse file path"))?,
        )?;

        self.db.remove(file).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to remove file from database: {}", err),
            )
        })?;

        if let Ok(key) = self.keyring.search(file) {
            key.invalidate().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to remove key from cache: {}", err),
                )
            })?;
        }

        if file == "contents" {
            self.is_archived = false;
        }

        Ok(())
    }

    /// List all files in the stash directory.
    ///
    /// This method lists all the files in the stash directory and returns their names as a string.
    /// If the stash directory does not exist, it returns an error.
    ///
    /// The file listing is obtained by executing the `ls` command on the stash directory. The output
    /// of the command is captured and converted to a string. The resulting string contains the names
    /// of the files in the stash directory, separated by newlines.
    ///
    /// # Returns
    ///
    /// A `Result` containing the file listing as a string if successful (`Ok`), or an error (`Err`) if
    /// the stash directory does not exist or if there was an error executing the `ls` command.
    ///
    /// # Errors
    ///
    /// This method can return various errors, including:
    /// - If the stash directory does not exist.
    /// - If there is an error executing the `ls` command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    ///
    /// let stash = Stash::new()?;
    /// let file_list = stash.list()?;
    /// println!("Files in stash: {}", file_list);
    /// ```
    ///
    pub fn list(&self) -> Result<String, Error> {
        if !self.path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "No stash found"));
        }

        let ls_output =
            Command::new("ls")
                .arg(self.path.to_str().ok_or_else(|| {
                    Error::new(ErrorKind::Other, "Failed to convert path to string")
                })?)
                .output()
                .map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to execute `ls` command: {}", err),
                    )
                })?
                .stdout;

        let contents = String::from_utf8_lossy(&ls_output).trim().to_string();

        Ok(contents)
    }

    /// Create a tarball from the current stash contents.
    ///
    /// This method creates a tarball from the files in the stash directory and encrypts it using a
    /// new secret (key and nonce). The encrypted tarball is then stored in the stash directory as
    /// the stash's contents. Additionally, the encryption secret is stored in the database and
    /// cached in the keyring for future retrieval.
    ///
    /// If the stash directory does not exist, an error is returned. If the stash is already archived,
    /// indicating that an archive already exists, it returns an error. If there are no files in the
    /// stash (the `.db` is empty), it returns an error as well.
    ///
    /// The tarball is created by calling the `create_tarball` method, which performs the necessary
    /// steps to generate a tarball from the stash contents. The resulting tarball is then encrypted
    /// using the newly generated secret. The description of the tarball, which is used for retrieval
    /// purposes, is derived from the tarball's file name.
    ///
    /// After the encryption and storage steps are completed, the stash's `is_archived` flag is set
    /// to `true` to indicate that an archive exists.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the archiving process was successful (`Ok`) or an error occurred
    /// (`Err`).
    ///
    /// # Errors
    ///
    /// This method can return various errors, including:
    /// - If the stash directory does not exist.
    /// - If the stash is already archived.
    /// - If there are no files in the stash.
    /// - If there is an error creating the tarball.
    /// - If there is an error encrypting the tarball.
    /// - If there is an error inserting the encryption secrets into the database.
    /// - If there is an error caching the encryption secrets in the keyring.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    ///
    /// let stash = Stash::new()?;
    /// stash.archive()?;
    /// ```
    ///
    pub fn archive(&mut self) -> Result<(), Error> {
        if !self.path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "No stash found"));
        } else if self.is_archived {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Archive already exists",
            ));
        } else if self.db.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "No files in stash: .db is empty",
            ));
        }

        let file_name = self.contents.file_name().ok_or(Error::new(
            ErrorKind::InvalidData,
            "Failed to get file name",
        ))?;

        let description = file_name.to_string_lossy().to_string();
        let secret = Secret::new();

        self.create_tarball().map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create tarball: {}", err),
            )
        })?;

        Self::encrypt(&self.contents, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to encrypt file: {}", err))
        })?;

        self.db
            .insert(description.as_bytes(), secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to add encryption secrets to database: {}", err),
                )
            })?;

        self.keyring
            .add_key(&description, &secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to cache encryption secrets: {}", err),
                )
            })?;

        self.is_archived = true;
        //zeroize_all!(description, secret);

        Ok(())
    }

    /// Extract the `contents` file from the stash archive.
    ///
    /// This method extracts the `contents` file from the stash archive. It first checks if the stash
    /// directory exists and if the stash is archived. If the stash directory doesn't exist, an error
    /// is returned. If the stash is not archived, indicating that no archive exists, it returns an
    /// error as well.
    ///
    /// The `contents` file is extracted by performing the following steps:
    /// 1. The file name of the archive is obtained from the `contents` path.
    /// 2. The description, used for retrieval purposes, is derived from the file name.
    /// 3. The encryption secret associated with the description is retrieved from either the keyring
    ///    or the database. If the secret is found in the keyring, it is read and cached, and the key
    ///    is invalidated. If it is found in the database, it is retrieved directly.
    /// 4. The `contents` file is decrypted using the retrieved secret.
    /// 5. The archive is extracted by calling the `extract_tarball` method, which handles the process
    ///    of extracting the archive contents.
    /// 6. The `contents` file is removed from the stash directory.
    /// 7. The encryption secret is removed from the database.
    /// 8. The `is_archived` flag is set to `false` to indicate that the stash is no longer archived.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the unpacking process was successful (`Ok`) or an error occurred
    /// (`Err`).
    ///
    /// # Errors
    ///
    /// This method can return various errors, including:
    /// - If the stash directory does not exist.
    /// - If the stash is not archived.
    /// - If the file name of the archive cannot be obtained.
    /// - If the encryption secret is not found in the keyring or the database.
    /// - If there is an error reading the encryption secrets from the keyring.
    /// - If there is an error decrypting the `contents` file.
    /// - If there is an error extracting the archive.
    /// - If there is an error removing the `contents` file.
    /// - If there is an error removing the encryption secrets from the database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Error;
    ///
    /// let stash = Stash::new()?;
    /// stash.unpack()?;
    /// ```
    ///
    pub fn unpack(&mut self) -> Result<(), Error> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        } else if !self.is_archived {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No archive exists",
            ));
        }

        let file_name = self.contents.file_name().ok_or(Error::new(
            ErrorKind::InvalidData,
            "Failed to get file name",
        ))?;

        let description = file_name.to_string_lossy().to_string();
        let secret;

        if let Ok(key) = self.keyring.search(&description) {
            let key_bytes = key.read_to_vec().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to read encryption secrets: {}", err),
                )
            })?;
            secret = Secret::from(&key_bytes);
            key.invalidate().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to remove encryption secrets from cache: {}", err),
                )
            })?;
            //key.zeroize();
        } else if let Some(value) = self.db.get(&description)? {
            secret = Secret::from(&value);
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Secret not found"));
        }

        Self::decrypt(&self.contents, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to decrypt file: {}", err))
        })?;

        self.extract_tarball().map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to extract archive: {}", err),
            )
        })?;

        fs::remove_file(&self.contents).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to remove `contents` file: {}", err),
            )
        })?;

        self.db.remove(description).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to remove encryption secrets from database: {}", err),
            )
        })?;

        self.is_archived = false;
        //zeroize_all!(tarball, secret);

        Ok(())
    }

    /// Encrypts a specified file in place using the provided secret.
    ///
    /// This function encrypts the file located at the specified `path` using the provided `secret`.
    /// The file is encrypted in place, meaning the original file is overwritten with the encrypted
    /// data. The encryption process involves the following steps:
    ///
    /// 1. The file is opened with read and write access.
    /// 2. The contents of the file are read into a buffer.
    /// 3. The encryption key and nonce are obtained from the `secret`.
    /// 4. The buffer is encrypted using the AES-256-GCM encryption algorithm with the key and nonce.
    /// 5. The encrypted data is written back to the file, overwriting its original contents.
    /// 6. The file length is set to the size of the encrypted data.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the path of the file to encrypt.
    /// * `secret` - A reference to the encryption secret to use for encryption.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the encryption process was successful (`Ok`) or an error occurred
    /// (`Err`).
    ///
    /// # Errors
    ///
    /// This function can return various errors, including:
    /// - If the file cannot be opened for reading and writing.
    /// - If there is an error reading the file.
    /// - If there is an error encrypting the file.
    /// - If there is an error seeking the file.
    /// - If there is an error writing the encrypted data to the file.
    /// - If there is an error setting the file length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use std::io::Error;
    ///
    /// let path = Path::new("file.txt");
    /// let secret = Secret::new();
    ///
    /// encrypt(path, &secret)?;
    /// ```
    ///
    fn encrypt(path: &Path, secret: &Secret) -> Result<(), Error> {
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let (key, nonce) = secret.split();
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read file: {}", err),
            )
        })?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let result = cipher.encrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut buffer);

        if let Err(err) = result {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to encrypt file: {}", err),
            ));
        }

        file.seek(io::SeekFrom::Start(0)).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to seek file: {}", err),
            )
        })?;
        file.write_all(&buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write encrypted data to file: {}", err),
            )
        })?;
        file.set_len(buffer.len() as u64).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to set file length: {}", err),
            )
        })?;

        Ok(())
    }

    /// Decrypts a file in place using the provided secret.
    ///
    /// This function decrypts the file located at the specified `path` using the provided `secret`.
    /// The file is decrypted in place, meaning the original encrypted file is overwritten with the
    /// decrypted data. The decryption process involves the following steps:
    ///
    /// 1. The file is opened with read and write access.
    /// 2. The contents of the file are read into a buffer.
    /// 3. The decryption key and nonce are obtained from the `secret`.
    /// 4. The buffer is decrypted using the AES-256-GCM decryption algorithm with the key and nonce.
    /// 5. The decrypted data is written back to the file, overwriting the original encrypted contents.
    /// 6. The file length is set to the size of the decrypted data.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the path of the file to decrypt.
    /// * `secret` - A reference to the encryption secret to use for decryption.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the decryption process was successful (`Ok`) or an error occurred
    /// (`Err`).
    ///
    /// # Errors
    ///
    /// This function can return various errors, including:
    /// - If the file cannot be opened for reading and writing.
    /// - If there is an error reading the file.
    /// - If there is an error decrypting the file.
    /// - If there is an error seeking the file.
    /// - If there is an error writing the decrypted data to the file.
    /// - If there is an error setting the file length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use std::io::Error;
    ///
    /// let path = Path::new("file.txt");
    /// let secret = Secret::new();
    ///
    /// decrypt(path, &secret)?;
    /// ```
    ///
    fn decrypt(path: &Path, secret: &Secret) -> Result<(), Error> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|err| Error::new(ErrorKind::Other, format!("Failed to open file: {}", err)))?;

        let (key, nonce) = secret.split();
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read file: {}", err),
            )
        })?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let result = cipher.decrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut buffer);

        if let Err(err) = result {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to decrypt file: {}", err),
            ));
        }

        file.seek(io::SeekFrom::Start(0)).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to seek file: {}", err),
            )
        })?;
        file.write_all(&buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write file: {}", err),
            )
        })?;
        file.set_len(buffer.len() as u64).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to set file length: {}", err),
            )
        })?;

        Ok(())
    }

    /// Creates a `.tar.gz` archive of the stash contents.
    ///
    /// This function creates a compressed `.tar.gz` archive of the stash contents. The archive is
    /// created using the `tar` command line utility. The following steps are performed during the
    /// archive creation:
    ///
    /// 1. The `tar` command is executed with the following arguments:
    ///    - `sh -c` to execute the command in a subshell.
    ///    - `cd` to change the directory to the stash location.
    ///    - `tar czf contents --remove-files ./*` to create the archive named `contents.tar.gz` and
    ///      remove the original files.
    /// 2. The output of the `tar` command is captured.
    /// 3. If the `tar` command is successful, the function returns `Ok(())`.
    /// 4. If the `tar` command fails, the error message from the stderr output is printed to the
    ///    console, and an error is returned with the corresponding error message.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the archive creation was successful (`Ok`) or an error occurred
    /// (`Err`).
    ///
    /// # Errors
    ///
    /// This function can return an error if there is a failure in executing the `tar` command or if the
    /// command does not exit successfully. The error message from the `tar` command is included in the
    /// returned `Error`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let stash = Stash::new("/path/to/stash");
    ///
    /// stash.create_tarball()?;
    /// ```
    ///
    fn create_tarball(&self) -> Result<(), Error> {
        let tar = Command::new("sh")
            .arg("-c")
            .arg("cd && tar czf contents --remove-files ./*")
            .output()
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to execute `tar` command: {}", err),
                )
            })?;

        if !tar.status.success() {
            let err_msg = String::from_utf8_lossy(&tar.stderr);
            eprintln!("{}", err_msg);
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to create tar archive: {}", err_msg),
            ));
        }

        Ok(())
    }

    /// Extracts a `.tar.gz` archive of the stash contents.
    ///
    /// This function extracts the contents of a compressed `.tar.gz` archive of the stash. The archive
    /// is extracted using the `tar` command line utility. The following steps are performed during the
    /// extraction:
    ///
    /// 1. The `tar` command is executed with the following arguments:
    ///    - `sh -c` to execute the command in a subshell.
    ///    - `cd` to change the directory to the stash location.
    ///    - `tar xzf contents` to extract the archive named `contents.tar.gz`.
    /// 2. The output of the `tar` command is captured.
    /// 3. If the `tar` command is successful, the function returns `Ok(())`.
    /// 4. If the `tar` command fails, the error message from the stderr output is printed to the
    ///    console, and an error is returned with the corresponding error message.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the archive extraction was successful (`Ok`) or an error occurred
    /// (`Err`).
    ///
    /// # Errors
    ///
    /// This function can return an error if there is a failure in executing the `tar` command or if the
    /// command does not exit successfully. The error message from the `tar` command is included in the
    /// returned `io::Error`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let stash = Stash::new("/path/to/stash");
    ///
    /// stash.extract_tarball()?;
    /// ```
    ///
    fn extract_tarball(&self) -> Result<(), io::Error> {
        let tar = Command::new("sh")
            .arg("-c")
            .arg("cd && tar xzf contents")
            .output()
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to execute `tar` command: {}", err),
                )
            })?;

        if !tar.status.success() {
            let err_msg = String::from_utf8_lossy(&tar.stderr);
            eprintln!("{}", err_msg);
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to unpack tar archive: {}", err_msg),
            ));
        }

        Ok(())
    }
}
