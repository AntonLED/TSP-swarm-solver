use crate::tsp_data::TspData;
use rand::prelude::*;
use std::time::Instant;

type PheromoneType = f32;

pub struct AcsTspSolver {
    data: TspData,
    n_ants: usize,
    n_iterations: usize,
    q0: f64,
    beta: f64,
    rho: f64,
    phi: f64,

    pheromone: Vec<PheromoneType>,
    candidates: Vec<Vec<usize>>,

    tau0: f64,

    pub best_tour: Vec<usize>,
    pub best_score: f64,
    pub history: Vec<f64>,
}

impl AcsTspSolver {
    pub fn new(data: TspData, n_ants: usize, n_iterations: usize, q0: f64, beta: f64) -> Self {
        let n = data.n;

        // Кандидаты: 25 для маленьких, 40 для больших графов
        // Для универсальности берем 30
        println!("Precomputing Candidate Lists (Top 30)...");
        let k_candidates = 30;
        let mut candidates = vec![Vec::new(); n];

        for i in 0..n {
            let mut dists: Vec<(usize, f64)> = (0..n)
                .filter(|&j| i != j)
                .map(|j| (j, data.dist(i, j)))
                .collect();
            dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            candidates[i] = dists.iter().take(k_candidates).map(|x| x.0).collect();
        }

        // --- УЛУЧШЕННЫЙ СТАРТ ---
        // Сразу оптимизируем жадный путь, чтобы задать высокую планку tau0
        let mut greedy_tour = Self::greedy_initial_tour(&data, &candidates);
        // Применяем 2-opt к жадному старту
        Self::static_two_opt(&data, &mut greedy_tour);

        let greedy_len = data.calculate_tour_length(&greedy_tour);
        // Формула ACS
        let tau0 = 1.0 / (n as f64 * greedy_len);

        println!("Optimized Baseline: {:.2}, Tau0: {:.6e}", greedy_len, tau0);

        let pheromone = vec![tau0 as PheromoneType; n * n];

        AcsTspSolver {
            data,
            n_ants,
            n_iterations,
            q0,
            beta,
            rho: 0.1,
            phi: 0.1,
            pheromone,
            candidates,
            tau0,
            best_tour: greedy_tour,
            best_score: greedy_len,
            history: Vec::new(),
        }
    }

    // Статический метод для оптимизации при инициализации
    fn static_two_opt(data: &TspData, tour: &mut Vec<usize>) {
        let n = tour.len();
        let mut improved = true;
        let mut passes = 0;
        while improved && passes < 5 {
            improved = false;
            passes += 1;
            for i in 1..(n - 1) {
                let limit = std::cmp::min(n, i + 200);
                for j in (i + 1)..limit {
                    if j - i == 1 {
                        continue;
                    }
                    let u = tour[i - 1];
                    let v = tour[i];
                    let w = tour[j];
                    let z = tour[(j + 1) % n];
                    if data.dist(u, w) + data.dist(v, z) < data.dist(u, v) + data.dist(w, z) - 1e-8
                    {
                        tour[i..=j].reverse();
                        improved = true;
                    }
                }
            }
        }
    }

    fn greedy_initial_tour(data: &TspData, candidates: &[Vec<usize>]) -> Vec<usize> {
        let n = data.n;
        let mut unvisited = vec![true; n];
        let mut tour = Vec::with_capacity(n);
        let mut curr = 0;
        tour.push(0);
        unvisited[0] = false;

        for _ in 1..n {
            let mut next_node = 0;
            let mut found = false;
            for &cand in &candidates[curr] {
                if unvisited[cand] {
                    next_node = cand;
                    found = true;
                    break;
                }
            }
            if !found {
                let mut best_dist = f64::INFINITY;
                for j in 0..n {
                    if unvisited[j] {
                        let d = data.dist(curr, j);
                        if d < best_dist {
                            best_dist = d;
                            next_node = j;
                        }
                    }
                }
            }
            curr = next_node;
            tour.push(curr);
            unvisited[curr] = false;
        }
        tour
    }

