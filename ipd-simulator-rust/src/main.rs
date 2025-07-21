mod agent;
mod grid;
mod video;
mod csv_export;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use tracing_subscriber;

use crate::csv_export::BufferedCsvExporter;
use crate::grid::Grid;
#[cfg(feature = "video")]
use crate::video::VideoEncoder;
#[cfg(not(feature = "video"))]
use crate::video::VideoEncoder;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Grid width
    #[arg(short = 'w', long, default_value_t = 100)]
    width: usize,
    
    /// Grid height
    #[arg(short = 'h', long, default_value_t = 100)]
    height: usize,
    
    /// Number of timesteps to simulate
    #[arg(short = 't', long, default_value_t = 1000)]
    timesteps: usize,
    
    /// Output video file path
    #[arg(short = 'o', long, default_value = "output.mp4")]
    output_video: PathBuf,
    
    /// Output CSV file path
    #[arg(short = 's', long, default_value = "statistics.csv")]
    output_csv: PathBuf,
    
    /// Video width in pixels
    #[arg(long, default_value_t = 1920)]
    video_width: u32,
    
    /// Video height in pixels
    #[arg(long, default_value_t = 1080)]
    video_height: u32,
    
    /// Video frames per second
    #[arg(long, default_value_t = 30)]
    fps: u32,
    
    /// Skip video generation
    #[arg(long)]
    no_video: bool,
    
    /// Q-learning alpha parameter
    #[arg(long, default_value_t = 0.2)]
    alpha: f32,
    
    /// Q-learning gamma parameter
    #[arg(long, default_value_t = 0.95)]
    gamma: f32,
    
    /// Q-learning epsilon parameter
    #[arg(long, default_value_t = 0.1)]
    epsilon: f32,
    
    /// Number of threads (0 = auto)
    #[arg(long, default_value_t = 0)]
    threads: usize,

    /// Print pass statistics for each timestep
    #[arg(long)]
    print_pass_stats: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Set thread pool size
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()?;
    }
    
    info!("IPD Simulator - High Performance Edition");
    info!("Grid size: {}x{} ({} agents)", args.width, args.height, args.width * args.height);
    info!("Timesteps: {}", args.timesteps);
    
    // Initialize grid
    let mut grid = Grid::new(args.width, args.height);
    grid.alpha = args.alpha;
    grid.gamma = args.gamma;
    grid.epsilon = args.epsilon;
    
    // Initialize video encoder
    let mut video_encoder = if !args.no_video {
        match VideoEncoder::new(
            &args.output_video,
            args.video_width,
            args.video_height,
            args.fps,
        ) {
            Ok(encoder) => Some(encoder),
            Err(e) => {
                warn!("Failed to initialize video encoder: {}. Continuing without video.", e);
                None
            }
        }
    } else {
        None
    };
    
    // Initialize CSV exporter
    let mut csv_exporter = BufferedCsvExporter::new(&args.output_csv, 100);
    
    // Progress bar
    let progress = ProgressBar::new(args.timesteps as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );
    
    // Performance tracking
    let mut total_sim_time = Duration::ZERO;
    let mut total_stats_time = Duration::ZERO;
    let mut total_export_time = Duration::ZERO;
    
    // Main simulation loop
    for timestep in 0..args.timesteps {
        // Simulation step
        let sim_start = Instant::now();
        grid.step();
        total_sim_time += sim_start.elapsed();
        
        // Calculate statistics
        let stats_start = Instant::now();
        let mut stats = grid.get_statistics();
        stats.pass_stats = grid.pass_stats.clone();
        total_stats_time += stats_start.elapsed();
        
        // Export data
        let export_start = Instant::now();
        
        // Add frame to video
        if let Some(encoder) = &mut video_encoder {
            encoder.add_frame(&grid, &stats, timestep)?;
        }
        
        // Write statistics to CSV
        csv_exporter.add_stats(timestep, stats.clone())?;
        
        total_export_time += export_start.elapsed();
        
        // Update progress
        progress.set_position(timestep as u64 + 1);
        progress.set_message(format!(
            "Agents: {} | Avg Fitness: {:.2} | Multi: {}",
            stats.total_agents,
            stats.avg_fitness(),
            stats.multicellular_agents
        ));
        
        // Log periodic updates
        if timestep % 100 == 0 && timestep > 0 {
            let elapsed = total_sim_time + total_stats_time + total_export_time;
            let fps = timestep as f64 / elapsed.as_secs_f64();
            info!(
                "Timestep {} | FPS: {:.2} | Sim: {:.2}s | Stats: {:.2}s | Export: {:.2}s",
                timestep,
                fps,
                total_sim_time.as_secs_f64(),
                total_stats_time.as_secs_f64(),
                total_export_time.as_secs_f64()
            );
            info!(
                "Pass Stats: Interactions: {} | Updates: {} | Cache: {}us | Gen: {}us | Proc: {}us | Update: {}us | Deferred: {}us",
                stats.pass_stats.num_interactions,
                stats.pass_stats.num_updates,
                stats.pass_stats.cache_update_time,
                stats.pass_stats.interaction_generation_time,
                stats.pass_stats.interaction_processing_time,
                stats.pass_stats.state_update_time,
                stats.pass_stats.deferred_op_time
            );
        }

        if args.print_pass_stats {
            println!(
                "{},{},{},{},{},{},{},{}",
                timestep,
                stats.pass_stats.num_interactions,
                stats.pass_stats.num_updates,
                stats.pass_stats.cache_update_time,
                stats.pass_stats.interaction_generation_time,
                stats.pass_stats.interaction_processing_time,
                stats.pass_stats.state_update_time,
                stats.pass_stats.deferred_op_time
            );
        }
    }
    
    progress.finish_with_message("Simulation complete!");
    
    // Finalize outputs
    if let Some(encoder) = video_encoder {
        info!("Finalizing video...");
        encoder.finish()?;
    }
    
    csv_exporter.finish()?;
    
    // Print performance summary
    let total_time = total_sim_time + total_stats_time + total_export_time;
    println!("\n=== Performance Summary ===");
    println!("Total time: {:.2}s", total_time.as_secs_f64());
    println!("Simulation: {:.2}s ({:.1}%)", 
        total_sim_time.as_secs_f64(),
        (total_sim_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!("Statistics: {:.2}s ({:.1}%)", 
        total_stats_time.as_secs_f64(),
        (total_stats_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!("Export: {:.2}s ({:.1}%)", 
        total_export_time.as_secs_f64(),
        (total_export_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!("Average FPS: {:.2}", args.timesteps as f64 / total_time.as_secs_f64());
    println!("Agents processed: {}", args.width * args.height);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_small_grid() {
        let mut grid = Grid::new(10, 10);
        
        // Run a few steps
        for _ in 0..10 {
            grid.step();
            let stats = grid.get_statistics();
            assert!(stats.total_agents > 0);
        }
    }
    
    #[test]
    fn test_large_grid_creation() {
        let grid = Grid::new(1000, 1000);
        assert_eq!(grid.agents.len(), 1_000_000);
    }
}
