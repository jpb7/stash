//  TODO: find a better way to set `stash_path` for testing
//  TODO: change tests for single-stash interface
/*
#[cfg(test)]
mod tests {
    use stash::*;
    use std::{
        fs,
        io::{self, ErrorKind, Read, Write},
    };
    use tempfile::TempDir;

    //  Tests for `move_file()`

    //  NOTE: these will need to be revisited once we move to a default stash
    //        at ~/.stash

    //  TODO: make label optional (path within stash) in `move_file()`

    //  TODO: detect optional path argument
    //  TODO: confirm valid label
    //  TODO: check for bad label
    //  TODO: check for empty label
    //  TODO: confirm file is encrypted
    //  TODO: confirm file appears in tar archive with `list_stash()`
    //  TODO: confirm file is still in original location
    //  TODO: modify and re-use other filesystem/naming tests from `init.rs`

    #[test]
    fn test_move_file() -> io::Result<()> {
        //  Create temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        //  Create source file with some text
        let src_path = temp_path.join("test.txt");
        let mut src = fs::File::create(&src_path).unwrap();
        src.write_all(b"Sample text").unwrap();

        //  Create stash directory and path
        let stash_dir = TempDir::new().unwrap();
        let stash_path = stash_dir.path();

        //  Create strings from paths
        let file = src_path.to_str().unwrap();
        let label = stash_path.to_str().unwrap();

        //  Move source file into stash directory
        let result = move_file(&file, &label);

        //  Should succeed
        assert!(result.is_ok());

        //  Check that file actually moved into stash
        let stashed_file = stash_path.join("test.txt");
        assert!(stashed_file.exists());

        //  Read the contents of the stashed file
        let mut stashed_contents = Vec::new();
        fs::File::open(&stashed_file)
            .unwrap()
            .read_to_end(&mut stashed_contents)
            .unwrap();

        //  Check that the contents of the stashed file are correct
        assert_eq!(stashed_contents, b"Sample text");

        Ok(())
    }

    #[test]
    fn test_move_file_source_not_found() {
        //  Create temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        //  Create nonexistent path to file that doesn't exist
        let src_path = temp_path.join("nonexistent.txt");

        //  Create stash directory to move file into
        let stash_path = temp_path.join("stash");
        fs::create_dir(&stash_path).unwrap();

        //  Create strings from paths
        let file = src_path.to_str().unwrap();
        let label = stash_path.to_str().unwrap();

        //  Try to move nonexistent file into stash
        let result = move_file(&file, &label);

        //  Should fail
        assert!(result.is_err());

        //  Check that correct error message is thrown
        let error = result.unwrap_err();
        assert_eq!(
            error.kind(),
            ErrorKind::NotFound,
            "Expected destination not found error"
        );
    }

    #[test]
    fn test_move_file_stash_not_found() {
        //  Create temp directory and path
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        //  Create source file in temp directory
        let src_path = temp_path.join("test.txt");
        fs::File::create(&src_path).unwrap();

        //  Add path in temp directory to stash which doesn't exist
        let stash_path = temp_path.join("nonexistent_stash");

        //  Create strings from paths
        let file = src_path.to_str().unwrap();
        let label = stash_path.to_str().unwrap();

        //  Try to move source file into nonexistent stash
        let result = move_file(&file, &label);

        //  Should fail
        assert!(result.is_err());

        //  Check that correct error message is thrown
        let error = result.unwrap_err();
        assert_eq!(
            error.kind(),
            ErrorKind::NotFound,
            "Expected destination not found error"
        );
    }
}
*/