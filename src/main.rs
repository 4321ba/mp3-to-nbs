mod cli;
mod note;
mod wave;
mod optimize;
mod debug;

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
    let test2 = wave::create_spectrum(samples, waveform.frame_rate_hz(), 4096, 1024);
    //println!("{} {}", test1.len(), test2.len());
    debug_save_as_image(&wave::spectrum_to_2d_vec(&test2), "image.png");
    let waveform_diff_pitch = wave::change_pitch(&waveform, 1.5);
    let test3 = wave::create_spectrum(
        waveform_diff_pitch.to_interleaved_samples(), waveform_diff_pitch.frame_rate_hz(), 4096, 1024);
    debug_save_as_image(&wave::spectrum_to_2d_vec(&test3), "image2.png");
    debug_save_as_wav(waveform, "wf1.wav");
    debug_save_as_wav(&waveform_diff_pitch, "wf2.wav");
}









use clap::Parser;
use crate::{cli::Args, note::Note};
fn main() {
    // Argument parsing
    let args = Args::parse();
    println!("{:?}", args);

    
    let waveform = wave::import_sound_file(&args.input_file);
    
    let cache = note::cache_instruments();
    optimize::optimize(&cache, &waveform);
    //test_main(&waveform);
    optimize::test_distances_for_instruments(&waveform, &cache);

}
