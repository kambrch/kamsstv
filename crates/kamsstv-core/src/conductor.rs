//! Pure half-duplex state machine. This is the *logic* half of the conductor;
//! the driver that owns the rings, spawns threads, and calls `Ptt::key`/`unkey`
//! lives in `kamsstv-app`. Keeping the transition logic here makes the
//! half-duplex invariant — "cannot receive while keyed" — testable with no
//! rings, threads, or hardware.
//!
//! TODO(you): implement the legal transitions under TDD. The invariant to pin:
//! no path constructs `Transmitting` while an RX session is consuming, and
//! vice versa.

/// Progress state of an in-flight receive. Owns pure decode progress, NOT rings.
pub struct RxSession;

/// Progress state of an in-flight transmit. Implies keyed. Owns no rings.
pub struct TxSession;

/// The half-duplex invariant made unrepresentable: at most one direction is
/// ever active, so "receiving while keyed" cannot be constructed.
pub enum Conductor {
    Idle,
    Receiving(RxSession),
    Transmitting(TxSession),
}
