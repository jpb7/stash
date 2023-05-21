# Stash



## Project vision

Stash is a command-line tool that allows the user to create and manage encrypted folders on their local Linux filesystem. The idea is to add an extra layer of privacy and security for sensitive files or values such as API keys.

Basically, stash provides a set of terminal commands that allow the user to quickly encrypt a given file or set of files into an encrypted folder (called a stash), and also to decrypt a file or files from a stash into the current directory.

For encryption and decryption, stash will use a Rust implementation of the AES-GCM-SIV algorithm which can be found at:

https://crates.io/crates/aes-gcm-siv.

## Usage

Stash will handle creation of a new stash with the command:

	stash init <stash_name> <path/to/stash>

The basic syntax of the primary commands will be:

	stash <cmd> <file> <stash_name>

So, to encrypt a copy of a given file and move it into the default stash, one could use:

	stash cp <file>

One could also move that file into the default stash by using:

	stash mv <file>

To restore an encrypted file from the default stash to the current directory, one can use:

	stash grab <file>

The contents of a given stash will be viewable with:

	stash ls <stash_name>

## Project status

We're off to a late start due to a complete change in project as originally proposed.

Our next steps will be to:

1. Create the program structure and needed TOML file.
2. Make a skeleton for handling command-line arguments and stubs for the functions that will be called upong receiving each of those arguments.
3. Write functions for filesystem operations such as creating directories and moving/copying files.
4. Integrate the `aes-gcm-siv` crate and its encryption/decryption operations.
5. Proceed to flex goals.

## Flex goals

1. Expand functionality to include multiple stashes.
2. Add initialization for `stash` Linux user and get password from user.
3. Implement tag system using a serialized hash map.
