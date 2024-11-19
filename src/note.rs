use std::path::Path;
use babycat::Waveform;
use babycat::Signal;
use microfft::Complex32;
use tracing::debug;
use crate::fourier;
use crate::wave;

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

pub type AmplitudeSpectrogram = Vec<Vec<f32>>;
pub type ComplexSpectrogram = Vec<Vec<Complex32>>;

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

pub fn cache_instruments(sounds_folder: &str) -> CachedInstruments {
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
        debug!("Loading {}", instr_filename);
        let sample_wf = wave::import_sound_file(&Path::new(sounds_folder).join(instr_filename).to_str().unwrap());
        for pitch in 0..PITCH_COUNT {
            let multiplier = 2.0f64.powf((pitch as i32 - 12) as f64 / 12.0);
            cached_instruments.waveforms[instr_idx].push(wave::change_pitch(&sample_wf, multiplier as f32));
        }
    }
    for instr_idx in 0..INSTRUMENT_COUNT {
        debug!("Calculating spectrums for {}", INSTRUMENT_FILENAMES[instr_idx]);
        for pitch in 0..PITCH_COUNT {
            let sample_wf_diff_pitch = &cached_instruments.waveforms[instr_idx][pitch];
            let complex_spectrogram = fourier::waveform_to_complex_spectrogram(sample_wf_diff_pitch, fourier::FFT_SIZE, fourier::HOP_SIZE, -1);
            let amplitude_spectrogram = fourier::complex_spectrogram_to_amplitude(&complex_spectrogram);
            cached_instruments.complex_spectrograms[instr_idx].push(complex_spectrogram);
            cached_instruments.amplitude_spectrograms[instr_idx].push(amplitude_spectrogram);
        }
    }
    cached_instruments
}


pub fn add_notes_together(notes: &[Note], cache: &CachedInstruments, multiplier: f32) -> Waveform {
    if notes.len() == 0 {
        return Waveform::new(cache.waveforms[0][0].frame_rate_hz(), cache.waveforms[0][0].num_channels(), vec![0.0; 10]);
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
