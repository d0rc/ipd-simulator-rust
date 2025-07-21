use crate::agent::{Agent, Action, CompactPolicy, DeferredOp};
use bitvec::prelude::*;
use crossbeam::queue::ArrayQueue;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use lru::LruCache;
use std::num::NonZeroUsize;

/// Payoff table for IPD game
pub struct PayoffTable {
    table: [[f32; 4]; 4],
}

impl PayoffTable {
    pub fn default() -> Self {
        let mut table = [[0.0; 4]; 4];
        // C, D, M, S
        table[0][0] = 8.0;  // CC
        table[0][1] = 0.0;  // CD
        table[0][2] = 8.0;  // CM
        table[0][3] = 0.0;  // CS
        
        table[1][0] = 10.0; // DC
        table[1][1] = 5.0;  // DD
        table[1][2] = 10.0; // DM
        table[1][3] = 0.0;  // DS
        
        table[2][0] = 8.0;  // MC
        table[2][1] = 0.0;  // MD
        table[2][2] = 0.0;  // MM
        table[2][3] = 0.0;  // MS
        
        table[3][0] = 0.0;  // SC
        table[3][1] = 0.0;  // SD
        table[3][2] = 0.0;  // SM
        table[3][3] = 0.0;  // SS
        
        Self { table }
    }
    
    pub fn get(&self, my_action: Action, opp_action: Action) -> f32 {
        self.table[my_action as usize][opp_action as usize]
    }
}

/// Shared policy table with LRU eviction
pub struct PolicyTable {
    policies: Arc<Mutex<LruCache<u64, CompactPolicy>>>,
}

impl PolicyTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            policies: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap()
            ))),
        }
    }
    
    pub fn get_or_create(&self, state_hash: u64) -> CompactPolicy {
        let mut cache = self.policies.lock().unwrap();
        cache.get_or_insert(state_hash, || CompactPolicy::new()).clone()
    }
    
    pub fn update(&self, state_hash: u64, policy: CompactPolicy) {
        let mut cache = self.policies.lock().unwrap();
        cache.put(state_hash, policy);
    }
}

/// Main grid structure for the simulation
pub struct Grid {
    pub agents: Vec<Agent>,
    pub active_mask: BitVec,
    pub grid_width: usize,
    pub grid_height: usize,
    pub chunk_size: usize,
    pub policy_table: PolicyTable,
    pub payoff_table: PayoffTable,
    pub deferred_ops: Arc<ArrayQueue<DeferredOp>>,
    pub next_agent_id: u32,
    
    // Q-learning parameters
    pub alpha: f32,
    pub gamma: f32,
    pub epsilon: f32,
}

impl Grid {
    pub fn new(width: usize, height: usize, chunk_size: usize) -> Self {
        let total_agents = width * height;
        let mut agents = Vec::with_capacity(total_agents);
        let active_mask = bitvec![1; total_agents];
        
        // Initialize agents
        for i in 0..total_agents {
            agents.push(Agent::new(i as u32));
        }
        
        Self {
            agents,
            active_mask,
            grid_width: width,
            grid_height: height,
            chunk_size,
            policy_table: PolicyTable::new(10_000_000), // 10M policies
            payoff_table: PayoffTable::default(),
            deferred_ops: Arc::new(ArrayQueue::new(100_000)),
            next_agent_id: total_agents as u32,
            alpha: 0.2,
            gamma: 0.95,
            epsilon: 0.1,
        }
    }
    
