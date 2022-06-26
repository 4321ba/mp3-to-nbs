use babycat::{Signal, Waveform, WaveformArgs};

fn maxslice(slice: &[f32]) -> &f32 {
    slice
        .iter()
        .max_by(|a,  b| a.partial_cmp(b).expect("No nan!!")).expect("slice shouldnt be empty")
}

fn main() {
    let waveform_args = WaveformArgs {
        convert_to_mono: true,
        ..Default::default()
    };
    let waveform =
        match Waveform::from_file("musictests/olddiscjing.mp3", waveform_args) {
            Ok(w) => w,
            Err(err) => {
                println!("Decoding error: {}", err);
                return;
            }
        };
    println!(
        "Decoded {} frames with {} channels at {} hz",
        waveform.num_frames(),
        waveform.num_channels(),
        waveform.frame_rate_hz(),
    );
    //let samples: &[f32] = waveform.to_interleaved_samples();
    println!("{:?}", &waveform.to_interleaved_samples()[105750..105800]);
    println!("{}", maxslice(waveform.to_interleaved_samples()));
}
