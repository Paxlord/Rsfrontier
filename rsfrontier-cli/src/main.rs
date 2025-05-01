use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use rsfrontier_core::{PackType, unpack_buffer};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    unpack: Option<bool>,

    #[arg(short, long)]
    compress: Option<u16>,

    #[arg(short, long)]
    encrypt: Option<bool>,

    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    #[arg(short, long, value_name = "DEST")]
    output: Option<PathBuf>,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    decrypt: Option<bool>,
}

fn main() {
    let cli = Args::parse();
    let start = Instant::now();
    let input_path = cli.input;
    let mut output_path = cli.output.unwrap_or(PathBuf::from("./"));
    if let Some(name) = cli.name.as_deref() {
        println!("Name arg detected : {}", name);
        output_path.push(name);
    } else {
        output_path.push(input_path.file_stem().unwrap());
    }

    let file_buf = fs::read(input_path.as_os_str()).unwrap();
    let mut out_buf: Vec<u8> = file_buf;

    if cli.unpack.is_some() {
        println!("Unpacking buffer");
        let unpacked_file = unpack_buffer(output_path.to_str().unwrap(), &out_buf);
        for (path, buf) in unpacked_file {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path.as_os_str(), &buf).unwrap();
        }
        let duration = start.elapsed();
        println!("Processed file in {:?}", duration);
    }

    if let Some(jpk_type) = cli.compress {
        println!("Packing buffer with jpk type : {}", jpk_type);
        out_buf = rsfrontier_core::pack_buffer(&out_buf, PackType::Jpk(jpk_type));
    }

    if cli.encrypt.is_some() {
        println!("Encrypting the buffer...");
        out_buf = rsfrontier_core::pack_buffer(&out_buf, PackType::Ecd);
    }

    if cli.decrypt.is_some() {
        println!("Decrypting the buffer...");
        out_buf = rsfrontier_core::ecd::decrypt_ecd(&out_buf);
    }
}
