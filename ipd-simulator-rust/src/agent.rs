
/// Actions that agents can take
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Cooperate = 0,
    Defect = 1,
    Merge = 2,
    Split = 3,
}

impl Action {
    pub fn from_u8(val: u8) -> Self {
        match val & 0b11 {
            0 => Action::Cooperate,
            1 => Action::Defect,
            2 => Action::Merge,
            3 => Action::Split,
            _ => unreachable!(),
        }
    }
}

/// Cache-line aligned agent structure (64 bytes)
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct Agent {
    // Core state (16 bytes)
    pub fitness: f32,
    pub memory_bits: u32,    // Bit-packed memory (up to 16 moves, 2 bits each)
    pub mem_length: u8,      // Current memory size (0-5)
    pub last_action: u8,     // Last action taken
    _padding1: [u8; 2],
    
    // Relationships (16 bytes)
    pub parent_1: u32,       // First parent ID (u32::MAX if none)
    pub parent_2: u32,       // Second parent ID
    pub child: u32,          // Child ID (for merged agents)
    pub generation: u32,     // Merge generation counter
    
    // Q-learning state (32 bytes)
    pub q_values: [f32; 8],  // Current Q-values for C,D,M,S with current state
    pub policy_hash: u64,    // Hash of current memory state
    pub epsilon: f32,        // Exploration rate
    pub _padding2: [u8; 4],
}

impl Agent {
    pub fn new(_id: u32) -> Self {
        Self {
            fitness: 0.001,
            memory_bits: 0,
            mem_length: (rand::random::<u8>() % 5) + 1, // 1-5
            last_action: 0,
            _padding1: [0; 2],
            
            parent_1: u32::MAX,
            parent_2: u32::MAX,
            child: u32::MAX,
            generation: 0,
            
            q_values: [0.1; 8],
            policy_hash: 0,
            epsilon: 0.1,
            _padding2: [0; 4],
        }
    }
    
    /// Add an action to memory (2 bits per action)
    pub fn add_to_memory(&mut self, my_action: Action, opp_action: Action) {
        if self.mem_length == 0 {
            return;
        }
        
        // Shift memory left by 4 bits (2 actions × 2 bits)
        self.memory_bits <<= 4;
        
        // Add new actions
        self.memory_bits |= (my_action as u32) << 2;
        self.memory_bits |= opp_action as u32;
        
        // Mask to keep only relevant bits
        let mask = (1u32 << (self.mem_length as u32 * 4)) - 1;
        self.memory_bits &= mask;
    }
    
    /// Get memory state as a hash for policy lookup
    pub fn get_memory_hash(&self, opp_memory: u32, opp_mem_length: u8) -> u64 {
        // Combine both agents' memories into a single hash
        let my_bits = self.memory_bits & ((1u32 << (self.mem_length as u32 * 4)) - 1);
        let opp_bits = opp_memory & ((1u32 << (opp_mem_length as u32 * 4)) - 1);
        
        // Pack into u64: [my_mem_length|opp_mem_length|my_bits|opp_bits]
        ((self.mem_length as u64) << 56) |
        ((opp_mem_length as u64) << 48) |
        ((my_bits as u64) << 32) |
        (opp_bits as u64)
    }
    
    /// Check if this agent is part of a multicellular organism
    pub fn is_multicellular(&self) -> bool {
        self.child != u32::MAX || (self.parent_1 != u32::MAX && self.parent_2 != u32::MAX)
    }
    
    /// Get the size of the organism this agent belongs to
    pub fn get_organism_size(&self) -> u32 {
        if !self.is_multicellular() {
            1
        } else {
            // This will be computed by traversing the parent/child tree
            // For now, return generation as a proxy
            self.generation + 1
        }
    }
}

/// Compact policy representation for memory efficiency
#[derive(Debug, Clone, Copy)]
pub struct CompactPolicy {
    pub q_values: [f32; 4], // Q-values for C, D, M, S
}

impl CompactPolicy {
    pub fn new() -> Self {
        Self {
            q_values: [
                rand::random::<f32>() * 0.1,
                rand::random::<f32>() * 0.1,
                rand::random::<f32>() * 0.1,
                rand::random::<f32>() * 0.1,
            ],
        }
    }
    
    /// Get action using epsilon-greedy strategy
    pub fn get_action(&self, epsilon: f32) -> Action {
        if rand::random::<f32>() < epsilon {
            // Random action
            Action::from_u8(rand::random::<u8>() & 0b11)
        } else {
            // Greedy action
            let max_idx = self.q_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(idx, _)| idx)
                .unwrap();
            Action::from_u8(max_idx as u8)
        }
    }
    
    /// Update Q-value using TD learning
    pub fn update(&mut self, action: Action, reward: f32, next_max_q: f32, alpha: f32, gamma: f32) {
        let idx = action as usize;
        let td_target = reward + gamma * next_max_q;
        let td_error = td_target - self.q_values[idx];
        self.q_values[idx] += alpha * td_error;
    }
}

/// Operations that need to be applied after parallel processing
#[derive(Debug, Clone)]
pub enum DeferredOp {
    Merge {
        agent1: u32,
        agent2: u32,
        new_fitness: f32,
        inherit_from: u32,
    },
    Split {
        agent: u32,
        parent1: u32,
        parent2: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_size() {
        assert_eq!(std::mem::size_of::<Agent>(), 64);
        assert_eq!(std::mem::align_of::<Agent>(), 64);
    }
    
    #[test]
    fn test_memory_packing() {
        let mut agent = Agent::new(0);
        agent.mem_length = 3;
        
        agent.add_to_memory(Action::Cooperate, Action::Defect);
        agent.add_to_memory(Action::Merge, Action::Split);
        agent.add_to_memory(Action::Defect, Action::Cooperate);
        
        // Should have: DC|SM|CD (newest to oldest)
        // In bits: 01|00|10|11|00|01
        assert_eq!(agent.memory_bits & 0xFFF, 0b010010110001);
    }
}
