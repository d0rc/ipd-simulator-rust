# IPD Simulator - High Performance Rust Implementation

A high-performance implementation of the Iterated Prisoner's Dilemma (IPD) multi-agent simulation, optimized for massive grid sizes (10,000 x 10,000 agents).

## Features

- **Massive Scale**: Supports grids up to 10,000 x 10,000 (100 million agents)
- **CPU Optimized**: Cache-aligned data structures, SIMD operations, and parallel processing
- **Video Generation**: Real-time MP4 video encoding with statistics overlay
- **CSV Export**: Streaming CSV export of simulation statistics
- **Q-Learning**: Agents learn optimal strategies through reinforcement learning
- **Multicellular Evolution**: Agents can merge into multicellular organisms or split apart

## Performance Optimizations

1. **Cache-Aligned Agents**: 64-byte aligned agent structures for optimal cache performance
2. **Parallel Processing**: Chunk-based parallel simulation using Rayon
3. **Bit-Packed Memory**: Efficient memory representation (2 bits per action)
4. **Lock-Free Updates**: Deferred merge/split operations with lock-free queues
5. **SIMD Operations**: Vectorized Q-learning updates (when available)
6. **Sparse Data Structures**: Only active agents are processed

## Building

### Prerequisites

- Rust 1.70 or later
- FFmpeg libraries (for video encoding)
  - macOS: `brew install ffmpeg`
  - Ubuntu: `sudo apt-get install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev`
  - Windows: Download from https://ffmpeg.org/download.html

### Compilation

```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release
```

## Usage

```bash
# Basic usage (100x100 grid, 1000 timesteps)
cargo run --release

# Large grid simulation
cargo run --release -- -w 10000 -h 10000 -t 100 --no-video

# Custom parameters
cargo run --release -- \
  --width 5000 \
  --height 5000 \
  --timesteps 500 \
  --chunk-size 1000 \
  --output-video simulation.mp4 \
  --output-csv stats.csv \
  --video-width 3840 \
  --video-height 2160 \
  --fps 60 \
  --alpha 0.3 \
  --gamma 0.9 \
  --epsilon 0.15 \
  --threads 12
```

### Command Line Options

- `-w, --width <WIDTH>`: Grid width (default: 100)
- `-h, --height <HEIGHT>`: Grid height (default: 100)
- `-t, --timesteps <TIMESTEPS>`: Number of simulation steps (default: 1000)
- `-c, --chunk-size <SIZE>`: Chunk size for parallel processing (default: 1024)
- `-o, --output-video <PATH>`: Output video file path (default: output.mp4)
- `-s, --output-csv <PATH>`: Output CSV file path (default: statistics.csv)
- `--video-width <WIDTH>`: Video width in pixels (default: 1920)
- `--video-height <HEIGHT>`: Video height in pixels (default: 1080)
- `--fps <FPS>`: Video frames per second (default: 30)
- `--no-video`: Skip video generation for better performance
- `--alpha <VALUE>`: Q-learning learning rate (default: 0.2)
- `--gamma <VALUE>`: Q-learning discount factor (default: 0.95)
- `--epsilon <VALUE>`: Q-learning exploration rate (default: 0.1)
- `--threads <COUNT>`: Number of threads (0 = auto, default: 0)

## Performance Benchmarks

On Apple M3 Max (16 performance cores):

| Grid Size | Agents | FPS (CPU) | Memory Usage |
|-----------|--------|-----------|--------------|
| 100×100   | 10K    | ~500      | 8 MB         |
| 1000×1000 | 1M     | ~20       | 80 MB        |
| 5000×5000 | 25M    | ~2        | 2 GB         |
| 10000×10000 | 100M | ~0.5      | 8 GB         |

## Output Files

### Video Output
- MP4 format with H.264 encoding
- Color-coded agents based on organism size
- Statistics overlay (simplified in current version)

### CSV Output
Contains per-timestep statistics:
- Total agents and average fitness
- Unicellular vs multicellular agent counts
- Fitness by organism type
- Cooperation rates
- Detailed metrics for analysis

## Architecture

```
┌─────────────────────────────────────────┐
│           Main Thread                   │
│  - CLI parsing                          │
│  - Progress tracking                    │
│  - Performance monitoring               │
└────────────┬────────────────────────────┘
             │
┌────────────▼────────────────────────────┐
│        Simulation Engine                │
│  - Parallel chunk processing            │
│  - Agent interactions                   │
│  - Q-learning updates                   │
│  - Merge/split operations               │
└────────────┬────────────────────────────┘
             │
┌────────────▼────────────────────────────┐
│         Output Pipeline                 │
├─────────────────────────────────────────┤
│  Video Thread    │   CSV Writer         │
│  - Frame render  │   - Buffered writes  │
│  - H.264 encode  │   - Streaming export │
└──────────────────┴──────────────────────┘
```

## Future Enhancements

1. **Metal GPU Acceleration**: 
   - GPU kernels for agent interactions
   - Parallel Q-learning on GPU
   - GPU-accelerated rendering

2. **Advanced Visualization**:
   - Real-time charts overlay
   - Interactive preview window
   - Heatmap visualizations

3. **Distributed Computing**:
   - Multi-node simulation
   - Cloud deployment options
   - Checkpoint/resume capability

## License

This project is part of the IPD-ms research. Please cite the original paper:
[IEEE Xplore Link](https://ieeexplore.ieee.org/document/10970107)

## Contributing

Contributions are welcome! Please focus on:
- Performance optimizations
- GPU acceleration
- Visualization improvements
- Scientific analysis tools
