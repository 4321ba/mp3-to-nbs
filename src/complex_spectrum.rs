// code copied from spectrum-analyzer library, and modified a little bit
// to export the complex result of the fft calculation

/*
MIT License

Copyright (c) 2023 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use spectrum_analyzer::error::SpectrumAnalyzerError;


use microfft::real;

/// The result of a FFT is always complex but because different FFT crates might
/// use different versions of "num-complex", each implementation exports
/// it's own version that gets used in lib.rs for binary compatibility.
use microfft::Complex32;

/// Calculates the real FFT by invoking the proper function corresponding to the
/// buffer length.
macro_rules! real_fft_n {
    ($buffer:expr, $( $i:literal ),*) => {
        match $buffer.len() {
            $(
                $i => {
                    let mut buffer: [_; $i] = $buffer.try_into().unwrap();
                    paste::paste! (
                        real::[<rfft_$i>]
                    )(&mut buffer).to_vec()
                }
            )*
            _ => { unimplemented!("unexpected buffer len") }
        }
    };
}

/// Real FFT using [`microfft::real`].
pub struct FftImpl;

impl FftImpl {
    /// Calculates the FFT For the given input samples and returns a Vector of
    /// of [`Complex32`] with length `samples.len() / 2 + 1`, where the first
    /// index corresponds to the DC component and the last index to the Nyquist
    /// frequency.
    ///
    /// # Parameters
    /// - `samples`: Array with samples. Each value must be a regular floating
    ///              point number (no NaN or infinite) and the length must be
    ///              a power of two. Otherwise, the function panics.
    #[inline]
    pub fn calc(samples: &[f32]) -> Vec<Complex32> {
        let mut fft_res: Vec<Complex32> =
            real_fft_n!(samples, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384);

        // `microfft::real` documentation says: the Nyquist frequency real value
        // is packed inside the imaginary part of the DC component.
        let nyquist_fr_pos_val = fft_res[0].im;
        fft_res[0].im = 0.0;
        // manually add the nyquist frequency
        fft_res.push(Complex32::new(nyquist_fr_pos_val, 0.0));
        fft_res
    }
}



/// Takes an array of samples (length must be a power of 2),
/// e.g. 2048, applies an FFT (using the specified FFT implementation) on it
/// and returns all frequencies with their volume/magnitude.
///
/// By default, no normalization/scaling is done at all and the results,
/// i.e. the frequency magnitudes/amplitudes/values are the raw result from
/// the FFT algorithm, except that complex numbers are transformed
/// to their magnitude.
///
/// * `samples` raw audio, e.g. 16bit audio data but as f32.
///             You should apply an window function (like Hann) on the data first.
///             The final frequency resolution is `sample_rate / (N / 2)`
///             e.g. `44100/(16384/2) == 5.383Hz`, i.e. more samples =>
///             better accuracy/frequency resolution. The amount of samples must
///             be a power of 2. If you don't have enough data, provide zeroes.
/// * `sampling_rate` sampling_rate, e.g. `44100 [Hz]`
/// * `frequency_limit` Frequency limit. See [`FrequencyLimit´]
/// * `scaling_fn` See [`crate::scaling::SpectrumScalingFunction`] for details.
///
/// ## Returns value
/// New object of type [`FrequencySpectrum`].
///
/// ## Examples
/// ### Scaling via dynamic closure
/// ```rust
/// use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
/// // get data from audio source
/// let samples = vec![0.0, 1.1, 5.5, -5.5];
/// let res = samples_fft_to_spectrum(
///         &samples,
///         44100,
///         FrequencyLimit::All,
///         Some(&|val, info| val - info.min),
///  );
/// ```
/// ### Scaling via static function
/// ```rust
/// use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
/// use spectrum_analyzer::scaling::scale_to_zero_to_one;
/// // get data from audio source
/// let samples = vec![0.0, 1.1, 5.5, -5.5];
/// let res = samples_fft_to_spectrum(
///         &samples,
///         44100,
///         FrequencyLimit::All,
///         Some(&scale_to_zero_to_one),
///  );
/// ```
///
/// ## Panics
/// * When `samples.len()` isn't a power of two less than or equal to `16384` and `microfft` is used
pub fn samples_fft_to_complex_spectrum(
    samples: &[f32],
) -> Result<Vec<Complex32>, SpectrumAnalyzerError> {
    // everything below two samples is unreasonable
    if samples.len() < 2 {
        return Err(SpectrumAnalyzerError::TooFewSamples);
    }
    // do several checks on input data
    if samples.iter().any(|x| x.is_nan()) {
        return Err(SpectrumAnalyzerError::NaNValuesNotSupported);
    }
    if samples.iter().any(|x| x.is_infinite()) {
        return Err(SpectrumAnalyzerError::InfinityValuesNotSupported);
    }
    if !samples.len().is_power_of_two() {
        return Err(SpectrumAnalyzerError::SamplesLengthNotAPowerOfTwo);
    }

    // With FFT we transform an array of time-domain waveform samples
    // into an array of frequency-domain spectrum samples
    // https://www.youtube.com/watch?v=z7X6jgFnB6Y

    // FFT result has same length as input
    // (but when we interpret the result, we don't need all indices)

    // applies the f32 samples onto the FFT algorithm implementation
    // chosen at compile time (via Cargo feature).
    // If a complex FFT implementation was chosen, this will internally
    // transform all data to Complex numbers.
    let fft_res = FftImpl::calc(samples);

    let frequency_vec = fft_res
        .into_iter()
        // See https://stackoverflow.com/a/4371627/2891595 for more information as well as
        // https://www.gaussianwaves.com/2015/11/interpreting-fft-results-complex-dft-frequency-bins-and-fftshift/
        //
        // The indices 0 to N/2 (inclusive) are usually the most relevant. Although, index
        // N/2-1 is declared as the last useful one on stackoverflow (because in typical applications
        // Nyquist-frequency + above are filtered out), we include everything here.
        // with 0..=(samples_len / 2) (inclusive) we get all frequencies from 0 to Nyquist theorem.
        //
        // Indices (samples_len / 2)..len() are mirrored/negative. You can also see this here:
        // https://www.gaussianwaves.com/gaussianwaves/wp-content/uploads/2015/11/realDFT_complexDFT.png
        .take(samples.len() / 2 + 1)
        .map(|v| Complex32 {re: v.re/samples.len() as f32, im: v.im/samples.len() as f32} )
        .collect::<Vec<Complex32>>();
    Ok(frequency_vec)
}
