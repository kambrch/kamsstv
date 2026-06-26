//! RGB ↔ YCrCb colour transform. Shared: TX encodes pixels to tone levels,
//! RX decodes tone levels back to pixels.
//!
//! TODO(you): implement `rgb_to_ycrcb` / `ycrcb_to_rgb` under TDD. The
//! invariant worth a property test: round-tripping should be near-identity
//! within a bounded rounding error.