    // Быстрый 2-opt (Windowed)
    fn two_opt_fast(&self, tour: &mut Vec<usize>) {
        let n = tour.len();
        let mut improved = true;
        let mut passes = 0;

        // Лимит 8 проходов - баланс скорости и качества
        while improved && passes < 8 {
            improved = false;
            passes += 1;

            for i in 1..(n - 1) {
                // Окно 200 для скорости на больших графах
                let limit = std::cmp::min(n, i + 200);
                for j in (i + 1)..limit {
                    if j - i == 1 {
                        continue;
                    }
                    let u = tour[i - 1];
                    let v = tour[i];
                    let w = tour[j];
                    let z = tour[(j + 1) % n];

                    if self.data.dist(u, w) + self.data.dist(v, z)
                        < self.data.dist(u, v) + self.data.dist(w, z) - 1e-8
                    {
                        tour[i..=j].reverse();
                        improved = true;
                    }
                }
            }
        }
    }

    // Or-opt (Heavy)
    fn or_opt(&self, tour: &mut Vec<usize>) -> bool {
        let n = tour.len();
        let mut improved = false;
        let mut pos = vec![0; n];
        for (i, &node) in tour.iter().enumerate() {
            pos[node] = i;
        }

        for &block_size in &[3, 2, 1] {
            let mut continue_search = true;
            let mut moves_limit = 0;

            // Разрешаем не более 10 перестановок за вызов, чтобы не виснуть
            while continue_search && moves_limit < 10 {
                continue_search = false;

                for i in 0..n {
                    if i + block_size >= n - 1 {
                        continue;
                    }

                    let start_node = tour[i];
                    let end_node = tour[i + block_size - 1];
                    let prev_node = tour[if i == 0 { n - 1 } else { i - 1 }];
                    let next_node = tour[i + block_size];

                    let cost_rem =
                        self.data.dist(prev_node, start_node) + self.data.dist(end_node, next_node);
                    let cost_add = self.data.dist(prev_node, next_node);
                    let reduction = cost_rem - cost_add;

                    // Ищем куда вставить, проверяя кандидатов start_node
                    for &target_node in &self.candidates[start_node] {
                        let target_idx = pos[target_node];
                        if target_idx >= (if i == 0 { n - 1 } else { i - 1 })
                            && target_idx <= i + block_size
                        {
                            continue;
                        }

                        let target_next_idx = (target_idx + 1) % n;
                        let target_next_node = tour[target_next_idx];

                        let cost_ins = self.data.dist(target_node, start_node)
                            + self.data.dist(end_node, target_next_node);
                        let cost_break = self.data.dist(target_node, target_next_node);

                        if reduction > (cost_ins - cost_break) + 1e-6 {
                            // Move block
                            let block: Vec<usize> = tour.drain(i..i + block_size).collect();
                            let mut new_target_idx = target_idx;
                            if target_idx > i {
                                new_target_idx -= block_size;
                            }

                            let insert_at = (new_target_idx + 1) % tour.len();
                            for (k, &node) in block.iter().enumerate() {
                                tour.insert(insert_at + k, node);
                            }

                            // Rebuild cache
                            for (idx, &val) in tour.iter().enumerate() {
                                pos[val] = idx;
                            }

                            improved = true;
                            continue_search = true;
                            moves_limit += 1;
                            break;
                        }
                    }
                    if continue_search {
                        break;
                    }
                }
            }
        }
        improved
    }

    fn select_next_city(&self, curr: usize, unvisited_mask: &[bool], rng: &mut ThreadRng) -> usize {
        // Fast candidate selection
        let mut candidates_vec: Vec<usize> = Vec::with_capacity(30);
        for &c in &self.candidates[curr] {
            if unvisited_mask[c] {
                candidates_vec.push(c);
            }
        }

        if !candidates_vec.is_empty() {
            if rng.random_range(0.0..1.0) <= self.q0 {
                let mut best_node = candidates_vec[0];
                let mut best_val = -1.0;
                for &node in &candidates_vec {
                    // Precompute distance? No, fast enough.
                    let val = self.pheromone[curr * self.data.n + node]
                        * (1.0 / self.data.dist(curr, node)).powf(self.beta) as PheromoneType;
                    if val > best_val {
                        best_val = val;
                        best_node = node;
                    }
                }
                return best_node;
            } else {
                let mut values = Vec::with_capacity(candidates_vec.len());
                let mut sum = 0.0;
                for &node in &candidates_vec {
                    let val = self.pheromone[curr * self.data.n + node]
                        * (1.0 / self.data.dist(curr, node)).powf(self.beta) as PheromoneType;
                    values.push(val);
                    sum += val;
                }
                if sum == 0.0 {
                    return candidates_vec[0];
                }
                let r = rng.random_range(0.0..1.0) * sum;
                let mut acc = 0.0;
                for (i, &v) in values.iter().enumerate() {
                    acc += v;
                    if acc >= r {
                        return candidates_vec[i];
                    }
                }
                return *candidates_vec.last().unwrap();
            }
        }

        // Fallback
        for i in 0..self.data.n {
            if unvisited_mask[i] {
                return i;
            }
        }
        0
    }

