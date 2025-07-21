use crate::agent::{Agent, Action, CompactPolicy, DeferredOp};
use bitvec::prelude::*;
use crossbeam::queue::ArrayQueue;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::Instant;

/// A single interaction between two agents
#[derive(Debug, Clone, Copy)]
struct Interaction {
    agent1_idx: u32,
    agent2_idx: u32,
}

/// State changes for a single agent after an interaction
#[derive(Debug, Clone, Copy)]
struct StateUpdate {
    agent_idx: u32,
    fitness_delta: f32,
    action: Action,
    policy_hash: u64,
    new_q_values: [f32; 4],
}

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
    pub policy_table: PolicyTable,
    pub payoff_table: PayoffTable,
    pub deferred_ops: Arc<ArrayQueue<DeferredOp>>,
    pub next_agent_id: u32,
    root_cache: Vec<u32>,
    
    // Q-learning parameters
    pub alpha: f32,
    pub gamma: f32,
    pub epsilon: f32,

    // Pass statistics
    pub pass_stats: PassStatistics,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
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
            policy_table: PolicyTable::new(10_000_000), // 10M policies
            payoff_table: PayoffTable::default(),
            deferred_ops: Arc::new(ArrayQueue::new(1_000_000)),
            next_agent_id: total_agents as u32,
            root_cache: vec![0; total_agents],
            alpha: 0.2,
            gamma: 0.95,
            epsilon: 0.1,
            pass_stats: PassStatistics::default(),
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
    
    /// Generate all interactions for a timestep
    fn generate_interactions(&self) -> Vec<Interaction> {
        (0..self.grid_width * self.grid_height)
            .into_par_iter()
            .filter(|&idx| self.active_mask[idx])
            .flat_map(|idx| {
                let agent_idx = self.root_cache[idx] as u32;
                let neighbors = self.get_neighbors(idx);
                
                if neighbors.is_empty() {
                    return Vec::new();
                }
                
                let opp_idx = neighbors[rand::random::<usize>() % neighbors.len()];
                let opp_root = self.root_cache[opp_idx] as u32;
                
                if opp_root != agent_idx {
                    vec![Interaction { agent1_idx: agent_idx, agent2_idx: opp_root }]
                } else {
                    Vec::new()
                }
            })
            .collect()
    }

    /// Process interactions and generate state updates
    fn process_interactions(&self, interactions: &[Interaction]) -> Vec<StateUpdate> {
        interactions
            .par_iter()
            .flat_map(|interaction| {
                let my_idx = interaction.agent1_idx as usize;
                let opp_idx = interaction.agent2_idx as usize;

                // This is safe because we are only reading from agents
                let my_agent = &self.agents[my_idx];
                let opp_agent = &self.agents[opp_idx];

                // Get current memory states and policies
                let my_state_hash = my_agent.get_memory_hash(opp_agent.memory_bits, opp_agent.mem_length);
                let opp_state_hash = opp_agent.get_memory_hash(my_agent.memory_bits, my_agent.mem_length);
                let my_policy = self.policy_table.get_or_create(my_state_hash);
                let opp_policy = self.policy_table.get_or_create(opp_state_hash);

                // Choose actions
                let my_action = my_policy.get_action(self.epsilon);
                let opp_action = opp_policy.get_action(self.epsilon);

                // Calculate payoffs
                let my_payoff = self.payoff_table.get(my_action, opp_action);
                let opp_payoff = self.payoff_table.get(opp_action, my_action);

                // --- Q-value updates ---

                // 1. Determine next state for my_agent
                let mut next_my_agent = my_agent.clone();
                next_my_agent.add_to_memory(my_action, opp_action);
                let next_my_state_hash = next_my_agent.get_memory_hash(opp_agent.memory_bits, opp_agent.mem_length);
                let next_my_policy = self.policy_table.get_or_create(next_my_state_hash);
                let next_max_q_my = next_my_policy.q_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

                // 2. Determine next state for opp_agent
                let mut next_opp_agent = opp_agent.clone();
                next_opp_agent.add_to_memory(opp_action, my_action);
                let next_opp_state_hash = next_opp_agent.get_memory_hash(my_agent.memory_bits, my_agent.mem_length);
                let next_opp_policy = self.policy_table.get_or_create(next_opp_state_hash);
                let next_max_q_opp = next_opp_policy.q_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

                // 3. Calculate new Q-values
                let my_new_q = my_policy.calculate_updated_q_values(my_action, my_payoff, next_max_q_my, self.alpha, self.gamma);
                let opp_new_q = opp_policy.calculate_updated_q_values(opp_action, opp_payoff, next_max_q_opp, self.alpha, self.gamma);

                // Handle Merge and Split actions
                if my_action == Action::Merge && opp_action == Action::Merge {
                    let new_fitness = my_agent.fitness + opp_agent.fitness;
                    let inherit_from = if my_agent.fitness > opp_agent.fitness { my_idx } else { opp_idx };
                    self.deferred_ops.push(DeferredOp::Merge {
                        agent1: my_idx as u32,
                        agent2: opp_idx as u32,
                        new_fitness,
                        inherit_from: inherit_from as u32,
                    }).ok();
                } else if my_action == Action::Split && my_agent.is_multicellular() {
                    self.deferred_ops.push(DeferredOp::Split {
                        agent: my_idx as u32,
                        parent1: my_agent.parent_1,
                        parent2: my_agent.parent_2,
                    }).ok();
                }

                // Create state updates
                vec![
                    StateUpdate {
                        agent_idx: my_idx as u32,
                        fitness_delta: my_payoff,
                        action: my_action,
                        policy_hash: my_state_hash,
                        new_q_values: my_new_q,
                    },
                    StateUpdate {
                        agent_idx: opp_idx as u32,
                        fitness_delta: opp_payoff,
                        action: opp_action,
                        policy_hash: opp_state_hash,
                        new_q_values: opp_new_q,
                    },
                ]
            })
            .collect()
    }

    /// Apply state updates to agents
    fn apply_state_updates(&mut self, updates: &[StateUpdate]) {
        // Group updates by agent ID
        let mut updates_by_agent: Vec<Vec<StateUpdate>> = vec![Vec::new(); self.agents.len()];
        for update in updates {
            updates_by_agent[update.agent_idx as usize].push(*update);
        }

        // Process updates in parallel for each agent
        self.agents
            .par_iter_mut()
            .zip(updates_by_agent.par_iter())
            .for_each(|(agent, updates)| {
                for update in updates {
                    agent.fitness += update.fitness_delta;
                    agent.last_action = update.action as u8;
                    
                    let new_policy = CompactPolicy { q_values: update.new_q_values };
                    self.policy_table.update(update.policy_hash, new_policy);
                }
            });
    }
    
    /// Run one timestep of the simulation
    pub fn step(&mut self) {
        self.pass_stats.reset();

        // === Pass 1: Update Root Cache ===
        let start = Instant::now();
        self.update_root_cache();
        self.pass_stats.cache_update_time = start.elapsed().as_micros();

        // === Pass 2: Generate Interactions ===
        let start = Instant::now();
        let interactions = self.generate_interactions();
        self.pass_stats.interaction_generation_time = start.elapsed().as_micros();
        self.pass_stats.num_interactions = interactions.len();

        // === Pass 3: Process Interactions ===
        let start = Instant::now();
        let updates = self.process_interactions(&interactions);
        self.pass_stats.interaction_processing_time = start.elapsed().as_micros();
        self.pass_stats.num_updates = updates.len();

        // === Pass 4: Apply State Updates ===
        let start = Instant::now();
        self.apply_state_updates(&updates);
        self.pass_stats.state_update_time = start.elapsed().as_micros();

        // === Pass 5: Apply Deferred Operations ===
        let start = Instant::now();
        self.apply_deferred_operations();
        self.pass_stats.deferred_op_time = start.elapsed().as_micros();
    }

    /// Update the root cache
    fn update_root_cache(&mut self) {
        let new_root_cache: Vec<u32> = (0..self.agents.len())
            .into_par_iter()
            .map(|i| self.find_root(i) as u32)
            .collect();

        if self.root_cache.len() != new_root_cache.len() {
            self.root_cache.resize(new_root_cache.len(), 0);
        }
        
        self.root_cache.copy_from_slice(&new_root_cache);
    }
    
    /// Apply merge and split operations
    fn apply_deferred_operations(&mut self) {
        while let Some(op) = self.deferred_ops.pop() {
            match op {
                DeferredOp::Merge { agent1, agent2, new_fitness, inherit_from } => {
                    // Ensure agents exist and are not already part of an organism
                    if agent1 as usize >= self.agents.len() || agent2 as usize >= self.agents.len() ||
                       self.agents[agent1 as usize].child != u32::MAX || self.agents[agent2 as usize].child != u32::MAX {
                        continue;
                    }

                    // Create new merged agent
                    let new_id = self.agents.len() as u32;
                    
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
                    self.active_mask.push(false); // The new agent is not on the grid
                    self.root_cache.push(0); // Add a placeholder to the root cache
                }
                DeferredOp::Split { agent, parent1, parent2 } => {
                    if parent1 != u32::MAX && parent2 != u32::MAX &&
                       (parent1 as usize) < self.agents.len() && (parent2 as usize) < self.agents.len() {
                        // Find empty cells for the new agents
                        let empty_cells = self.find_empty_cells(2);
                        if empty_cells.len() < 2 {
                            continue;
                        }

                        // Copy fitness and q_values before modifying
                        let fitness = self.agents[agent as usize].fitness;
                        
                        // Restore parent agents
                        self.agents[parent1 as usize].child = u32::MAX;
                        self.agents[parent2 as usize].child = u32::MAX;
                        self.agents[parent1 as usize].fitness = fitness / 2.0;
                        self.agents[parent2 as usize].fitness = fitness / 2.0;

                        // Place new agents on the grid
                        self.active_mask.set(empty_cells[0], true);
                        self.root_cache[empty_cells[0]] = parent1;
                        self.active_mask.set(empty_cells[1], true);
                        self.root_cache[empty_cells[1]] = parent2;
                    }
                }
            }
        }
    }
    
    /// Get statistics for the current state
    pub fn get_statistics(&self) -> Statistics {
        let mut stats = Statistics::default();
        
        // Use the root cache to iterate over active agents only
        let active_agents: Vec<_> = (0..self.grid_width * self.grid_height)
            .into_par_iter()
            .filter_map(|i| {
                if self.active_mask[i] {
                    Some(self.root_cache[i] as usize)
                } else {
                    None
                }
            })
            .collect();

        let partial_stats: Vec<Statistics> = active_agents
            .par_chunks(10000)
            .map(|chunk| {
                let mut local_stats = Statistics::default();
                
                for &agent_idx in chunk {
                    let agent = &self.agents[agent_idx];
                    
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

    /// Find empty cells on the grid
    fn find_empty_cells(&self, count: usize) -> Vec<usize> {
        let mut empty_cells = Vec::with_capacity(count);
        for (i, is_active) in self.active_mask.iter().enumerate() {
            if !*is_active {
                empty_cells.push(i);
                if empty_cells.len() == count {
                    break;
                }
            }
        }
        empty_cells
    }
}

#[derive(Debug, Default, Clone)]
pub struct PassStatistics {
    pub num_interactions: usize,
    pub num_updates: usize,
    pub cache_update_time: u128,
    pub interaction_generation_time: u128,
    pub interaction_processing_time: u128,
    pub state_update_time: u128,
    pub deferred_op_time: u128,
}

impl PassStatistics {
    pub fn reset(&mut self) {
        *self = Self::default();
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
    pub pass_stats: PassStatistics,
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
