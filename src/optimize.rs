
use crate::note;
use crate::wave;
use crate::debug;
use crate::wave::complex_spectrogram_to_amplitude;
use crate::wave::waveform_to_complex_spectrogram;
use crate::wave::waveform_to_spectrogram;

use argmin::core::Gradient;
use argmin::core::State;
use argmin::solver::gradientdescent::SteepestDescent;
use argmin::solver::linesearch::MoreThuenteLineSearch;
use argmin_math::Rng;
use babycat::{Signal, Waveform};
use microfft::Complex32;





use std::cmp::max;
fn calculate_distance(song_part: &[Vec<f32>], sample: &[Vec<f32>], dist: &dyn Fn(f32, f32) -> f32, sample_volume: f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though dfferent vectors could still be different sizes
    let mut distance = 0.0_f32;
    for x in 0..bigger_width {
        for y in 0..song_part[0].len() {
            let song_part_val = match song_part.get(x) { Some(v) => v[y], None => 0.0 };
            let sample_val = match sample.get(x) { Some(v) => v[y], None => 0.0 };
            distance += dist(song_part_val, sample_val * sample_volume);
        }
    }
    distance
}
fn calculate_distance_complex(song_part: &[Vec<Complex32>], sample: &[Vec<Complex32>], dist: &dyn Fn(Complex32, Complex32) -> f32) -> f32 {
    let bigger_width = max(song_part.len(), sample.len());
    assert!(song_part[0].len() == sample[0].len()); // though dfferent vectors could still be different sizes
    let mut distance = 0.0_f32;
    for x in 0..bigger_width {
        for y in 0..song_part[0].len() {
            let song_part_val = match song_part.get(x) { Some(v) => v[y], None => 0.0.into() };
            let sample_val = match sample.get(x) { Some(v) => v[y], None => 0.0.into() };
            distance += dist(song_part_val, sample_val);
        }
    }
    distance
}

//TODO double m?
fn calculate_assymetric_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, sample_volume: f32) -> f32 {
    calculate_distance(song_part, sample, &|sp, sa| if sp >= sa {0.0} else {(sp-sa) * (sp-sa)}, sample_volume)
} // TODO where do we cut?
fn calculate_symetric_distance(song_part: &[Vec<Complex32>], sample: &[Vec<Complex32>]) -> f32 {
    let cut_sample = if sample.len() > song_part.len() {
        &sample[0..song_part.len()]
    } else { sample };
    let cut_songpart = if sample.len() < song_part.len() {
        &song_part[0..sample.len()]
    } else { song_part };
    //calculate_distance(cut_songpart, cut_sample, &|sp, sa| (sp-sa) * (sp-sa), sample_volume)
    //much better with linear distance/error
    calculate_distance_complex(cut_songpart, cut_sample, &|sp, sa| (sp-sa).norm_sqr())
}


use debug::debug_save_as_image;
use debug::debug_save_as_wav;
use note::Note;

pub fn test_distances_for_instruments(spectrogram_slice: &note::SpectrogramSlice, cache: &note::CachedInstruments) -> Vec<note::Note> {
    let mut test_found_notes: Vec<note::Note> = Vec::new();

    let fft_size = 4096;
    if spectrogram_slice.len() < 40 {
        return Vec::new();
    }//  so that it doesnt panic sometimes
    let song_part = &&spectrogram_slice[0..40];//TODO dont we need hopstocomp here?
    debug_save_as_image(song_part, "song_part.png");
    for instr_idx in 0..note::INSTRUMENT_COUNT {
        print!("\ninstr idx: {}\n", instr_idx);
        for pitch in 0..note::PITCH_COUNT {
            let sample_2dvec = &cache.amplitude_spectrograms[instr_idx][pitch];
            //debug_save_as_image(&wave::subtract_2d_vecs(song_part, &sample_2dvec), &format!("{instr_idx}_pitch{pitch:02}.png"));

            let TEMP_volume = 0.2; // TODO it was 0.5
            let diff = calculate_assymetric_distance(song_part, &sample_2dvec, TEMP_volume);
            
            let silence = [vec![0.0; sample_2dvec[0].len()]; 1];
            let compensation = calculate_assymetric_distance(&silence, &sample_2dvec, TEMP_volume);
            let compensated_val = diff / compensation;
            println!("{pitch:02}: {diff:.5}, comp:{compensation:.5}, compensated: {compensated_val:.5}");

            if compensated_val < 0.035 {// TODO it was 0.015, threshold for guessing if there's a note there
                test_found_notes.push(Note {instrument_id: instr_idx, pitch, volume: TEMP_volume});
                println!("Added this note!");
            }
        }
    }


    //let found_wf = note::add_notes_together(&test_found_notes, cache, 1.0);
    //debug_save_as_wav(&found_wf, "test_found_notes.wav");
    test_found_notes
}




