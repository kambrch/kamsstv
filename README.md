# kamsstv

An SSTV (slow-scan television) receiver and transmitter in Rust.
Development of this software is primarly motivated by lack of cross-platform (mainly Linux/Windows) SSTV software as well as personal coding excercise.

## Status

Early.

## Roadmap

Mirrors the architecture in [docs/design.md](docs/design.md). Checked items are
implemented and tested; the rest are planned.

### Infrastructure

- [x] Three-crate workspace (`core` / `io` / `app`), compiler-enforced core purity
- [ ] File-loopback test harness (encode → WAV → decode → image-diff)

### Transmit path

- [x] NCO — phase-wrapped oscillator (property + mutation tested)
- [ ] Timing/segment layer (Duration → samples, Bresenham fractional carry)
- [ ] `ModeSpec` table (Martin M1, Robot 36)
- [ ] Colour transform (RGB ↔ YCrCb)
- [ ] Modulator (`Iterator<Item = f32>`)
- [ ] WAV writer
- [ ] PTT backends (`Noop`/`Vox` first; serial/rigctl/gpio later)

### Receive path *(primary open risk — see design.md)*

- [ ] AGC / normalise
- [ ] IQ downconvert → instantaneous frequency
- [ ] Sync detector + clamped slant (ε) estimator
- [ ] Pixel sampler
- [ ] Image assembler

### Orchestration

- [ ] `Conductor` state machine (pure, in `core`)
- [ ] `ConductorDriver` (rings / threads / PTT, in `app`)
- [ ] Audio I/O (`cpal`)
- [ ] GUI

## Design

kamsstv is built as ports-and-adapters around a pure, I/O-free core. Because
dataflow reverses between TX and RX, audio I/O and the GUI are both edges around
the codec — neither sits "above" or "below" it. A `Conductor` orchestrates state,
mode, and PTT; lock-free rings carry samples between the real-time audio callbacks
and the DSP loop.

See **[docs/design.md](docs/design.md)** for the full design: the mode table,
concurrency model, transmit and receive signal paths, the half-duplex typed
invariant, the testing strategy, and known limitations.
