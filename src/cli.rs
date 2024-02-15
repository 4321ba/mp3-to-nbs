use clap::Parser;


#[derive(Parser, Debug)]
#[command(author = "1234ab", version, about, long_about = "mp3 to nbs converter")]
pub struct Args {
    #[arg(short, long, long_help = "The file that should be parsed to nbs.\nExample song.mp3", required = true)]
    pub input_file: String,
}
