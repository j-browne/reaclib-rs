use clap::Parser;
use reaclib::{Format, Iter};
use serde_json::to_writer_pretty;
use std::{
    error::Error,
    fs::File,
    io::{stdout, BufReader},
};

/// Example program for converting from the raclib format to json
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    /// The reaclib format of the file (1, 2).
    #[arg(short, long, value_parser = format_parse)]
    format: Format,

    /// File to read from.
    file: String,
}

fn format_parse(s: &str) -> Result<Format, String> {
    match s.parse::<u8>() {
        Ok(1) => Ok(Format::Reaclib1),
        Ok(2) => Ok(Format::Reaclib2),
        _ => Err("Only '1' and '2' are valid formats".to_string()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let file = File::open(cli.file)?;
    let file = BufReader::new(file);

    let v = Iter::new(file, cli.format).collect::<Result<Vec<_>, _>>()?;
    let writer = stdout().lock();
    to_writer_pretty(writer, &v)?;

    Ok(())
}
