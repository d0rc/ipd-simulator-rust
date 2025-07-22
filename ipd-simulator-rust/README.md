# IPD Simulator - High-Performance Rust Implementation

This project is a high-performance implementation of the Iterated Prisoner's Dilemma (IPD) multi-agent simulation, written in Rust. It is designed to study emergent multicellularity and evolutionary dynamics at scales previously not possible with the original JavaScript implementation.

## Features

- **High Performance**: Achieves massive speedups (e.g., 500+ FPS on smaller grids) through multi-threading, cache-optimization, and other advanced techniques.
- **Large-Scale Simulations**: Capable of simulating millions of agents on large grids (e.g., 1000x1000).
- **Parallel Processing**: Utilizes all available CPU cores for near-linear performance scaling.
- **Video Generation**: Asynchronously generates MP4 videos of the simulation, visualizing organism evolution.
- **Data Export**: Exports comprehensive statistics to CSV files for analysis.
- **Configurable**: A command-line interface allows for easy configuration of simulation parameters.

## Performance

The Rust implementation is significantly faster than the original JavaScript version, enabling much larger and more complex simulations. For detailed benchmarks and a technical breakdown of the optimizations, please see [performance_comparison.md](performance_comparison.md).

| Test Case         | Dimensions      | Timesteps | Avg. FPS |
| ----------------- | --------------- | --------- | -------- |
| **Small Grid**    | 50x50           | 1,000     | 509.36   |
| **Medium Grid**   | 200x200         | 1,000     | 54.88    |
| **Large Grid**    | 500x500         | 500       | 8.13     |
| **Very Large Grid** | 1000x1000       | 200       | 1.87     |

For a full summary of the implementation and its features, see [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md).

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
