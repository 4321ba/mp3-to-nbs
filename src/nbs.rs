use nbs::{
    header::Header,
    noteblocks::{instrument, instrument::CustomInstruments, layer::Layer, note::Note, NoteBlocks},
    Nbs, NbsFormat,
};
use std::fs::File;

use crate::note;
/*
const INSTRUMENT_: &[&str] = &[
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

pub fn export_notes(notes: Vec<Vec<crate::note::Note>>, timestamps: Vec<usize>) {
    assert_eq!(notes.len(), timestamps.len(), "Amount of note vecs should be the amount of timestamps!");
    let max_layer_count = notes.iter().max_by_key(|v| v.len()).unwrap().len();
    let mut file = File::create("out.nbs").unwrap();
    let mut header = Header::new(NbsFormat::OpenNoteBlockStudio(4)); // Create a header.
    header.song_name = String::from("test"); // Change the name to `test`.
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
                timestamps[i] as i16 / 4,//TODO temp /4
                Note::new(
                    if note.instrument_id==0 {instrument::DOUBLE_BASS} else {instrument::PIANO},//TODO in a hashmap
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