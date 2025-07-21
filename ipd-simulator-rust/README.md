# IPD Simulator - High Performance Rust Implementation

A high-performance implementation of the Iterated Prisoner's Dilemma (IPD) multi-agent simulation, written in Rust.

## Building and Running

### Prerequisites

- Rust 1.70 or later
- FFmpeg libraries (for video encoding)

### Compilation

```bash
# For the best performance, build in release mode
cargo build --release
```

### Usage

```bash
# Run with default settings (100x100 grid, 1000 timesteps)
./target/release/ipd_simulator

# Run a larger simulation and disable video output for speed
./target/release/ipd_simulator --width 1000 --height 1000 --timesteps 1000 --no-video

# See all available options
./target/release/ipd_simulator --help
```

