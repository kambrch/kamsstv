# kamsstv design

This document describes the architecture and design rationale of kamsstv. For a
project overview and status, see the [README](../README.md).

## Design overview

kamsstv is designed as ports-and-adapters around a pure core. Because dataflow reverses between TX and RX, audio I/O and the GUI are both edges around the codec — neither sits "above" or "below" it.

```text
            ┌─────────────────────────────┐
   GUI ───▶ │   Conductor (orchestration) │ ◀─── audio I/O
            │  state machine · mode · PTT  │
            └──────────────┬──────────────┘
                           │
                    ┌──────▼──────┐
                    │  Pure core  │   (image, ModeSpec, SampleRate) → samples   [TX]
                    │  I/O-free   │   push state machine                         [RX]
                    └─────────────┘
```

### Modes are a table, not a scalar

A "mode" carries per-line timing and colorspace, and that table is most of the core's surface area. The core is pure, but it is not trivially simple: it's this spec, plus a colour transform, plus the NCO.

```rust
struct ModeSpec {
    vis_code: u8,
    width: u16,
    height: u16,
    color: ColorSpace,            // Rgb | YCrCb
    channel_order: ChannelOrder,  // R,G,B (Scottie/Martin) vs Y,Cr,Cb (Robot/PD)
    pixel_time: Duration,
    sync_pulse: Duration,
    sync_porch: Duration,
    // Tone freqs (black 1500 / white 2300 / sync 1200 Hz) are consts across
    // supported modes; promote into ModeSpec only if a future mode breaks that.
}
```

## Concurrency model

Three clocks meet — the real-time audio callback(s), the DSP loop, and the 60 fps UI.

| Seam | Channel | Why |
|---|---|---|
| audio ↔ DSP | `rtrb` (SPSC, lock-free) | the callback must never block or allocate |
| DSP ↔ UI | `crossbeam` (MPMC) | nothing here has a hard deadline |

- Unidirectional channels, so two of each: separate RX and TX rings, separate event and command channels. A shared bidirectional queue would reintroduce contention on the one seam that must stay lock-free.
- No async runtime: long-lived threads + lock-free queues is what threads do best. `tokio` buys nothing for a sample loop and a real-time callback you can't `.await` in — it only imposes function colouring.

| Seam | Failure | Response | Mitigation |
|---|---|---|---|
| RX input (driver → ring) | ring full → overrun | drop oldest | size ring for burst tolerance |
| TX output (ring → driver) | ring empty → underrun | emit zeros | prime ring before keying; size for worst-case jitter |

You cannot drop on TX: the driver asks for N samples and plays whatever you return. An empty ring mid-image is an audible gap and corrupted lines.

- TX priming: fill the TX ring to a watermark before `key()`. Never start keyed on an empty buffer. Size the watermark and ring for the loosest host you target — that's WASAPI on Windows, not PipeWire.
- Underrun counter: a mid-stream underrun is a defect and must be observable; a tail underrun is benign (the silence pad covers it). Surface the count to the conductor.

### The DSP thread has a throughput floor, not a deadline

The hard deadlines live only in the audio callbacks — that is the whole reason the lock-free seam matters. The DSP loop's sole obligation is *average* throughput: consume the input ring at least as fast as it fills. SSTV occupies ~3 kHz; the demod is cheap and clears any modern core by orders of magnitude.

---

## Transmit path

- Single phase accumulator (NCO): continuous phase, never per-pixel tone bursts — phase discontinuities splatter energy outside the SSB passband.
- Modulator is an `Iterator`: it produces at the consumer's pace; the RX side accepts whatever arrives. The TX/RX shapes differ on purpose — don't force symmetry.
- PTT is guarded on both ends: `key()` precedes the first sample by `TX_SETTLE`; `unkey()` follows modulator-`None` by the buffer tail (ring drain + device latency). A trailing silence pad turns that hard deadline into a soft one.

### PTT is a config-time `enum`

Keying fires ~twice per ~90-second image, so dispatch cost is irrelevant — the real question is whether the rig interface is chosen at runtime. A closed enum gives a settings-menu choice with no vtable, no allocation, and exhaustiveness checking:

