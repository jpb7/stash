#[cfg(test)]
mod tests {
    use stash::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_stash_valid_label_succeeds() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");

        let temp_path = temp_dir.path().to_str().unwrap();

        assert!(init_stash(temp_path, "my_stash").is_ok());
    }

    #[test]
    fn test_init_stash_recursive_label_fails() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");

        let temp_path = temp_dir.path().to_str().unwrap();

        assert!(init_stash(temp_path, "path/to/my_stash").is_err());
    }

    #[test]
    fn test_init_stash_glob_label_fails() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");

        let temp_path = temp_dir.path().to_str().unwrap();

        assert!(init_stash(temp_path, "my_invalid_label/*").is_err());
    }

    // Add more test functions for other scenarios
}
