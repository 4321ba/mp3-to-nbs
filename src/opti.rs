use argmin::core::CostFunction;
use argmin_math::Error;
use babycat::Signal;

use crate::{note, optimize::calculate_symetric_distance, wave};

pub struct Opti<'a> {
    pub cache: &'a note::CachedInstruments,
    pub multiplier: f32,
    pub song_part: &'a note::SpectrogramSlice,
    pub found_notes: &'a [note::Note],
    pub hops_to_compare: usize,
}

impl CostFunction for Opti<'_> {
    type Param = Vec<f32>; // it should be found_notes.len() long
    type Output = f32;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        assert_eq!(
            param.len(),
            self.found_notes.len(),
            "Volume guess vec should be as long as the notes vec to guess"
        );

        //for nelder-mead
        //if param.iter().any(|x| *x < 0.0) {return Ok(1000.0);} // very expensive

        let wf = note::add_notes_together_merge_from_stsp(
            self.found_notes,
            param,
            self.cache,
            self.multiplier,
        ); //TODO only add the necessary length together

        let fft_size = 4096;

        let spectrogram = wave::create_spectrum(
            wf.to_interleaved_samples(),
            wf.frame_rate_hz(),
            fft_size,
            1024,
            self.hops_to_compare as isize,
        );

        let spectrogram_2dvec = wave::spectrum_to_2d_vec(&spectrogram);
        let found_part = &spectrogram_2dvec[0..self.hops_to_compare];

        assert_eq!(
            found_part.len(),
            spectrogram_2dvec.len(),
            "The count limit should have been applied previously as well, to save performance!"
        );

        let diff = calculate_symetric_distance(self.song_part, found_part, 1.0); //TODO 1.0?

        Ok(diff)

        //Ok((param[0]-0.34) *(param[0]-0.34)+ (param[1]-0.36) *(param[1]-0.36))
    }
}
