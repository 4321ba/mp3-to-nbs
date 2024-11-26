#!/bin/bash
: '
Use this main function!

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .init();
    let args = Args::parse();
    debug!("Command line args: {:?}", args);

    let waveform = wave::import_sound_file(&args.input_file);

    let tps_approx = tempo::guess_tps_aubio(&waveform);
    let onsets = get_onsets_aubio(&waveform);
    let tps_guessed = tempo::guess_exact_tps(&onsets, waveform.frame_rate_hz(), tps_approx);
    let tps = if args.tps > 0.0 { args.tps } else { tps_guessed };
    print!("{},{},", &args.input_file, tps_guessed);
    return;
    ...
}
'

for f in wave/*
do
name=$(basename $f .ogg)
../target/debug/mp3-to-nbs --input-file "$f" --output-file "qwe.nbs"
./get_tps.py "nbs/$name.nbs"
done
