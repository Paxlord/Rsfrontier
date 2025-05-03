use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{Parser, Subcommand};
use rsfrontier_core::{PackType, jpk, pack_buffer, pack_folder, recursive_pack, unpack_buffer};

#[derive(Parser)]
#[command(author="Pax", version="0.0.1", about="Took for packing and unpacking mhfz files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Pack {
        #[arg(short, long, value_name = "PATH")]
        input: PathBuf,

        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        #[arg(short, long)]
        compression: Option<u8>,

        #[arg(short, long)]
        encrypt: bool,
    },

    Unpack {
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,
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
        } => {
            let packed_data = if input.is_dir() {
                pack_folder(&input)
            } else {
                let file_buf = fs::read(&input).unwrap();
                if let Some(jpk_type) = compression {
                    pack_buffer(&file_buf, PackType::Jpk(jpk_type as u16))
                } else {
                    file_buf
                }
            };

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
        Commands::Unpack { input, output } => {
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
