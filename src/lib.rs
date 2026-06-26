// Numerically Controlled Oscillator
// Nco is stateful (evolve phase from step to next step)

use std::f32::consts::PI;

// Note: f32 should be suffiecient for SSTV (worth testing later)
pub struct Nco {
    phase: f32,      // φ[k] -- phase accumulator
    step_scale: f32, // 2π/fs -- precomputed phase scalar
}

impl Nco {
    pub fn new(fs: f32) -> Self {
        debug_assert!(fs > 0.0);
        Nco {
            phase: 0.0, // we are free to choose φ[0]=0 (convention)
            step_scale: 2.0 * PI / fs,
        }
    }
    pub fn next_sample(&mut self, f: f32) -> f32 {
        // Convention: ADVANCE-then-READ. phase moves first, then we read sin().
        // φ[k+1] = φ[k] + (2π/fs)·f mod 2π
        // .rem_euclid() helps to avoid issues with modulo aithmetics in Rust
        self.phase = (self.phase + self.step_scale * f).rem_euclid(2.0 * PI);
        self.phase.sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*; // pull in Nco from parent module
    use proptest::prelude::*;
    const FS: f32 = 48_000.0; // sample rate; MUST be the same value used in Nco::new and in the bound
    const EPS: f32 = 1e-6; // absorbs rounding errors
    const TOL: f32 = 5e-3;

    // Schedule-generation bounds for the property test below.
    // SSTV video subcarrier band: 1500 Hz (black) .. 2300 Hz (white).
    const TONE_LO: f32 = 1500.0;
    const TONE_HI: f32 = 2300.0;
    const MAX_HOLD: usize = 64; // longest a single tone is held, in samples
    const MIN_SEGMENTS: usize = 2; // >= 2 guarantees at least one seam (tone change)
    const MAX_SEGMENTS: usize = 16;
    const ACCURACY_SAMPLES: usize = 4096;

    #[test]
    fn first_sample_reflects_advanced_phase() {
        let f = 1500.0_f32;
        let mut nco = Nco::new(FS);
        let first = nco.next_sample(f);
        let expected = (2.0 * PI * f / FS).sin();
        assert!((expected - first).abs() < EPS);
    }

    // Max possible |sample[k+1] - sample[k]| for a sine of frequency `f` at rate `fs`.
    // sin(φ+Δφ) - sin(φ) = 2·cos(φ+Δφ/2)·sin(Δφ/2); |cos| ≤ 1, Δφ = 2π·f/fs
    // => supremum = 2·sin(π·f/fs). Attainable, so callers compare with a tolerance.
    fn max_adjacent_step(f: f32, fs: f32) -> f32 {
        2.0 * (PI * f / fs).sin()
    }
    fn tone_energy_fraction(samples: &[f32], f: f32, fs: f32) -> f32 {
        let n = samples.len();
        let phi_step = 2.0 * PI * f / fs; // Δφ per sample

        let energy: f32 = samples.iter().map(|s| s.powi(2)).sum();
        let (c_re, c_im, _) = samples
            .iter()
            .fold((0.0_f32, 0.0_f32, 0.0_f32), |(re, im, phase), s| {
                (re + s * phase.cos(), im + s * phase.sin(), phase + phi_step)
            });

        (c_re * c_re + c_im * c_im) / energy / n as f32 // // → 0.5 for pure unit sine (Parseval)
    }

    proptest! {
        #[test]
        fn samples_stay_in_unit_range(f in TONE_LO..=TONE_HI) {
            let mut nco = Nco::new(FS);
            for _ in 0..ACCURACY_SAMPLES {
            let s = nco.next_sample(f);
            prop_assert!((-1.0..=1.0).contains(&s), "{} out of [-1, 1]", s);
            }
        }
        #[test]
        fn phase_continuity_bound(
            schedule in prop::collection::vec(
                (TONE_LO..=TONE_HI, 1usize..=MAX_HOLD),
                MIN_SEGMENTS..=MAX_SEGMENTS,
            ),
        ) {
            // Synthesize the waveform through a SINGLE nco, so phase carries across
            // tone changes -- that seam continuity is exactly what this test checks.
            // (A fresh nco per segment would reset phase to 0 and destroy the property.)
            let mut nco = Nco::new(FS);
            let mut samples = Vec::new();
            for &(tone, hold) in &schedule {
                for _ in 0..hold {
                    samples.push(nco.next_sample(tone));
                }
            }
            // max tone across the schedule. `|&(tone, _)|` derefs the &(f32, usize)
            // from iter() and binds `tone` by value; `_` drops the hold. fold seeds at
            // 0.0 (safe ONLY because tones are positive) and reduces with f32::max as a
            // fn-value -- can't use Iterator::max(), f32 isn't Ord (NaN breaks ordering).
            let f_max = schedule.iter().map(|&(tone, _)| tone).fold(0.0f32, f32::max);
            let max_diff = samples.windows(2)
                .map(|w| (w[0] - w[1]).abs())
                .fold(0.0f32, f32::max);
            let bound = max_adjacent_step(f_max, FS);
            prop_assert!(f_max <= FS / 2.0);
            prop_assert!(max_diff <= bound + EPS, "max_diff {} exceeded bound {}", max_diff, bound);
        }
        #[test]
        fn energy_concentrates_at_given_tone(f in TONE_LO..=TONE_HI){
            let mut nco = Nco::new(FS);
            let samples: Vec<f32> = (0..ACCURACY_SAMPLES).map(|_|nco.next_sample(f)).collect();
            let frac = tone_energy_fraction(&samples, f, FS);
            prop_assert!((frac-0.5).abs() < TOL, "frac {} off 0.5 at f={}", frac, f)
        }
    }
}
