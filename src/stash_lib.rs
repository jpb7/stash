use std::fs;

//As of now this function simply creates a folder with the given name and
//creates the path
pub fn init_stash(label:&str, path: &str) -> Result<(), std::io::Error>{
    let pathway = format!("{}/{}", path, label);
    fs::create_dir(pathway)?;
    Ok(())
}

