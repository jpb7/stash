use aes_gcm::{
    aead::{AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm,
};
use linux_keyutils::{KeyRing, KeyRingIdentifier};
use std::{
    env, fs,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
    process::Command,
};
#[cfg(test)]
use tempfile::TempDir;
use zeroize::Zeroize;

//  TODO: find a way to test this
macro_rules! zeroize_all {
    ($($arg:expr),*) => {
        $(
            $arg.zeroize();
        )*
    };
}

#[derive(Debug, Clone)]
pub struct Stash {
    path: PathBuf,
    contents: PathBuf,
    keyring: KeyRing, // TODO: confirm permissions on this
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
        let session_keyring = KeyRing::from_special_id(KeyRingIdentifier::Session, false).unwrap();

        Stash {
            path: stash_path,
            contents: contents_path,
            keyring: session_keyring,
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

        let mut file_key = Aes256Gcm::generate_key(OsRng);
        let mut nonce = Aes256Gcm::generate_nonce(OsRng);

        Self::encrypt(&dst_path, &file_key, &nonce)?;

        let description = src_path.to_str().unwrap();
        let mut secret = Self::join_key_and_nonce(file_key.as_ref(), nonce.as_ref());
        self.keyring.add_key(description, &secret).unwrap();

        zeroize_all!(file_key, nonce, secret);

        Ok(())
    }

    //  Copy `file` into stash
    pub fn copy(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let src_path = Path::new(file);
        let dst_path = self.path.join(src_path.file_name().unwrap());
        fs::copy(src_path, &dst_path).unwrap();

        let mut file_key = Aes256Gcm::generate_key(OsRng);
        let mut nonce = Aes256Gcm::generate_nonce(OsRng);

        Self::encrypt(&dst_path, &file_key, &nonce)?;

        let description = src_path.to_str().unwrap();
        let mut secret = Self::join_key_and_nonce(file_key.as_ref(), nonce.as_ref());
        self.keyring.add_key(description, &secret).unwrap();

        zeroize_all!(file_key, nonce, secret);

        Ok(())
    }

    //  Move `file` from stash into current directory
    pub fn grab(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let sys_key = self.keyring.search(file).unwrap();
        let (mut file_key, mut nonce) = Self::split_secret(&sys_key.read_to_vec().unwrap());

        let src_path = self.path.join(file);
        Self::decrypt(&src_path, &file_key, &nonce).unwrap();

        let dst_path = env::current_dir()?.join(file);
        fs::rename(src_path, dst_path)?;

        sys_key.invalidate().unwrap();
        //Self::zeroize_key(secret);
        zeroize_all!(file_key, nonce);

        Ok(())
    }

    //  Copy `file` from stash into current directory
    pub fn r#use(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let sys_key = self.keyring.search(file).unwrap();
        let (mut file_key, mut nonce) = Self::split_secret(&sys_key.read_to_vec().unwrap());

        let src_path = self.path.join(file);
        let dst_path = env::current_dir()?.join(file);

        fs::copy(src_path, &dst_path)?;
        Self::decrypt(&dst_path, &file_key, &nonce).unwrap();

        //Self::zeroize_key(sys_key);
        zeroize_all!(file_key, nonce);

        Ok(())
    }

    //  Delete `file` in stash
    pub fn delete(&self, file: &str) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let target_path = self.path.join(file);
        fs::remove_file(target_path.to_str().unwrap())?;

        let sys_key = self.keyring.search(file).unwrap();
        sys_key.invalidate().unwrap();

        //Self::zeroize_key(secret);

        Ok(())
    }

    //  Create a tarball from current stash contents
    pub fn archive(&mut self) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        self.create_tarball()?;

        let mut file_key = Aes256Gcm::generate_key(OsRng);
        let mut nonce = Aes256Gcm::generate_nonce(OsRng);

        Self::encrypt(&self.contents, &file_key, &nonce)?;

        let description = &self.contents.to_str().unwrap();
        let mut secret = Self::join_key_and_nonce(file_key.as_ref(), nonce.as_ref());
        self.keyring.add_key(description, &secret).unwrap();

        zeroize_all!(file_key, nonce, secret);

        Ok(())
    }

    //  Extract the `contents` file
    pub fn unpack(&mut self) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
        }
        let description = &self.contents.to_str().unwrap();
        let sys_key = self.keyring.search(description).unwrap();
        let (mut file_key, mut nonce) = Self::split_secret(&sys_key.read_to_vec().unwrap());

        Self::decrypt(&self.contents, &file_key, &nonce).unwrap();
        self.extract_tarball()?;
        fs::remove_file(&self.contents)?;
        sys_key.invalidate().unwrap();

        //Self::zeroize_key(sys_key);
        zeroize_all!(file_key, nonce);

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

    // Join `file_key` and `nonce` into single `secret` for storage.
    fn join_key_and_nonce(file_key: &[u8], nonce: &[u8]) -> Vec<u8> {
        let mut secret = Vec::with_capacity(file_key.len() + nonce.len());
        secret.extend_from_slice(file_key);
        secret.extend_from_slice(nonce);

        secret
    }

    // Split a stored `secret` into `file_key` and `nonce`
    fn split_secret(secret: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let file_key = secret[..32].to_vec();
        let nonce = secret[32..].to_vec();

        (file_key, nonce)
    }

    // TODO: get this to work
    /*
    fn zeroize_key(key: &mut Key) {
        key.0 = Default::default();
    }
    */
}
