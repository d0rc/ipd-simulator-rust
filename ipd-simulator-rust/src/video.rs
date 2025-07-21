use crate::grid::{Grid, Statistics};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use std::thread;

#[cfg(feature = "video")]
use crossbeam::channel::{bounded, Sender, Receiver};

#[cfg(feature = "video")]
struct FrameData {
    frame_number: usize,
    width: u32,
    height: u32,
    data: Arc<Vec<u8>>,
}

pub struct VideoEncoder {
    width: u32,
    height: u32,
    fps: u32,
    frames_dir: PathBuf,
    frame_count: usize,
    #[cfg(feature = "video")]
    frame_sender: Option<Sender<FrameData>>,
    #[cfg(feature = "video")]
    encoder_threads: Vec<thread::JoinHandle<()>>,
}

impl VideoEncoder {
    pub fn new(output_path: &Path, width: u32, height: u32, fps: u32) -> Result<Self, Box<dyn std::error::Error>> {
        // Create frames directory
        let frames_dir = output_path.parent()
            .unwrap_or(Path::new("."))
            .join("frames");
        
        fs::create_dir_all(&frames_dir)?;
        
        #[cfg(feature = "video")]
        {
            // Create bounded channel with capacity for 10 frames
            let (sender, receiver) = bounded::<FrameData>(10);
            
            // Determine number of encoder threads (use half of available CPUs)
            let num_threads = std::cmp::max(1, num_cpus::get() - 1);
            let mut encoder_threads = Vec::with_capacity(num_threads);
            
            // Spawn encoder threads
            for _ in 0..num_threads {
                let rx = receiver.clone();
                let frames_dir = frames_dir.clone();
                
                let handle = thread::spawn(move || {
                    Self::encoder_worker(rx, frames_dir);
                });
                
                encoder_threads.push(handle);
            }
            
            Ok(Self {
                width,
                height,
                fps,
                frames_dir,
                frame_count: 0,
                frame_sender: Some(sender),
                encoder_threads,
            })
        }
        
        #[cfg(not(feature = "video"))]
        {
            Ok(Self {
                width,
                height,
                fps,
                frames_dir,
                frame_count: 0,
            })
        }
    }
    
    #[cfg(feature = "video")]
    fn encoder_worker(receiver: Receiver<FrameData>, frames_dir: PathBuf) {
        while let Ok(frame_data) = receiver.recv() {
            let filename = frames_dir.join(format!("frame_{:06}.bmp", frame_data.frame_number));
            // Write BMP file directly - much faster than PNG
            let _ = Self::write_bmp(
                &filename,
                frame_data.width,
                frame_data.height,
                &frame_data.data
            );
        }
    }
    
    #[cfg(feature = "video")]
    fn write_bmp(path: &Path, width: u32, height: u32, data: &[u8]) -> std::io::Result<()> {
        // BMP rows are padded to a multiple of 4 bytes
        let bytes_per_row = width * 3;
        let padding_size = (4 - (bytes_per_row % 4)) % 4;
        let padded_row_size = bytes_per_row + padding_size;
        
        let image_size = padded_row_size * height;
        let file_size = 54 + image_size;
        
        // Pre-allocate buffer for the entire BMP file
        let mut bmp_data = Vec::with_capacity(file_size as usize);
        
        // BMP Header (14 bytes)
        bmp_data.extend_from_slice(b"BM");
        bmp_data.extend_from_slice(&file_size.to_le_bytes());
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Reserved
        bmp_data.extend_from_slice(&54u32.to_le_bytes()); // Data offset
        
        // DIB Header (40 bytes)
        bmp_data.extend_from_slice(&40u32.to_le_bytes()); // Header size
        bmp_data.extend_from_slice(&(width as i32).to_le_bytes()); // Width
        bmp_data.extend_from_slice(&(height as i32).to_le_bytes()); // Height (positive for bottom-up)
        bmp_data.extend_from_slice(&1u16.to_le_bytes()); // Planes
        bmp_data.extend_from_slice(&24u16.to_le_bytes()); // Bits per pixel
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Compression (none)
        bmp_data.extend_from_slice(&image_size.to_le_bytes());
        bmp_data.extend_from_slice(&2835u32.to_le_bytes()); // X pixels per meter (72 DPI)
        bmp_data.extend_from_slice(&2835u32.to_le_bytes()); // Y pixels per meter (72 DPI)
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Colors used
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Important colors
        
        // Write pixel data (bottom-up)
        let padding_bytes = vec![0u8; padding_size as usize];
        for y in (0..height).rev() { // Iterate rows in reverse for bottom-up BMP
            let row_start = (y * bytes_per_row) as usize;
            
            // Convert RGB to BGR for the entire row
            for x in 0..width {
                let idx = row_start + (x * 3) as usize;
                bmp_data.push(data[idx + 2]); // B
                bmp_data.push(data[idx + 1]); // G
                bmp_data.push(data[idx]);     // R
            }
            
            // Add padding
            bmp_data.extend_from_slice(&padding_bytes);
        }
        
        // Write the entire BMP data in a single system call
        std::fs::write(path, &bmp_data)?;
        
        Ok(())
    }
    
