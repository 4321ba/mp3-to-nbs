use std::usize;

use crate::note::AmplitudeSpectrogram;

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

pub fn get_interesting_hopcounts(spectrogram: &AmplitudeSpectrogram) -> Vec<usize> {
    let sumvec = spectrogram.iter().map(|v| v.iter().sum()).collect::<Vec<f32>>();
    //println!("Amplitude sums: {:?}", sumvec);
    let mut ret = Vec::new();

    if sumvec[0] > 0.1 { ret.push(0) } // TODO magic numbers everywhere xddd
    for i in 0..(sumvec.len()-1) {
        if (i < 1 || sumvec[i-1] * 1.2/* TODO magic number */ < sumvec[i])
            && (i >= sumvec.len()-1 || i < 1 || sumvec[i] - sumvec[i-1] > sumvec[i+1] - sumvec[i])
            && (ret.len() < 1 || (ret[ret.len()-1] as i32) < i as i32-2) {
            ret.push(i);
        }
    }

    //println!("Interesting hopcounts: {:?}", ret);
    println!("Interesting hopcounts:");
    for i in &ret {
        //print!("{}: {};   ", i, sumvec[*i]);
        println!("HC: {}\t sec: {},\t tick:{}", i, *i as f64*1024.0/44100.0, (*i as f64*1024.0/44100.0*10.)as usize);
    }
    println!();
    ret
}

use aubio::{Tempo, Onset, Smpl, Notes};
use babycat::{Signal, Waveform};

pub fn guess_tps_aubio(wf: &Waveform) -> f64 {
    const BUF_SIZE: usize = 512;
    const HOP_SIZE: usize = 256;
    let mut tempo = Tempo::new(aubio::OnsetMode::Energy, BUF_SIZE, HOP_SIZE, wf.frame_rate_hz()).unwrap();
    //tempo.set_tatum_signature(4);
    //tempo.set_threshold(0.1);

    let period = 1.0 / wf.frame_rate_hz() as Smpl;

    let mut time = 0.0;
    let mut offset = 0;

    loop {
        let block = wf.to_interleaved_samples()
            .into_iter()
            .skip(offset)
            .take(HOP_SIZE)
            .map(|s| *s)
            .collect::<Vec<Smpl>>();

        if block.len() == HOP_SIZE {
            let res = tempo.do_result(block.as_slice().as_ref()).unwrap();
            if res > 0.0 {
                //println!("T: {},\ttick: {},\tres: {}", time, (10.0*time) as usize, res);
            }
        }

        offset += block.len();
        time = offset as Smpl * period;
        //print!("k");

        if block.len() < HOP_SIZE {
            break;
        }
    }

    println!("T: {} bpm: {}", time, tempo.get_bpm());

    tempo.get_bpm() as f64 * 4.0 / 60.0
}


pub fn get_onsets_aubio(wf: &Waveform) -> Vec<usize> {
    const BUF_SIZE: usize = 512;
    const HOP_SIZE: usize = 256;
    let mut onset = Onset::new(aubio::OnsetMode::Energy, BUF_SIZE, HOP_SIZE, wf.frame_rate_hz()).unwrap();
    onset.set_threshold(1.0);

    let period = 1.0 / wf.frame_rate_hz() as Smpl;

    let mut time = 0.0;
    let mut offset = 0;

    let mut ret: Vec<usize> = Vec::new();

    loop {
        let block = wf.to_interleaved_samples()
            .into_iter()
            .skip(offset)
            .take(HOP_SIZE)
            .map(|s| *s)
            .collect::<Vec<Smpl>>();

        if block.len() == HOP_SIZE {
            let res = onset.do_result(block.as_slice().as_ref()).unwrap();
            if res > 0.0 {
                println!("T: {},\ttick: {},\tres: {},\tHC: {}", time, (10.0*time-0.5) as usize, res, 
                    (time*wf.frame_rate_hz()as f32 / 1024.0 + 0.5)as usize);
                ret.push(onset.get_last());
            }
        }

        offset += block.len();
        time = offset as Smpl * period;
        //print!("k");

        if block.len() < HOP_SIZE {
            break;
        }
    }
    ret
}

pub fn convert_onsets_to_hopcounts_uneven_with_filler(onsets: &[usize], tps: f64, hop_size: usize, frame_rate_hz: u32) -> Vec<usize> {
    let mut extended_onsets: Vec<usize> = onsets.windows(2)
        .flat_map(|w| {
            let tick_count = ((w[1] - w[0]) as f64 / frame_rate_hz as f64 * tps + 0.5) as usize;
            let sample_diff = (w[1] - w[0]) / tick_count;
            (0..tick_count).map(move |c| w[0] + c * sample_diff)
        })
        .collect();
    extended_onsets.push(*onsets.last().unwrap());
    extended_onsets.into_iter().map(|o| (o + hop_size / 2) / hop_size).collect()
}

pub fn even_out_onsets(onsets: &[usize], tps: f64, hop_size: usize, frame_rate_hz: u32) -> Vec<usize> {
    let mut extended_onsets: Vec<usize> = onsets.windows(2)
        .flat_map(|w| {
            let tick_count = ((w[1] - w[0]) as f64 / frame_rate_hz as f64 * tps + 0.5) as usize;
            (0..tick_count).map(move |c| w[0] + c * ((w[1] - w[0]) / tick_count))
        })
        .collect();
    extended_onsets.push(*onsets.last().unwrap());
    
    let tick_count = extended_onsets.len();
    let sample_diff = frame_rate_hz as f64 / tps;//(onsets.last().unwrap() - onsets[0]) as f64 / (tick_count - 1) as f64;
    let first_sample = extended_onsets.iter().enumerate()
        .map(|(i, o)| *o as f64 - i as f64 * sample_diff).sum::<f64>() / tick_count as f64;
    let last_sample = extended_onsets.iter().enumerate()
        .map(|(i, o)| *o as f64 + (tick_count - i - 1) as f64 * sample_diff).sum::<f64>() / tick_count as f64;
    dbg!(extended_onsets.first().unwrap());
    dbg!(extended_onsets.last().unwrap());
    dbg!(first_sample);
    dbg!(last_sample);
    let mod_first_sample = 0.0;// TODO //if first_sample >= 0.0 { first_sample } else { 0.0 };
    //let mod_sample_diff = (last_sample - first_sample) / (tick_count - 1) as f64;
    (0..tick_count).map(|i| (mod_first_sample + i as f64 * sample_diff + 0.5) as usize).collect()
}
pub fn onsets_to_hopcounts(onsets: &[usize], hop_size: usize) -> Vec<usize> {
    onsets.into_iter().map(|o| (o + hop_size / 2) / hop_size).collect()
}

pub fn guess_exact_tps(onsets: &Vec<usize>, hop_size: usize, frame_rate_hz: u32, approx_tps: f64) -> f64 {
    // https://stackoverflow.com/questions/75178232/how-to-get-the-adjacent-difference-of-a-vec
    let tick_sum: usize = onsets
        .windows(2)
        .map(|s| ((s[1] - s[0]) as f64 / frame_rate_hz as f64 * approx_tps + 0.5) as usize)
        .sum();
    let tps = tick_sum as f64 / ((onsets.last().unwrap() - onsets[0]) as f64 / frame_rate_hz as f64);
    dbg!(tps);
    ((tps * 4.0 + 0.5) as u32) as f64 / 4.0 // rounding to 0.25
}