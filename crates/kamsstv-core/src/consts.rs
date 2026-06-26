//! Tone frequencies and band geometry, shared across modes (per design.md).
//!
//! These are facts of the SSTV format, not tunable parameters. Promote a tone
//! into `ModeSpec` only if a future mode breaks the assumption that it is constant.

/// Black level subcarrier.
pub const BLACK_HZ: f32 = 1500.0;
/// White level subcarrier.
pub const WHITE_HZ: f32 = 2300.0;
/// Sync pulse tone.
pub const SYNC_HZ: f32 = 1200.0;
/// Band centre — RX IQ downconversion mixes the input down to here.
pub const BAND_CENTRE_HZ: f32 = 1900.0;
