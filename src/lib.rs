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
    io::{self, Read, Seek, Write},
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
    keyring: KeyRing,
    //secret: Secret,
    db: Db,
}
impl Default for Stash {
    fn default() -> Self {
        Self::new()
    }
}
impl Stash {
    pub fn new() -> Self {
        let home = env::var("HOME").expect("Failed to get `HOME` environment variable");
        let path = PathBuf::from(&home);

        let contents = path.join("contents");
        //let secret_path = path.join(".secret");
        let db_path = path.join(".db");

        //  TODO: set up session-based encryption/decryption
        let keyring = KeyRing::from_special_id(KeyRingIdentifier::Session, false).unwrap();
        //let secret = Self::get_secret(&secret_path);
        let db = Self::get_db(&db_path);

        Stash {
            path,
            contents,
            keyring,
            //secret,
            db,
        }
    }

    // Open `sled` database if it exists; otherwise create it
    fn get_db(db_path: &Path) -> Db {
        if db_path.exists() {
            sled::open(db_path.to_str().unwrap()).unwrap()
        } else {
            let config = Config::new().path(db_path.to_str().unwrap());
            config.open().unwrap()
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

    // Return current value of stash path
    #[cfg(test)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    #[cfg(test)]
    //  Create a temp directory and stash path for testing
    pub fn set_test_path(&mut self) {
        let temp_dir = TempDir::new().unwrap();
        self.path = temp_dir.path().join("test_stash");
        self.contents = self.path.join("contents");
    }

    #[cfg(test)]
    //  Create a new stash in test directory
    pub fn new_test_stash(&mut self) -> io::Result<()> {
        if self.path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Stash already exists",
            ));
        }
        fs::create_dir_all(self.path.to_str().unwrap())?;

