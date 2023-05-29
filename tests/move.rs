#[cfg(test)]
mod tests {
    use stash::*;
    use tempfile::tempdir;
    use std::{
        fs,
        io::{self, ErrorKind},
    };

    #[test]
    fn test_move_file_source_not_found() {
        let src_path = "test_files/nonexistent_file.txt";
        let dst_path = "test_files/dst_file.txt";
        let result = move_file(src_path, dst_path);

        assert!(result.is_err(), "Expected move_file to return an error");
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::NotFound, "Expected source not found error");
    }

    #[test]
   fn test_move_file_destination_not_found(){
    let src_path = "test_files/dst_file.txt";
    let dst_path = "test_files/nonexistent_file.txt";
    let result = move_file(src_path, dst_path);

    assert!(result.is_err(), "Expected move_file to return an error");
    let error = result.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::NotFound, "Expected destination not found error");
   }

   #[test]
   fn test_move_file() -> io::Result<()> {
    // Create a temporary directory
    let src_dir = tempdir()?;
    let dst_dir = tempdir()?;

    // Create a source file in the temporary directory
    let src_file = src_dir.path().join("test.txt");
    fs::write(&src_file, "")?;

    // Call the move_file function
    move_file(src_file.to_str().unwrap(), dst_dir.path().to_str().unwrap())?;

    // Assert that the file was moved to the destination directory
    let dst_file = dst_dir.path().join("test.txt");
    assert!(dst_file.exists());

    Ok(())
}

}