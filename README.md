# stash

## DISCLAIMER ##

It should be stated at the outset that this is a student project. Use this program at your own risk, and don't assume it uses security best practices or even employs `AES` encryption properly. This code has not been audited by anyone in relation to anything.

## About

`stash` is a Linux command-line tool that allows the user to create and manage a directory of encrypted files. The idea is to add an extra layer of privacy and security for sensitive files.

`stash` provides a few simple commands which allow the user to move files into and out of a locked directory called the stash, encrypting or decrypting those files in the process.

For encryption and decryption, `stash` uses the [`aes-gcm`](https://crates.io/crates/aes-gcm) crate. Specifically, it uses the `AES-256` variant. Both encryption and decryption are performed byte-by-byte, and have been (casually) tested on various file types including text, audio, and video.

This program uses the [`sled`](https://crates.io/crates/sled) and [`linux-keyutils`](https://crates.io/crates/linux-keyutils) crates for persistent storage and caching, respectively, of encryption secrets. The man page for Linux `keyrings` can be found [here](https://man7.org/linux/man-pages/man7/keyrings.7.html). If you'd like to manually observe or modify key operations related to `stash`, you can do so with the [`keyctl`](https://man7.org/linux/man-pages/man1/keyctl.1.html) program.

## Linux specifics

Note that `stash` is intended to run on modern, single-user Linux distributions. It has only been tested on Ubuntu 22.04. Note too that in order to install and run `stash`, you will need `root` privileges as well as the `useradd` and `sudo` commands. They will be used to create and authenticate the `stash` user.

By default, the `stash` user will be added to the group of the `UID` that creates it. Typically, then, for primary human user with `UID` of `1000`, `stash` will be added to `GID` of `1000`. Specifics might vary according to your local configuration, but the intent here is to give read and write permissions to the `stash` user across your home directory.

In addition to the command mentioned above, this program also makes use of `id`, `sh`, `tar`, and `ls`. It assumes the user has configured `HOME` and `USER` environment variables.

## Usage

Upon initialization, you will be prompted to create a password for the `stash` user. Make sure you can remember or access it if needed, because from this point on you will be using your `sudo` password to run the program. A new stash will then be created at `~/.stash` after successful creation of the `stash` user.

To encrypt a given file and add it to the stash, use:

	stash add <file>

To encrypt a copy of that file into the stash, use:

	stash add -c <file>

To decrypt a stashed file and drop it into the current directory, use:

	stash grab <file>

To decrypt a copy of that stashed file instead, use:

	stash grab -c <file>

To delete a stashed file, use:

	stash delete <file>

The contents of the stash are viewable with:

	stash list

All stashed files and directories can be archived into a `.tar.gz` file with:
```
stash archive
```
This will replace everything in the stash with an encrypted tarball called `contents`. It will also prevent you from adding anything else to the stash, or from grabbing anything except `contents`. To unpack that tarball and exit archive mode, use:
```
stash unpack
```
You can also delete the `contents` file in order to get out of archive mode.

`NOTE`: it may seem like a silly limitation, but it's important to point out that we have only been manually testing this using `cargo run`: throughout testing, we have only been working with files _in the current directory_ of the project. In other words, if you use filesystem paths to point anywhere else, you're going to get an OS error. Sadly, this program is (currently) only a proof of concept. Supporting real-world paths, directories, globbing, `ls` flags, etc, are all goals for continuing development.

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
- Added Linux keyrings support for managing key/nonce pairs.
- Added the `unpack` command for un-archiving.
- Added the `borrow` command for copying files out of the stash.
- Switched to `sudo` for authentication to avoid repeated password re-entry.
- Switched to `sled` database for secret storage.
- Added Linux keys again to cache secrets, avoid disk I/O.
- Added `is_archived` field to `Stash` struct.
- Combined `add()`/`copy()`, and `grab()`/`borrow()`.
- Changed command interface to use `-c` flag for copy behavior.
- Added detailed, verbose error handling.
- Added extensive doc comments via ChatGPT.
- Added MIT license.
- Removed bot comments, replaced with human comments.
- Added unit tests for valid cases of all core methods.

Future goals:

- Zeroize all sensitive data.
- Prevent OS from creating graphical login for `stash` user.
- Support more flexible file paths.
- Implement automatic, session-based encryption/decryption of database using `std::thread`.
- Handle multiple files per `add` or `grab` command.
- Support directory encryption/decryption.
- Support passing options to `ls` command via `list`.
- Possibly support globbing.
