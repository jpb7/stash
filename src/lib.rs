use aes_gcm::{
    aead::{Aead, AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm,
};
use std::io::{self, Read, Seek, Write};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
#[cfg(test)]
use tempfile::TempDir;

//  Specify `~/.stash` as default path; use temp directory when testing
//  TODO: add Result as return value
fn get_stash_path() -> PathBuf {
    #[cfg(not(test))]
    {
        let home = match env::var("HOME") {
            Ok(path) => PathBuf::from(path),
            Err(_) => panic!("Failed to get `HOME` environment variable"),
        };
        home.join(".stash")
    }
    #[cfg(test)]
    {
        let temp_dir = TempDir::new().unwrap();
        temp_dir.path().join("test_stash")
    }
}

//  Create a new stash in user's home directory
pub fn init() -> io::Result<()> {
    let stash_path = get_stash_path();
    //let contents = stash_path.join("contents");

    if stash_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Stash already exists.",
        ));
    }

    fs::create_dir_all(stash_path.to_str().unwrap())?;
    //create_tarball(&contents)?;

    //let stash_key = Aes256Gcm::generate_key(OsRng);
    //encrypt(&contents, &stash_key)?;

    //  TODO: store key on keyring

    Ok(())
}

//  Create a `.tar.gz` archive
#[allow(dead_code)]
fn create_tarball(archive_path: &Path) -> Result<(), io::Error> {
    let stash_path = get_stash_path();
    let output = std::process::Command::new("tar")
        .arg("czf")
        .arg(archive_path)
        .arg(stash_path)
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", err_msg);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create tar archive: {}", err_msg),
        ));
    }

    Ok(())
}

//  Encrypt a specified file in place
fn encrypt(path: &Path, key: &[u8]) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    //  TODO: not sure about empty byte string here
    cipher.encrypt_in_place(&nonce, b"", &mut buffer).unwrap();

    file.seek(io::SeekFrom::Start(0))?;
    file.write_all(&buffer)?;

    Ok(())
}

//  Create an encrypted copy of a specified file
fn encrypt_a_copy(path: &Path, key: &[u8]) -> io::Result<()> {
    let stash_path = get_stash_path();
    if !stash_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
    }
    let output_path = stash_path.join(path.file_name().unwrap());

    let mut input_file = fs::File::open(path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut input_buffer = Vec::new();
    input_file.read_to_end(&mut input_buffer)?;

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted_buffer = cipher.encrypt(&nonce, input_buffer.as_ref()).unwrap();
    output_file.write_all(&encrypted_buffer)?;

    Ok(())
}

//  List all files in stash directory
pub fn list() -> io::Result<String> {
    let stash_path = get_stash_path();
    if !stash_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
    }

    //  TODO: this will need to decrypt the `contents` file
    //  TODO: it will then need to run some form of `tar -t` instead
    let ls = std::process::Command::new("ls")
        .arg(stash_path.to_str().unwrap())
        .output()
        .expect("Failed to execute ls command")
        .stdout;

    let contents = String::from_utf8_lossy(&ls).trim().to_string();

    //  TODO: re-encrypt

    Ok(contents)
}

//  Add `file` to stash
pub fn add(file: &str) -> io::Result<()> {
    let stash_path = get_stash_path();
    if !stash_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
    }

    let src_path = Path::new(file);
    let dst_path = stash_path.join(src_path.file_name().unwrap());
    fs::rename(src_path, &dst_path).unwrap();

    let file_key = Aes256Gcm::generate_key(OsRng);
    encrypt(&dst_path, &file_key)?;

    //  TODO: decrypt `contents` file (tarball) using `stash_key`
    //  TODO: unpack decrypted tarball
    //  TODO: create new tarball of stash contents
    //  TODO: re-encrypt `contents` using `stash_key`

    Ok(())
}

//  Copy `file` into stash
pub fn copy(file: &str) -> io::Result<()> {
    let stash_path = get_stash_path();
    if !stash_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
    }

    let src_path = Path::new(file);
    let dst_path = stash_path.join(src_path.file_name().unwrap());
    fs::copy(src_path, &dst_path).unwrap();

    let file_key = Aes256Gcm::generate_key(OsRng);
    encrypt_a_copy(&dst_path, &file_key)?;

    //  TODO: decrypt `contents` file (tarball) using `stash_key`
    //  TODO: unpack decrypted tarball
    //  TODO: create new tarball of stash contents
    //  TODO: re-encrypt `contents` using `stash_key`

    Ok(())
}

//  Move `file` from stash into current directory
pub fn grab(file: &str) -> io::Result<()> {
    let stash_path = get_stash_path();
    if !stash_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No stash found"));
    }
    let src_path = stash_path.join(file);
    let dst_path = env::current_dir()?.join(file);

    fs::rename(src_path, dst_path)?;

    Ok(())
}
