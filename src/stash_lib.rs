use std::{env, fs, io, path::Path};

//  Create a new stash directory at `label` in current directory.
pub fn init_stash(path: &str, label: &str) -> io::Result<()> {
    let new_stash = format!("{}/{}", path, label);
    fs::create_dir(new_stash)?;

    Ok(())
}

//  List all files in stash directory at `label` in current directory.
pub fn list_stash(label: &str) -> io::Result<()> {
    let stash_path = env::current_dir()?.join(label);
    let dir = fs::read_dir(stash_path)?;

    for file in dir {
        let path = file?.path();
        if path.is_file() {
            println!("{}", path.display());
        }
    }

    Ok(())
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
