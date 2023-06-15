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

#[allow(unused_macros)]
macro_rules! zeroize_all {
    ($($arg:expr),*) => {
        $(
            $arg.zeroize();
        )*
    };
}

/// Represents a secret consisting of a key and nonce.
///
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Secret {
    key: Vec<u8>,
    nonce: Vec<u8>,
}
impl Secret {
    ///
    /// Creates a new `Secret` with random key and nonce.
    ///
    fn new() -> Self {
        Secret {
            key: Aes256Gcm::generate_key(OsRng).to_vec(),
            nonce: Aes256Gcm::generate_nonce(OsRng).to_vec(),
        }
    }

    /// Creates a `Secret` object out of a combined key/nonce pair.
    ///
    fn from(secret: &[u8]) -> Self {
        Secret {
            key: secret[..32].to_vec(),
            nonce: secret[32..].to_vec(),
        }
    }

    /// Returns the concatenated key/nonce pair.
    ///
    fn join(&self) -> Vec<u8> {
        let mut secret = Vec::with_capacity(self.key.len() + self.nonce.len());
        secret.extend_from_slice(&self.key);
        secret.extend_from_slice(&self.nonce);

        secret
    }

    /// Returns key and nonce as a tuple.
    ///
    fn split(&self) -> (Vec<u8>, Vec<u8>) {
        (self.key.clone(), self.nonce.clone())
    }
}

/// Represents a stash that holds encrypted files.
///
#[derive(Debug, Clone)]
pub struct Stash {
    path: PathBuf,
    contents: PathBuf,
    is_archived: bool,
    keyring: KeyRing,
    db: Db,
}