        Ok(())
    }

    //  List all files in stash directory
    pub fn list(&self) -> io::Result<String> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let ls_output = std::process::Command::new("ls")
            .arg(self.path.to_str().unwrap())
            .output()
            .expect("Failed to execute ls command")
            .stdout;

        let contents = String::from_utf8_lossy(&ls_output).trim().to_string();

        /*
        println!("\n{:?}", self.keyring.get_links(64 as usize).unwrap());
        for key in self.db.iter().keys() {
            println!("Key in database: {:?}", String::from_utf8_lossy(&key?));
        }
        */

        Ok(contents)
    }

    //  Add `file` to stash
    pub fn add(&mut self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = Path::new(file);
        let dst_path = self.path.join(src_path.file_name().unwrap());

        if dst_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File is already in stash",
            ));
        }
        let secret = Secret::new();
        let description = src_path.to_string_lossy().to_string();

        fs::rename(src_path, &dst_path).unwrap();
        Self::encrypt(&dst_path, &secret).unwrap();
        self.db
            .insert(description.as_bytes(), secret.join())
            .unwrap();
        self.keyring.add_key(&description, &secret.join()).unwrap();
        //zeroize_all!(src_path, dst_path, secret, description, key);

        Ok(())
    }

    //  Copy `file` into stash
    pub fn copy(&mut self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = Path::new(file);
        let dst_path = self.path.join(src_path.file_name().unwrap());

        if dst_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File is already in stash",
            ));
        }
        let secret = Secret::new();
        let description = src_path.to_string_lossy().to_string();

        fs::copy(src_path, &dst_path).unwrap();
        Self::encrypt(&dst_path, &secret)?;
        self.db
            .insert(description.as_bytes(), secret.join())
            .unwrap();
        self.keyring.add_key(&description, &secret.join()).unwrap();
        //zeroize_all!(src_path, dst_path, secret, description, key);

        Ok(())
    }

    //  Move `file` from stash into current directory
    pub fn grab(&mut self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = self.path.join(file);
        let dst_path = env::current_dir()?.join(file);
        let secret;

        if dst_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File already exists in current directory",
            ));
        }
        //  Get secret from sys key if it exists; otherwise, use db
        if let Ok(key) = self.keyring.search(file) {
            secret = Secret::from(&key.read_to_vec().unwrap());
            key.invalidate().unwrap();
            //key.zeroize();
        } else if let Some(value) = self.db.get(file)? {
            secret = Secret::from(&value);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Secret not found in stash",
            ));
        }
        Self::decrypt(&src_path, &secret).unwrap();
        fs::rename(src_path, dst_path)?;
        self.db.remove(file)?;
        //zeroize_all!(src_path, dst_path, secret);

        Ok(())
    }

    //  Copy `file` from stash into current directory
    pub fn borrow(&mut self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "No stash found",
            ));
        }
        let src_path = self.path.join(file);
        let dst_path = env::current_dir()?.join(file);
        let secret;

        if dst_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File already exists in current directory",
            ));
        }
        //  Get secret from sys key if it exists; otherwise, use db
        if let Ok(key) = self.keyring.search(file) {
            secret = Secret::from(&key.read_to_vec().unwrap());
        } else if let Some(value) = self.db.get(file)? {
            secret = Secret::from(&value);
            self.keyring.add_key(file, &secret.join()).unwrap();
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Secret not found in stash",
            ));
        }
        Self::decrypt(&src_path, &secret).unwrap();
        fs::copy(src_path, dst_path)?;
        //zeroize_all!(src_path, dst_path, secret);

        Ok(())
    }

    //  !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    //  TODO: should refuse to delete `.secret` and `.db`
    //  !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    //  Delete `file` in stash
    pub fn delete(&mut self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let target_path = self.path.join(file);
        fs::remove_file(target_path.to_str().unwrap())?;
        self.db.remove(file)?;
        if let Ok(key) = self.keyring.search(file) {
            key.invalidate().unwrap();
        }

        Ok(())
    }

    //  Create a tarball from current stash contents
    pub fn archive(&mut self) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        } else if self.contents.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Archive already exists",
            ));
        }
        //  TODO: use "contents" only, not full path
        let description = self.contents.to_string_lossy().to_string();
        let secret = Secret::new();

        self.create_tarball()?;
        Self::encrypt(&self.contents, &secret)?;
        self.db
            .insert(description.as_bytes(), secret.join())
            .unwrap();
        self.keyring.add_key(&description, &secret.join()).unwrap();
        //zeroize_all!(description, secret);

        Ok(())
    }

    //  Extract the `contents` file
    pub fn unpack(&mut self) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        } else if !self.contents.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No archive to unpack",
            ));
        }
        //  TODO: change this to filename only, not full path
        let tarball = &self.contents.to_str().unwrap();
        let secret;

        if let Ok(key) = self.keyring.search(tarball) {
            secret = Secret::from(&key.read_to_vec().unwrap());
            key.invalidate().unwrap();
            //key.zeroize();
        } else if let Some(value) = self.db.get(tarball)? {
            secret = Secret::from(&value);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Secret not found in stash",
            ));
        }
        Self::decrypt(&self.contents, &secret).unwrap();
        self.extract_tarball()?;
        fs::remove_file(&self.contents)?;
        self.db.remove(tarball)?;
        //zeroize_all!(tarball, secret);

        Ok(())
    }

    // Encrypt a specified file in place
    fn encrypt(path: &Path, secret: &Secret) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let (key, nonce) = secret.split();
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        cipher
            .encrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut buffer)
            .unwrap();

        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(&buffer)?;
        file.set_len(buffer.len() as u64)?;

        Ok(())
    }

    // Decrypt a file in place
    fn decrypt(path: &Path, secret: &Secret) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let (key, nonce) = secret.split();
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        cipher
            .decrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut buffer)
            .unwrap();

        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(&buffer)?;
        file.set_len(buffer.len() as u64)?;

        Ok(())
    }

    // Create a `.tar.gz` archive of stash contents
    fn create_tarball(&self) -> Result<(), io::Error> {
        let tar = Command::new("sh")
            .arg("-c")
            .arg("cd && tar czf contents --remove-files ./*")
            .output()?;

        if !tar.status.success() {
            let err_msg = String::from_utf8_lossy(&tar.stderr);
            eprintln!("{}", err_msg);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create tar archive: {}", err_msg),
            ));
        }

        Ok(())
    }

    // Extract a `.tar.gz` of stash contents
    fn extract_tarball(&self) -> Result<(), io::Error> {
        let tar = Command::new("sh")
            .arg("-c")
            .arg("cd ~ && tar xzf contents")
            .output()?;

        if !tar.status.success() {
            let err_msg = String::from_utf8_lossy(&tar.stderr);
            eprintln!("{}", err_msg);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to unpack tar archive: {}", err_msg),
            ));
        }

        Ok(())
    }
}
