
// index is the ID of the instrument
pub const INSTRUMENT_FILENAMES: &[&str] = &[
    "dbass.ogg",
    "harp.ogg",
    //"pling.ogg",
    "sdrum.ogg",
    //"bdrum.ogg",
    "click.ogg",
];
/*
pub const INSTRUMENT_FILENAMES: &[&str] = &[
    "banjo.ogg",
    "bdrum.ogg",
    "bell.ogg",
    "bit.ogg",
    "click.ogg",
    "cow_bell.ogg",
    "dbass.ogg",
    "didgeridoo.ogg",
    "flute.ogg",
    "guitar.ogg",
    "harp.ogg",
    "icechime.ogg",
    "iron_xylophone.ogg",
    "pling.ogg",
    "sdrum.ogg",
    "xylobone.ogg",
];*/

pub const INSTRUMENT_COUNT: usize = INSTRUMENT_FILENAMES.len();
pub const PITCH_COUNT: usize = 25;

pub type SpectrogramSlice = [Vec<f32>];
pub type AmplitudeSpectrogram = Vec<Vec<f32>>;
pub type ComplexSpectrogram = Vec<Vec<Complex32>>;

use std::path::Path;

use babycat::Waveform;
use babycat::Signal;
use microfft::Complex32;
pub struct CachedInstruments {
    pub waveforms: [Vec<Waveform>; INSTRUMENT_COUNT], // Vec will be PITCH_COUNT long; Waveform hz should be the same
    pub complex_spectrograms: [Vec<ComplexSpectrogram>; INSTRUMENT_COUNT],
    pub amplitude_spectrograms: [Vec<AmplitudeSpectrogram>; INSTRUMENT_COUNT],
}

#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub instrument_id: usize, // 0..INSTRUMENT_COUNT
    pub pitch: usize, // 0..PITCH_COUNT
    pub volume: f32, // 0.0..1.0 usually, but can be bigger
}

//pub type NoteStateSpace = [[f32; PITCH_COUNT]; INSTRUMENT_COUNT];
pub type NoteStateSpace = Vec<f32>; // it should be PITCH_COUNT*INSTRUMENT_COUNT big
pub fn get_volume_from_state_space(nss: &NoteStateSpace, instrument_id: usize, pitch: usize) -> f32 {
    assert!(instrument_id < INSTRUMENT_COUNT, "Overindexing!");
    assert!(pitch < PITCH_COUNT, "Overindexing!");
    nss[instrument_id * INSTRUMENT_COUNT + pitch]
}

use crate::wave;
use crate::wave::complex_spectrogram_to_amplitude;
use crate::wave::waveform_to_complex_spectrogram;
pub fn cache_instruments(sounds_folder: &str) -> CachedInstruments {
    let fft_size = 4096;
    let hop_size = 1024;

    const WAVEFORM_VEC: Vec<Waveform> = Vec::new();
    const A_SPECTROGRAM_VEC: Vec<AmplitudeSpectrogram> = Vec::new();
    const C_SPECTROGRAM_VEC: Vec<ComplexSpectrogram> = Vec::new();
    let mut cached_instruments: CachedInstruments = CachedInstruments {
        waveforms: [WAVEFORM_VEC; INSTRUMENT_COUNT],
        amplitude_spectrograms: [A_SPECTROGRAM_VEC; INSTRUMENT_COUNT],
        complex_spectrograms: [C_SPECTROGRAM_VEC; INSTRUMENT_COUNT],
    };
    
    for instr_idx in 0..INSTRUMENT_COUNT {
        let instr_filename = INSTRUMENT_FILENAMES[instr_idx];
        print!("Loading {}\n", instr_filename);
        let sample_wf = wave::import_sound_file(&Path::new(sounds_folder).join(instr_filename).to_str().unwrap());
        for pitch in 0..PITCH_COUNT {
            let multiplier = 2.0f64.powf((pitch as i32 - 12) as f64 / 12.0);
            cached_instruments.waveforms[instr_idx].push(wave::change_pitch(&sample_wf, multiplier as f32));
        }
    }
    for instr_idx in 0..INSTRUMENT_COUNT {
        print!("Calculating spectrums for {}\n", INSTRUMENT_FILENAMES[instr_idx]);
        for pitch in 0..PITCH_COUNT {
            let sample_wf_diff_pitch = &cached_instruments.waveforms[instr_idx][pitch];
            let complex_spectrogram = waveform_to_complex_spectrogram(sample_wf_diff_pitch, fft_size, hop_size, -1);
            let amplitude_spectrogram = complex_spectrogram_to_amplitude(&complex_spectrogram);
            cached_instruments.complex_spectrograms[instr_idx].push(complex_spectrogram);
            cached_instruments.amplitude_spectrograms[instr_idx].push(amplitude_spectrogram);
        }
    }
    cached_instruments
}


