mod cli;

fn max_of_slice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No NaN should be here")).expect("Slice shouldn't be empty")
}

//use std::hint;

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

use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencySpectrum};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::scaling::divide_by_N;

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

fn spectrum_to_2d_vec(spectrogram: &Vec<FrequencySpectrum>) -> Vec<Vec<f32>> {
    // https://stackoverflow.com/questions/13212212/creating-two-dimensional-arrays-in-rust
    let width = spectrogram.len();
    let height = spectrogram[0].data().iter().count();

    // we index with x first! so that it's easier to cut in the time dimension
    let mut array = vec![vec![0.0_f32; height]; width];
    for x in 0..width {
        for y in 0..height {
            let frequency_value = spectrogram[x].data()[y].1;
            array[x][y] = frequency_value.val();
        }
    }
    array
}

fn debug_save_as_image(spectrogram: &Vec<FrequencySpectrum>, filename: &str) {

    let mut bytes: Vec<u8> = Vec::new();
    // Write a &str in the file (ignoring the result).
    let array2d = spectrum_to_2d_vec(spectrogram);
    let width = array2d.len();
    let height = array2d[0].len();
    for y in (0..height).rev() {
        for x in 0..width {
            let strength = (array2d[x][y] * 256.0*16.0) as u8;
            bytes.push(strength);
            bytes.push(strength);
            bytes.push(strength);
        }
    }
    image::save_buffer(filename, &bytes, width as u32, height as u32, image::ColorType::Rgb8).unwrap()
}

fn debug_save_as_wav(wf: Waveform, filename: &str) {
    wf.to_wav_file(filename).unwrap();
}

// multiplier: between 0.5 and 2.0 usually, those mean 1 octave higher and one octave lower
fn change_pitch(wf: &Waveform, multiplier: f32) -> Waveform {
    let original_hz = wf.frame_rate_hz();
    println!("Converted {:?}", wf);
    let new_wf = wf.resample((original_hz as f32 / multiplier) as u32).unwrap();
    println!("Through {:?}", new_wf);
    let even_newer_wf = Waveform::from_interleaved_samples(original_hz, new_wf.num_channels(), new_wf.to_interleaved_samples());
    println!("To {:?}", even_newer_wf);
    even_newer_wf
}

use clap::Parser;
use crate::cli::Args;
fn main() {
    // Argument parsing
    let args = Args::parse();
    println!("{:?}", args);
    let waveform = import_sound_file(&args.input_file);
    let samples = waveform.to_interleaved_samples();

    let harp_wf = import_sound_file("Sounds/harp.ogg");

    //println!("{:?}", &samples[105750..105800]);
    println!("{}", max_of_slice(samples));

    /*
    let spectrum_hann_window = transform_fourier(&samples[0..16], waveform.frame_rate_hz());

    for (fr, fr_val) in spectrum_hann_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }*/

    //let test1 = create_spectrum(samples, waveform.frame_rate_hz(), 128, 1024);
    let test2 = create_spectrum(samples, waveform.frame_rate_hz(), 4096, 1024);
    //println!("{} {}", test1.len(), test2.len());
    debug_save_as_image(&test2, "image.png");
    let waveform_diff_pitch = change_pitch(&waveform, 1.5);
    let test3 = create_spectrum(
        waveform_diff_pitch.to_interleaved_samples(), waveform_diff_pitch.frame_rate_hz(), 4096, 1024);
    debug_save_as_image(&test3, "image2.png");
    debug_save_as_wav(waveform, "wf1.wav");
    debug_save_as_wav(waveform_diff_pitch, "wf2.wav");
}
