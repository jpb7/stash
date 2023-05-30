#[cfg(test)]
mod tests {
    use stash::*;
    use std::fs;
    use std::io::{Read, Write};
    use tempfile::TempDir;

    //  Tests for `copy_file()`

    //  NOTE: these will need to be revisited once we move to a default stash
    //        at ~/.stash

    //  TODO: make label optional (path within stash) in `copy_file()`

    //  TODO: detect optional path argument
    //  TODO: confirm valid label
    //  TODO: check for bad label
    //  TODO: confirm file is encrypted
    //  TODO: confirm file appears in tar archive with `list_stash()`
    //  TODO: modify and re-use filesystem/naming tests from `init.rs`

    #[test]
    fn test_copy_file_valid() {
        //  Create temp directory and path
        let src_dir = TempDir::new().unwrap();
        let src_path = src_dir.path().join("test.txt");

        //  Create file with some text in it
        let mut src = fs::File::create(&src_path).unwrap();
        src.write_all(b"Sample text").unwrap();

        //  Create destination directory to simulate a stash
        let dst_dir = TempDir::new().unwrap();
        let dst_path = dst_dir.path();

        //  Convert paths to strings
        let test_src = src_path.to_str().unwrap();
        let test_dst = dst_path.to_str().unwrap();

        //  Copy source file into the simulated stash
        copy_file(&test_src, &test_dst).unwrap();

        //  Read in contents of source file
        let mut mock = Vec::new();
        fs::File::open(&src_path)
            .unwrap()
            .read_to_end(&mut mock)
            .unwrap();

        //  Read in contents of destination file
        let mut test = Vec::new();
        fs::File::open(&dst_path.join("test.txt"))
            .unwrap()
            .read_to_end(&mut test)
            .unwrap();

        //  Check that both files are the same
        assert_eq!(mock, test);
    }

    #[test]
    fn test_copy_file_src_file_not_found() {
        //  Create temp directory with a path to nonexistent source file
        let src_dir = TempDir::new().unwrap();
        let src_path = src_dir.path().join("nonexistent.txt");

        //  Create stash directory to copy nonexistent file into
        let stash_dir = TempDir::new().unwrap();
        let stash_path = stash_dir.path();

        //  Convert paths to strings
        let file = src_path.to_str().unwrap();
        let label = stash_path.to_str().unwrap();

        //  Try to copy file into nonexistent stash directory
        let result = copy_file(&file, &label);

        //  Make sure result is an error
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_file_stash_dir_not_found() {
        //  Create temp directory with path to source file
        let src_dir = TempDir::new().unwrap();
        let src_path = src_dir.path().join("test.txt");

        //  Create source file and give it some content
        let mut src = fs::File::create(&src_path).unwrap();
        src.write_all(b"Sample text").unwrap();

        //  Create path to nonexistent stash directory
        let stash_path = src_dir.path().join("nonexistent_dir/");

        //  Convert paths to strings
        let file = src_path.to_str().unwrap();
        let label = stash_path.to_str().unwrap();

        //  Try to copy file into nonexistent stash directory
        let result = copy_file(&file, &label);

        //  Make sure result is an error
        assert!(result.is_err());
    }
}
