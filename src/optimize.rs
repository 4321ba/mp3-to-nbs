
use crate::fourier;
use crate::note;
use crate::observer::TracingLogger;
use note::Note;
use tracing::debug;
use crate::fourier::complex_spectrogram_to_amplitude;
use crate::fourier::waveform_to_complex_spectrogram;

use argmin::core::Gradient;
use argmin::core::State;
use argmin::solver::gradientdescent::SteepestDescent;
use argmin::solver::linesearch::MoreThuenteLineSearch;
use babycat::{Signal, Waveform};
use microfft::Complex32;
use argmin::core::{CostFunction, Error, Executor};
use argmin::core::observers::ObserverMode;


pub fn calculate_asymmetric_distance(song_part: &[Vec<f32>], sample: &Vec<Vec<f32>>, sample_volume: f32) -> f32 {
    fourier::calculate_distance(song_part, sample, &|sp, sa| if sp >= sa {0.0} else {(sp-sa) * (sp-sa)}, sample_volume)
}
pub fn test_distances_for_instruments(song_part: &[Vec<f32>], cache: &note::CachedInstruments) -> Vec<note::Note> {
    let volume_to_test_with = 0.2; // volume for the sound sample to test with
    let guess_threshold = 0.035; // threshold for guessing if there's a note there
    let mut found_notes: Vec<note::Note> = Vec::new();
    //debug_save_as_image(song_part, "song_part.png");
    for instr_idx in 0..note::INSTRUMENT_COUNT {
        for pitch in 0..note::PITCH_COUNT {
            let sample_spectrogram = &cache.amplitude_spectrograms[instr_idx][pitch];
            //debug_save_as_image(&wave::subtract_2d_vecs(song_part, &sample_spectrogram), &format!("{instr_idx}_pitch{pitch:02}.png"));

            let diff = calculate_asymmetric_distance(song_part, &sample_spectrogram, volume_to_test_with);
            
            let silence = [vec![0.0; sample_spectrogram[0].len()]; 1];
            let compensation = calculate_asymmetric_distance(&silence, &sample_spectrogram, volume_to_test_with);
            let compensated_val = diff / compensation;
            
            if compensated_val < guess_threshold {
                found_notes.push(Note {instrument_id: instr_idx, pitch, volume: volume_to_test_with});
                debug!("Added note: instr:{instr_idx}, pitch:{pitch:02}, diff: {diff:.5}, diff to silence: {compensation:.5}, ratio: {compensated_val:.5}");
            }
        }
    }
    found_notes
}



struct OptiProblem<'a> {
    cache: &'a note::CachedInstruments,
    multiplier: f32,
    song_part: &'a [Vec<Complex32>],
    previous_part: &'a [Vec<Complex32>],
    found_notes: &'a [note::Note],
    hops_to_compare: usize,
}
impl CostFunction for OptiProblem<'_> {
    type Param = Vec<f32>; // it should be found_notes.len() long
    type Output = f32;
    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        assert_eq!(param.len(), self.found_notes.len(), "Volume guess vec should be as long as the notes vec to guess");
        let added_spectrogram = note::add_note_spectrograms(self.found_notes, param, self.cache, self.multiplier);
        let with_previous = fourier::add_spectrograms(&added_spectrogram, self.previous_part);
        let found_part = &with_previous[0..std::cmp::min(self.hops_to_compare, with_previous.len())];
        let diff = fourier::calculate_distance_complex(self.song_part, found_part, &|sp, fp| (sp-fp).norm_sqr());
        Ok(diff)
    }
}
impl Gradient for OptiProblem<'_> {
    type Param = Vec<f32>; // it should be found_notes.len() long
    type Gradient = Vec<f32>;
    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
        // cost function: pn = param n, spn = spectrogram n
        // cost = squaredsum(amplitude(p1 * sp1 + p2 * sp2 + ... + pn * spn + spprevious - original))
        // cost = (p1*sp1[00re] + p2*sp2[00re] + ... + spprevious[00re] - original[00re])^2 + (p1*sp1[00im] + p2*sp2[00im] + ... + spprevious[00im] - original[00im])^2
        // + same thing for [01] + [02] + ... + [10] + [11] + ...
        // dcost/dp1 = 2 * sp1[00re] * (p1*sp1[00re] + p2*sp2[00re] + ... + spprevious[00re] - original[00re]) + 2 * sp1[00im] * (p1*sp1[00im] + p2*sp2[00im] + ... + spprevious[00im] - original[00im])
        // + same thing for [01] + [02] + ... + [10] + [11] + ...
        assert_eq!(param.len(), self.found_notes.len(), "Volume guess vec should be as long as the notes vec to guess");
        let added_spectrogram = note::add_note_spectrograms(self.found_notes, param, self.cache, self.multiplier);
        let with_previous = fourier::add_spectrograms(&added_spectrogram, self.previous_part);
        let subtracted = fourier::sub_spectrograms(&with_previous, self.song_part);
        let found_part = &subtracted[0..std::cmp::min(self.hops_to_compare, subtracted.len())];
        let grad = self.found_notes.iter().map(|note| {
            let note_spectrogram = &self.cache.complex_spectrograms[note.instrument_id][note.pitch];
            let cut_note_spectrogram = &note_spectrogram[0..std::cmp::min(self.hops_to_compare, note_spectrogram.len())];
            let diff = fourier::calculate_distance_complex(found_part, &cut_note_spectrogram, &|fp, no| fp.re * no.re + fp.im * no.im);
            2.0 * diff
        }).collect();
        Ok(grad)
    }
}

