# stash

## Project vision

`stash` is a command-line tool that allows the user to create and manage encrypted folders on their local Linux filesystem. The idea is to add an extra layer of privacy and security for sensitive files or values such as API keys.

Basically, `stash` provides a set of terminal commands that allow the user to quickly encrypt a given file or set of files into an encrypted folder (called a stash), and also to decrypt a file or files from a stash into the current directory.

For encryption and decryption, `stash` uses the `aes-gcm` crate.

## Usage

`stash` will create a new stash at `~/.stash` using:

	stash init

The contents of a given stash will be viewable with:

	stash list

The basic syntax of the remaining commands will be:

	stash <cmd> <file>

So, to encrypt a given file and add it to the stash, one could use:

	stash add <file>

One could also encrypt and copy that file into the stash by using:

	stash copy <file>

To grab an encrypted file from the stash, decrypt it, and drop it into the current directory, one can use:

	stash grab <file>

## Project status

So far we have completed these tasks:
- Create the program structure and needed `TOML` file.
- Make a skeleton for handling command-line arguments and stubs for the functions that will be called upon receiving each of those arguments.
- Write functions for basic filesystem operations such as creating directories and moving/copying files.
- Implement some unit testing on what we have so far.
- Choose an encryption crate and integrate it as a dependency.
- Begin implementing encryption along with our filesystem operations.
- Rewritten our interface and logic to use a single, default stash.
- Integrated encryption into all of our core functions.

Our next steps will be to:

1. Implement decryption and add it to our core functions.
2. Design a secure key management system (ie. encrypted, serialized hash map).
3. Implement that key management system.
4. Continue writing unit tests and rewriting core functions as needed.
5. Proceed to flex goals.

## Flex goals

1. Add initialization for `stash` Linux user and get password from user.
2. Use that password as a master key for our key management system.
3. Make error handling more descriptive and robust.
4. Find a better solution to the `stash_path` problem.
5. Find a better testing arrangement.
6. Support nested filepaths in the stash using optional `path` argument.
