# Algorithm Description for `grid-copy.js`

## 1. Overview

The script implements a multi-agent simulation on a 2D grid where agents interact via an extended Iterated Prisoner's Dilemma (IPD). Agents use Q-learning to adapt their strategies, which can include merging into multicellular structures or splitting apart. The simulation tracks the evolution of agent size, fitness, and cooperation over time.

## 2. Initialization

The simulation is initialized on a grid of size `G x G`, where `G` is `gridSize`. The state of all `G*G` potential agents is stored in `agList`.

Each agent `i` is an object:
`A_i = { parentlst, childlst, fitness, memlist, nbrs, policy, mem_size }`

-   **`policy`**: A Q-table, `Q_i(s, a)`, mapping a state `s` to Q-values for each action `a`.
-   **`mem_size`**: An integer `L_i` specifying the agent's memory capacity.
-   **`fitness`**: The agent's accumulated score, initialized to a small positive value.

## 3. Simulation Loop

The simulation proceeds in discrete time steps `t`. In each step:

1.  **Agent Selection**:
    *   An agent `A_i` is chosen uniformly at random from the set of all currently active agents.
    *   An opponent `A_j` is chosen uniformly at random from `A_i`'s neighbors.

2.  **Game Interaction (`fight`)**: The two agents play one round of the game.

## 4. Core Game Mechanics (`fight`)

### 4.1. State Representation (`stitchmem`)

The state `s` for an agent is a string derived from its own action history (`M_i`) and its opponent's (`M_j`).

1.  First, each agent's memory is truncated to its `mem_size` (`L_i`, `L_j`):
    `M'_i = M_i.slice(-L_i)`
    `M'_j = M_j.slice(-L_j)`

2.  Next, the longer of the two truncated memories is further trimmed to match the length of the shorter one. Let `k = min(length(M'_i), length(M'_j))`.
    `M''_i = M'_i.slice(-k)`
    `M''_j = M'_j.slice(-k)`

3.  The final state string `s_i` for agent `i` is formed by interleaving the two histories in reverse order:
    `s_i = M''_i[k-1] + M''_j[k-1] + M''_i[k-2] + M''_j[k-2] + ... + M''_i[0] + M''_j[0]`

The opponent's state `s_j` is constructed symmetrically.

### 4.2. Action Selection

Agents use an **ε-greedy strategy** to select one of four actions:
`Actions = {C (Cooperate), D (Defect), M (Merge), S (Split)}`

Given state `s_i`, the action `a_i` is chosen as:
`a_i =`
-   `argmax_{a'}(Q_i(s_i, a'))` with probability `1 - ε`
-   A random action from `Actions` with probability `ε`

### 4.3. Fitness Update

Fitness is updated based on the joint action `(a_i, a_j)` using a payoff matrix `P`:
`fitness_i = fitness_i + P(a_i, a_j)`
`fitness_j = fitness_j + P(a_j, a_i)`

### 4.4. Q-Learning Update

The Q-table is updated using the reward `r = P(a_i, a_j)` and the resulting next state `s'_i`:

`Q_{t+1}(s_i, a_i) = (1 - α)Q_t(s_i, a_i) + α * [r + γ * max_{a'}(Q_t(s'_i, a'))]`

-   `α`: Learning Rate
-   `γ`: Discount Factor

## 5. Agent Transformation (Multicellularity)

### 5.1. Merge (`M`)

If `a_i = 'M'` or `a_j = 'M'`, the two agents `A_i` and `A_j` are replaced by a new "super" agent `A_k`.

-   **Hierarchy**: `A_i` and `A_j` become inactive "parents". `A_k` is their "child".
-   **Inheritance**: `A_k` inherits its policy, memory, and `mem_size` from the parent with the higher fitness.
-   **Fitness**: `fitness_k = (fitness_i + fitness_j) / 2`
-   **Neighbors**: The neighbors of `A_k` are the union of its parents' neighbors.

### 5.2. Split (`S`)

If agent `A_k` (with size > 1) plays `'S'`, it dissolves.

-   `A_k` is removed.
-   Its parent agents become active again.
-   The resurrected parents inherit the policy, fitness, and memory of `A_k`.

## 6. Agent Hierarchy and Size

A tree-like data structure tracks agent lineage.

-   **`getchildren(id)`**: Finds the active agent occupying a grid cell by traversing from the initial cell `id` down the merger tree.
-   **`getparents(id)`**: Finds all the original grid cells that constitute an active agent `id` by traversing up the merger tree.
-   **Agent Size**: The size of an agent `id` is `length(getparents(id))`.
