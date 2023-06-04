#![allow(unused_variables)]
#![allow(dead_code)]

use aes_gcm::{
    aead::{AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm,
};
use std::{
    env, fs,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};
#[cfg(test)]
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct Stash {
    path: PathBuf,
    contents: PathBuf,
}

// Call `new()` for default implementation
impl Default for Stash {
    fn default() -> Self {
        Self::new()
    }
}

impl Stash {
    //  Initialize paths at `~/.stash` and `~/.stash/contents`
    pub fn new() -> Self {
        let home = env::var("HOME").expect("Failed to get `HOME` environment variable");
        let stash_path = PathBuf::from(&home);
        let contents_path = stash_path.join("contents");

        Stash {
            path: stash_path,
            contents: contents_path,
        }
    }

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
        //  TODO: this will need to decrypt the `contents` file
        //  TODO: it will then need to run some form of `tar -t` instead
        let ls_output = std::process::Command::new("ls")
            .arg(self.path.to_str().unwrap())
            .output()
            .expect("Failed to execute ls command")
            .stdout;

        let contents = String::from_utf8_lossy(&ls_output).trim().to_string();

        Ok(contents)
    }

    //  Add `file` to stash
    pub fn add(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = Path::new(file);
        let dst_path = self.path.join(src_path.file_name().unwrap());
        fs::rename(src_path, &dst_path).unwrap();

        let file_key = Aes256Gcm::generate_key(OsRng);
        let nonce = Aes256Gcm::generate_nonce(OsRng);

        Self::encrypt(&dst_path, &file_key, &nonce)?;
        //  For testing that decryption works
        //Self::decrypt(&dst_path, &file_key, &nonce)?;

        //  TODO: decrypt `contents` file (tarball) using `stash_key`
        //  TODO: unpack decrypted tarball
        //  TODO: create new tarball of stash contents
        //  TODO: re-encrypt `contents` using `stash_key`

        Ok(())
    }

    //  Copy `file` into stash
    pub fn copy(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = Path::new(file);
        let dst_path = self.path.join(src_path.file_name().unwrap());
        fs::copy(src_path, dst_path).unwrap();

        let file_key = Aes256Gcm::generate_key(OsRng);
        let nonce = Aes256Gcm::generate_nonce(OsRng);

        //  TODO: decrypt `contents` file (tarball) using `stash_key`
        //  TODO: unpack decrypted tarball
        //  TODO: create new tarball of stash contents
        //  TODO: re-encrypt `contents` using `stash_key`

        Ok(())
    }

    //  Move `file` from stash into current directory
    pub fn grab(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = self.path.join(file);
        let dst_path = env::current_dir()?.join(file);
        fs::rename(src_path, dst_path)?;

        Ok(())
    }

    //  Create a tarball from current stash contents.
    pub fn archive(&mut self) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        self.create_tarball()?;

        let stash_key = Aes256Gcm::generate_key(OsRng);
        let nonce = Aes256Gcm::generate_nonce(OsRng);
        Self::encrypt(&self.contents, &stash_key, &nonce)?;
        //Self::decrypt(&self.contents, &stash_key, &nonce)?;

        //  TODO: create keyring
        //  TODO: store key/nonce on keyring

        Ok(())
    }

    // Encrypt a specified file in place
    fn encrypt(path: &Path, key: &[u8], nonce: &[u8]) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        let cipher = Aes256Gcm::new(key.into());
        cipher
            .encrypt_in_place(nonce.into(), b"", &mut buffer)
            .unwrap();

        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(&buffer)?;

        //  Truncate the file to the new size
        file.set_len(buffer.len() as u64)?;

        Ok(())
    }

    // Decrypt a file in place
    fn decrypt(path: &Path, key: &[u8], nonce: &[u8]) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        let cipher = Aes256Gcm::new(key.into());
        cipher
            .decrypt_in_place(nonce.into(), b"", &mut buffer)
            .unwrap();

        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(&buffer)?;

        //  Truncate the file to the new size
        file.set_len(buffer.len() as u64)?;

        Ok(())
    }

    // Create a `.tar.gz` archive of stash contents
    fn create_tarball(&self) -> Result<(), io::Error> {
        //  TODO: successfully create archive, then remove originals
        let tar = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "tar czf {} --remove-files {}/*",
                self.contents.display(),
                self.path.display()
            ))
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
}
