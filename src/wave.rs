use babycat::{Signal, Waveform, WaveformArgs};
pub fn import_sound_file(filename: &str) -> Waveform {
    let waveform_args = WaveformArgs {
        convert_to_mono: true, // We convert everything to mono for now
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

pub fn transform_fourier(samples: &[f32], sampling_rate: u32) -> FrequencySpectrum {
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

pub fn create_spectrum(samples: &[f32], sampling_rate: u32, fft_size: usize, hop_size: usize, hop_count: isize/*<0 if full conversion*/) -> Vec<FrequencySpectrum> {
    let mut padded_samples: Vec<f32> = vec![0.0; fft_size / 2];
    padded_samples.extend(samples);
    padded_samples.resize(samples.len() + fft_size, 0.0);

    let last_sample = if hop_count < 0 { samples.len() } else { hop_size * hop_count as usize };
    (0..last_sample).step_by(hop_size)
    .map(|begin| transform_fourier(&padded_samples[begin..begin+fft_size], sampling_rate))
    .collect::<Vec<FrequencySpectrum>>()
}

pub fn spectrum_to_2d_vec(spectrogram: &Vec<FrequencySpectrum>) -> Vec<Vec<f32>> {
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

pub fn waveform_to_spectrogram(wf: &Waveform, fft_size: usize, hop_size: usize) -> note::Spectrogram {
    let spectrogram = create_spectrum(wf.to_interleaved_samples(), wf.frame_rate_hz(), fft_size, hop_size, -1);
    spectrum_to_2d_vec(&spectrogram)
}

pub fn waveform_to_spectrogram_countlimited(wf: &Waveform, fft_size: usize, hop_size: usize, hop_count: usize) -> note::Spectrogram {
    let spectrogram = create_spectrum(wf.to_interleaved_samples(), wf.frame_rate_hz(), fft_size, hop_size, hop_count as isize);
    spectrum_to_2d_vec(&spectrogram)
}

pub fn get_interesting_hopcounts(spectrogram: &note::Spectrogram) -> Vec<usize> {
    let sumvec = spectrogram.iter().map(|v| v.iter().sum()).collect::<Vec<f32>>();
    println!("Amplitude sums: {:?}", sumvec);
    let mut ret = Vec::new();

    if sumvec[0] > 0.1 { ret.push(0) } // TODO magic numbers everywhere xddd
    for i in 0..(sumvec.len()-1) {
        if sumvec[i] * 1.2/* TODO magic number */ < sumvec[i+1]
            && (ret.len() < 1 || ret[ret.len()-1] < i-2) {
            ret.push(i);
        }
    }

    println!("Interesting hopcounts: {:?}", ret);
    ret
}



use std::cmp::max;

use crate::note;
pub fn subtract_2d_vecs(one: &[Vec<f32>], other: &[Vec<f32>]) -> Vec<Vec<f32>> {
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

// multiplier: between 0.5 and 2.0 usually, those mean 1 octave higher and one octave lower
pub fn change_pitch(wf: &Waveform, multiplier: f32) -> Waveform {
    let original_hz = wf.frame_rate_hz();
    //println!("Converted {:?}", wf);
    let new_wf = wf.resample((original_hz as f32 / multiplier) as u32).unwrap();
    //println!("Through {:?}", new_wf);
    let even_newer_wf = Waveform::from_interleaved_samples(original_hz, new_wf.num_channels(), new_wf.to_interleaved_samples());
    //println!("To {:?}", even_newer_wf);
    even_newer_wf
}