impl Default for Stash {
    ///
    /// Creates a new `Stash` with default configuration.
    ///
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Stash {
    ///
    /// Creates a new instance of the `Stash` struct.
    ///
    pub fn new() -> Result<Self, Error> {
        //
        let home = env::var("HOME").map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to get `HOME` environment variable: {}", err),
            )
        })?;
        let path = PathBuf::from(&home);

        //  Bail if stash path doesn't exist
        //
        if !path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "Stash path does not exist"));
        }
        let contents = path.join("contents");
        let mut is_archived = false;

        //  Turn on archive mode if tarball exists
        //
        if contents.exists() {
            is_archived = true;
        }
        let db_path = path.join(".db");
        let keyring = KeyRing::from_special_id(KeyRingIdentifier::Session, false).unwrap();
        let db = Self::get_db(&db_path)?;

        Ok(Stash {
            path,
            contents,
            is_archived,
            keyring,
            db,
        })
    }

    #[cfg(test)]
    ///
    /// Creates a new test stash at a specified directory.
    ///
    pub fn test(dir: &Path) -> Self {
        //
        let path = dir.join("test_stash");
        fs::create_dir(&path).unwrap();

        let contents = path.join("contents");
        let mut is_archived = false;

        //  Turn on archive mode if tarball exists
        //
        if contents.exists() {
            is_archived = true;
        }
        let keyring = KeyRing::from_special_id(KeyRingIdentifier::Session, false).unwrap();
        let db_path = path.join(".db");
        let db = Self::get_db(&db_path).unwrap();

        Stash {
            path,
            contents,
            is_archived,
            keyring,
            db,
        }
    }

    /// Retrieves or creates a `sled` database at the specified path.
    ///
    fn get_db(db_path: &Path) -> Result<Db, Error> {
        //
        let db_str = db_path
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Invalid database path"))?;

        //  Open the database if it exists
        //
        if db_path.exists() {
            sled::open(db_str).map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to open database: {}", err),
                )
            })
            //  Otherwise create it
            //
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

    /// Add a file to the stash, optionally as a copy.
    ///
    pub fn add(&mut self, file: &str, copy: bool) -> Result<(), Error> {
        //
        //  Refuse to add file if stash is archived
        //
        if self.is_archived {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Stash is in archive mode. Call `stash unpack` before adding more files",
            ));
        }
        //  Refuse to add a directory
        //
        let src_path = Path::new(file);
        if src_path.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Source file is a directory",
            ));
        }
        //  Take file name of target file
        //
        let dst_path = self.path.join(src_path.file_name().ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput, "Failed to resolve new file path")
        })?);

        //  Refuse to overwrite existing stashed file
        //
        if dst_path.exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "File already in stash",
            ));
        }

        let secret = Secret::new();
        let description = src_path.to_string_lossy().to_string();

        //  Copy or move depending on option
        //
        if copy {
            fs::copy(src_path, &dst_path)?;
        } else {
            fs::rename(src_path, &dst_path)?;
        }

        //  Encrypt file in place
        //
        Self::encrypt(&dst_path, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to encrypt file: {}", err))
        })?;

        //  Add filename and secret to database
        //
        self.db
            .insert(description.as_bytes(), secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to add encryption key to database: {}", err),
                )
            })?;

        //  Cache filename and secret in keyring
        //
        self.keyring
            .add_key(&description, &secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to cache encryption key: {}", err),
                )
            })?;

        Ok(())
    }

    /// Move a file from the stash into the current directory.
    ///
    pub fn grab(&mut self, file: &str, copy: bool) -> Result<(), Error> {
        //
        //  Bail if archived and not copying the tarball
        //
        if self.is_archived && !copy && file != "contents" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Stash is in archive mode. Use `stash unpack` to unpack",
            ));
        }
        let src_path = self.path.join(file);
        let dst_path = env::current_dir()?.join(file);
        let secret;

        //  Refuse to overwrite existing file
        //
        if dst_path.exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "File already exists in current directory",
            ));
        }

        //  Get secret from keyring if it's there
        //
        if let Ok(key) = self.keyring.search(file) {
            secret = Secret::from(&key.read_to_vec().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to read encryption key: {}", err),
                )
            })?);
            //      Remove secret if file is moved
            //
            if !copy {
                key.invalidate().map_err(|err| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Failed to remove cached key: {}", err),
                    )
                })?;
            }
            //  Or check database for file secrets
            //
        } else if let Some(value) = self.db.get(file)? {
            secret = Secret::from(&value);
            //
            //      Or throw
        } else {
            return Err(Error::new(ErrorKind::NotFound, "Secret not found"));
        }

        //  Decrypt file in place
        //
        Self::decrypt(&src_path, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to decrypt file: {}", err))
        })?;

        //  Copy depending on option passed in
        //
        if copy {
            fs::copy(src_path, dst_path).map_err(|err| {
                Error::new(ErrorKind::Other, format!("Failed to copy file: {}", err))
            })?;
            //      Or move
        } else {
            fs::rename(src_path, dst_path).map_err(|err| {
                Error::new(ErrorKind::Other, format!("Failed to move file: {}", err))
            })?;
            //      And remove encryption secrets from database
            //
            self.db.remove(file).map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to remove file from database: {}", err),
                )
            })?;
        }

        //  Toggle archive mode if tarball was removed
        //
        if !copy && file == "contents" {
            self.is_archived = false;
        }

        Ok(())
    }

    /// Delete `file` in the stash.
    ///
    pub fn delete(&mut self, file: &str) -> Result<(), Error> {
        //
        //  Bail if archived and not deleting `contents`
        //
        if self.is_archived && file != "contents" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Stash is in archive mode. Use `stash unpack` to unpack",
            ));
            //  Otherwise, bail if deleting program files
            //
        } else if file == ".db" || file == ".secret" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Cannot delete program file {}", file),
            ));
        }
        //  Make sure specified file exists
        //
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

        //  Remove file secret from database
        //
        self.db.remove(file).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to remove file from database: {}", err),
            )
        })?;

        //  Remove file secret from keyring if cached
        //
        if let Ok(key) = self.keyring.search(file) {
            key.invalidate().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to remove key from cache: {}", err),
                )
            })?;
        }
        //  End archive mode if tarball deleted
        //
        if file == "contents" {
            self.is_archived = false;
        }

        Ok(())
    }

    /// List all files in the stash directory.
    ///
    pub fn list(&self) -> Result<String, Error> {
        //
        //  Call `ls` command at the stash path
        //
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
    pub fn archive(&mut self) -> Result<(), Error> {
        //
        //  Bail if already archived
        //
        if self.is_archived {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Archive already exists",
            ));
            //  Or bail if the database is empty
            //
        } else if self.db.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "No files in stash: .db is empty",
            ));
        }

        //  Get file name of tarball
        //
        let file_name = self.contents.file_name().ok_or(Error::new(
            ErrorKind::InvalidData,
            "Failed to get file name",
        ))?;

        let description = file_name.to_string_lossy().to_string();
        let secret = Secret::new();

        //  Create tarball `contents` and remove original files
        //
        self.create_tarball().map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create tarball: {}", err),
            )
        })?;

        //  Encrypt the new tarball
        //
        Self::encrypt(&self.contents, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to encrypt file: {}", err))
        })?;

        //  Add its encryption secrets to the database
        //
        self.db
            .insert(description.as_bytes(), secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to add encryption secrets to database: {}", err),
                )
            })?;

        //  Also cache its secrets in the keyring
        //
        self.keyring
            .add_key(&description, &secret.join())
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to cache encryption secrets: {}", err),
                )
            })?;

        self.is_archived = true;

        Ok(())
    }

    /// Extract the `contents` file from the stash archive.
    ///
    pub fn unpack(&mut self) -> Result<(), Error> {
        //
        //  Bail if nothing is archived
        //
        if !self.is_archived {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No archive exists",
            ));
        }
        //  Get contents file name to use for secret lookup
        //
        let file_name = self.contents.file_name().ok_or(Error::new(
            ErrorKind::InvalidData,
            "Failed to get `contents` file name",
        ))?;

        let description = file_name.to_string_lossy().to_string();
        let secret;

        //  Look for cached secret on keyring
        //
        if let Ok(key) = self.keyring.search(&description) {
            let key_bytes = key.read_to_vec().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to read encryption secrets: {}", err),
                )
            })?;

            secret = Secret::from(&key_bytes);

            //  Remove key if secret is found
            //
            key.invalidate().map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to remove encryption secrets from cache: {}", err),
                )
            })?;
            //    Or check database
            //
        } else if let Some(value) = self.db.get(&description)? {
            secret = Secret::from(&value);
            //
            //          Or throw
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Secret not found"));
        }

        //  Decrypt the tarball in place
        //
        Self::decrypt(&self.contents, &secret).map_err(|err| {
            Error::new(ErrorKind::Other, format!("Failed to decrypt file: {}", err))
        })?;

        //  Extract its contents into stash
        //
        self.extract_tarball().map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to extract archive: {}", err),
            )
        })?;

        //  Delete `contents` file
        //
        fs::remove_file(&self.contents).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to remove `contents` file: {}", err),
            )
        })?;

        //  Remove `file` encryption secrets from database
        //
        self.db.remove(description).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to remove encryption secrets from database: {}", err),
            )
        })?;

        self.is_archived = false;

        Ok(())
    }

    /// Encrypts a specified file in place using the provided secret.
    ///
    fn encrypt(path: &Path, secret: &Secret) -> Result<(), Error> {
        //
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let (key, nonce) = secret.split();
        let mut buffer = Vec::new();

        //  Read file contents into buffer
        //
        file.read_to_end(&mut buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read file: {}", err),
            )
        })?;

        //  Create ciphertext using key, encrypt to buffer using nonce
        //
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let result = cipher.encrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut buffer);

        if let Err(err) = result {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to encrypt file: {}", err),
            ));
        }

        //  Go back to beginning of file
        //
        file.seek(io::SeekFrom::Start(0)).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to seek file: {}", err),
            )
        })?;

        //  Write encrypted contents to file
        //
        file.write_all(&buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write encrypted data to file: {}", err),
            )
        })?;

        //  Trim file to new length
        //
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
    fn decrypt(path: &Path, secret: &Secret) -> Result<(), Error> {
        //
        //  Open file at `path` for reading and writing
        //
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|err| Error::new(ErrorKind::Other, format!("Failed to open file: {}", err)))?;

        let (key, nonce) = secret.split();
        let mut buffer = Vec::new();

        //  Read contents to buffer
        //
        file.read_to_end(&mut buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read file: {}", err),
            )
        })?;

        //  Create ciphertext using key, decrypt to buffer using nonce
        //
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let result = cipher.decrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut buffer);

        if let Err(err) = result {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to decrypt file: {}", err),
            ));
        }

        //  Go back to beginning of file
        //
        file.seek(io::SeekFrom::Start(0)).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to seek file: {}", err),
            )
        })?;

        //  Write encrypted contents to file
        //
        file.write_all(&buffer).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write file: {}", err),
            )
        })?;

        //  Trim file to new length
        //
        file.set_len(buffer.len() as u64).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to set file length: {}", err),
            )
        })?;

        Ok(())
    }

    /// Creates `.tar.gz` archive of stash contents and removes files.
    ///
    fn create_tarball(&self) -> Result<(), Error> {
        //
        //  Change to stash directory and unpack
        //
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

        //  Throw error on failure
        //
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
    fn extract_tarball(&self) -> Result<(), io::Error> {
        //  Change to stash directory and extract tarball
        //
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

        //  Throw error on failure
        //
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

//  Unit tests
//
#[cfg(test)]
//
mod tests {
    use crate::*;
    use serial_test::serial;
    use std::fs::File;
    use tempfile::TempDir;

    #[allow(dead_code)]
    //
    fn setup() {
        // todo
    }

    #[test]
    #[serial]
    //
    fn test_valid_new() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let stash_path = path.join("test_stash");
        let db_path = stash_path.join(".db");

        let _ = Stash::test(path);

        assert!(stash_path.exists() && stash_path.is_dir());
        assert!(db_path.exists() && db_path.is_dir());
    }

    #[test]
    #[serial]
    //
    fn test_valid_add() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        let stash_path = dir_path.join("test_stash");
        let file_path = dir_path.join("test.txt");
        let file_os_str = file_path.file_name().unwrap();
        let file_str = file_os_str.to_str().unwrap();
        env::set_current_dir(&dir_path).unwrap();

        let mut stash = Stash::test(dir_path);

        let mut file = File::create(&file_path).unwrap();
        let stashed_file = stash_path.join("test.txt");
        let test_str = "Testing: one, two...";
        writeln!(file, "{}", test_str).unwrap();

        stash.add(&file_str, false).unwrap();

        assert!(stashed_file.exists() && !file_path.exists());
        let encrypted = fs::read(&stashed_file).unwrap();
        assert_ne!(test_str.as_bytes(), encrypted);
    }

    #[test]
    #[serial]
    //
    fn test_valid_copy() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        let stash_path = dir_path.join("test_stash");
        let file_path = dir_path.join("test.txt");
        let file_os_str = file_path.file_name().unwrap();
        let file_str = file_os_str.to_str().unwrap();
        env::set_current_dir(&dir_path).unwrap();

        let mut stash = Stash::test(dir_path);

        let mut file = File::create(&file_path).unwrap();
        let stashed_file = stash_path.join("test.txt");
        let test_str = "Testing: one, two...";
        writeln!(file, "{}", test_str).unwrap();

        stash.add(&file_str, true).unwrap();

        assert!(stashed_file.exists() && file_path.exists());
        let encrypted = fs::read(&stashed_file).unwrap();
        assert_ne!(test_str.as_bytes(), encrypted);
    }

    #[test]
    #[serial]
    //
    fn test_valid_grab() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        let stash_path = dir_path.join("test_stash");
        let file_path = dir_path.join("test.txt");
        let file_os_str = file_path.file_name().unwrap();
        let file_str = file_os_str.to_str().unwrap();
        env::set_current_dir(&dir_path).unwrap();

        let mut stash = Stash::test(dir_path);

        let mut file = File::create(&file_path).unwrap();
        let stashed_file = stash_path.join("test.txt");
        let test_str = "Testing: one, two...";
        writeln!(file, "{}", test_str).unwrap();

        stash.add(&file_str, false).unwrap();

        assert!(stashed_file.exists() && !file_path.exists());
        let encrypted = fs::read(&stashed_file).unwrap();
        assert_ne!(test_str.as_bytes(), encrypted);

        stash.grab(&file_str, false).unwrap();

        assert!(!stashed_file.exists());
        assert!(file_path.exists());
    }
}
