use core::panic;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

use clap::{Parser, Subcommand};
use rsfrontier_core::{
    FolderPackType, PackType,
    ecd::{decrypt_ecd, is_buf_ecd},
    pack_buffer, pack_em_folder, pack_folder, unpack_buffer,
};

/// A command-line tool for packing and unpacking various file formats
/// used in Monster Hunter Frontier Z (MHFZ).
///
/// Supports automatic handling of:
/// - ECD Encryption/Decryption
/// - JPK Compression/Decompression (Types 0, 2, 3, 4)
/// - Simple Archive Packing/Unpacking
/// - MHA/ABN Archive Packing/Unpacking
#[derive(Parser)]
#[command(author="Pax", version="0.0.1", about="Tool for packing and unpacking mhfz files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Packs a single file or an entire directory recursively.
    ///
    /// When packing a directory:
    /// - By default, it creates a 'Simple Archive'.
    /// - Subdirectories become nested Simple Archives.
    /// - Files matching known extensions (.bin, .fmod, .fskl) inside directories
    ///   are automatically compressed using JPK Type 4 (Huffman+LZ) before archiving.
    /// - Use the --mha flag to create an MHA archive instead (requires --capacity and --baseid).
    ///
    /// When packing a single file:
    /// - Use --compression to apply JPK compression.
    ///
    /// Use --encrypt to apply ECD encryption to the final output (after any packing/compression).
    Pack {
        /// Path to the input file or directory to pack.
        #[arg(short, long, value_name = "PATH")]
        input: PathBuf,

        /// Path to the output file.
        /// If omitted, the packed data is written to standard output (stdout).
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// JPK compression type to apply (only when the input is a single file).
        /// Valid types:
        ///   0: Raw (no compression)
        ///   2: HuffmanRW
        ///   3: LZ
        ///   4: Huffman + LZ
        /// If input is a directory, JPK compression is applied automatically where appropriate.
        #[arg(short, long)]
        compression: Option<u8>,

        /// Encrypt the final output buffer using ECD encryption.
        /// This happens *after* all packing and compression steps.
        #[arg(short, long)]
        encrypt: bool,

        /// Pack the input directory as an MHA archive instead of a Simple Archive.
        /// This flag is only effective when the input path is a directory.
        /// Requires --capacity and --baseid to be specified.
        #[arg(long)]
        mha: bool,

        /// Set the 'capacity' field for the MHA archive header.
        /// Required if --mha is used.
        #[arg(long, value_name = "COUNT", requires = "mha")]
        capacity: Option<u16>,

        /// Set the 'base file ID' for the MHA archive header.
        /// Files within the MHA will be assigned IDs starting from this value.
        /// Required if --mha is used.
        #[arg(long, value_name = "ID", requires = "mha")]
        baseid: Option<u16>,

        /// Flag for specially handling monster archives that need a special packing scheme.
        #[arg(long)]
        em: bool,
    },

    /// Unpacks an MHFZ file recursively, handling nested archives and compressions.
    ///
    /// Automatically detects and handles:
    /// - ECD encryption
    /// - JPK compression (Types 0, 2, 3, 4)
    /// - Simple Archives
    /// - MHA Archives
    ///
    /// Unpacking continues until raw file data is reached. File extensions (.dds, .png, .ogg, etc.)
    /// are automatically determined based on magic bytes where possible, otherwise defaults to '.bin'.
    Unpack {
        /// Path to the input file to unpack (e.g., .bin, .dat, .pak).
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        // Path to the output directory.
        /// - If omitted, creates a directory in the current location named after the
        ///   input file (e.g., unpacking 'mhfdat.bin' creates './mhfdat/').
        /// - If a directory path is provided, unpacked files are placed inside that directory,
        ///   within a subdirectory named after the input file stem
        ///   (e.g., unpacking 'data.pak' with -o /tmp creates '/tmp/data/').
        /// - If a non-existent path ending not with '/' or '\' is provided, it might be treated
        ///   as a base path/prefix, potentially leading to unexpected results if it conflicts
        ///   with directory creation logic. It's safer to provide directory paths.
        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,

        /// Only decrypt the input file using ECD without unpacking.
        /// This is useful for decrypting files without further processing.
        #[arg(long)]
        decrypt: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.command {
        Commands::Pack {
            input,
            output,
            compression,
            encrypt,
            mha,
            capacity,
            baseid,
            em,
        } => {
            let packed_data;

            if input.is_dir() {
                if mha {
                    if em {
                        panic!("--em cannot be used with --mha. Use --mha only for MHA archives.");
                    }
                    let capacity = capacity.expect("--capacity is required with --mha");
                    let baseid = baseid.expect("--baseid is required with --mha");
                    packed_data = pack_folder(&input, FolderPackType::MHA(baseid, capacity));
                } else if em {
                    packed_data = pack_em_folder(&input);
                } else {
                    if compression.is_some() {
                        panic!(
                            "--compression cannot be used when packing a directory into a Simple Archive (default). JPK is applied automatically inside."
                        );
                    }
                    packed_data = pack_folder(&input, FolderPackType::Simple);
                }
            } else {
                if mha {
                    panic!(
                        "--mha, --capacity, --baseid flags can only be used when the input is a directory."
                    );
                }
                if em {
                    panic!("--em cannot be used when packing a single file.");
                }
                let file_buf = fs::read(&input).unwrap();
                if let Some(jpk_type) = compression {
                    match jpk_type {
                        0 | 2 | 3 | 4 => {
                            packed_data = pack_buffer(&file_buf, PackType::Jpk(jpk_type as u16));
                        }
                        _ => {
                            panic!(
                                "Invalid JPK compression type: {}. Valid types are 0, 2, 3, 4.",
                                jpk_type
                            );
                        }
                    }
                } else {
                    packed_data = file_buf;
                }
            }

            if packed_data.is_empty() {
                eprintln!(
                    "Warning: Resulting packed buffer is empty. Input directory might have been empty or contained only hidden files."
                );
            }

            let out_data = if encrypt {
                pack_buffer(&packed_data, PackType::Ecd)
            } else {
                packed_data
            };

            if let Some(path) = output {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(path, out_data).unwrap();
            } else {
                io::stdout().write_all(&out_data).unwrap();
            }
        }
        Commands::Unpack {
            input,
            output,
            decrypt,
        } => {
            let output_path = if let Some(path) = output {
                let derived_path = if path.is_dir() {
                    let mut new_path = path.clone();
                    let input_file_name = input.file_stem().unwrap_or_default();
                    new_path.push(input_file_name);
                    new_path
                } else {
                    path
                };
                derived_path
            } else {
                let stem = input
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                PathBuf::from(stem)
            };

            let file_buf = fs::read(&input).unwrap();
            if decrypt {
                if !is_buf_ecd(&file_buf) {
                    panic!("Input file is not ECD encrypted.");
                }
                let decrypted_buf = decrypt_ecd(&file_buf);
                fs::write(&output_path, decrypted_buf).unwrap();
                return;
            }
            let unpacked_files = unpack_buffer(&output_path.to_string_lossy(), &file_buf);

            for (path, buf) in unpacked_files {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }

                fs::write(path, buf).unwrap();
            }
        }
    }
    let duration = start.elapsed();
    println!("Processed command in {:?}", duration);
}
