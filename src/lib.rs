use aes_gcm::{
    aead::{Aead, AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm,
};
use std::io::{self, Read, Seek, Write};
use std::{env, fs, path::Path};

//  Create a new stash directory at `label` in current directory.
pub fn init_stash(path: &str, label: &str) -> io::Result<()> {
    let new_stash = format!("{}/{}", path, label);
    fs::create_dir(&new_stash)?;

    //  Create a `tar.gz` of new, empty directory
    let tarball = format!("{}/contents", new_stash);
    create_tarball(&new_stash, &tarball)?;
    let contents = Path::new(&tarball);

    //  Generate key and perform encryption
    let key = Aes256Gcm::generate_key(OsRng);
    encrypt(contents, &key)?;
    //encrypt_a_copy(contents, &key)?;

    //  TODO: store key on keyring
    //  TODO: reset authentication timeout (eventually)

    Ok(())
}

//  TODO: slice of strings or space-separated filepaths as `label`
fn create_tarball(path: &str, label: &str) -> Result<(), io::Error> {
    let output = std::process::Command::new("tar")
        .arg("czf")
        .arg(label)
        .arg(path)
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

#[allow(dead_code)]
//  Encrypt a specified file in place.
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

#[allow(dead_code)]
//  Create an encrypted copy of a specified file.
fn encrypt_a_copy(path: &Path, key: &[u8]) -> io::Result<()> {
    //  TODO: specify src/dst somehow
    let mut input_file = fs::File::open(path)?;
    let output_path = path.with_extension("enc");
    let mut output_file = fs::File::create(output_path)?;

    let mut input_buffer = Vec::new();
    input_file.read_to_end(&mut input_buffer)?;

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted_buffer = cipher.encrypt(&nonce, input_buffer.as_ref()).unwrap();
    output_file.write_all(&encrypted_buffer)?;

    Ok(())
}

//  List all files in stash directory at `label` in current directory.
pub fn list_stash(label: &str) -> io::Result<String> {
    let stash_path = env::current_dir()?.join(label);
    let ls = std::process::Command::new("ls")
        .arg(&stash_path)
        .output()
        .expect("Failed to execute ls command")
        .stdout;

    let contents = String::from_utf8_lossy(&ls).trim().to_string();

    Ok(contents)
}

//  Append `dst` to `src` path, validate, and return both paths.
fn get_paths(src: &str, dst: &str) -> (Box<Path>, Box<Path>) {
    let src_path = Path::new(src).to_owned().into_boxed_path();
    let dst_path = Path::new(dst)
        .join(
            src_path
                .file_name()
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid source file path",
                ))
                .unwrap(),
        )
        .into_boxed_path();

    (src_path, dst_path)
}

//  Move file from `src` to `dst`.
pub fn move_file(src: &str, dst: &str) -> io::Result<()> {
    let (src_path, dst_path) = get_paths(src, dst);
    fs::rename(src_path, dst_path)?;

    Ok(())
}

//  Copy file from `src` to `dst`.
pub fn copy_file(src: &str, dst: &str) -> io::Result<()> {
    let (src_path, dst_path) = get_paths(src, dst);
    fs::copy(src_path, dst_path)?;

    Ok(())
}

//  Move `file` from stash `label` to `file` in current directory.
pub fn grab_file(file: &str, label: &str) -> io::Result<()> {
    let src = format!("{}/{}", label, file);
    let src_path = Path::new(&src);

    let dst = format!("{}/{}", env::current_dir()?.display(), file);
    let dst_path = Path::new(&dst);

    fs::rename(src_path, dst_path)?;

    Ok(())
}
