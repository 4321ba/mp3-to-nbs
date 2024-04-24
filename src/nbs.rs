use nbs::{
    header::Header,
    noteblocks::{instrument, instrument::CustomInstruments, layer::Layer, note::Note, NoteBlocks},
    Nbs, NbsFormat,
};
use std::fs::File;

use crate::note;

use phf::phf_map;
static COUNTRIES: phf::Map<&str, nbs::noteblocks::instrument::Instrument> = phf_map! {
    "UK" => instrument::PIANO,
    "Sounds/banjo.ogg" => instrument::BANJO,
    "Sounds/bdrum.ogg" => instrument::BASS_DRUM,
    "Sounds/bell.ogg" => instrument::BELL,
    "Sounds/bit.ogg" => instrument::BIT,
    "Sounds/click.ogg" => instrument::CLICK,
    "Sounds/cow_bell.ogg" => instrument::COW_BELL,
    "Sounds/dbass.ogg" => instrument::DOUBLE_BASS,
    "Sounds/didgeridoo.ogg" => instrument::DIDGERIDOO,
    "Sounds/flute.ogg" => instrument::FLUTE,
    "Sounds/guitar.ogg" => instrument::GUITAR,
    "Sounds/harp.ogg" => instrument::PIANO,
    "Sounds/icechime.ogg" => instrument::CHIME,
    "Sounds/iron_xylophone.ogg" => instrument::IRON_XYLOPHONE,
    "Sounds/pling.ogg" => instrument::PLING,
    "Sounds/sdrum.ogg" => instrument::SNARE_DRUM,
    "Sounds/xylobone.ogg" => instrument::XYLOPHONE,
};

pub fn guess_tps(hopcounts: &Vec<usize>, hop_size: usize, frame_rate_hz: u32) -> f64 {
    // https://stackoverflow.com/questions/75178232/how-to-get-the-adjacent-difference-of-a-vec
    let mut diff: Vec<usize> = hopcounts.windows(2).map(|s| s[1] - s[0]).collect();
    //dbg!(diff);
    // inefficient and not so correct median implementation
    diff.sort();
    let median = diff[diff.len() / 2];
    //(median, frame_rate_hz as f64 / (hop_size * median) as f64)
    let tps = frame_rate_hz as f64 / (hop_size * median) as f64;
    ((tps * 4.0 + 0.5) as u32) as f64 / 4.0 // rounding to 0.25
}

pub fn convert_hopcounts_to_ticks(hopcounts: &Vec<usize>, tps: f64, hop_size: usize, frame_rate_hz: u32) -> Vec<usize> {
    let diff: Vec<usize> = hopcounts.windows(2).map(|s| s[1] - s[0]).collect();
    //hopcounts.iter().map(|c| ((frame_rate_hz as f64 / (*c * hop_size) as f64) / tps + 0.5) as usize).collect()
    let mut tickdiff: Vec<usize> = diff.iter().map(|c| ((*c * hop_size) as f64 / frame_rate_hz as f64 * tps + 0.5) as usize).collect();
    // https://users.rust-lang.org/t/inplace-cumulative-sum-using-iterator/56532
    let mut acc = 0usize;
    for t in &mut tickdiff {
        acc += *t;
        *t = acc;
    }
    tickdiff.insert(0, 0);
    tickdiff
}

pub fn export_notes(notes: &Vec<Vec<crate::note::Note>>, timestamps: &Vec<usize>, tps: f64) {
    assert_eq!(notes.len(), timestamps.len(), "Amount of note vecs should be the amount of timestamps!");
    let max_layer_count = notes.iter().max_by_key(|v| v.len()).unwrap().len();
    let mut file = File::create("out.nbs").unwrap();
    let mut header = Header::new(NbsFormat::OpenNoteBlockStudio(4)); // Create a header.
    header.song_name = String::from("test"); // Change the name to `test`.
    header.song_tempo = (tps * 100.0 + 0.5) as i16;
    let mut noteblocks = NoteBlocks::new();
    for _ in 0..max_layer_count {
        // Create a new Layer.
        noteblocks
            .layers
            .push(Layer::from_format(NbsFormat::OpenNoteBlockStudio(4)));
    }
    // Insert notes into the layers
    for i in 0..timestamps.len() {
        for j in 0..notes[i].len() {
            let note = &notes[i][j];
            let vol = (std::cmp::min(std::cmp::max((note.volume.abs()*100.0) as i32, 0), 100)) as i8;
            if vol < 10 {
                continue;
            }
            noteblocks.layers[j].notes.insert(
                timestamps[i] as i16,
                Note::new(
                    COUNTRIES[note::INSTRUMENT_FILENAMES[note.instrument_id]],
                    //if note.instrument_id==0 {instrument::DOUBLE_BASS} else {instrument::PIANO},//-> in a hashmap
                    (33 + note.pitch) as i8,
                    Some(vol),
                    Some(100),
                    Some(0),
                ),
            );
        }
    }
    let custom_instruments = CustomInstruments::new(); // Create a empty list of custom instruments.
    let mut nbs = Nbs::from_componets(header, noteblocks, custom_instruments); // Assamble everything together.
    nbs.update(); // Update certian fields in the header to match the rest of the file.
    nbs.encode(&mut file).unwrap(); // save!
}