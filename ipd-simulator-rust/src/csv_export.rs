use crate::grid::Statistics;
use csv::Writer;
use std::path::{Path, PathBuf};
use std::error::Error;

/// Buffered CSV writer for high-performance streaming
pub struct BufferedCsvExporter {
    path: PathBuf,
    buffer: Vec<StatsRecord>,
    buffer_size: usize,
}

#[derive(Debug, Clone)]
struct StatsRecord {
    timestep: usize,
    stats: Statistics,
}

impl BufferedCsvExporter {
    pub fn new(path: &Path, buffer_size: usize) -> Self {
        Self {
            path: path.to_owned(),
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }
    
    pub fn add_stats(&mut self, timestep: usize, stats: Statistics) -> Result<(), Box<dyn Error>> {
        self.buffer.push(StatsRecord { timestep, stats });
        
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }
        
        Ok(())
    }
    
    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        let file_exists = self.path.exists();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        
        let mut writer = Writer::from_writer(file);
        
        // Write header if file is new
        if !file_exists {
            writer.write_record(&[
                "timestep",
                "total_agents",
                "avg_fitness",
                "unicellular_agents",
                "multicellular_agents",
                "avg_unicellular_fitness",
                "avg_multicellular_fitness",
                "unicellular_cooperation_rate",
                "multicellular_cooperation_rate",
                "total_fitness",
                "unicellular_fitness",
                "multicellular_fitness",
                "unicellular_cooperators",
                "multicellular_cooperators",
            ])?;
        }
        
        // Write all buffered records
        for record in &self.buffer {
            writer.write_record(&[
                record.timestep.to_string(),
                record.stats.total_agents.to_string(),
                record.stats.avg_fitness().to_string(),
                record.stats.unicellular_agents.to_string(),
                record.stats.multicellular_agents.to_string(),
                record.stats.avg_unicellular_fitness().to_string(),
                record.stats.avg_multicellular_fitness().to_string(),
                record.stats.unicellular_cooperation_rate().to_string(),
                record.stats.multicellular_cooperation_rate().to_string(),
                record.stats.total_fitness.to_string(),
                record.stats.unicellular_fitness.to_string(),
                record.stats.multicellular_fitness.to_string(),
                record.stats.unicellular_cooperation.to_string(),
                record.stats.multicellular_cooperation.to_string(),
            ])?;
        }
        
        writer.flush()?;
        self.buffer.clear();
        
        Ok(())
    }
    
    pub fn finish(mut self) -> Result<(), Box<dyn Error>> {
        self.flush()?;
        Ok(())
    }
}
