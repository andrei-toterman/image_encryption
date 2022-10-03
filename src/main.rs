use clap::Parser;

use image_encryption::{decrypt_image, encrypt_image, load_image, write_image};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Mode {
    Enc,
    Dec,
}

/// simple image encryption program
#[derive(Debug, Parser)]
struct Args {
    /// encrypt an image or decrypt an encrypted one
    #[clap(value_enum)]
    mode: Mode,
    /// the encryption/decryption key
    key: u64,
    /// image input path
    input: String,
    /// image output path
    /// if omitted, input file is overwritten
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut img = match load_image(&args.input) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    match args.mode {
        Mode::Enc => encrypt_image(&mut img, args.key),
        Mode::Dec => decrypt_image(&mut img, args.key),
    }

    if let Err(err) = write_image(args.output.unwrap_or(args.input), img) {
        eprintln!("{}", err)
    };
}
