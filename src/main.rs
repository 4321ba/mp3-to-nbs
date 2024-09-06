use clap::Parser;
use mp3_to_nbs::{cli::Args, parse_and_export, wave::import_sound_file};

fn main() {
    tracing_subscriber::fmt::init();

    // Argument parsing
    let args = Args::parse();

    // println!("{:?}", args);

    parse_and_export(&import_sound_file(&args.input_file));
}
