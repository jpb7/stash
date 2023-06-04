# stash

## Project vision

`stash` is a command-line tool that allows the user to create and manage an encrypted folder on their local Linux filesystem. The idea is to add an extra layer of privacy and security for sensitive files or values such as API keys.

Basically, `stash` provides a set of terminal commands that allow the user to quickly encrypt a given file or set of files into an encrypted folder (called the stash), and also to decrypt a file or files from the stash into the current directory.

For encryption and decryption, `stash` uses the `aes-gcm` crate.

## Usage

Upon initialization, the user is prompted to create a password for the `stash` user. A new stash will then be created at `~/.stash`.

The contents of the stash are viewable with:

	stash list

Basic syntax of the remaining commands is:

	stash <cmd> <file>

So, to encrypt a given file and add it to the stash, use:

	stash add <file>

To encrypt a copy of that file into the stash, use:

	stash copy <file>

To decrypt a stashed file and drop it into the current directory, use:

	stash grab <file>

All stashed files and directories can be archived into a `.tar.gz` file with:
```
stash archive
```
This will replace everything in the stash with an encrypted tarball called `contents`.

## Project status

So far we have completed these tasks:
- Created the program structure and needed `TOML` file.
- Made a skeleton for handling command-line arguments and stubs for the functions that will be called upon receiving each of those arguments.
- Wrote functions for basic filesystem operations such as creating directories and moving/copying files.
- Implemented some unit testing on what we have so far.
- Chose an encryption crate and integrated it as a dependency.
- Began implementing encryption along with our filesystem operations.
- Rewrote our interface and logic to use a single, default stash.
- Integrated encryption into all of our core functions.
- Implemented decryption and added it to our core functions.
- Moved our core functions into a `Stash` struct that manages its own path.
- Implemented and tested a bespoke key management system as a proof of concept. (Then scrapped it for Linux keyrings.)
- Added initialization and authentication for `stash` Linux user.
- Removed `init` command, added `archive` instead.

Our next steps will be to:

1. Use Linux keyrings to store key/nonce pairs for stashed files.
2. Rewrite unit tests to use our `Stash` object, and re-integrate them into the project.
3. Add `view` and `unpack` commands.
4. Add `PAM` initialization to configure sessions and timeouts for `stash` user.
5. Make error handling more descriptive and robust.
6. Review and refactor for greater efficiency.
