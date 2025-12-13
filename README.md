# TSP Swarm Solver

## Overview
This project is a Traveling Salesman Problem (TSP) solver using Ant Colony Optimization (ACO). It includes a Rust-based implementation for solving TSP instances and a Python-based visualization tool for analyzing the results.

## Prerequisites

### Rust
- Install Rust using [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- Ensure `cargo` (Rust's package manager) is available:
  ```bash
  cargo --version
  ```

### Python (Optional for Visualization)
- Install Python 3.8 or higher.
- Install the required Python libraries:
  ```bash
  pip install -r requirements.txt
  ```

## Build && Run Instructions
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd tsp-swarm-solver
   ```
2. Run the project using Cargo:
   ```bash
   cargo run --release
   ```
3. Check the output in the `answer.txt` file. Each line contains the test index, score, and the tour.

## Visualization
1. Navigate to the `visualization` folder:
   ```bash
   cd visualization
   ```
2. Open the Jupyter Notebook or the presaved images.
3. Run the notebook to visualize the TSP tours.

## Project Structure
- `src/`: Contains the Rust source code.
- `data/`: Contains TSP instance files for testing.
- `visualization/`: Contains the Python visualization tool.
- `answer.txt`: Stores the results of the solver.
- `logs.txt`: Shows presaved execution logs (to confirm the integrity of the work).

## License
This project is licensed under the MIT License.