    fn local_update(&mut self, u: usize, v: usize) {
        let idx = u * self.data.n + v;
        let idx2 = v * self.data.n + u;
        self.pheromone[idx] = (1.0 - self.phi) as PheromoneType * self.pheromone[idx]
            + (self.phi * self.tau0) as PheromoneType;
        self.pheromone[idx2] = self.pheromone[idx];
    }

    fn global_update(&mut self) {
        let deposit = 1.0 / self.best_score;
        let n = self.data.n;
        for i in 0..n {
            let u = self.best_tour[i];
            let v = self.best_tour[(i + 1) % n];
            let idx = u * n + v;
            let idx2 = v * n + u;
            self.pheromone[idx] = (1.0 - self.rho) as PheromoneType * self.pheromone[idx]
                + (self.rho * deposit) as PheromoneType;
            self.pheromone[idx2] = self.pheromone[idx];
        }
    }

    pub fn run(&mut self) {
        println!(
            "Starting Robust ACS (beta={}, ants={})...",
            self.beta, self.n_ants
        );
        let start = Instant::now();
        let mut rng = rand::rng();
        let n = self.data.n;

        for it in 0..self.n_iterations {
            // Храним лучшего муравья этой итерации
            let mut iter_best_score = f64::INFINITY;
            let mut iter_best_tour = Vec::new();

            for _k in 0..self.n_ants {
                let mut tour = Vec::with_capacity(n);
                let start_node = rng.random_range(0..n);
                tour.push(start_node);

                let mut mask = vec![true; n];
                mask[start_node] = false;
                let mut count = 1;
                let mut curr = start_node;

                while count < n {
                    let next = self.select_next_city(curr, &mask, &mut rng);
                    mask[next] = false;
                    self.local_update(curr, next);
                    tour.push(next);
                    curr = next;
                    count += 1;
                }
                self.local_update(tour[n - 1], tour[0]);

                // 1. Все делают быстрый 2-opt
                self.two_opt_fast(&mut tour);
                let score = self.data.calculate_tour_length(&tour);

                // Запоминаем лучшего в итерации
                if score < iter_best_score {
                    iter_best_score = score;
                    iter_best_tour = tour;
                }
            }

            // 2. СТРАТЕГИЯ "ЧЕМПИОН ИТЕРАЦИИ"
            // Только победитель гонки получает Or-opt.
            // Это гарантирует, что мы всегда пытаемся улучшить лучший результат,
            // но не тратим время на остальных.
            if self.or_opt(&mut iter_best_tour) {
                // Если улучшили, пересчитываем скор
                // iter_best_score = self.data.calculate_tour_length(&iter_best_tour);
                // И полируем еще раз 2-opt'ом
                self.two_opt_fast(&mut iter_best_tour);
                iter_best_score = self.data.calculate_tour_length(&iter_best_tour);
            }

            // Обновляем глобальный рекорд
            if iter_best_score < self.best_score {
                self.best_score = iter_best_score;
                self.best_tour = iter_best_tour;
                println!("Iter {}: NEW RECORD {:.2}", it, self.best_score);
            }

            // Глобальное обновление феромонов по глобальному лучшему
            self.global_update();
            self.history.push(self.best_score);
        }
        println!(
            "Done in {:.2?}s. Best: {:.2}",
            start.elapsed(),
            self.best_score
        );
    }
}
