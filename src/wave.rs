use babycat::{Signal, Waveform, WaveformArgs};
use tracing::debug;


pub fn import_sound_file(filename: &str) -> Waveform {
    let waveform_args = WaveformArgs {
        convert_to_mono: true, // We convert everything to mono for now
        ..Default::default()
    };
    let waveform = Waveform::from_file(filename, waveform_args).expect("Decoding error");
    debug!(
        "Decoded {} frames with {} channels at {} hz",
        waveform.num_frames(),
        waveform.num_channels(),
        waveform.frame_rate_hz(),
    );
    waveform
}


// multiplier: between 0.5 and 2.0 usually, those mean 1 octave higher and one octave lower
pub fn change_pitch(wf: &Waveform, multiplier: f32) -> Waveform {
    let original_hz = wf.frame_rate_hz();
    //println!("Converted {:?}", wf);
    let new_wf = wf.resample((original_hz as f32 / multiplier) as u32).unwrap();
    //println!("Through {:?}", new_wf);
    let even_newer_wf = Waveform::from_interleaved_samples(original_hz, new_wf.num_channels(), new_wf.to_interleaved_samples());
    //println!("To {:?}", even_newer_wf);
    even_newer_wf
}

pub fn add_waveforms_delayed(orig: &Waveform, delayed: &Waveform, delay: usize) -> Waveform {
    assert_eq!(orig.frame_rate_hz(), delayed.frame_rate_hz(), "Sampling rate should be the same!");
    assert_eq!(orig.num_channels(), delayed.num_channels(), "Channel count should be the same!");
    let bigger_width = std::cmp::max(
        orig.to_interleaved_samples().len(), 
        delayed.to_interleaved_samples().len() + delay
    );
    let mut samples = vec![0.0; bigger_width];
    for i in 0..bigger_width {
        samples[i] += orig.to_interleaved_samples().get(i).unwrap_or(&0.0);
        samples[i] += delayed.to_interleaved_samples().get(i.overflowing_sub(delay).0).unwrap_or(&0.0);
    }
    Waveform::from_interleaved_samples(orig.frame_rate_hz(), orig.num_channels(), &samples)
}
