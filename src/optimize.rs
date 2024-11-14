
use crate::note;
use crate::wave;
use crate::debug;
use crate::wave::waveform_to_spectrogram;

use argmin::core::State;
use argmin_math::Rng;
use babycat::{Signal, Waveform};





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

//TODO double m?
fn calculate_assymetric_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, sample_volume: f32) -> f32 {
    calculate_distance(song_part, sample, &|sp, sa| if sp >= sa {0.0} else {(sp-sa) * (sp-sa)}, sample_volume)
} // TODO where do we cut?
fn calculate_symetric_distance(song_part: &note::SpectrogramSlice, sample: &note::SpectrogramSlice, sample_volume: f32) -> f32 {
    let cut_sample = if sample.len() > song_part.len() {
        &sample[0..song_part.len()]
    } else { sample };
    let cut_songpart = if sample.len() < song_part.len() {
        &song_part[0..sample.len()]
    } else { song_part };
    //calculate_distance(cut_songpart, cut_sample, &|sp, sa| (sp-sa) * (sp-sa), sample_volume)
    //much better with linear distance/error
    calculate_distance(cut_songpart, cut_sample, &|sp, sa| (sp-sa).abs(), sample_volume)
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
    song_part: &'a note::SpectrogramSlice,
    found_notes: &'a [note::Note],
    hops_to_compare: usize,
}
impl CostFunction for Opti<'_> {
    type Param = Vec<f32>; // it should be found_notes.len() long
    type Output = f32;
    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {

        assert_eq!(param.len(), self.found_notes.len(), "Volume guess vec should be as long as the notes vec to guess");
        
        //for nelder-mead
        //if param.iter().any(|x| *x < 0.0) {return Ok(1000.0);} // very expensive
        
        let wf = note::add_notes_together_merge_from_stsp(self.found_notes, param, self.cache, self.multiplier); //TODO only add the necessary length together
        let fft_size = 4096;
        let spectrogram = wave::create_spectrum(wf.to_interleaved_samples(), wf.frame_rate_hz(), fft_size, 1024, self.hops_to_compare as isize);
        let spectrogram_2dvec = wave::spectrum_to_2d_vec(&spectrogram);

        let added_cx_sptr = note::add_cx_spectrograms(self.found_notes, param, self.cache, self.multiplier);
        let real_from_cx = wave::complex_spectrogram_to_amplitude(&added_cx_sptr);
        //println!("QWE {:?}", spectrogram_2dvec);
        //println!("ASD {:?}", real_from_cx);
        let mut qwe = vec![vec![0.0.into(); real_from_cx[0].len()]; 10];
        let found_part = &spectrogram_2dvec[0..self.hops_to_compare];
        let cut_part_from_cx = if real_from_cx.len()>=10 {
            &real_from_cx[0..self.hops_to_compare]
        } else {
            for (i, row) in real_from_cx.iter().enumerate() {
                qwe[i] = row.clone();
            }
            &qwe
        };
        //debug::debug_save_as_image(cut_part_from_cx, "TESTcut_part_from_cx.png");
        //debug::debug_save_as_image(&spectrogram_2dvec, "TESTspec2d.png");

        assert!(approximately_equal_2d(cut_part_from_cx, &spectrogram_2dvec, 0.0001), "The complex-to-amplitude and spectrum-analyzer calculations differ!");

        assert_eq!(found_part.len(), spectrogram_2dvec.len(), "The count limit should have been applied previously as well, to save performance!");
        let diff = calculate_symetric_distance(self.song_part, found_part, 1.0);//TODO 1.0?



        Ok(diff)

        //Ok((param[0]-0.34) *(param[0]-0.34)+ (param[1]-0.36) *(param[1]-0.36))
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

pub fn optimize(cache: &note::CachedInstruments, spectrogram_slice: &note::SpectrogramSlice, found_notes: &[note::Note]) -> Vec<note::Note>  {
    if found_notes.len() == 0 {
        return Vec::new();
    }
    let hopstocomp = 10;//TODO ..10?? it depends on self.song_part.len() as well
    let spectrogram = &spectrogram_slice[0..hopstocomp];
    assert_eq!(spectrogram.len(), hopstocomp, "Just to make sure the above function works well - nevermind it got replaced");

    let cost_function = Opti {cache, multiplier: 1.0, song_part: spectrogram, found_notes, hops_to_compare: hopstocomp};//TODO multiplier?

    let solver = get_nm_solver(found_notes);

    let res = Executor::new(cost_function, solver)
        .configure(|state| state.max_iters(200))
        .add_observer(SlogLogger::term(), ObserverMode::Always).run().unwrap();

    // Print Result
    println!("{res}");

    //let found_positions = &res.state.get_param().unwrap().position; // PSO
    let found_positions = &res.state.get_best_param().unwrap();//temp



    let guess_wf = note::add_notes_together_merge_from_stsp(found_notes, found_positions, cache, 1.0);
    debug_save_as_image(&wave::subtract_2d_vecs(
        spectrogram, &waveform_to_spectrogram(&guess_wf, 4096, 1024))[0..hopstocomp], 
        "test_diff_found_notes.png");

    let mut owned_notes: Vec<Note> = found_notes.to_vec();
    for i in 0..owned_notes.len() {
        owned_notes[i].volume = found_positions[i];
    }
    owned_notes

}

pub fn full_optimize_timestamp(cache: &note::CachedInstruments, spectrogram: &note::AmplitudeSpectrogram, start_hop: usize) -> Vec<note::Note>  {// TODO this only needs to be done even less frequently
    let found_notes = test_distances_for_instruments(&spectrogram[start_hop..], &cache);
    let better_found_notes = optimize(&cache, &spectrogram[start_hop..], &found_notes);
    better_found_notes
}
//TODO: overamplification and bpm as parameters at first, and try to guess them later; tuning as well???