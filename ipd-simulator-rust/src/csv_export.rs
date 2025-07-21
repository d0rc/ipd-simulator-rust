use crate::grid::Statistics;
use csv::Writer;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::error::Error;

pub struct CsvExporter {
    writer: Writer<File>,
}

impl CsvExporter {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let writer = Writer::from_path(path)?;
        Ok(Self { writer })
    }
    
    pub fn write_header(&mut self) -> Result<(), Box<dyn Error>> {
        self.writer.write_record(&[
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
        self.writer.flush()?;
        Ok(())
    }
    
    pub fn write_stats(&mut self, timestep: usize, stats: &Statistics) -> Result<(), Box<dyn Error>> {
        self.writer.write_record(&[
            timestep.to_string(),
            stats.total_agents.to_string(),
            stats.avg_fitness().to_string(),
            stats.unicellular_agents.to_string(),
            stats.multicellular_agents.to_string(),
            stats.avg_unicellular_fitness().to_string(),
            stats.avg_multicellular_fitness().to_string(),
            stats.unicellular_cooperation_rate().to_string(),
            stats.multicellular_cooperation_rate().to_string(),
            stats.total_fitness.to_string(),
            stats.unicellular_fitness.to_string(),
            stats.multicellular_fitness.to_string(),
            stats.unicellular_cooperation.to_string(),
            stats.multicellular_cooperation.to_string(),
        ])?;
        
        // Flush periodically for real-time updates
        if timestep % 10 == 0 {
            self.writer.flush()?;
        }
        
        Ok(())
    }
    
    pub fn finish(mut self) -> Result<(), Box<dyn Error>> {
        self.writer.flush()?;
        Ok(())
    }
}

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

/// Export detailed agent data for analysis
pub struct DetailedExporter {
    writer: Writer<File>,
}

impl DetailedExporter {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let writer = Writer::from_path(path)?;
        Ok(Self { writer })
    }
    
    pub fn write_header(&mut self) -> Result<(), Box<dyn Error>> {
        self.writer.write_record(&[
            "timestep",
            "agent_id",
            "x",
            "y",
            "fitness",
            "organism_size",
            "is_multicellular",
            "generation",
            "memory_length",
            "last_action",
            "parent_1",
            "parent_2",
            "child",
        ])?;
        self.writer.flush()?;
        Ok(())
    }
    
    pub fn write_agents(&mut self, timestep: usize, grid: &crate::grid::Grid) -> Result<(), Box<dyn Error>> {
        for (idx, agent) in grid.agents.iter().enumerate() {
            if agent.child != u32::MAX {
                continue; // Skip merged agents
            }
            
            let x = idx % grid.grid_width;
            let y = idx / grid.grid_width;
            
            self.writer.write_record(&[
                timestep.to_string(),
                idx.to_string(),
                x.to_string(),
                y.to_string(),
                agent.fitness.to_string(),
                agent.get_organism_size().to_string(),
                agent.is_multicellular().to_string(),
                agent.generation.to_string(),
                agent.mem_length.to_string(),
                agent.last_action.to_string(),
                if agent.parent_1 == u32::MAX { "".to_string() } else { agent.parent_1.to_string() },
                if agent.parent_2 == u32::MAX { "".to_string() } else { agent.parent_2.to_string() },
                if agent.child == u32::MAX { "".to_string() } else { agent.child.to_string() },
            ])?;
        }
        
        self.writer.flush()?;
        Ok(())
    }
    
    pub fn finish(mut self) -> Result<(), Box<dyn Error>> {
        self.writer.flush()?;
        Ok(())
    }
}
