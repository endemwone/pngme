use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<PngMeArgs>,
}

#[derive(Subcommand)]
pub enum PngMeArgs {
    Encode(EncodeArgs),
    Decode(DecodeArgs),
    Remove(RemoveArgs),
    Print(PrintArgs),
}

#[derive(Args, Debug)]
pub struct EncodeArgs {
    pub file_path: PathBuf,
    pub chunk_type: String,
    pub message: String,
    pub output_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct DecodeArgs {
    pub file_path: PathBuf,
    pub chunk_type: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    pub file_path: PathBuf,
    pub chunk_type: String,
}

#[derive(Args, Debug)]
pub struct PrintArgs {
    pub file_path: PathBuf,
}