    /// Get neighbors for an agent (8-connected)
    #[inline]
    pub fn get_neighbors(&self, idx: usize) -> Vec<usize> {
        let x = idx % self.grid_width;
        let y = idx / self.grid_width;
        let mut neighbors = Vec::with_capacity(8);
        
        // Use branchless neighbor calculation
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && nx < self.grid_width as i32 && 
                   ny >= 0 && ny < self.grid_height as i32 {
                    neighbors.push((ny as usize * self.grid_width) + nx as usize);
                }
            }
        }
        
        neighbors
    }
    
    /// Find the root agent (following child links)
    pub fn find_root(&self, mut idx: usize) -> usize {
        while self.agents[idx].child != u32::MAX {
            idx = self.agents[idx].child as usize;
            if idx >= self.agents.len() {
                break;
            }
        }
        idx
    }
    
    /// Process a single chunk of the grid
    fn process_chunk(&self, chunk_x: usize, chunk_y: usize) {
        let start_x = chunk_x * self.chunk_size;
        let start_y = chunk_y * self.chunk_size;
        let end_x = (start_x + self.chunk_size).min(self.grid_width);
        let end_y = (start_y + self.chunk_size).min(self.grid_height);
        
        // Process all agents in this chunk
        for y in start_y..end_y {
            for x in start_x..end_x {
                let idx = y * self.grid_width + x;
                
                if !self.active_mask[idx] {
                    continue;
                }
                
                // Find actual agent (might be merged)
                let agent_idx = self.find_root(idx);
                if agent_idx >= self.agents.len() {
                    continue;
                }
                
                // Get neighbors and pick one randomly
                let neighbors = self.get_neighbors(idx);
                if neighbors.is_empty() {
                    continue;
                }
                
                let opp_idx = neighbors[rand::random::<usize>() % neighbors.len()];
                let opp_root = self.find_root(opp_idx);
                
                if opp_root >= self.agents.len() || opp_root == agent_idx {
                    continue;
                }
                
                // Simulate interaction
                self.simulate_interaction(agent_idx, opp_root);
            }
        }
    }
    
    /// Simulate interaction between two agents
    fn simulate_interaction(&self, my_idx: usize, opp_idx: usize) {
        // Get agents (we'll update them through unsafe pointers for performance)
        let agents_ptr = self.agents.as_ptr() as *mut Agent;
        
        unsafe {
            let my_agent = &mut *agents_ptr.add(my_idx);
            let opp_agent = &mut *agents_ptr.add(opp_idx);
            
            // Get memory states
            let my_state_hash = my_agent.get_memory_hash(opp_agent.memory_bits, opp_agent.mem_length);
            let opp_state_hash = opp_agent.get_memory_hash(my_agent.memory_bits, my_agent.mem_length);
            
            // Get policies
            let mut my_policy = self.policy_table.get_or_create(my_state_hash);
            let mut opp_policy = self.policy_table.get_or_create(opp_state_hash);
            
            // Choose actions
            let my_action = my_policy.get_action(my_agent.epsilon);
            let opp_action = opp_policy.get_action(opp_agent.epsilon);
            
            // Calculate payoffs
            let my_payoff = self.payoff_table.get(my_action, opp_action);
            let opp_payoff = self.payoff_table.get(opp_action, my_action);
            
            // Update fitness
            my_agent.fitness += my_payoff;
            opp_agent.fitness += opp_payoff;
            
            // Update memories
            my_agent.add_to_memory(my_action, opp_action);
            opp_agent.add_to_memory(opp_action, my_action);
            
            // Get next states for Q-learning
            let my_next_hash = my_agent.get_memory_hash(opp_agent.memory_bits, opp_agent.mem_length);
            let opp_next_hash = opp_agent.get_memory_hash(my_agent.memory_bits, my_agent.mem_length);
            
            let my_next_policy = self.policy_table.get_or_create(my_next_hash);
            let opp_next_policy = self.policy_table.get_or_create(opp_next_hash);
            
            // Update Q-values
            let my_next_max = my_next_policy.q_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let opp_next_max = opp_next_policy.q_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            
            my_policy.update(my_action, my_payoff, my_next_max, self.alpha, self.gamma);
            opp_policy.update(opp_action, opp_payoff, opp_next_max, self.alpha, self.gamma);
            
            // Update policies in table
            self.policy_table.update(my_state_hash, my_policy);
            self.policy_table.update(opp_state_hash, opp_policy);
            
            // Handle merge/split operations
            match (my_action, opp_action) {
                (Action::Merge, _) | (_, Action::Merge) => {
                    let better_agent = if my_agent.fitness > opp_agent.fitness { my_idx } else { opp_idx };
                    let _ = self.deferred_ops.push(DeferredOp::Merge {
                        agent1: my_idx as u32,
                        agent2: opp_idx as u32,
                        new_fitness: (my_agent.fitness + opp_agent.fitness) / 2.0,
                        inherit_from: better_agent as u32,
                    });
                }
                (Action::Split, _) if my_agent.is_multicellular() => {
                    let _ = self.deferred_ops.push(DeferredOp::Split {
                        agent: my_idx as u32,
                        parent1: my_agent.parent_1,
                        parent2: my_agent.parent_2,
                    });
                }
                (_, Action::Split) if opp_agent.is_multicellular() => {
                    let _ = self.deferred_ops.push(DeferredOp::Split {
                        agent: opp_idx as u32,
                        parent1: opp_agent.parent_1,
                        parent2: opp_agent.parent_2,
                    });
                }
                _ => {}
            }
        }
    }
    
    /// Run one timestep of the simulation
    pub fn step(&mut self) {
        // Clear deferred operations
        while self.deferred_ops.pop().is_some() {}
        
        // Process chunks in parallel
        let chunk_coords: Vec<(usize, usize)> = (0..self.grid_height / self.chunk_size + 1)
            .flat_map(|cy| (0..self.grid_width / self.chunk_size + 1).map(move |cx| (cx, cy)))
            .collect();
        
        chunk_coords.par_iter().for_each(|&(cx, cy)| {
            self.process_chunk(cx, cy);
        });
        
        // Apply deferred operations
        self.apply_deferred_operations();
    }
    
    /// Apply merge and split operations
    fn apply_deferred_operations(&mut self) {
        while let Some(op) = self.deferred_ops.pop() {
            match op {
                DeferredOp::Merge { agent1, agent2, new_fitness, inherit_from } => {
                    // Create new merged agent
                    let new_id = self.next_agent_id;
                    self.next_agent_id += 1;
                    
                    let mut new_agent = self.agents[inherit_from as usize].clone();
                    new_agent.fitness = new_fitness;
                    new_agent.parent_1 = agent1;
                    new_agent.parent_2 = agent2;
                    new_agent.generation += 1;
                    
                    // Update parent agents
                    self.agents[agent1 as usize].child = new_id;
                    self.agents[agent2 as usize].child = new_id;
                    
                    // Add new agent
                    self.agents.push(new_agent);
                }
                DeferredOp::Split { agent, parent1, parent2 } => {
                    if parent1 != u32::MAX && parent2 != u32::MAX {
                        // Copy fitness and q_values before modifying
                        let fitness = self.agents[agent as usize].fitness;
                        let q_values = self.agents[agent as usize].q_values;
                        
                        // Restore parent agents
                        self.agents[parent1 as usize].child = u32::MAX;
                        self.agents[parent2 as usize].child = u32::MAX;
                        self.agents[parent1 as usize].fitness = fitness;
                        self.agents[parent2 as usize].fitness = fitness;
                        self.agents[parent1 as usize].q_values = q_values;
                        self.agents[parent2 as usize].q_values = q_values;
                    }
                }
            }
        }
    }
    
    /// Get statistics for the current state
    pub fn get_statistics(&self) -> Statistics {
        let mut stats = Statistics::default();
        
        // Parallel computation of statistics
        let partial_stats: Vec<Statistics> = self.agents
            .par_chunks(10000)
            .map(|chunk| {
                let mut local_stats = Statistics::default();
                
                for agent in chunk {
                    if agent.child != u32::MAX {
                        continue; // Skip merged agents
                    }
                    
                    local_stats.total_agents += 1;
                    local_stats.total_fitness += agent.fitness as f64;
                    
                    if agent.is_multicellular() {
                        local_stats.multicellular_agents += 1;
                        local_stats.multicellular_fitness += agent.fitness as f64;
                    } else {
                        local_stats.unicellular_agents += 1;
                        local_stats.unicellular_fitness += agent.fitness as f64;
                    }
                    
                    if agent.last_action == Action::Cooperate as u8 {
                        if agent.is_multicellular() {
                            local_stats.multicellular_cooperation += 1;
                        } else {
                            local_stats.unicellular_cooperation += 1;
                        }
                    }
                }
                
                local_stats
            })
            .collect();
        
        // Combine partial statistics
        for partial in partial_stats {
            stats.total_agents += partial.total_agents;
            stats.total_fitness += partial.total_fitness;
            stats.unicellular_agents += partial.unicellular_agents;
            stats.multicellular_agents += partial.multicellular_agents;
            stats.unicellular_fitness += partial.unicellular_fitness;
            stats.multicellular_fitness += partial.multicellular_fitness;
            stats.unicellular_cooperation += partial.unicellular_cooperation;
            stats.multicellular_cooperation += partial.multicellular_cooperation;
        }
        
        stats
    }
}