```rust
enum Ptt {
    Noop,              // tests + waveform-only work
    Vox,               // keyed by audio presence; control is a real no-op
    Serial(SerialPtt), // DTR/RTS
    Rigctl(RigctlPtt), // hamlib
    Gpio(GpioPtt),
}
impl Ptt {
    fn key(&mut self) -> Result<(), PttError>;
    fn unkey(&mut self) -> Result<(), PttError>;
}
```

`Ptt::Noop` makes the entire TX path testable with no hardware: the same orchestration runs, the keying calls are real no-ops, and tests assert on the WAV that drops out.

---

## Receive path

```text
capture → AGC/normalize → IQ downconvert (mix + LPF) → instantaneous frequency
   → { sync detector → line clock + ε estimator (clamped),  pixel sampler }
   → image assembler
```

- Demodulation: IQ downconversion → instantaneous frequency: mix the input down by the band centre (~1900 Hz), lowpass to isolate the analytic signal, then recover `f_inst = (fs / 2π) · d/dt arg(z)` and map 1500 Hz → black, 2300 Hz → white.
  - Why IQ and not FFT-Hilbert: the IQ path runs sample-by-sample, preserving the RX push state machine. A block-based FFT/Hilbert would silently re-impose block semantics the RX side deliberately rejects. Per-window Goertzel (cheap, robust) and a PLL (best at low SNR, more state) are the documented fallbacks.
- AGC ahead of sync detection: instantaneous-frequency demod is amplitude-blind, but sync *thresholding* is not, and real input swings tens of dB. A peak-tracking normaliser (fast attack, slow decay) sits between capture and sync detection.
- Slant correction is first-class — and clamped: sound-card clock ppm mismatch shears the image; ε is estimated from sync-pulse spacing and the stream is resampled. But ε is derived from the very pulses that fading degrades first, so the estimator can diverge as SNR drops. ε is therefore clamped to a physically plausible range (configurable, default ≈ ±1000 ppm). The clamp rejects divergence — percent-level runaway when the estimator locks onto noise — rather than modelling the clock; a clamped bad estimate leaves a recoverable, slightly-sheared image instead of shredding it.
- Manual mode override always available: VIS is the first thing to die on a weak signal, so auto-detect is never the only path.

---

## Half-duplex is a typed invariant

SSTV is half-duplex, and with two independent rings nothing structurally forbids "both live" unless the type system says so. The conductor makes RX/TX mutual exclusion unrepresentable:

```rust
enum Conductor {
    Idle,
    Receiving(RxSession),     // TX ring not consumed here
    Transmitting(TxSession),  // implies keyed; RX ring not consumed here
}
```

The rings still exist independently; consumption is gated by the active variant, so "receiving while keyed" can't be constructed.

---

## Testing

- File loopback is both the dev harness and the regression suite: encode → WAV → decode → compare, with injected clock error, AWGN, and buffer latency. Head-clip surfaces as VIS-decode failure; tail-clip as garbled bottom lines — one image-diff catches keying and demod regressions alike.
- Low-SNR sweep on the sync → ε path: sweep SNR downward and assert ε stays inside its clamp and doesn't diverge once sync detection starts missing pulses. This guards the two features that fail first and together: slant correction and auto-mode.
- TX mid-stream underrun regression: starve the TX ring mid-stream; assert the underrun counter fires and the gap stays localized rather than failing the whole image.
- Measure device latency, don't hardcode it: `cpal` reports the stream config and timestamps; the right value differs across a PipeWire box and a WASAPI host.

---

## Known limitations

- The RX demodulator + low-SNR sync recovery is the primary open risk: IQ-demod + peak AGC is the first bet; the PLL fallback is held in reserve for real HF recordings where sync detection fails before the image is otherwise unrecoverable. The low-SNR sweep is the test that will tell you which.
- ε estimation and sync detection share a failure source (fading sync pulses), so the clamp is a backstop, not a cure.

---

## Non-goals

- No async runtime, by design.
- Not full-duplex.
- Auto-detection never replaces manual mode selection.
