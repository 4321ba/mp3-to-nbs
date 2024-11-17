use clap::Parser;


#[derive(Parser, Debug)]
#[command(author = "1234ab", version, about, long_about = "mp3 to nbs converter")]
pub struct Args {
    #[arg(short, long, long_help = "The waveform file that should be parsed to nbs.\nExample: song.mp3", required = true)]
    pub input_file: String,
    #[arg(short, long, long_help = "The output NBS file.\nExample: song.nbs", required = true)]
    pub output_file: String,
    #[arg(short, long, long_help = "The folder where the sounds are.\nExample: Sounds", required = false, default_value = "Sounds")]
    pub sounds_folder: String,
    #[arg(short, long, long_help = "The tempo in ticks per second.\nExample: 6.75", required = false, default_value = "-1.0")]
    pub tps: f64,
}
