#[cfg(test)]
mod tests {
    use stash::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_stash_valid_label_succeeds() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_dir_path = temp_dir.path();

        let result = list_stash(temp_dir_path.to_str().unwrap());

        assert!(result.is_ok());
    }

    #[test]
    #[should_panic]
    fn test_list_stash_on_nonexistent_directory_fails() {
        let bad_label = "directory/does/not/exist";

        let result = list_stash(bad_label);

        assert!(result.is_err());
    }

    #[test]
    fn test_list_stash_output_succeeds() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let stash_path = temp_dir.path().to_str().unwrap();

        // Create some files in the temporary directory
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        std::fs::File::create(&file1_path).unwrap();
        std::fs::File::create(&file2_path).unwrap();

        let test_output = list_stash(stash_path).unwrap();

        // Get the output of the `ls` command
        let ls_output = std::process::Command::new("ls")
            .arg(&stash_path)
            .output()
            .expect("Failed to execute ls command")
            .stdout;

        let ls_output = String::from_utf8_lossy(&ls_output).trim().to_string();

        assert_eq!(test_output, ls_output);
    }
}
