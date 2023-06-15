#![allow(dead_code)] // keyring and secret

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
//  TODO: add comments
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Secret {
    key: Vec<u8>,
    nonce: Vec<u8>,
}
impl Secret {
    fn new() -> Self {
        Secret {
            key: Aes256Gcm::generate_key(OsRng).to_vec(),
            nonce: Aes256Gcm::generate_nonce(OsRng).to_vec(),
        }
    }
    fn from(secret: &[u8]) -> Self {
        Secret {
            key: secret[..32].to_vec(),
            nonce: secret[32..].to_vec(),
        }
    }
    fn join(&self) -> Vec<u8> {
        let mut secret = Vec::with_capacity(self.key.len() + self.nonce.len());
        secret.extend_from_slice(&self.key);
        secret.extend_from_slice(&self.nonce);

        secret
    }
    fn split(&self) -> (Vec<u8>, Vec<u8>) {
        (self.key.clone(), self.nonce.clone())
    }
}

//  TODO: zeroize on drop
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
    fn default() -> Self {
        Self::new().unwrap()
    }
}
impl Stash {
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

    // Open `sled` database if it exists; otherwise create it
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

    //  Add `file` to stash, optionally as a copy
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

    //  Move `file` from stash into current directory
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

    //  Delete `file` in stash
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

    //  List all files in stash directory
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

        /*
        println!("Archive mode: {}", self.is_archived);
        println!("\n{:?}", self.keyring.get_links(64 as usize).unwrap());
        for key in self.db.iter().keys() {
            println!("Key in database: {:?}", String::from_utf8_lossy(&key?));
        }
        */

        Ok(contents)
    }

    //  Create a tarball from current stash contents
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

    //  Extract the `contents` file
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

    // Encrypt a specified file in place
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

    // Decrypt a file in place
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

    // Create a `.tar.gz` archive of stash contents
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

    // Extract a `.tar.gz` of stash contents
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
