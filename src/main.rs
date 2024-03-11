mod cli;
mod note;
mod wave;

fn max_of_slice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No NaN should be here")).expect("Slice shouldn't be empty")
}

//use std::hint;

use std::char::MAX;

use babycat::{Signal, Waveform, WaveformArgs};
use note::CachedInstruments;


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

use std::cmp::max;
fn calculate_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, dist: &dyn Fn(f32, f32) -> f32, sample_volume: f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though dfferent vectors could still be different sizes
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

fn calculate_assymetric_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, sample_volume: f32) -> f32 {
    calculate_distance(song_part, sample, &|sp, sa| if sp >= sa {0.0} else {(sp-sa) * (sp-sa)}, sample_volume)
}

fn test_distances_for_instruments(waveform: &Waveform, cache: &CachedInstruments) {
    let mut test_found_notes: Vec<note::Note> = Vec::new();

    let fft_size = 4096;
    let samples = waveform.to_interleaved_samples();
    let spectrogram = wave::create_spectrum(samples, waveform.frame_rate_hz(), fft_size, 1024);
    let spectrogram_2dvec = wave::spectrum_to_2d_vec(&spectrogram);
    let song_part = &spectrogram_2dvec[0..40];
    debug_save_as_image(song_part, "song_part.png");
    for instr_idx in 0..note::INSTRUMENT_COUNT {
        print!("\ninstr idx: {}\n", instr_idx);
        for pitch in 0..note::PITCH_COUNT {
            let sample_2dvec = &cache.spectrograms[instr_idx][pitch];
            debug_save_as_image(&wave::subtract_2d_vecs(song_part, &sample_2dvec), &format!("{instr_idx}_pitch{pitch:02}.png"));

            let TEMP_volume = 0.5; // TODO
            let diff = calculate_assymetric_distance(song_part, &sample_2dvec, TEMP_volume); // TODO
            
            let silence = [vec![0.0; sample_2dvec[0].len()]; 1];
            let compensation = calculate_assymetric_distance(&silence, &sample_2dvec, TEMP_volume); // TODO
            let compensated_val = diff / compensation;
            println!("{pitch:02}: {diff:.5}, comp:{compensation:.5}, compensated: {compensated_val:.5}");

            if compensated_val < 0.015 {
                test_found_notes.push(Note {instrument_id: instr_idx, pitch, volume: TEMP_volume});
                println!("Added this note!");
            }
        }
    }


    let found_wf = note::add_notes_together(&test_found_notes, cache, 1.0);
    debug_save_as_wav(&found_wf, "test_found_notes.wav");
}

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
    //test_main(&waveform);
    test_distances_for_instruments(&waveform, &cache);

}
