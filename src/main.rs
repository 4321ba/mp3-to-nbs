use babycat::{Signal, Waveform, WaveformArgs};

use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::scaling::divide_by_N;

fn maxslice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No nan!!")).expect("slice shouldnt be empty")
}

fn main() {
    let waveform_args = WaveformArgs {
        convert_to_mono: true,
        ..Default::default()
    };
    let waveform =
        match Waveform::from_file("musictests/olddiscjing.mp3", waveform_args) {
            Ok(w) => w,
            Err(err) => {
                println!("Decoding error: {}", err);
                return;
            }
        };
    println!(
        "Decoded {} frames with {} channels at {} hz",
        waveform.num_frames(),
        waveform.num_channels(),
        waveform.frame_rate_hz(),
    );
    
    let samples = waveform.to_interleaved_samples();
    println!("{:?}", &samples[105750..105800]);
    println!("{}", maxslice(samples));


    // apply hann window for smoothing; length must be a power of 2 for the FFT
    // 2048 is a good starting point with 44100 kHz
    let hann_window = hann_window(&samples[0..2048]);
    // calc spectrum
    let spectrum_hann_window = samples_fft_to_spectrum(
        // (windowed) samples
        &hann_window,
        // sampling rate
        waveform.frame_rate_hz(),
        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
        FrequencyLimit::All,
        // optional scale
        Some(&divide_by_N),
    ).unwrap();

    for (fr, fr_val) in spectrum_hann_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }
}
