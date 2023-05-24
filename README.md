# stash

## Project vision

`stash` is a command-line tool that allows the user to create and manage encrypted folders on their local Linux filesystem. The idea is to add an extra layer of privacy and security for sensitive files or values such as API keys.

Basically, `stash` provides a set of terminal commands that allow the user to quickly encrypt a given file or set of files into an encrypted folder (called a stash), and also to decrypt a file or files from a stash into the current directory.

For encryption and decryption, `stash` will use a Rust implementation of an algorithm to be determined.

## Usage

`stash` will handle creation of a new stash with the command:

	stash init <path/to/stash> <label>

The contents of a given stash will be viewable with:

	stash list <label>

The basic syntax of the remaining commands will be:

	stash <cmd> <file> <label>

So, to encrypt a given file and move it into the stash referred to by `label`, one could use:

	stash move <file> <label>

One could also encrypt and copy that file into a given stash by using:

	stash copy <file> <label>

To restore an encrypted file from a stash to the current directory, one can use:

	stash grab <file> <label>

## Project status

So far we have completed these tasks:
- Create the program structure and needed `TOML` file.
- Make a skeleton for handling command-line arguments and stubs for the functions that will be called upon receiving each of those arguments.
- Write functions for basic filesystem operations such as creating directories and moving/copying files.

Our next steps will be to:

1. Implement some unit testing on what we have so far.
2. Choose an encryption crate and integrate it as a dependency.
3. Implement encryption along with our filesystem operations.
4. Design and implement a way to manage and coordinate stash labels (ie. `stash_map`).
5. Proceed to other flex goals.

## Flex goals

1. Expand functionality to include multiple stashes (`stash_map`).
2. Add initialization for `stash` Linux user and get password from user.
3. Implement tag system using a serialized hash map.
