use nbs::{
    header::Header,
    noteblocks::{instrument, instrument::CustomInstruments, layer::Layer, note::Note, NoteBlocks},
    Nbs, NbsFormat,
};
use std::fs::File;
use phf::phf_map;
use crate::note;

static INSTRUMENTS: phf::Map<&str, nbs::noteblocks::instrument::Instrument> = phf_map! {
    "banjo.ogg" => instrument::BANJO,
    "bdrum.ogg" => instrument::BASS_DRUM,
    "bell.ogg" => instrument::BELL,
    "bit.ogg" => instrument::BIT,
    "click.ogg" => instrument::CLICK,
    "cow_bell.ogg" => instrument::COW_BELL,
    "dbass.ogg" => instrument::DOUBLE_BASS,
    "didgeridoo.ogg" => instrument::DIDGERIDOO,
    "flute.ogg" => instrument::FLUTE,
    "guitar.ogg" => instrument::GUITAR,
    "harp.ogg" => instrument::PIANO,
    "icechime.ogg" => instrument::CHIME,
    "iron_xylophone.ogg" => instrument::IRON_XYLOPHONE,
    "pling.ogg" => instrument::PLING,
    "sdrum.ogg" => instrument::SNARE_DRUM,
    "xylobone.ogg" => instrument::XYLOPHONE,
};


pub fn clean_quiet_notes(notes: &Vec<Vec<crate::note::Note>>) -> Vec<Vec<crate::note::Note>> {
    let mut ret = Vec::new();
    for tick in notes {
        ret.push(Vec::new());
        let last = ret.len() - 1;
        for note in tick {
            let mut vol = note.volume.abs();
            while vol >= 0.15 { // lower threshold for volume for including a note block
                ret[last].push(crate::note::Note {
                    instrument_id: note.instrument_id,
                    pitch: note.pitch,
                    volume: if vol <= 1.0 { vol } else { 1.0 }
                });
                vol -= 1.0;
            }
        }
    }
    ret
}

pub fn export_notes(notes: &Vec<Vec<crate::note::Note>>, timestamps: &Vec<usize>, tps: f64, output_file: &str) {
    assert_eq!(notes.len(), timestamps.len(), "Amount of note vecs should be the amount of timestamps!");
    let layer_counts_by_instrid: Vec<usize> = (0..note::INSTRUMENT_COUNT).map(|instr_id|
        notes.iter().map(|tick|
            tick.iter().filter(|note| note.instrument_id == instr_id).count()
        ).max().unwrap()
    ).collect();
    let max_layer_count = layer_counts_by_instrid.iter().sum();
    //let max_layer_count = notes.iter().max_by_key(|v| v.len()).unwrap().len();
    let mut file = File::create(output_file).unwrap();
    let mut header = Header::new(NbsFormat::OpenNoteBlockStudio(4)); // Create a header.
    header.song_name = String::from("Recognized song"); // Change the name.
    header.song_tempo = (tps * 100.0 + 0.5) as i16;
    let mut noteblocks = NoteBlocks::new();
    for _ in 0..max_layer_count {
        // Create a new Layer.
        noteblocks
            .layers
            .push(Layer::from_format(NbsFormat::OpenNoteBlockStudio(4)));
    }
    // Insert notes into the layers
    for i in 0..notes.len() {
        let mut current_instrid = 0;
        let mut instrid_count = 0;
        let mut current_instrid_beginning: usize = 0;
        for j in 0..notes[i].len() {
            let note = &notes[i][j];
            let vol = (std::cmp::min(std::cmp::max((note.volume.abs()*10.0 + 0.5) as i32 * 10, 0), 100)) as i8;
            if current_instrid != note.instrument_id {
                current_instrid = note.instrument_id;
                current_instrid_beginning = (0..note.instrument_id).map(|i| layer_counts_by_instrid[i]).sum();
                instrid_count = 0;
            }
            noteblocks.layers[current_instrid_beginning + instrid_count].notes.insert(
                timestamps[i] as i16,
                Note::new(
                    INSTRUMENTS[note::INSTRUMENT_FILENAMES[note.instrument_id]],
                    (33 + note.pitch) as i8,
                    Some(vol),
                    Some(100),
                    Some(0),
                ),
            );
            instrid_count += 1;
        }
    }
    let custom_instruments = CustomInstruments::new(); // Create a empty list of custom instruments.
    let mut nbs = Nbs::from_componets(header, noteblocks, custom_instruments); // Assamble everything together.
    nbs.update(); // Update certian fields in the header to match the rest of the file.
    nbs.encode(&mut file).unwrap(); // save!
}