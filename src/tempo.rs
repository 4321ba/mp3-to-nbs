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
                println!("T: {},\ttick: {},\tres: {}", time, (10.0*time) as usize, res);
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

/*
    const BUF_SIZE: usize = 512;
    const HOP_SIZE: usize = 256;


    let mut samples = wf.to_interleaved_samples();
    let mut notes = Notes::new(BUF_SIZE, HOP_SIZE, wf.frame_rate_hz()).unwrap();

    let period = 1.0 / wf.frame_rate_hz() as Smpl;

    let mut time = 0.0;
    let mut offset = 0;

    loop {
        let block = samples
            .into_iter()
            .skip(offset)
            .take(HOP_SIZE)
            .map(|s| *s)
            .collect::<Vec<Smpl>>();

        if block.len() == HOP_SIZE {
            for note in notes.do_result(block.as_slice().as_ref()).unwrap() {
                if note.velocity > 0.0 {
                    print!("{}\t{}\t", note.pitch, time);
                } else {
                    println!("{}", time);
                }
            }
        }

        offset += block.len();
        time = offset as Smpl * period;
        print!("k");

        if block.len() < HOP_SIZE {
            break;
        }
    }

    println!("{}", time);
*/
    tempo.get_bpm() as f64 * 4.0 / 60.0
}

pub fn guess_exact_tps(hopcounts: &Vec<usize>, hop_size: usize, frame_rate_hz: u32, approx_tps: f64) -> f64 {
    // https://stackoverflow.com/questions/75178232/how-to-get-the-adjacent-difference-of-a-vec
    let tick_sum: usize = hopcounts
        .windows(2)
        .map(|s| (((s[1] - s[0]) * hop_size) as f64 / frame_rate_hz as f64 * approx_tps + 0.5) as usize)
        .sum();
    //(median, frame_rate_hz as f64 / (hop_size * median) as f64)
    let tps = tick_sum as f64 / (((hopcounts.last().unwrap() - hopcounts[0]) * hop_size) as f64 / frame_rate_hz as f64);
    dbg!(tps);
    ((tps * 4.0 + 0.5) as u32) as f64 / 4.0 // rounding to 0.25
}