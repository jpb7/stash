#[cfg(test)]
mod tests {
    use stash::*;
    use std::fs;
    use tempfile::TempDir;

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
    fn test_init_stash_valid_label_succeeds() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Create stash using valid label
        let result = init_stash(temp_path, "my_stash");

        //  Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_stash_empty_label_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Try to create stash with an empty label
        let result = init_stash(temp_path, "");

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_recursive_label_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Try to create stash with a recursive label
        let result = init_stash(temp_path, "path/to/my_stash");

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_glob_label_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Try to create stash with glob characters in label
        let result = init_stash(temp_path, "my_glob_label/*");

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    #[should_panic]
    fn test_init_stash_label_with_invalid_characters_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Try to create stash with invalid characters in label
        let result = init_stash(temp_path, "my:stash?");

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_long_label_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Create a label greater than 255 bytes (ext4 limit)
        let mut long_label = String::new();
        while long_label.len() < 256 {
            long_label.push_str("X");
        }

        //  Try to create stash with label that's too long
        let result = init_stash(temp_path, &long_label);

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_at_existing_directory_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        //  Create a stash in temp directory
        let stash_path = temp_path.join("test");
        fs::create_dir(&stash_path).unwrap();

        //  Create strings from paths
        let path = stash_path.to_str().unwrap();
        let label = temp_path.to_str().unwrap();

        //  Try to create new stash at same path as stash above
        let result = init_stash(&path, &label);

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_label_shadowed_by_file_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Create a file where new stash will try to init
        let temp_file = format!("{}/my_stash", temp_path);
        fs::File::create(&temp_file).expect("Failed to create temp file");

        //  Try to initialize stash at same path as file
        let result = init_stash(temp_path, "my_stash");

        //  Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_in_nonexistent_directory_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("nonexistent_dir");

        //  Add path to directory which doesn't exist
        let bogus_path = temp_path.to_str().unwrap();

        //  Try to initialize stash in nonexistent directory
        let result = init_stash(bogus_path, "my_stash");

        assert!(result.is_err());
    }

    #[test]
    fn test_init_stash_in_readonly_directory_fails() {
        //  Create a temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        //  Create a sub-directory in temp directory
        let readonly_directory = format!("{}/my_stash", temp_path);
        fs::create_dir(&readonly_directory).unwrap();

        //  Set readonly permissions on the sub-directory
        let mut permissions = fs::metadata(&readonly_directory).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&readonly_directory, permissions).unwrap();

        //  Try to initialize stash in readonly directory
        let result = init_stash(temp_path, "my_stash");

        //  Should fail
        assert!(result.is_err());
    }
}
