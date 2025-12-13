use crate::tsp_data::TspData; // Импортируем структуру данных из соседнего модуля
use rand::prelude::*;
use std::time::Instant;

pub struct AcsTspSolver {
    data: TspData,
    n_ants: usize,
    n_iterations: usize,
    q0: f64,
    beta: f64,
    rho: f64,
    phi: f64,
    pheromone: Vec<f64>,
    visibility: Vec<f64>,
    tau0: f64,

    // Результаты делаем публичными, чтобы main мог их читать
    pub best_tour: Vec<usize>,
    pub best_score: f64,
    pub history: Vec<f64>,
}

impl AcsTspSolver {
    pub fn new(data: TspData, n_ants: usize, n_iterations: usize, q0: f64, beta: f64) -> Self {
        let n = data.n;

        // Видимость
        let mut visibility = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    let d = data.dist(i, j);
                    if d > 1e-10 {
                        visibility[i * n + j] = 1.0 / d;
                    }
                }
            }
        }

        // Жадный старт
        let greedy_tour = Self::greedy_initial_tour(&data);
        let greedy_len = data.calculate_tour_length(&greedy_tour);
        let tau0 = 1.0 / (n as f64 * greedy_len);

        println!(
            "ACS Initialized. Baseline (Greedy): {:.2}, Tau0: {:.6}",
            greedy_len, tau0
        );

        let pheromone = vec![tau0; n * n];

        AcsTspSolver {
            data,
            n_ants,
            n_iterations,
            q0,
            beta,
            rho: 0.1,
            phi: 0.1,
            pheromone,
            visibility,
            tau0,
            best_tour: greedy_tour,
            best_score: greedy_len,
            history: Vec::new(),
        }
    }

    fn greedy_initial_tour(data: &TspData) -> Vec<usize> {
        let n = data.n;
        let mut unvisited: Vec<bool> = vec![true; n];
        let mut tour = Vec::with_capacity(n);
        let mut curr = 0;
        tour.push(curr);
        unvisited[curr] = false;

        for _ in 1..n {
            let mut best_dist = f64::INFINITY;
            let mut best_node = 0;
            for next_node in 0..n {
                if unvisited[next_node] {
                    let d = data.dist(curr, next_node);
                    if d < best_dist {
                        best_dist = d;
                        best_node = next_node;
                    }
                }
            }
            curr = best_node;
            tour.push(curr);
            unvisited[curr] = false;
        }
        tour
    }

    fn two_opt_fast(&self, mut tour: Vec<usize>) -> Vec<usize> {
        let n = tour.len();
        let mut improved = true;
        while improved {
            improved = false;
            'outer: for i in 1..(n - 1) {
                for j in (i + 1)..n {
                    if j - i == 1 {
                        continue;
                    }
                    let u = tour[i - 1];
                    let v = tour[i];
                    let w = tour[j];
                    let z = tour[(j + 1) % n];

                    let dist_old = self.data.dist(u, v) + self.data.dist(w, z);
                    let dist_new = self.data.dist(u, w) + self.data.dist(v, z);

                    if dist_new < dist_old {
                        tour[i..=j].reverse();
                        improved = true;
                        break 'outer;
                    }
                }
            }
        }
        tour
    }

    fn select_next_city(&self, curr: usize, unvisited: &[usize], rng: &mut ThreadRng) -> usize {
        let n = self.data.n;
        if unvisited.len() == 1 {
            return unvisited[0];
        }

        // ИСПРАВЛЕНИЕ: random_range(0.0..1.0)
        if rng.random_range(0.0..1.0) <= self.q0 {
            let mut best_val = -1.0;
            let mut best_node = unvisited[0];
            for &node in unvisited {
                let val = self.pheromone[curr * n + node]
                    * self.visibility[curr * n + node].powf(self.beta);
                if val > best_val {
                    best_val = val;
                    best_node = node;
                }
            }
            best_node
        } else {
            let mut values = Vec::with_capacity(unvisited.len());
            let mut sum_val = 0.0;
            for &node in unvisited {
                let val = self.pheromone[curr * n + node]
                    * self.visibility[curr * n + node].powf(self.beta);
                values.push(val);
                sum_val += val;
            }
            if sum_val == 0.0 {
                return unvisited[rng.random_range(0..unvisited.len())];
            }

            let r = rng.random_range(0.0..1.0) * sum_val;
            let mut acc = 0.0;
            for (idx, &val) in values.iter().enumerate() {
                acc += val;
                if acc >= r {
                    return unvisited[idx];
                }
            }
            *unvisited.last().unwrap()
        }
    }

    fn local_pheromone_update(&mut self, u: usize, v: usize) {
        let idx = u * self.data.n + v;
        let idx_sym = v * self.data.n + u;
        let new_val = (1.0 - self.phi) * self.pheromone[idx] + self.phi * self.tau0;
        self.pheromone[idx] = new_val;
        self.pheromone[idx_sym] = new_val;
    }

    fn global_pheromone_update(&mut self) {
        let deposit = 1.0 / self.best_score;
        let n = self.data.n;
        for i in 0..n {
            let u = self.best_tour[i];
            let v = self.best_tour[(i + 1) % n];
            let idx = u * n + v;
            let idx_sym = v * n + u;
            let new_val = (1.0 - self.rho) * self.pheromone[idx] + self.rho * deposit;
            self.pheromone[idx] = new_val;
            self.pheromone[idx_sym] = new_val;
        }
    }

    pub fn run(&mut self) {
        println!("Starting ACS (q0={}, ants={})...", self.q0, self.n_ants);
        let start_time = Instant::now();
        let mut rng = rand::rng();
        let n = self.data.n;

        for it in 0..self.n_iterations {
            for k in 0..self.n_ants {
                let mut tour = Vec::with_capacity(n);
                let start_node = rng.random_range(0..n);
                tour.push(start_node);

                // Оптимизация удаления: swap_remove меняет порядок, но для случайного выбора это ок
                let mut unvisited: Vec<usize> = (0..n).filter(|&x| x != start_node).collect();
                let mut curr = start_node;

                while !unvisited.is_empty() {
                    let next_node = self.select_next_city(curr, &unvisited, &mut rng);
                    if let Some(pos) = unvisited.iter().position(|&x| x == next_node) {
                        unvisited.swap_remove(pos);
                    }
                    self.local_pheromone_update(curr, next_node);
                    tour.push(next_node);
                    curr = next_node;
                }
                self.local_pheromone_update(tour[n - 1], tour[0]);

                let improved_tour = self.two_opt_fast(tour);
                let improved_len = self.data.calculate_tour_length(&improved_tour);

                if improved_len < self.best_score {
                    self.best_score = improved_len;
                    self.best_tour = improved_tour;
                    println!(
                        "Iter {}, Ant {}: New Global Best -> {:.2}",
                        it, k, self.best_score
                    );
                }
            }
            self.global_pheromone_update();
            self.history.push(self.best_score);
        }
        println!(
            "Finished in {:.2?}s. Best: {:.2}",
            start_time.elapsed(),
            self.best_score
        );
    }
}
