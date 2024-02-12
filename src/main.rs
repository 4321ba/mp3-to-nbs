mod cli;

fn max_of_slice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No NaN should be here")).expect("Slice shouldn't be empty")
}

use babycat::{Signal, Waveform, WaveformArgs};
fn import_sound_file(filename: &str) -> Waveform {
    let waveform_args = WaveformArgs {
        convert_to_mono: true,
        ..Default::default()
    };
    let waveform = Waveform::from_file(filename, waveform_args).expect("Decoding error");
    println!(
        "Decoded {} frames with {} channels at {} hz",
        waveform.num_frames(),
        waveform.num_channels(),
        waveform.frame_rate_hz(),
    );
    waveform
}

use clap::Parser;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencySpectrum};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::scaling::divide_by_N;

use crate::cli::Args;
fn transform_fourier(samples: &[f32], sampling_rate: u32) -> FrequencySpectrum {
    // apply hann window for smoothing; length must be a power of 2 for the FFT
    // 2048 is a good starting point with 44100 kHz
    let hann_window = hann_window(samples);
    // calc spectrum
    samples_fft_to_spectrum(
        // (windowed) samples
        &hann_window,
        // sampling rate
        sampling_rate,
        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
        FrequencyLimit::All,
        // optional scale
        Some(&divide_by_N),
    ).expect("Something went wrong with calculating fft")
}

fn create_spectrum(samples: &[f32], sampling_rate: u32, fft_size: usize, hop_size: usize) -> Vec<FrequencySpectrum> {
    let mut padded_samples: Vec<f32> = vec![0.0; fft_size / 2];
    padded_samples.extend(samples);
    padded_samples.resize(samples.len() + fft_size, 0.0);

    (0..samples.len()).step_by(hop_size)
    .map(|begin| transform_fourier(&padded_samples[begin..begin+fft_size], sampling_rate))
    .collect::<Vec<FrequencySpectrum>>()
}

fn main() {
    // Argument parsing
    let args = Args::parse();

    let waveform = import_sound_file(&args.input_file);
    let samples = waveform.to_interleaved_samples();

    println!("{:?}", &samples[105750..105800]);
    println!("{}", max_of_slice(samples));

    let spectrum_hann_window = transform_fourier(&samples[0..16], waveform.frame_rate_hz());

    for (fr, fr_val) in spectrum_hann_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }

    let test1 = create_spectrum(samples, waveform.frame_rate_hz(), 128, 1024);
    let test2 = create_spectrum(samples, waveform.frame_rate_hz(), 4096, 1024);
    println!("{} {}", test1.len(), test2.len());

    let width = test2.len() as u32;
    let height = test2[0].data().iter().count() as u32;
    let mut bytes: Vec<u8> = Vec::new();
    // Write a &str in the file (ignoring the result).
    for spectr in test2 {
        for (_, fr_val) in spectr.data().iter() {
            bytes.push((fr_val.val() * 256.0*128.0) as u8);
            bytes.push(0);
            bytes.push(0);
        }
    }
    image::save_buffer("image.png", &bytes, height, width, image::ColorType::Rgb8).unwrap()
}