// TODO add previous tick's ending?? maybe
pub fn add_notes_together(notes: &[Note], cache: &CachedInstruments, multiplier: f32) -> Waveform {
    if notes.len() == 0 { // TODO is this really the best way?
        return Waveform::new(cache.waveforms[0][0].frame_rate_hz(), cache.waveforms[0][0].num_channels(), vec![0.0; 1]);
    }

    let max_len_note = notes.iter().max_by_key(
        |note| cache.waveforms[note.instrument_id][note.pitch].to_interleaved_samples().len()
    ).unwrap();
    let max_len = cache.waveforms[max_len_note.instrument_id][max_len_note.pitch].to_interleaved_samples().len();
    let mut samples = vec![0.0; max_len];
    for note in notes {
        let samples_to_add = cache.waveforms[note.instrument_id][note.pitch].to_interleaved_samples();
        for i in 0..samples_to_add.len() {
            samples[i] += samples_to_add[i] * note.volume * multiplier;
            /*if samples[i] > 1.0 {
                samples[i] = 1.0;
            }*/
        }
    }
    assert_eq!(cache.waveforms[0][0].num_channels(), 1, "We are expecting everything to be mono for now.");
    Waveform::new(cache.waveforms[0][0].frame_rate_hz(), cache.waveforms[0][0].num_channels(), samples)
}

pub fn add_notes_together_statespace(notes: &NoteStateSpace, cache: &CachedInstruments, multiplier: f32) -> Waveform {
    let mut notes_vec: Vec<Note> = Vec::new();
    for instrid in 0..INSTRUMENT_COUNT {
        for pitch in 0..PITCH_COUNT {
            notes_vec.push(Note { instrument_id: instrid, pitch, volume: get_volume_from_state_space(notes, instrid, pitch) })
        }
    }
    add_notes_together(&notes_vec, cache, multiplier)
}
pub fn add_notes_together_merge_from_stsp(notes: &[Note], volumes: &[f32], cache: &CachedInstruments, multiplier: f32) -> Waveform {
    let mut notes_vec: Vec<Note> = Vec::new();
    for idx in 0..notes.len() {
        notes_vec.push(Note { instrument_id: notes[idx].instrument_id, pitch: notes[idx].pitch, volume: volumes[idx] })
    }
    add_notes_together(&notes_vec, cache, multiplier)
}


pub fn add_note_spectrograms(notes: &[Note], volume_override: &[f32], cache: &CachedInstruments, multiplier: f32) -> ComplexSpectrogram {
    if notes.len() == 0 { 
        return vec![vec![0.0.into(); cache.complex_spectrograms[0][0][0].len()]; 1];
    }

    let notes_vec: Vec<Note> = if volume_override.len() > 0 {
        (0..notes.len()).map(|idx| Note { instrument_id: notes[idx].instrument_id, pitch: notes[idx].pitch, volume: volume_override[idx] }).collect()
    } else {
        notes.to_vec()
    };

    let max_len_note = notes_vec.iter().max_by_key(
        |note| cache.complex_spectrograms[note.instrument_id][note.pitch].len()
    ).unwrap();
    let max_width = cache.complex_spectrograms[max_len_note.instrument_id][max_len_note.pitch].len();
    let height = cache.complex_spectrograms[0][0][0].len();
    let mut ret = vec![vec![0.0.into(); height]; max_width];
    for x in 0..max_width {
        for y in 0..height {
            for note in &notes_vec {
                ret[x][y] += match cache.complex_spectrograms[note.instrument_id][note.pitch].get(x)
                 { Some(v) => v[y] * note.volume * multiplier, None => 0.0.into() };
            }
        }
    }
    ret
}



pub fn add_spectrograms(one: &[Vec<Complex32>], other: &[Vec<Complex32>]) -> ComplexSpectrogram {
    let bigger_width = std::cmp::max(one.len(), other.len());
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
    let bigger_width = std::cmp::max(one.len(), other.len());
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
