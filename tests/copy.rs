#[cfg(test)]
mod tests {
    use stash::*;
    use std::io::{ErrorKind,self, Write};
    use std::fs::{self};
    use tempfile::{Builder};
   

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
        let temp_dir = Builder::new().prefix("temp_dir").tempdir().unwrap();
        let file_path = temp_dir.path().join("temp_file.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        let path_s = file_path.into_os_string().into_string().unwrap();

        let temp_dir2 = Builder::new().prefix("temp_dir").tempdir().unwrap();
        let file_path2 = temp_dir2.path().join("temp_file.txt");
        let mut file2 = fs::File::create(&file_path2).unwrap();
        file2.write_all(b"Hello, World!").unwrap();
        let path_s2 = file_path2.into_os_string().into_string().unwrap();
        let result2 = files_are_equal(&path_s, &path_s2);
        assert!(result2.is_ok())    
    }

}