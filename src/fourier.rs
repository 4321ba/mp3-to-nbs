use microfft::Complex32;
use spectrum_analyzer::windows::hann_window;
use std::cmp::max;
use crate::{complex_spectrum::samples_fft_to_complex_spectrum, note::ComplexSpectrogram};


pub const HOP_SIZE: usize = 1024;
pub const FFT_SIZE: usize = 4096;


fn transform_fourier_complex(samples: &[f32]) -> Vec<Complex32> {
    let hann_window = hann_window(samples);
    samples_fft_to_complex_spectrum(&hann_window).expect("Something went wrong with calculating fft")
}


fn create_complex_spectrogram(samples: &[f32], fft_size: usize, hop_size: usize, hop_count: isize/*<0 if full conversion*/) -> Vec<Vec<Complex32>> {
    let mut padded_samples: Vec<f32> = vec![0.0; fft_size / 2];
    padded_samples.extend(samples);
    padded_samples.resize(max(samples.len() as isize, hop_size as isize*hop_count) as usize + fft_size, 0.0); // so that empty samples are converted well as well

    let last_sample = if hop_count < 0 { samples.len() } else { hop_size * hop_count as usize };
    (0..last_sample).step_by(hop_size)
    .map(|begin| transform_fourier_complex(&padded_samples[begin..begin+fft_size]))
    .collect::<Vec<Vec<Complex32>>>()
}


pub fn waveform_to_complex_spectrogram(wf: &babycat::Waveform, fft_size: usize, hop_size: usize, hop_count: isize) -> Vec<Vec<Complex32>> {
    create_complex_spectrogram(wf.to_interleaved_samples(), fft_size, hop_size, hop_count)
}

pub fn complex_spectrogram_to_amplitude(spectrogram: &[Vec<Complex32>]) -> Vec<Vec<f32>> {
    spectrogram.into_iter().map(
        |spectrum| spectrum.into_iter().map(
            |cx| (cx.re*cx.re + cx.im*cx.im).sqrt()
        ).collect()
    ).collect()
}

#[allow(dead_code)]
pub fn subtract_amplitude_spectrograms(one: &[Vec<f32>], other: &[Vec<f32>]) -> Vec<Vec<f32>> {
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

pub fn add_spectrograms(one: &[Vec<Complex32>], other: &[Vec<Complex32>]) -> ComplexSpectrogram {
    let bigger_width = max(one.len(), other.len());
    let height = one[0].len();
    assert!(other.len() == 0 || height == other[0].len()); // though dfferent vectors could still be different sizes
    let mut ret = vec![vec![0.0.into(); height]; bigger_width];
    for x in 0..bigger_width {
        for y in 0..height {
            let one_val = match one.get(x) { Some(v) => v[y], None => 0.0.into() };
            let other_val = match other.get(x) { Some(v) => v[y], None => 0.0.into() };
            ret[x][y] = one_val + other_val;
        }
    }
    ret
}

pub fn sub_spectrograms(one: &[Vec<Complex32>], other: &[Vec<Complex32>]) -> ComplexSpectrogram {
    let bigger_width = max(one.len(), other.len());
    let height = one[0].len();
    assert!(other.len() == 0 || height == other[0].len()); // though dfferent vectors could still be different sizes
    let mut ret = vec![vec![0.0.into(); height]; bigger_width];
    for x in 0..bigger_width {
        for y in 0..height {
            let one_val = match one.get(x) { Some(v) => v[y], None => 0.0.into() };
            let other_val = match other.get(x) { Some(v) => v[y], None => 0.0.into() };
            ret[x][y] = one_val - other_val;
        }
    }
    ret
}


pub fn calculate_distance(song_part: &[Vec<f32>], sample: &[Vec<f32>], dist: &dyn Fn(f32, f32) -> f32, sample_volume: f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though different vectors could still be different sizes
    let mut distance = 0.0_f32;
    for x in 0..bigger_width {
        for y in 0..song_part[0].len() {
            let song_part_val = match song_part.get(x) { Some(v) => v[y], None => 0.0 };
            let sample_val = match sample.get(x) { Some(v) => v[y], None => 0.0 };
            distance += dist(song_part_val, sample_val * sample_volume);
        }
    }
    distance
}

pub fn calculate_distance_complex(song_part: &[Vec<Complex32>], sample: &[Vec<Complex32>], dist: &dyn Fn(Complex32, Complex32) -> f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though different vectors could still be different sizes
    let mut distance = 0.0_f32;
    for x in 0..bigger_width {
        for y in 0..song_part[0].len() {
            let song_part_val = match song_part.get(x) { Some(v) => v[y], None => 0.0.into() };
            let sample_val = match sample.get(x) { Some(v) => v[y], None => 0.0.into() };
            distance += dist(song_part_val, sample_val);
        }
    }
    distance
}

