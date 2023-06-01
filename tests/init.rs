//  TODO: find a better way to set `stash_path` for testing
/*
#[cfg(test)]
mod tests {
    use stash::*;
    //use std::path::PathBuf;
    //use tempfile::TempDir;

    //  Tests for `init_stash()`

    //  NOTE: many of the following tests will no longer apply to this function
    //        once we move to a default stash at `~/.stash`.
    //
    //        They can, however, be modified for our other functions.

    //  TODO: test that stash is created at `~/.stash`
    //  TODO: test that stash is created with encrypted `contents` tarball
    //  TODO: test that tarball decrypts properly
    //  TODO: change test for duplicate stash creation

    #[test]
    #[ignore = "try in lib"]
    fn test_init_stash_succeeds() {
        let result = init_stash();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "need to fix first test"]
    fn test_init_stash_with_existing_stash_fails() {
        let mock = init_stash();
        assert!(mock.is_ok());

        let test = init_stash();
        assert!(test.is_err());
    }
}
*/

//    #[test]
//    fn test_init_stash_valid_label_succeeds() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//
//        let result = init_stash(temp_path, "my_stash");
//
//        assert!(result.is_ok());
//    }
//
//    #[test]
//    fn test_init_stash_empty_label_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//
//        let result = init_stash(temp_path, "");
//
//        assert!(result.is_err());
//    }
//
//    #[test]
//    fn test_init_stash_recursive_label_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//
//        let result = init_stash(temp_path, "path/to/my_stash");
//
//        assert!(result.is_err());
//    }
//
//    #[test]
//    fn test_init_stash_glob_label_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//
//        let result = init_stash(temp_path, "my_glob_label/*");
//
//        assert!(result.is_err());
//    }
//
//    #[test]
//    #[cfg(unix)]
//    #[should_panic]
//    fn test_init_stash_label_with_invalid_characters_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//        let invalid_label = "my:stash?";
//
//        let result = init_stash(temp_path, invalid_label);
//
//        assert!(result.is_err());
//    }
//
//    #[test]
//    fn test_init_stash_long_label_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//        let mut long_label = String::new();
//
//        //  Create a label greater than 255 bytes (ext4 limit)
//        while long_label.len() < 256 {
//            long_label.push_str("X");
//        }
//        let result = init_stash(temp_path, &long_label);
//
//        assert!(result.is_err());
//    }
//
//    #[test]
//    fn test_init_stash_label_shadowed_by_file_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//
//        //  Create a file where new stash will try to init
//        let temp_file = format!("{}/my_stash", temp_path);
//        fs::File::create(&temp_file).expect("Failed to create temp file");
//
//        let result = init_stash(temp_path, "my_stash");
//
//        //  Try to initialize stash at same path as file
//        assert!(result.is_err());
//    }
//
//    #[test]
//    fn test_init_stash_in_nonexistent_directory_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().join("nonexistent_dir");
//
//        //  Add path to directory which doesn't exist
//        let bogus_path = temp_path.to_str().unwrap();
//
//        //  Try to initialize stash in nonexistent directory
//        let result = init_stash(bogus_path, "my_stash");
//
//        assert!(result.is_err());
//    }
//
//    #[test]
//    fn test_init_stash_in_readonly_directory_fails() {
//        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
//        let temp_path = temp_dir.path().to_str().unwrap();
//
//        //  Create a directory with no write permissions
//        let readonly_directory = format!("{}/my_stash", temp_path);
//        fs::create_dir(&readonly_directory).expect("Failed to create temporary directory");
//        let mut permissions = fs::metadata(&readonly_directory).unwrap().permissions();
//        permissions.set_readonly(true);
//        fs::set_permissions(&readonly_directory, permissions).unwrap();
//
//        let result = init_stash(temp_path, "my_stash");
//
//        //  Try to initialize stash in readonly directory
//        assert!(result.is_err());
//    }
