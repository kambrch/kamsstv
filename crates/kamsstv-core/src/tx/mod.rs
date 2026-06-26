//! Transmit path — generation-side building blocks.
//!
//! The NCO only ever *generates*; the RX path demodulates via IQ downconversion
//! and never touches it. Hence the oscillator is TX machinery, not a shared root
//! primitive.

pub mod nco;