#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub total_agents: usize,
    pub total_fitness: f64,
    pub unicellular_agents: usize,
    pub multicellular_agents: usize,
    pub unicellular_fitness: f64,
    pub multicellular_fitness: f64,
    pub unicellular_cooperation: usize,
    pub multicellular_cooperation: usize,
}

impl Statistics {
    pub fn avg_fitness(&self) -> f64 {
        if self.total_agents > 0 {
            self.total_fitness / self.total_agents as f64
        } else {
            0.0
        }
    }
    
    pub fn avg_unicellular_fitness(&self) -> f64 {
        if self.unicellular_agents > 0 {
            self.unicellular_fitness / self.unicellular_agents as f64
        } else {
            0.0
        }
    }
    
    pub fn avg_multicellular_fitness(&self) -> f64 {
        if self.multicellular_agents > 0 {
            self.multicellular_fitness / self.multicellular_agents as f64
        } else {
            0.0
        }
    }
    
    pub fn unicellular_cooperation_rate(&self) -> f64 {
        if self.unicellular_agents > 0 {
            self.unicellular_cooperation as f64 / self.unicellular_agents as f64
        } else {
            0.0
        }
    }
    
    pub fn multicellular_cooperation_rate(&self) -> f64 {
        if self.multicellular_agents > 0 {
            self.multicellular_cooperation as f64 / self.multicellular_agents as f64
        } else {
            0.0
        }
    }
}
