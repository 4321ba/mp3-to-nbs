
// index is the ID of the instrument
const INSTRUMENT_FILENAMES: &[&str] = &[
    "Sounds/dbass.ogg",
    "Sounds/harp.ogg",
];
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

pub const INSTRUMENT_COUNT: usize = INSTRUMENT_FILENAMES.len();
pub const PITCH_COUNT: usize = 25;

pub type SpectrogramSlice = [Vec<f32>];
pub type Spectrogram = Vec<Vec<f32>>;

use babycat::Waveform;
use babycat::Signal;
pub struct CachedInstruments {
    pub waveforms: [Vec<Waveform>; INSTRUMENT_COUNT], // Vec will be PITCH_COUNT long
    pub spectrograms: [Vec<Spectrogram>; INSTRUMENT_COUNT],
}

pub struct Note {
    pub instrument_id: usize, // 0..INSTRUMENT_COUNT
    pub pitch: usize, // 0..PITCH_COUNT
    pub volume: f32, // 0.0..1.0
}

#[path = "wave.rs"]
mod wave;
pub fn cache_instruments() -> CachedInstruments {
    let fft_size = 4096;
    let hop_size = 1024;

    const WAVEFORM_VEC: Vec<Waveform> = Vec::new();
    const SPECTROGRAM_VEC: Vec<Spectrogram> = Vec::new();
    let mut cached_instruments: CachedInstruments = CachedInstruments {
        waveforms: [WAVEFORM_VEC; INSTRUMENT_COUNT],
        spectrograms: [SPECTROGRAM_VEC; INSTRUMENT_COUNT],
    };
    
    for instr_idx in 0..INSTRUMENT_COUNT {
        let instr_filename = INSTRUMENT_FILENAMES[instr_idx];
        print!("Loading {}\n", instr_filename);
        let sample_wf = wave::import_sound_file(instr_filename);
        for pitch in 0..PITCH_COUNT {
            let multiplier = 2.0_f64.powf((pitch as i32 - 12) as f64 / 12.0);
            cached_instruments.waveforms[instr_idx].push(wave::change_pitch(&sample_wf, multiplier as f32));
        }
    }
    for instr_idx in 0..INSTRUMENT_COUNT {
        print!("Calculating spectrums for {}\n", INSTRUMENT_FILENAMES[instr_idx]);
        for pitch in 0..PITCH_COUNT {
            let sample_wf_diff_pitch = &cached_instruments.waveforms[instr_idx][pitch];
            let sample_spectrogram = wave::create_spectrum(
                sample_wf_diff_pitch.to_interleaved_samples(), sample_wf_diff_pitch.frame_rate_hz(), fft_size, hop_size);
            let sample_2dvec = wave::spectrum_to_2d_vec(&sample_spectrogram);
            cached_instruments.spectrograms[instr_idx].push(sample_2dvec);
        }
    }
    cached_instruments
}



fn add_notes_together(notes: &[Note], cache: &CachedInstruments) {
    
}