///FROM GPT, temp
fn approximately_equal_2d(v1: &[Vec<f32>], v2: &[Vec<f32>], epsilon: f32) -> bool {
    // First, check if the dimensions are the same
    if v1.len() != v2.len() {
        return false;
    }

    // Check each row for approximate equality
    for (row1, row2) in v1.iter().zip(v2.iter()) {
        if row1.len() != row2.len() {
            return false;
        }

        for (el1, el2) in row1.iter().zip(row2.iter()) {
            if (f32::abs(el1 - el2) >= epsilon) {
                return false;
            }
        }
    }

    true
}








use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::particleswarm::ParticleSwarm;
use argmin::solver::neldermead::NelderMead;
use argmin::core::observers::ObserverMode;
use argmin_observer_slog::SlogLogger;
struct Opti<'a> {
    cache: &'a note::CachedInstruments,
    multiplier: f32,
    song_part: &'a [Vec<Complex32>],
    previous_part: &'a [Vec<Complex32>],
    found_notes: &'a [note::Note],
    hops_to_compare: usize,
}
impl CostFunction for Opti<'_> {
    type Param = Vec<f32>; // it should be found_notes.len() long
    type Output = f32;
    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {

        assert_eq!(param.len(), self.found_notes.len(), "Volume guess vec should be as long as the notes vec to guess");
        let added_spectrogram = note::add_note_spectrograms(self.found_notes, param, self.cache, self.multiplier); //TODO limit ide??
        let with_previous = note::add_spectrograms(&added_spectrogram, self.previous_part);
        //let amplitude_spectrogram = wave::complex_spectrogram_to_amplitude(&with_previous);
        let found_part = &with_previous[0..std::cmp::min(self.hops_to_compare, with_previous.len())];

        //assert_eq!(found_part.len(), spectrogram_2dvec.len(), "The count limit should have been applied previously as well, to save performance!");
        let diff = calculate_symetric_distance(self.song_part, found_part);//TODO 1.0??? why / why not multiplier wtf? and why anyway?



        Ok(diff)
    }
}
impl Gradient for Opti<'_> {
    type Param = Vec<f32>; // it should be found_notes.len() long
    type Gradient = Vec<f32>;
    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
        // cost function: pn = param n, spn = spectrogram n
        // cost = squaredsum(amplitude(p1 * sp1 + p2 * sp2 + ... + pn * spn + spprevious - original))
        // cost = (p1*sp1[00re] + p2*sp2[00re] + ... + spprevious[00re] - original[00re])^2 + (p1*sp1[00im] + p2*sp2[00im] + ... + spprevious[00im] - original[00im])^2
        // + same thing for [01] + [02] + ... + [10] + [11] + ...
        // dcost/dp1 = sp1[00re] * (p1*sp1[00re] + p2*sp2[00re] + ... + spprevious[00re] - original[00re]) + sp1[00im] * (p1*sp1[00im] + p2*sp2[00im] + ... + spprevious[00im] - original[00im])
        // + same thing for [01] + [02] + ... + [10] + [11] + ...
        assert_eq!(param.len(), self.found_notes.len(), "Volume guess vec should be as long as the notes vec to guess");
        let added_spectrogram = note::add_note_spectrograms(self.found_notes, param, self.cache, self.multiplier); //TODO limit ide??
        let with_previous = note::add_spectrograms(&added_spectrogram, self.previous_part);
        let subtracted = note::sub_spectrograms(&with_previous, self.song_part);
        let found_part = &subtracted[0..std::cmp::min(self.hops_to_compare, subtracted.len())];

        //assert_eq!(found_part.len(), spectrogram_2dvec.len(), "The count limit should have been applied previously as well, to save performance!");
        let grad = self.found_notes.iter().map(|note| {
            let note_spectrogram = &self.cache.complex_spectrograms[note.instrument_id][note.pitch];
            let cut_note_spectrogram = &note_spectrogram[0..std::cmp::min(self.hops_to_compare, note_spectrogram.len())];
            let diff = calculate_distance_complex(found_part, &cut_note_spectrogram, &|fp, no| fp.re * no.re + fp.im * no.im);//TODO 1.0??? why / why not multiplier wtf? and why anyway?
            diff
        }).collect();



        Ok(grad)
    }
}

fn get_nm_solver(found_notes: &[Note]) -> NelderMead<Vec<f32>, f32> {
    let param_number = found_notes.len();
    let mut paramsvec_nm: Vec<Vec<f32>> = Vec::new();
    for i in 0..=param_number {
        paramsvec_nm.push(vec![1.0;param_number]);
        if i < param_number {
            paramsvec_nm[i][i] = 0.0;
        }// else { paramsvec_nm[i] = vec![1.0;param_number]; }
    }
    let solverNM = NelderMead::new(paramsvec_nm)
    .with_sd_tolerance(0.0001).unwrap();
    solverNM
}
fn get_pso_solver(found_notes: &[Note]) -> ParticleSwarm<Vec<f32>, f32, rand::rngs::StdRng> {
    let param_number = found_notes.len();
    let solverPSO = ParticleSwarm::new((vec![0.0; param_number], vec![1.0; param_number]), 40); // TODO it could be bigger than 1.0
    solverPSO
}

