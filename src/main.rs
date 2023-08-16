mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use args::{Cli, PngMeArgs};
use clap::Parser;
use commands::{decode, encode, print_chunks, remove};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(PngMeArgs::Encode(encode_args)) => encode(encode_args)?,
        Some(PngMeArgs::Decode(decode_args)) => decode(decode_args)?,
        Some(PngMeArgs::Remove(remove_args)) => remove(remove_args)?,
        Some(PngMeArgs::Print(print_args)) => print_chunks(print_args)?,
        None => println!("No command provided"),
    };

    Ok(())
}
