use std::convert::TryFrom;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

use crate::args::{DecodeArgs, EncodeArgs, PrintArgs, RemoveArgs};
use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::Png;
use crate::Result;

/// Encodes a message into a PNG file and saves the result
pub fn encode(encode_args: EncodeArgs) -> Result<()> {
    let bytes = get_bytes_from_path(&encode_args.file_path);
    let mut png = Png::try_from(bytes.as_slice())?;

    // Add the new chunk to the end of the file
    let i_end = png.remove_chunk("IEND")?;
    png.append_chunk(Chunk::new(
        ChunkType::from_str(encode_args.chunk_type.as_str())?,
        encode_args.message.as_bytes().to_vec(),
    ));
    png.append_chunk(i_end);

    let output_path = encode_args
        .output_path
        .unwrap_or(PathBuf::from("output.png"));

    fs::write(output_path, png.as_bytes())?;
    println!("Message encoded successfully!");

    Ok(())
}

/// Searches for a message hidden in a PNG file and prints the message if one is found
pub fn decode(args: DecodeArgs) -> Result<()> {
    let bytes = get_bytes_from_path(&args.file_path);
    let png = Png::try_from(bytes.as_slice())?;

    let target = png
        .chunk_by_type(args.chunk_type.as_str())
        .expect("Chunk not found");

    println!("Hidden message: {}", target.data_as_string()?);

    Ok(())
}

/// Removes a chunk from a PNG file and saves the result
pub fn remove(args: RemoveArgs) -> Result<()> {
    let bytes = get_bytes_from_path(&args.file_path);
    let mut png = Png::try_from(bytes.as_slice())?;

    png.remove_chunk(args.chunk_type.as_str())?;

    let output_path = args.file_path;
    fs::write(output_path, png.as_bytes())?;

    println!("Chunk removed successfully!");

    Ok(())
}

/// Prints all of the chunks in a PNG file
pub fn print_chunks(args: PrintArgs) -> Result<()> {
    let bytes = get_bytes_from_path(&args.file_path);
    let png = Png::try_from(bytes.as_slice())?;

    for chunk in png.chunks() {
        println!("{}", chunk);
    }

    Ok(())
}

fn get_bytes_from_path(path: &PathBuf) -> Vec<u8> {
    let mut f = File::open(path).expect("Unable to open file");
    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer).expect("Unable to read file");
    buffer
}
