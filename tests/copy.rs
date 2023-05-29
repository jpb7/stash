#[cfg(test)]
mod tests {
    use stash::*;
    use std::io::{ErrorKind,self, Read, Write};
    use std::fs::{self};
    use tempfile::TempDir;
   

    #[test]
    fn test_copy_file_source_not_found() {
        let src_path = "test_files/nonexistent_file.txt";
        let dst_path = "test_files/dst_file.txt";
        let result = copy_file(src_path, dst_path);

        assert!(result.is_err(), "Expected move_file to return an error");
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::NotFound, "Expected source not found error");
    }
    #[test]
    fn test_copy_file_dest_not_found() {
        let src_path = "test_files/dst_file.txt";
        let dst_path = "test_files/nonexistent_file.txt";
        let result = copy_file(src_path, dst_path);

        assert!(result.is_err(), "Expected move_file to return an error");
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::NotFound, "Expected source not found error");
    }
  
    fn files_are_equal(file1: &str, file2: &str) -> io::Result<bool> {
        let file_contents = fs::read_to_string(file1)?;
        let file_contents1 = fs::read_to_string(file2)?;
        
        Ok(file_contents == file_contents1)
    }
    #[test]
    fn test_copy_file_valid(){
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_path = temp_dir.path();

        let src_path = temp_dir_path.join("src.txt");
        let mut src = fs::File::create(&src_path).unwrap();
        src.write_all(b"Hello, World!").unwrap();

        let mock_path = temp_dir.path().join("mock.txt");
        fs::copy(&src_path, &mock_path).unwrap();

        let test_path = temp_dir.path().join("test.txt");

        let src_label = format!("src_path: {}", src_path.display());
        let test_label = format!("test_path: {}", test_path.display());
        println!("{}", &src_label);
        println!("{}", &test_label);

        copy_file(&src_label, &test_label).unwrap();

        // Read the contents of the files
        let mut src_contents = Vec::new();
        fs::File::open(src_path).unwrap().read_to_end(&mut src_contents).unwrap();

        let mut mock_contents = Vec::new();
        fs::File::open(mock_path).unwrap().read_to_end(&mut mock_contents).unwrap();

        let mut test_contents = Vec::new();
        fs::File::open(test_path).unwrap().read_to_end(&mut test_contents).unwrap();

        // Assert the contents are equal
        assert_eq!(src_contents, mock_contents);
        assert_eq!(src_contents, test_contents);
    }

}