fn get_linesearch_solver(found_notes: &[Note]) -> SteepestDescent<MoreThuenteLineSearch<Vec<f32>, Vec<f32>, f32>> {
    // https://github.com/argmin-rs/argmin/blob/main/examples/steepestdescent/src/main.rs

    // Pick a line search.
    // let linesearch = HagerZhangLineSearch::new();
    let linesearch = MoreThuenteLineSearch::new();

    // Set up solver
    let solver = SteepestDescent::new(linesearch);
    solver
}

pub fn optimize(cache: &note::CachedInstruments, spectrogram_slice: &[Vec<Complex32>], found_notes: &[note::Note], previous_part: &[Vec<Complex32>]) -> Vec<note::Note>  {
    if found_notes.len() == 0 {
        return Vec::new();
    }
    let hopstocomp = (44100.0/10.0/1024.0) as usize;//TODO ..10?? it depends on self.song_part.len() as well
    let spectrogram = &spectrogram_slice[0..hopstocomp];
    assert_eq!(spectrogram.len(), hopstocomp, "Just to make sure the above function works well - nevermind it got replaced");
    let previous = &previous_part[0..std::cmp::min(hopstocomp, previous_part.len())];

    let cost_function = Opti {cache, multiplier: 1.0, song_part: spectrogram, previous_part: previous, found_notes, hops_to_compare: hopstocomp};//TODO multiplier?

    let solver = get_linesearch_solver(found_notes);
    // Define initial parameter vector
    let init_param: Vec<f32> = vec![0.5; found_notes.len()];

    let res = Executor::new(cost_function, solver)
        .configure(|state| state.param(init_param).max_iters(60))
        .add_observer(SlogLogger::term(), ObserverMode::Always).run().unwrap();

    // Print Result
    println!("{res}");

    //let found_positions = &res.state.get_param().unwrap().position; // PSO
    let found_positions = &res.state.get_best_param().unwrap();//temp


    let added_spectrogram = note::add_note_spectrograms(found_notes, &found_positions, cache, 1.0);
    let with_previous = note::add_spectrograms(&added_spectrogram, previous_part);
    let amplitude_spectrogram = wave::complex_spectrogram_to_amplitude(&with_previous);
    let found_part = &amplitude_spectrogram[0..std::cmp::min(hopstocomp, amplitude_spectrogram.len())];

    let dbg_ampl_spectr = complex_spectrogram_to_amplitude(spectrogram);
    debug_save_as_image(&wave::subtract_2d_vecs(
        &dbg_ampl_spectr, &found_part), 
        "test_diff_found_notes.png");
        debug_save_as_image(&dbg_ampl_spectr[0..hopstocomp], "test_orig_notes.png");

    let mut owned_notes: Vec<Note> = found_notes.to_vec();
    for i in 0..owned_notes.len() {
        owned_notes[i].volume = found_positions[i];
    }
    owned_notes

}

pub fn full_optimize_timestamp(cache: &note::CachedInstruments, spectrogram: &note::AmplitudeSpectrogram, start_hop: usize, previous_part: &Waveform, wf: &Waveform, onset: usize) -> Vec<note::Note>  {// TODO this only needs to be done even less frequently
    let hopstocomp_bigger = 40; //TODO fftsize and hopsize as variables as well
    if wf.to_interleaved_samples().len() <= onset + hopstocomp_bigger * 1024 {
        return Vec::new();
    }
    let cut_wf = Waveform::from_interleaved_samples(
        wf.frame_rate_hz(),
        wf.num_channels(),
        &wf.to_interleaved_samples()[onset..(onset+hopstocomp_bigger*1024)]
    );
    let cut_spectrogram = waveform_to_complex_spectrogram(&cut_wf, 4096, 1024, -1);

    let found_notes = test_distances_for_instruments(&complex_spectrogram_to_amplitude(&cut_spectrogram), &cache);

    let short_vec = vec![0.0.into();1];
    let cut_previous_wf = Waveform::from_interleaved_samples(
        previous_part.frame_rate_hz(),
        previous_part.num_channels(),
        if previous_part.to_interleaved_samples().len() <= onset {
            &short_vec
        } else {
            &previous_part.to_interleaved_samples()[onset..]
        }
    );
    let cut_previous_spectrogram = waveform_to_complex_spectrogram(&cut_previous_wf, 4096, 1024, -1);

    let better_found_notes = optimize(&cache, &cut_spectrogram, &found_notes, &cut_previous_spectrogram);
    better_found_notes
}
//TODO: overamplification and bpm as parameters at first, and try to guess them later; tuning as well???