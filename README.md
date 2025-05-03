# rsfrontier

[![crates.io](https://img.shields.io/crates/v/rsfrontier-cli.svg?style=flat-square)](https://crates.io/crates/rsfrontier-cli) <!-- Optional: Add if you publish to crates.io -->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](./LICENSE) <!-- Optional: Update if you choose a different license -->

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
    *   MHA Archive (`.mha`)

## Installation

### Option 1: Pre-compiled Binaries (Recommended)

Check the **[Releases](https://github.com/[your-username/rsfrontier]/releases)** page for pre-compiled binaries for your operating system. Download the appropriate executable, place it somewhere in your system's `PATH`, or run it directly from its location.

### Option 2: Using Cargo

If you have the Rust toolchain installed (`rustup`), you can install directly from the source repository:

```bash
cargo install --git https://github.com/[your-username/rsfrontier].git rsfrontier-cli