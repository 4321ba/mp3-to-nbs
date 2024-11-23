mod note;
mod wave;
mod optimize;
mod debug;
mod nbs;
mod tempo;
mod fourier;

mod observer;
mod complex_spectrum;

use babycat::{Signal, Waveform};
use clap::Parser;
use tracing::debug;
use note::add_notes_together;
use tempo::get_onsets_aubio;
use tempo::onsets_to_hopcounts;
use wave::add_waveforms_delayed;



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


fn main() {
    tracing_subscriber::fmt()
    //    .with_max_level(tracing::Level::DEBUG)
        .init();
    let args = Args::parse();
    debug!("Command line args: {:?}", args);

    let waveform = wave::import_sound_file(&args.input_file);

    let tps_approx = tempo::guess_tps_aubio(&waveform);
    let onsets = get_onsets_aubio(&waveform);
    let tps_guessed = tempo::guess_exact_tps(&onsets, waveform.frame_rate_hz(), tps_approx);
    let tps = if args.tps > 0.0 { args.tps } else { tps_guessed };
    println!("Recognizing {}, using tps {}", &args.input_file, tps);
    debug!("Approximate tps: {}, exact guessed tps: {}, given tps: {}, final choice: {}", tps_approx, tps_guessed, args.tps, tps);
    let evened_onsets = tempo::even_onsets(tps, waveform.frame_rate_hz(), waveform.num_samples());
    debug!("Onsets: {:?}", evened_onsets);

    let cache = note::cache_instruments(&args.sounds_folder);

    // multithreading cannot directly be used sadly, as we use the previous guessed notes for the next guess
    //let all_found_notes = hopcounts.par_iter().map(|i| optimize::full_optimize_timestamp(&cache, &spectrogram, *i)).collect();
    
    let mut all_found_notes = Vec::new();
    let mut accumulator_waveform = Waveform::from_frames_of_silence(waveform.frame_rate_hz(), waveform.num_channels(), 10);
    for onset in &evened_onsets {
        let percentage = *onset as f32 / *evened_onsets.last().unwrap() as f32 * 100.0;
        println!("Recognizing {}, currently at {}%", &args.input_file, percentage);
        debug!("Starting recognition at onset {}, at {}%", onset, percentage);
        let notes = optimize::full_optimize_timestamp(&cache, &accumulator_waveform, &waveform, *onset, tps);
        debug!("Found notes: {:?}", notes);

        let current_notes = add_notes_together(&notes, &cache, 1.0);
        accumulator_waveform = add_waveforms_delayed(&accumulator_waveform, &current_notes, *onset);

        all_found_notes.push(notes);
    }

    let hopcounts = onsets_to_hopcounts(&evened_onsets, fourier::HOP_SIZE);
    let timestamps = tempo::convert_hopcounts_to_ticks(&hopcounts, tps, fourier::HOP_SIZE, waveform.frame_rate_hz());
    debug!("Valid tick positions: {:?}", timestamps);
    nbs::export_notes(&nbs::clean_quiet_notes(&all_found_notes), &timestamps, tps, &args.output_file);
}
