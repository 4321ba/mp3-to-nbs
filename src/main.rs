mod cli;
mod note;
mod wave;
mod optimize;
mod debug;
mod nbs;
mod complex_lib;
mod tempo;

fn max_of_slice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No NaN should be here")).expect("Slice shouldn't be empty")
}

//use std::hint;

use std::char::MAX;

use babycat::{Signal, Waveform, WaveformArgs};




use debug::debug_save_as_image;
use debug::debug_save_as_wav;
fn test_main(waveform: &Waveform) {

    let samples = waveform.to_interleaved_samples();
    let harp_wf = wave::import_sound_file("Sounds/harp.ogg");

    //println!("{:?}", &samples[105750..105800]);
    println!("{}", max_of_slice(samples));

    /*
    let spectrum_hann_window = transform_fourier(&samples[0..16], waveform.frame_rate_hz());

    for (fr, fr_val) in spectrum_hann_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }*/

    //let test1 = create_spectrum(samples, waveform.frame_rate_hz(), 128, 1024);
    let test2 = wave::create_spectrum(samples, waveform.frame_rate_hz(), 4096, 1024, -1);
    //println!("{} {}", test1.len(), test2.len());
    debug_save_as_image(&wave::spectrum_to_2d_vec(&test2), "image.png");
    let waveform_diff_pitch = wave::change_pitch(&waveform, 1.5);
    let test3 = wave::create_spectrum(
        waveform_diff_pitch.to_interleaved_samples(), waveform_diff_pitch.frame_rate_hz(), 4096, 1024, -1);
    debug_save_as_image(&wave::spectrum_to_2d_vec(&test3), "image2.png");
    debug_save_as_wav(waveform, "wf1.wav");
    debug_save_as_wav(&waveform_diff_pitch, "wf2.wav");
}










use clap::Parser;
use crate::wave::waveform_to_spectrogram;
use crate::{cli::Args, note::Note};
use rayon::prelude::*;
fn main() {
    // Argument parsing
    let args = Args::parse();
    println!("{:?}", args);

    
    let waveform = wave::import_sound_file(&args.input_file);


    //let cache = note::cache_instruments();
    //test_main(&waveform);
    
    let spectrogram = wave::waveform_to_spectrogram(&waveform, 4096, 1024);

    let hopcounts = tempo::get_interesting_hopcounts(&spectrogram);
    //let hopcounts2: &[usize] = &hopcounts;

    let tps2 = tempo::guess_tps_aubio(&waveform);
    let tps3 = tempo::guess_exact_tps(&hopcounts, 1024, waveform.frame_rate_hz(), tps2);

    let tps = tempo::guess_tps(&hopcounts, 1024, waveform.frame_rate_hz());
    //let tps = 10.0;//TODO hardcoded for now
    dbg!(tps);
    dbg!(tps2);
    dbg!(tps3);

/*
    //let all_found_notes = hopcounts.par_iter().map(|i| optimize::full_optimize_timestamp(&cache, &spectrogram, *i)).collect();
    let mut all_found_notes = Vec::new();
    let mut accumulator_spectrogram = Vec::new();
    for i in &hopcounts { // TODO parallelization
        let notes = optimize::full_optimize_timestamp(&cache, &spectrogram, *i, &accumulator_spectrogram);
        println!("Found notes: {:?}", notes);

        let mut padded_spectrogram: note::ComplexSpectrogram = vec![vec![0.0.into(); spectrogram[0].len()]; *i];
        padded_spectrogram.extend(note::add_note_spectrograms(&notes, &Vec::new(), &cache, 1.0));
        accumulator_spectrogram = note::add_spectrograms(&padded_spectrogram, &accumulator_spectrogram);

        debug::debug_save_as_image(&wave::complex_spectrogram_to_amplitude(&accumulator_spectrogram), "accumulated.png");

        all_found_notes.push(notes);
    }
    
    println!("Found all notes: {:?}", all_found_notes);

    dbg!(&hopcounts);
    let timestamps = tempo::convert_hopcounts_to_ticks(&hopcounts, tps, 1024, waveform.frame_rate_hz());
    dbg!(&timestamps);
    nbs::export_notes(&nbs::clean_quiet_notes(&all_found_notes), &timestamps, tps);*/
}
