use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

//As of now this function simply creates a folder with the given name and
//creates the path
pub fn init_stash(label:&str, path: &str) -> Result<(), std::io::Error>{
    let pathway = format!("{}/{}", path, label);
    fs::create_dir(pathway)?;
    Ok(())
}

//function that lists files in an existing directory given the direcotry name
pub fn list_stash(directory_name: &str) -> Result<(), std::io::Error>{
    let current_dir = std::env::current_dir()?;
    let pathway = current_dir.join(directory_name);
    let dir = fs::read_dir(pathway)?;
    for file in dir {
        let test = file?;
        let path = test.path();
        if path.is_file() {
            println!("{}", path.display());
        }
    }

    Ok(())
}

//Basic copy function - One Question - currently this function creates the new file in the 
//directory provided and if it doesnt exist, it throws an error.  But could easily be changed
//so that if the direcotry doesnt exist it creates it and then copies the file
pub fn copy_file(source_file_name: &str, destination_path: &str) -> io::Result<()> {
    let source_path = PathBuf::from(source_file_name);
    let mut source_file = fs::File::open(&source_path)?;
    let mut contents = Vec::new();
    source_file.read_to_end(&mut contents)?;
    let destination_path = Path::new(destination_path);
    if !destination_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Destination directory does not exist",
        ));
    }
    let destination_file_path = destination_path.join(source_path.file_name().unwrap());
    let mut destination_file = fs::File::create(&destination_file_path)?;
    destination_file.write_all(&contents)?;
    Ok(())
}
