#[cfg(test)]
mod tests {
    use stash::*;
    use std::fs;
    use tempfile::TempDir;

    //  Tests for `list_stash()`

    //  NOTE: the following TODO items are blocked until we move to a single
    //        stash with a default path of `~/.stash`

    //  TODO: test default stash (call without args)

    //  TODO: make label optional (path within stash) and test
    //  TODO: test top-level path within stash
    //  TODO: test recursive path within stash

    #[test]
    fn test_list_stash_valid_label_succeeds() {
        //  Create temp directory and a valid label for it
        let test_dir = TempDir::new().unwrap();
        let valid_label = test_dir.path().to_str().unwrap();

        //  Get result of call to `list_stash()`
        let result = list_stash(&valid_label);

        //  Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_stash_output_multiple_files_succeeds() {
        //  Create temp directory to simulate stash
        let temp_dir = TempDir::new().unwrap();
        let stash_path = temp_dir.path().to_str().unwrap();

        //  Create some files in the temporary directory
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        let file3_path = temp_dir.path().join("file3.txt");
        fs::File::create(&file1_path).unwrap();
        fs::File::create(&file2_path).unwrap();
        fs::File::create(&file3_path).unwrap();

        //  Get the output of the `ls` command as a string
        let ls = std::process::Command::new("ls")
            .arg(&stash_path)
            .output()
            .expect("Failed to execute ls command")
            .stdout;
        let ls_output = String::from_utf8_lossy(&ls).trim().to_string();

        //  Get output of `list_stash()` as a string
        let test_output = list_stash(&stash_path).unwrap();

        //  Should succeed
        assert_eq!(ls_output, test_output);
    }

    #[test]
    fn test_list_stash_output_multiple_dirs_succeeds() {
        //  Create temp directory to simulate stash
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().to_str().unwrap();

        //  Create some files in the temporary directory
        let dir1_path = temp_dir.path().join("dir1/");
        let dir2_path = temp_dir.path().join("dir2/");
        let dir3_path = temp_dir.path().join("dir3/");
        fs::create_dir(&dir1_path).unwrap();
        fs::create_dir(&dir2_path).unwrap();
        fs::create_dir(&dir3_path).unwrap();

        //  Get the output of the `ls` command as a string
        let ls = std::process::Command::new("ls")
            .arg(&test_path)
            .output()
            .expect("Failed to execute ls command")
            .stdout;
        let ls_output = String::from_utf8_lossy(&ls).trim().to_string();

        //  Get output of `list_stash()` as a string
        let test_output = list_stash(&test_path).unwrap();

        //  Should succeed
        assert_eq!(ls_output, test_output);
    }

    #[test]
    #[should_panic]
    fn test_list_stash_on_nonexistent_directory_fails() {
        //  Create temp directory and a path to nonexistent sub-directory
        let temp_dir = TempDir::new().unwrap();
        let bad_path = temp_dir.path().join("nonexistent/");
        let bad_label = bad_path.to_str().unwrap();

        //  Get result of call to `list_stash()`
        let result = list_stash(&bad_label);

        //  Should fail
        assert!(result.is_err());
    }
}
