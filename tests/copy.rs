#[cfg(test)]
mod tests {
    use stash::*;
    use std::fs::{self};
    use std::io::{ErrorKind, Read, Write};
    use tempfile::TempDir;

    #[test]
    fn test_copy_file_valid() {
        //  Create directory with source file
        let src_dir = TempDir::new().expect("Failed to create temporary directory");
        let src_path = src_dir.path().join("test.txt");
        let mut src = fs::File::create(&src_path).unwrap();
        src.write_all(b"Sample text").unwrap();

        //  Create destination directory to simulate a stash
        let dst_dir = TempDir::new().expect("Failed to create temporary directory");
        let dst_path = dst_dir.path();

        //  Convert paths to strings
        let test_src = format!("{}", src_path.display());
        let test_dst = format!("{}", dst_path.display());

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
        let src_dir = TempDir::new().expect("Failed to create temporary directory");
        let src_path = src_dir.path().join("nonexistent.txt");

        //  Create temp directory to copy nonexistent file into
        let dst_dir = TempDir::new().expect("Failed to create temporary directory");
        let dst_path = dst_dir.path();

        //  Convert paths to strings and call function
        let src = format!("{}", src_path.display());
        let dst = format!("{}", dst_path.display());
        let result = copy_file(&src, &dst);

        //  Make sure result is an error
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_file_dst_dir_not_found() {
        //  Create temp directory with source file
        let src_dir = TempDir::new().expect("Failed to create temporary directory");
        let src_path = src_dir.path().join("test.txt");
        let mut src = fs::File::create(&src_path).unwrap();
        src.write_all(b"Sample text").unwrap();

        //  Create path to nonexistent destination directory
        let dst_path = src_dir.path().join("nonexistent_dir/");

        //  Convert paths to strings and call function
        let src = format!("{}", src_path.display());
        let dst = format!("{}", dst_path.display());
        let result = copy_file(&src, &dst);

        //  Make sure result is an error
        assert!(result.is_err());
    }

}
