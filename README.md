# kamsstv

An SSTV (slow-scan television) receiver and transmitter in Rust.
Development of this software is primarly motivated by lack of cross-platform (mainly Linux/Windows) SSTV software as well as personal coding excercise.

## Status

Early.

## Design

kamsstv is built as ports-and-adapters around a pure, I/O-free core. Because
dataflow reverses between TX and RX, audio I/O and the GUI are both edges around
the codec — neither sits "above" or "below" it. A `Conductor` orchestrates state,
mode, and PTT; lock-free rings carry samples between the real-time audio callbacks
and the DSP loop.

See **[docs/design.md](docs/design.md)** for the full design: the mode table,
concurrency model, transmit and receive signal paths, the half-duplex typed
invariant, the testing strategy, and known limitations.
