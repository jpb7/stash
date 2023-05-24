use std::{env, fs, io, path::Path};

//  Create a new stash directory at `label` in current directory.
pub fn init_stash(path: &str, label: &str) -> io::Result<()> {
    let new_stash = format!("{}/{}", path, label);
    println!("\n{:?}", new_stash);
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

//  Copy file from `src` to `dst`.
pub fn copy_file(src: &str, dst: &str) -> io::Result<()> {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst).join(src_path.file_name().ok_or(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid source file path",
    ))?);
    fs::copy(src_path, dst_path)?;

    Ok(())
}
