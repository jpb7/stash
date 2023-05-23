use std::fs;

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
