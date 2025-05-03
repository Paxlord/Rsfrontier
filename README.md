# rsfrontier

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](./LICENSE) 

`rsfrontier` is a command-line tool written in Rust for packing and unpacking various file formats used in the Monster Hunter Frontier Z (MHFZ) game client.

## Overview

This tool aims to provide a simple and efficient way to work with common MHFZ container and compression formats. It handles nested structures recursively, automatically detecting and processing formats like ECD encryption, JPK compression, Simple Archives, and MHA archives until the raw file data is extracted or the desired packing level is achieved.

## Features

*   **Packing:**
    *   Pack single files.
    *   Pack entire directories into Simple Archives (default) or MHA archives.
    *   Automatic JPK Type 4 (Huffman+LZ) compression for known file types (`.bin`, `.fmod`, `.fskl`) when packing directories.
    *   Optional JPK compression (Types 0, 2, 3, 4) for single files.
    *   Optional ECD encryption for the final packed output.
*   **Unpacking:**
    *   Recursively unpack archives and compressed files.
    *   Automatic detection and handling of:
        *   ECD Encryption
        *   JPK Compression (Types 0, 2, 3, 4)
        *   Simple Archives
        *   MHA Archives
    *   Automatic file extension detection based on magic bytes (e.g., `.dds`, `.png`, `.ogg`, `.fmod`, `.fskl`) where possible, defaulting to `.bin`.
*   **Cross-Platform:** Built with Rust, compilable for Windows, macOS, and Linux.
*   **Performance:** Designed with performance in mind, leveraging Rust's strengths.

## Supported Formats

The tool can automatically recognize and process the following formats during unpacking:

*   **Encryption:** ECD
*   **Compression:** JPK (Types 0, 2, 3, 4)
*   **Archives:**
    *   Simple Archive (often seen in `.pac` or nested within other files)
    *   MHA Archive (`.abn`)

## Installation

### Option 1: Pre-compiled Binaries (Recommended)

Check the **[Releases](https://github.com/[your-username/rsfrontier]/releases)** page for pre-compiled binaries for your operating system. Download the appropriate executable, place it somewhere in your system's `PATH`, or run it directly from its location.

### Option 2: Using Cargo

If you have the Rust toolchain installed (`rustup`), you can install directly from the source repository:

```bash
cargo install --git https://github.com/[your-username/rsfrontier].git rsfrontier-cli

```

Or, after cloning the repository (see Building from Source):

```bash
# Navigate to the cloned repository directory
cargo install --path ./rsfrontier-cli
```

This will compile and install the `rsfrontier` binary to your Cargo binary path (`~/.cargo/bin/` by default).

## Usage

The tool operates through two main subcommands: `pack` and `unpack`. You can get detailed help for each command:

```bash
rsfrontier --help
rsfrontier pack --help
rsfrontier unpack --help
```

### Packing

Use the `pack` command to combine files or directories.

**Syntax:**

```bash
rsfrontier pack -i <input-path> -o <output-file> [options]
```

**Examples:**

1.  **Pack a single file (no compression):**
    ```bash
    rsfrontier pack -i model.fskl -o packed_model.bin
    ```

2.  **Pack a single file with JPK Type 4 compression:**
    ```bash
    rsfrontier pack -i 000_model.fmod -o 000_model.bin -c 4
    ```
    *(Valid types for `-c`/`--compression-type` are 0, 2, 3, 4)*

3.  **Pack a directory into a Simple Archive (default):**
    *(Files like `.bin`, `.fmod`, `.fskl` inside `my_assets/` will be auto-compressed)*
    ```bash
    rsfrontier pack -i my_archive/ -o my_archive.pac
    ```

4.  **Pack a directory into an MHA Archive:**
    *(Requires base ID and capacity)*
    ```bash
    rsfrontier pack -i ./mha_source_files/ --mha --baseid 500 --capacity 500 -o custom_archive.abn
    ```

5.  **Pack a directory and encrypt the result:**
    ```bash
    rsfrontier pack -i ./data/ -o encrypted_archive.dat --encrypt
    ```

6.  **Pack to standard output:**
    ```bash
    rsfrontier pack -i file.dds | other_command
    ```

### Unpacking

Use the `unpack` command to extract files from archives or decompress/decrypt single files.

**Syntax:**

```bash
rsfrontier unpack -i <input-file> [-o <output-directory>]
```

**Examples:**

1.  **Unpack a file to a default directory:**
    *(If input is `mhfdat.bin`, creates `./mhfdat/` and unpacks contents there)*
    ```bash
    rsfrontier unpack -i mhfdat.bin
    ```

2.  **Unpack a file to a specific output directory:**
    *(If input is `resource.pak`, creates `./extracted/resource/` and unpacks contents there)*
    ```bash
    rsfrontier unpack -i resource.pac -o ./extracted/
    ```

3.  **Unpack an encrypted and compressed file:**
    *(Handles ECD decryption then JPK decompression automatically)*
    ```bash
    rsfrontier unpack -i encrypted_compressed.bin -o ./final_data/
    ```

## Building from Source

1.  **Install Rust:** If you don't have it, get it from [rustup.rs](https://rustup.rs/).
2.  **Clone the repository:**
    ```bash
    git clone https://github.com/Paxlord/rsfrontier.git
    cd rsfrontier
    ```
3.  **Build:**
    ```bash
    cargo build --release
    ```
4.  **Run:** The executable will be located at `target/release/rsfrontier-cli` (or `target\release\rsfrontier-cli.exe` on Windows). You can copy this file to a location in your system's `PATH`.

## License

This project is licensed under the [MIT License](./LICENSE). 

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## Acknowledgements

*   This project is heavily inspired by **ReFrontier**, developed by **MhVuze**. ReFrontier served as the primary reference for creating `rsfrontier`. Many thanks to MhVuze for his work.

