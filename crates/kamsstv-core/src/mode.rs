//! `ModeSpec`: the per-mode timing + colorspace table. Shared by TX and RX —
//! a mode's timing is identical whether you are encoding or decoding it.
//!
//! TODO(you): populate the supported-mode table (Martin M1, Robot 36, ...) and
//! its constructors under TDD when the mode layer is built.

use std::time::Duration;

/// Colour model a mode transmits in.
pub enum ColorSpace {
    Rgb,
    YCrCb,
}

/// Order channels are sent per line.
pub enum ChannelOrder {
    /// R, G, B — Scottie / Martin.
    Rgb,
    /// Y, Cr, Cb — Robot / PD.
    YCrCb,
}

/// Per-mode timing and colour description. Most of the pure core's surface area.
pub struct ModeSpec {
    pub vis_code: u8,
    pub width: u16,
    pub height: u16,
    pub color: ColorSpace,
    pub channel_order: ChannelOrder,
    pub pixel_time: Duration,
    pub sync_pulse: Duration,
    pub sync_porch: Duration,
}
