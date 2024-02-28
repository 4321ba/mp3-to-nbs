mod cli;

fn max_of_slice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No NaN should be here")).expect("Slice shouldn't be empty")
}

//use std::hint;

use std::char::MAX;

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

fn subtract_2d_vecs(one: &[Vec<f32>], other: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let bigger_width = max(one.len(), other.len());
    let height = one[0].len();
    assert!(height == other[0].len()); // though dfferent vectors could still be different sizes
    let mut ret = vec![vec![0.0_f32; one[0].len()]; bigger_width];
    for x in 0..bigger_width {
        for y in 0..height {
            let one_val = match one.get(x) { Some(v) => v[y], None => 0.0 };
            let other_val = match other.get(x) { Some(v) => v[y], None => 0.0 };
            ret[x][y] = one_val - other_val;
        }
    }
    ret
}

fn debug_save_as_image(array2d: &[Vec<f32>], filename: &str) {

    let mut bytes: Vec<u8> = Vec::new();
    // Write a &str in the file (ignoring the result).
    //let array2d = spectrum_to_2d_vec(spectrogram);
    let width = array2d.len();
    let height = array2d[0].len();
    for y in (0..height/8).rev() {//0 instead of height*7/8 to print the whole thing
        for i in 0..4 {
            for x in 0..width {
                for j in 0..4 {
                    let strength = (array2d[x][y] * 256.0*16.0) as i8;
                    if strength >= 0 {
                        bytes.push(strength as u8);
                        bytes.push(strength as u8);
                        bytes.push(strength as u8);
                    } else {
                        bytes.push((-(strength as i16)) as u8);
                        bytes.push(0);
                        bytes.push(0);
                    }
                }
            }

        }
    }
    image::save_buffer(filename, &bytes, 4*width as u32, 4*(height/8) as u32, image::ColorType::Rgb8).unwrap()
}

fn debug_save_as_wav(wf: &Waveform, filename: &str) {
    wf.to_wav_file(filename).unwrap();
}

// multiplier: between 0.5 and 2.0 usually, those mean 1 octave higher and one octave lower
fn change_pitch(wf: &Waveform, multiplier: f32) -> Waveform {
    let original_hz = wf.frame_rate_hz();
    //println!("Converted {:?}", wf);
    let new_wf = wf.resample((original_hz as f32 / multiplier) as u32).unwrap();
    //println!("Through {:?}", new_wf);
    let even_newer_wf = Waveform::from_interleaved_samples(original_hz, new_wf.num_channels(), new_wf.to_interleaved_samples());
    //println!("To {:?}", even_newer_wf);
    even_newer_wf
}

use std::cmp::max;
fn calculate_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, dist: &dyn Fn(f32, f32) -> f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though dfferent vectors could still be different sizes
    let mut distance = 0.0_f32;
    for x in 0..bigger_width {
        for y in 0..song_part[0].len() {
            let song_part_val = match song_part.get(x) { Some(v) => v[y], None => 0.0 };
            let sample_val = match sample.get(x) { Some(v) => v[y], None => 0.0 };
            distance += dist(song_part_val, sample_val/2.0); // TODO
        }
    }
    distance
}

fn calculate_assymetric_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>) -> f32 {
    calculate_distance(song_part, sample, &|sp, sa| if sp >= sa {0.0} else {(sp-sa) * (sp-sa)} )
}


fn test_distances_for_instruments(waveform: &Waveform) {
    let fft_size = 4096;
    let samples = waveform.to_interleaved_samples();
    let spectrogram = create_spectrum(samples, waveform.frame_rate_hz(), fft_size, 1024);
    let spectrogram_2dvec = spectrum_to_2d_vec(&spectrogram);
    let song_part = &spectrogram_2dvec[0..40];
    debug_save_as_image(song_part, "song_part.png");
/*
    const INSTRUMENT_FILENAMES: &[&str] = &[
        "Sounds/banjo.ogg",
        "Sounds/bdrum.ogg",
        "Sounds/bell.ogg",
        "Sounds/bit.ogg",
        "Sounds/click.ogg",
        "Sounds/cow_bell.ogg",
        "Sounds/dbass.ogg",
        "Sounds/didgeridoo.ogg",
        "Sounds/flute.ogg",
        "Sounds/guitar.ogg",
        "Sounds/harp.ogg",
        "Sounds/icechime.ogg",
        "Sounds/iron_xylophone.ogg",
        "Sounds/pling.ogg",
        "Sounds/sdrum.ogg",
        "Sounds/xylobone.ogg",
    ];*/

    const INSTRUMENT_FILENAMES: &[&str] = &[
        "Sounds/dbass.ogg",
        "Sounds/harp.ogg",
    ];
    
    for instr_filename in INSTRUMENT_FILENAMES {
        print!("\n{}\n", instr_filename);
        let sample_wf = import_sound_file(instr_filename);
        for pitch in 0..=24 {
            let multiplier = 2.0_f64.powf((pitch - 12) as f64 / 12.0);
            let sample_wf_diff_pitch = change_pitch(&sample_wf, multiplier as f32);
            let sample_spectrogram = create_spectrum(
                sample_wf_diff_pitch.to_interleaved_samples(), sample_wf_diff_pitch.frame_rate_hz(), fft_size, 1024);
            let sample_2dvec = spectrum_to_2d_vec(&sample_spectrogram);
            debug_save_as_image(&subtract_2d_vecs(song_part, &sample_2dvec), &format!("{instr_filename}_pitch{pitch:02}.png"));

            let diff = calculate_assymetric_distance(song_part, &sample_2dvec);
            println!("{pitch:02}: {diff:.5}");
            
        }
    }
}

fn test_main(waveform: &Waveform) {

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
    debug_save_as_image(&spectrum_to_2d_vec(&test2), "image.png");
    let waveform_diff_pitch = change_pitch(&waveform, 1.5);
    let test3 = create_spectrum(
        waveform_diff_pitch.to_interleaved_samples(), waveform_diff_pitch.frame_rate_hz(), 4096, 1024);
    debug_save_as_image(&spectrum_to_2d_vec(&test3), "image2.png");
    debug_save_as_wav(waveform, "wf1.wav");
    debug_save_as_wav(&waveform_diff_pitch, "wf2.wav");
}

use clap::Parser;
use crate::cli::Args;
fn main() {
    // Argument parsing
    let args = Args::parse();
    println!("{:?}", args);
    let waveform = import_sound_file(&args.input_file);

    //test_main(&waveform);
    test_distances_for_instruments(&waveform);

}
