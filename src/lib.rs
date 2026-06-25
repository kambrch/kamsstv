// Numerically Controlled Oscillator
// Nco is stateful (evolve phase from step to next step)
pub struct Nco {
    phase: f32,      // φ[k] -- phase accumulator
    step_scale: f32, // 2π/fs -- precomputed phase scalar
}

impl Nco {
    pub fn new(fs: f32) -> Self {
        Nco {
            phase: 0.0, // oscillation begins at sin(0)=0 (convention)
            step_scale: 2.0 * std::f32::consts::PI / fs,
        }
    }
    pub fn next_sample(&mut self, f: f32) -> f32 {
        // Convention: ADVANCE-then-READ. phase moves first, then we read sin().
        // Consequence: sample 0 = sin(step*f), NOT sin(0)=0. A future
        // "starts at zero" test would need the two lines swapped (read-then-advance).
        self.phase += self.step_scale * f; // φ[k+1] = φ[k] + (2π/fs)·f  -- the accumulator IS the integral of frequency
        self.phase.sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*; // pull in Nco from parent module
    use proptest::prelude::*;
    const FS: f32 = 48_000.0; // sample rate; MUST be the same value used in Nco::new and in the bound
    const EPS: f32 = 1e-6; // absorbs rounding errors 
    #[test]
    fn dc_input_yields_constant_output() {
        let mut nco = Nco::new(FS);
        let a = nco.next_sample(0.0);
        let b = nco.next_sample(0.0);
        assert!(a == b);
    }

    proptest! {
        #[test]
        // SINGLE-TONE for now: positive bound-coverage only
        // To catch the bug, swap `f` for a randomized (tone, hold) schedule with seams.
        fn phase_continuity_bound(f in 1500.0f32..=2300.0) {
            let mut nco = Nco::new(FS);
            // waveform: 100 samples at one constant tone -> NO seam (can't trip the bug yet)
            let samples: Vec<f32> = (0..100).map(|_| nco.next_sample(f)).collect();
            let max_diff = samples.windows(2)
                .map(|w| (w[0] - w[1]).abs())
                .fold(0.0f32, f32::max);

            let bound = 2.0 * (std::f32::consts::PI * f / FS).sin();
            prop_assert!(max_diff <= bound + EPS, "max_diff {} exceeded bound {}", max_diff, bound);
            }
    }
}
