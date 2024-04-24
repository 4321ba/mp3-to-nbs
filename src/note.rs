
// index is the ID of the instrument
pub const INSTRUMENT_FILENAMES: &[&str] = &[
    "Sounds/dbass.ogg",
    "Sounds/harp.ogg",
    //"Sounds/pling.ogg",
    //"Sounds/sdrum.ogg",
//    "Sounds/bdrum.ogg",
    //"Sounds/click.ogg",
];
/*
pub const INSTRUMENT_FILENAMES: &[&str] = &[
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
    pub waveforms: [Vec<Waveform>; INSTRUMENT_COUNT], // Vec will be PITCH_COUNT long; Waveform hz should be the same
    pub spectrograms: [Vec<Spectrogram>; INSTRUMENT_COUNT],
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
            let multiplier = 2.0f64.powf((pitch as i32 - 12) as f64 / 12.0);
            cached_instruments.waveforms[instr_idx].push(wave::change_pitch(&sample_wf, multiplier as f32));
        }
    }
    for instr_idx in 0..INSTRUMENT_COUNT {
        print!("Calculating spectrums for {}\n", INSTRUMENT_FILENAMES[instr_idx]);
        for pitch in 0..PITCH_COUNT {
            let sample_wf_diff_pitch = &cached_instruments.waveforms[instr_idx][pitch];
            let sample_spectrogram = wave::create_spectrum(
                sample_wf_diff_pitch.to_interleaved_samples(), sample_wf_diff_pitch.frame_rate_hz(), fft_size, hop_size, -1);
            let sample_2dvec = wave::spectrum_to_2d_vec(&sample_spectrogram);
            cached_instruments.spectrograms[instr_idx].push(sample_2dvec);
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
            if samples[i] > 1.0 {
                samples[i] = 1.0;
            }
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