    pub fn add_frame(&mut self, grid: &Grid, stats: &Statistics, _timestep: usize) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "video")]
        {
            let frame_data = self.render_frame(grid, stats)?;
            
            // Send frame to encoding queue
            if let Some(sender) = &self.frame_sender {
                let frame = FrameData {
                    frame_number: self.frame_count,
                    width: self.width,
                    height: self.height,
                    data: Arc::new(frame_data),
                };
                
                // Try to send, but don't block if queue is full
                match sender.try_send(frame) {
                    Ok(_) => {},
                    Err(crossbeam::channel::TrySendError::Full(_)) => {
                        // Queue is full, wait for space
                        // This ensures we don't drop frames
                        if let Some(sender) = &self.frame_sender {
                            let frame = FrameData {
                                frame_number: self.frame_count,
                                width: self.width,
                                height: self.height,
                                data: Arc::new(self.render_frame(grid, stats)?),
                            };
                            sender.send(frame)?;
                        }
                    },
                    Err(e) => return Err(Box::new(e)),
                }
            }
            
            self.frame_count += 1;
        }
        
        #[cfg(not(feature = "video"))]
        {
            let _ = (grid, stats, _timestep); // Suppress unused warnings
        }
        
        Ok(())
    }
    
    fn render_frame(&self, grid: &Grid, stats: &Statistics) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut frame = vec![0u8; (self.width * self.height * 3) as usize];
        
        // Calculate scale factor
        let scale_x = self.width as f32 / grid.grid_width as f32;
        let scale_y = self.height as f32 / grid.grid_height as f32;
        
        // Render grid
        for y in 0..grid.grid_height {
            for x in 0..grid.grid_width {
                let idx = y * grid.grid_width + x;
                let agent_idx = grid.find_root(idx);
                
                if agent_idx >= grid.agents.len() {
                    continue;
                }
                
                let agent = &grid.agents[agent_idx];
                let color = self.get_agent_color(agent.get_organism_size());
                
                // Calculate pixel coordinates
                let px_start = (x as f32 * scale_x) as u32;
                let px_end = ((x + 1) as f32 * scale_x) as u32;
                let py_start = (y as f32 * scale_y) as u32;
                let py_end = ((y + 1) as f32 * scale_y) as u32;
                
                // Fill pixels
                for py in py_start..py_end.min(self.height) {
                    for px in px_start..px_end.min(self.width) {
                        let pixel_idx = ((py * self.width + px) * 3) as usize;
                        frame[pixel_idx] = color.0;
                        frame[pixel_idx + 1] = color.1;
                        frame[pixel_idx + 2] = color.2;
                    }
                }
            }
        }
        
        // Overlay statistics text
        self.overlay_stats(&mut frame, stats);
        
        Ok(frame)
    }
    
    fn get_agent_color(&self, organism_size: u32) -> (u8, u8, u8) {
        // Color based on organism size (similar to original spectral colors)
        match organism_size {
            1 => (158, 1, 66),      // Dark red for single cells
            2 => (213, 62, 79),     // Red
            3 => (244, 109, 67),    // Orange
            4 => (253, 174, 97),    // Light orange
            5 => (254, 224, 139),   // Yellow
            6 => (230, 245, 152),   // Light yellow
            7 => (171, 221, 164),   // Light green
            8 => (102, 194, 165),   // Green
            9 => (50, 136, 189),    // Blue
            10 => (94, 79, 162),    // Purple
            _ => (140, 81, 255),    // Violet for large organisms
        }
    }
    
    fn overlay_stats(&self, frame: &mut [u8], stats: &Statistics) {
        // Add a semi-transparent dark bar at the top for stats
        let bar_height = 40;
        let bar_color = (30, 30, 30); // Dark gray
        
        // Fill top bar with semi-transparency effect
        for y in 0..bar_height.min(self.height) {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 3) as usize;
                // Blend with existing pixels for semi-transparency
                let alpha = 0.8;
                frame[idx] = (frame[idx] as f32 * (1.0 - alpha) + bar_color.0 as f32 * alpha) as u8;
                frame[idx + 1] = (frame[idx + 1] as f32 * (1.0 - alpha) + bar_color.1 as f32 * alpha) as u8;
                frame[idx + 2] = (frame[idx + 2] as f32 * (1.0 - alpha) + bar_color.2 as f32 * alpha) as u8;
            }
        }
        
        // Draw simple stat indicators using colored blocks
        // This is a simplified visualization - in production you'd use a text rendering library
        let block_size = 10;
        let margin = 10;
        let y_pos = 15;
        
        // Draw agent count indicator (green block)
        let agent_color = (0, 255, 0);
        let agent_width = ((stats.total_agents as f32 / 2500.0) * 100.0) as u32;
        self.draw_block(frame, margin, y_pos, agent_width.min(100), block_size, agent_color);
        
        // Draw fitness indicator (blue block)
        let fitness_color = (0, 100, 255);
        let avg_fitness = if stats.total_agents > 0 {
            stats.total_fitness as f64 / stats.total_agents as f64
        } else {
            0.0
        };
        let fitness_width = ((avg_fitness / 20000.0) * 100.0) as u32;
        self.draw_block(frame, margin + 120, y_pos, fitness_width.min(100), block_size, fitness_color);
        
        // Draw multi-cell indicator (purple block)
        let multi_color = (255, 0, 255);
        let multi_width = if stats.total_agents > 0 {
            ((stats.multicellular_agents as f32 / stats.total_agents as f32) * 100.0) as u32
        } else {
            0
        };
        self.draw_block(frame, margin + 240, y_pos, multi_width.min(100), block_size, multi_color);
    }
    
    fn draw_block(&self, frame: &mut [u8], x: u32, y: u32, width: u32, height: u32, color: (u8, u8, u8)) {
        for py in y..((y + height).min(self.height)) {
            for px in x..((x + width).min(self.width)) {
                let idx = ((py * self.width + px) * 3) as usize;
                if idx + 2 < frame.len() {
                    frame[idx] = color.0;
                    frame[idx + 1] = color.1;
                    frame[idx + 2] = color.2;
                }
            }
        }
    }
    
    pub fn finish(mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "video")]
        {
            // Close the channel to signal encoder threads to finish
            self.frame_sender.take();
            
            // Wait for all encoder threads to complete
            for thread in self.encoder_threads {
                thread.join().map_err(|_| "Encoder thread panicked")?;
            }
            
            // Generate ffmpeg command for the user
            println!("\nVideo frames saved to: {}", self.frames_dir.display());
            println!("\nTo create video, run:");
            println!("ffmpeg -r {} -i {}/frame_%06d.bmp -c:v libx264 -preset medium -crf 23 -pix_fmt yuv420p output.mp4", 
                self.fps, self.frames_dir.display());
        }
        
        Ok(())
    }
}
