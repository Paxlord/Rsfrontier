use core::panic;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

use clap::{Parser, Subcommand};
use rsfrontier_core::{FolderPackType, PackType, pack_buffer, pack_folder, unpack_buffer};

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

        #[arg(short, long)]
        mha: bool,

        #[arg(short, long)]
        capacity: Option<u16>,

        #[arg(short, long)]
        baseid: Option<u16>,
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
            mha,
            capacity,
            baseid,
        } => {
            let packed_data;

            if input.is_dir() {
                if mha {
                    if let Some(capacity) = capacity {
                        if let Some(baseid) = baseid {
                            packed_data =
                                pack_folder(&input, FolderPackType::MHA(baseid, capacity));
                        } else {
                            panic!("Base file id needed for MHA repacking");
                        };
                    } else {
                        panic!("Capcity needed for MHA repacking");
                    }
                } else {
                    packed_data = pack_folder(&input, FolderPackType::Simple);
                }
            } else {
                let file_buf = fs::read(&input).unwrap();
                if let Some(jpk_type) = compression {
                    packed_data = pack_buffer(&file_buf, PackType::Jpk(jpk_type as u16));
                } else {
                    packed_data = file_buf;
                }
            }

            if packed_data.is_empty() {
                panic!("Resulting packed buffer ended up empty");
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
