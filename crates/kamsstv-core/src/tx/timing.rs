use std::time::Duration;

pub struct Segment {
    pub frequency: f32, // Hz — f32 is fine (< 8 significant digits needed)
    pub duration: Duration,
}

/// Distribute segments across integer sample counts while minimising
/// cumulative rounding error.
///
/// The algorithm keeps a running *exact* (f64) sample accumulator and a
/// running *committed* integer count.  Each segment is given
/// `round(acc_ideal) - acc_int` samples, so rounding error never grows
/// beyond ±0.5 samples and the global sum always equals
/// `round(Σ d·fs)`.
pub fn plan(segments: &[Segment], fs: u32) -> Vec<u32> {
    let mut acc_ideal: f64 = 0.0; // running exact (fractional) sample count
    let mut acc_int: u64 = 0; // running committed integer sample count
    segments
        .iter()
        .map(|s| {
            // as_secs_f64() is lossless to f64 precision (~15 digits),
            // avoiding the f32 underflow / precision-collision hazard.
            acc_ideal += s.duration.as_secs_f64() * fs as f64;
            let next_int = acc_ideal.round() as u64;
            let count = next_int - acc_int;
            acc_int = next_int;
            count as u32
        })
        .collect()
}

// ── test helpers ──────────────────────────────────────────────────────────────

#[cfg(test)]
fn micros(us: u64) -> Duration {
    Duration::from_micros(us)
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const FS: u32 = 48_000;

    /// Empty input must return an empty Vec without panicking.
    /// Both invariants hold trivially: sum = 0 = round(0).
    #[test]
    fn empty_slice_returns_empty_vec() {
        assert_eq!(plan(&[], FS), Vec::<u32>::new());
    }

    /// Global invariant: Σ counts == round(Σ d·fs).
    ///
    /// The expected value is computed with the same multiply-then-sum order
    /// as plan() to avoid any discrepancy from floating-point associativity.
    #[test]
    fn counts_sum_to_rounded_total_duration() {
        let n = 100;
        // 457 µs ≈ 21.936 samples @ 48 kHz — deliberately non-integer.
        let segments: Vec<Segment> = (0..n)
            .map(|_| Segment {
                frequency: 1500.0,
                duration: micros(457),
            })
            .collect();

        let counts: Vec<u32> = plan(&segments, FS);
        let counts_sum: u64 = counts.iter().map(|&c| c as u64).sum();

        let expected: u64 = segments
            .iter()
            .map(|s| s.duration.as_secs_f64() * FS as f64)
            .sum::<f64>()
            .round() as u64;

        assert_eq!(counts_sum, expected);
    }

    /// Local invariant: each count is strictly within 1 sample of its own
    /// ideal (non-integer) length.  The carry mechanism guarantees the
    /// error never reaches exactly ±1.0.
    #[test]
    fn each_count_within_one_sample_of_ideal() {
        let n = 100;
        let segments: Vec<Segment> = (0..n)
            .map(|_| Segment {
                frequency: 1500.0,
                duration: micros(457),
            })
            .collect();

        let counts = plan(&segments, FS);

        for (i, (count, seg)) in counts.iter().zip(segments.iter()).enumerate() {
            let ideal = seg.duration.as_secs_f64() * FS as f64;
            let diff = (*count as f64 - ideal).abs();
            assert!(
                diff < 1.0,
                "segment {i}: count={count}, ideal={ideal:.4}, |diff|={diff:.4} >= 1"
            );
        }
    }
}

// ── property tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Realistic audio sample rates (Hz).
    const RATES: &[u32] = &[
        8_000, 11_025, 16_000, 22_050, 44_100, 48_000, 96_000, 192_000,
    ];

    proptest! {
        /// Both invariants must hold for arbitrary segment counts, durations,
        /// and sample rates — including the empty-slice case (vec size 0..=200).
        ///
        /// Durations are expressed as integer microseconds so the strategy
        /// itself is free of f32/f64 representation ambiguity.
        #[test]
        fn both_invariants_hold_for_arbitrary_segments(
            // 1 µs … 2 s, up to 200 segments, including the empty case.
            duration_us in prop::collection::vec(1_u64..=2_000_000_u64, 0..=200),
            // Index into RATES to avoid generating invalid sample rates.
            fs_idx in 0..RATES.len(),
        ) {
            let fs = RATES[fs_idx];
            let segments: Vec<Segment> = duration_us
                .iter()
                .map(|&us| Segment {
                    frequency: 440.0,
                    duration: Duration::from_micros(us),
                })
                .collect();

            let counts = plan(&segments, fs);

            // ── global invariant ──────────────────────────────────────────────
            // u64 accumulator: 200 segments × 2 s × 192 000 Hz = 76.8 M > u32::MAX.
            let total_ideal: f64 = segments
                .iter()
                .map(|s| s.duration.as_secs_f64() * fs as f64)
                .sum();
            let expected_total = total_ideal.round() as u64;
            let actual_total: u64 = counts.iter().map(|&c| c as u64).sum();

            prop_assert_eq!(
                actual_total,
                expected_total,
                "sum mismatch at fs={}: got {}, expected {}", fs, actual_total, expected_total
            );

            // ── local invariant ───────────────────────────────────────────────
            for (i, (count, seg)) in counts.iter().zip(segments.iter()).enumerate() {
                let ideal = seg.duration.as_secs_f64() * fs as f64;
                let diff = (*count as f64 - ideal).abs();
                prop_assert!(
                    diff < 1.0,
                    "segment {i} at fs={fs}: count={count}, ideal={ideal:.4}, |diff|={diff:.4} >= 1"
                );
            }
        }
    }
}