pub fn optimize(
        cache: &note::CachedInstruments,
        spectrogram_slice: &[Vec<Complex32>],
        found_notes: &[note::Note],
        previous_part: &[Vec<Complex32>],
        hops_to_compare: usize
    ) -> Vec<note::Note> {

    if found_notes.len() == 0 {
        return Vec::new();
    }
    let spectrogram = &spectrogram_slice[0..hops_to_compare];
    let previous = &previous_part[0..std::cmp::min(hops_to_compare, previous_part.len())];

    let cost_function = OptiProblem {cache, multiplier: 1.0, song_part: spectrogram, previous_part: previous, found_notes, hops_to_compare};

    // https://github.com/argmin-rs/argmin/blob/main/examples/steepestdescent/src/main.rs
    // Pick a line search.
    // let linesearch = HagerZhangLineSearch::new();
    let linesearch = MoreThuenteLineSearch::new();
    // Set up solver
    let solver = SteepestDescent::new(linesearch);

/*
    // set up line search
    let linesearch = MoreThuenteLineSearch::new();
    let beta_method = PolakRibiere::new();

    // Set up nonlinear conjugate gradient method
    let solver = NonlinearConjugateGradient::new(linesearch, beta_method)
        // Set the number of iterations when a restart should be performed
        // This allows the algorithm to "forget" previous information which may not be helpful anymore.
        .restart_iters(10)
        // Set the value for the orthogonality measure.
        // Setting this parameter leads to a restart of the algorithm (setting beta = 0) after two
        // consecutive search directions are not orthogonal anymore. In other words, if this condition
        // is met:
        //
        // `|\nabla f_k^T * \nabla f_{k-1}| / | \nabla f_k ||^2 >= v`
        //
        // A typical value for `v` is 0.1.
        .restart_orthogonality(0.1);
*/
    // Define initial parameter vector
    let init_param: Vec<f32> = vec![0.5; found_notes.len()];

    let res = Executor::new(cost_function, solver)
        .configure(|state| state.param(init_param).max_iters(80))
//        .configure(|state| state.param(init_param).max_iters(80).target_cost(0.0))
        .add_observer(TracingLogger::new(), ObserverMode::Always).run().unwrap();

    // Print Result
    debug!("{res}");

    let found_positions = &res.state.get_best_param().unwrap();

    /*
    let added_spectrogram = note::add_note_spectrograms(found_notes, &found_positions, cache, 1.0);
    let with_previous = fourier::add_spectrograms(&added_spectrogram, previous_part);
    let amplitude_spectrogram = fourier::complex_spectrogram_to_amplitude(&with_previous);
    let found_part = &amplitude_spectrogram[0..std::cmp::min(hopstocomp, amplitude_spectrogram.len())];
    let dbg_ampl_spectr = complex_spectrogram_to_amplitude(spectrogram);
    debug_save_as_image(&fourier::subtract_amplitude_spectrograms(
        &dbg_ampl_spectr, &found_part), 
        "test_diff_found_notes.png");
    debug_save_as_image(&dbg_ampl_spectr[0..hopstocomp], "test_orig_notes.png");
    */

    let mut owned_notes: Vec<Note> = found_notes.to_vec();
    for i in 0..owned_notes.len() {
        owned_notes[i].volume = found_positions[i];
    }
    owned_notes
}

pub fn full_optimize_timestamp(cache: &note::CachedInstruments, previous_part: &Waveform, wf: &Waveform, onset: usize, tps: f64) -> Vec<note::Note> {
    let hop_count_to_compare = 40;
    let mut samples = wf.to_interleaved_samples()
        [onset..std::cmp::min(onset + hop_count_to_compare * fourier::HOP_SIZE, wf.to_interleaved_samples().len())].to_vec();
    samples.resize(hop_count_to_compare * fourier::HOP_SIZE, 0.0);
    let cut_wf = Waveform::from_interleaved_samples(wf.frame_rate_hz(), wf.num_channels(), &samples);
    let cut_spectrogram = waveform_to_complex_spectrogram(&cut_wf, fourier::FFT_SIZE, fourier::HOP_SIZE, -1);

    let found_notes = test_distances_for_instruments(&complex_spectrogram_to_amplitude(&cut_spectrogram), &cache);

    let short_vec = vec![0.0.into(); 10];
    let cut_previous_wf = Waveform::from_interleaved_samples(
        previous_part.frame_rate_hz(),
        previous_part.num_channels(),
        if previous_part.to_interleaved_samples().len() <= onset {
            &short_vec
        } else {
            &previous_part.to_interleaved_samples()[onset..]
        }
    );
    let cut_previous_spectrogram = waveform_to_complex_spectrogram(&cut_previous_wf, fourier::FFT_SIZE, fourier::HOP_SIZE, -1);

    let hops_to_compare = (wf.frame_rate_hz() as f64 / tps / fourier::HOP_SIZE as f64) as usize;
    let optimized_notes = optimize(&cache, &cut_spectrogram, &found_notes, &cut_previous_spectrogram, hops_to_compare);
    optimized_notes
}