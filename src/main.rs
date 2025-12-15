mod tsp_data;
mod tsp_solvers;

use ::colored::Colorize;
use std::{fs, path::Path};
use tsp_data::TspData;
use tsp_solvers::{AcsTspSolver, PsoTspSolver};

struct SolverConfig {
    n_ants: usize,
    n_iterations: usize,
    q0: f64,
    beta: f64,
}

struct TestsConfig<'a> {
    filename: &'a str,
    score_min: f64,
    score_max: f64,
    solver_configs: SolverConfig,
}

enum TestResult {
    Passed5,
    Passed7,
    Failed,
}

fn run_simple_test(config: &TestsConfig) -> Result<(TestResult, Vec<usize>), ()> {
    let filename = config.filename;

    if !Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found.", filename);
        eprintln!(
            "Please make sure you have a 'data' folder next to Cargo.toml containing the test file."
        );
        return Err(());
    }

    match TspData::new(filename) {
        Ok(data) => {
            println!("Loaded {} cities.", data.n);

            let num_particles = 128;
            let iterations = 512;
            let w = 0.7;
            let c1 = 1.5;
            let c2 = 1.5;

            let mut solver = PsoTspSolver::new(data, num_particles, iterations, w, c1, c2);

            solver.run();

            println!("Final PSO Best Length: {:.2}", solver.gbest_score);

            if solver.gbest_score <= config.score_min && solver.gbest_score >= config.score_max {
                println!("{}", "5 points test passed!".yellow());
                Ok((TestResult::Passed5, solver.gbest_tour))
            } else if solver.gbest_score >= config.score_min {
                println!("{}", "0 points test failed!".red());
                Ok((TestResult::Failed, solver.gbest_tour))
            } else {
                println!("{}", "7 points test passed!".green());
                Ok((TestResult::Passed7, solver.gbest_tour))
            }
        }
        Err(e) => {
            eprintln!("Error loading data: {}", e);
            Err(())
        }
    }
}

fn run_test(config: &TestsConfig) -> Result<(TestResult, Vec<usize>), ()> {
    let filename = config.filename;

    if !Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found.", filename);
        eprintln!(
            "Please make sure you have a 'data' folder next to Cargo.toml containing the test file."
        );
        return Err(());
    }

    match TspData::new(filename) {
        Ok(data) => {
            println!("Loaded {} cities from {}", data.n, filename);

            let mut solver = AcsTspSolver::new(
                data.clone(),
                config.solver_configs.n_ants,
                config.solver_configs.n_iterations,
                config.solver_configs.q0,
                config.solver_configs.beta,
            );

            solver.run();

            println!(
                "Final Best Tour Length: {:.2} / {:.2} {:.2}",
                solver.best_score, config.score_min, config.score_max
            );

            if solver.best_score <= config.score_min && solver.best_score >= config.score_max {
                println!("{}", "5 points test passed!".yellow());
                Ok((TestResult::Passed5, solver.best_tour))
            } else if solver.best_score >= config.score_min {
                println!("{}", "0 points test failed!".red());
                Ok((TestResult::Failed, solver.best_tour))
            } else {
                println!("{}", "7 points test passed!".green());
                Ok((TestResult::Passed7, solver.best_tour))
            }
        }
        Err(e) => {
            eprintln!("Error loading data: {}", e);
            Err(())
        }
    }
}

fn main() {
    let tests: Vec<TestsConfig> = vec![
        TestsConfig {
            filename: "./data/tsp_51_1",
            score_min: 482.0,
            score_max: 430.0,
            solver_configs: SolverConfig {
                n_ants: 32,
                n_iterations: 128,
                q0: 0.9,
                beta: 2.0,
            },
        },
        TestsConfig {
            filename: "./data/tsp_100_3",
            score_min: 23_433.0,
            score_max: 20_800.0,
            solver_configs: SolverConfig {
                n_ants: 32,
                n_iterations: 256,
                q0: 0.9,
                beta: 2.0,
            },
        },
        TestsConfig {
            filename: "./data/tsp_200_2",
            score_min: 35_985.0,
            score_max: 30_000.0,
            solver_configs: SolverConfig {
                n_ants: 32,
                n_iterations: 256,
                q0: 0.9,
                beta: 2.0,
            },
        },
        TestsConfig {
            filename: "./data/tsp_574_1",
            score_min: 40_000.0,
            score_max: 37_600.0,
            solver_configs: SolverConfig {
                n_ants: 128,
                n_iterations: 2048,
                q0: 0.9,
                beta: 3.0,
            },
        },
        TestsConfig {
            filename: "./data/tsp_1889_1",
            score_min: 378_069.0,
            score_max: 323_000.0,
            solver_configs: SolverConfig {
                n_ants: 256,
                n_iterations: 8,
                q0: 0.9,
                beta: 2.0,
            },
        },
        TestsConfig {
            filename: "./data/tsp_33810_1",
            score_min: 78_478_868.0,
            score_max: 67_700_000.0,
            solver_configs: SolverConfig {
                n_ants: 32,
                n_iterations: 1 * 0,
                q0: 0.9,
                beta: 2.0,
            },
        },
    ];
    let mut simple_results = Vec::new();
    let mut simple_answer = String::new();

    let mut results = Vec::new();
    let mut answer = String::new();

    let simple_answer_filename = "./answers/classic_pso_answer.txt";
    let answer_filename = "./answers/improved_acs_answer.txt";

    // Running simple tests
    for config in tests.iter() {
        println!(
            "{1} {}",
            "Running simple test on file:".white().bold(),
            config.filename
        );
        match run_simple_test(config) {
            Ok(result) => simple_results.push(result),
            Err(_) => println!("{}", "Test could not be completed due to an error.".red()),
        }
        println!("----------------------------------------");
    }

    // Running tests
    for config in tests.iter() {
        println!(
            "{1} {}",
            "Running test on file:".white().bold(),
            config.filename
        );
        match run_test(config) {
            Ok(result) => results.push(result),
            Err(_) => println!("{}", "Test could not be completed due to an error.".red()),
        }
        println!("----------------------------------------");
    }

    for i in 0..tests.len() {
        match &simple_results[i] {
            (TestResult::Passed5, tour) => {
                println!("Test {i} PSO: {}", "5 points!".yellow());
                simple_answer.push_str(&format!("{} {} {}\n", i, 5, format!("{:?}", tour)));
            }
            (TestResult::Passed7, tour) => {
                println!("Test {i} PSO: {}", "7 points!".green());
                simple_answer.push_str(&format!("{} {} {}\n", i, 7, format!("{:?}", tour)));
            }
            (TestResult::Failed, tour) => {
                println!("Test {i} PSO: {}", "0 points!".red());
                simple_answer.push_str(&format!("{} {} {}\n", i, 0, format!("{:?}", tour)));
            }
        }

        match &results[i] {
            (TestResult::Passed5, tour) => {
                println!("Test {i} ACS: {}", "5 points!".yellow());
                answer.push_str(&format!("{} {} {}\n", i, 5, format!("{:?}", tour)));
            }
            (TestResult::Passed7, tour) => {
                println!("Test {i} ACS: {}", "7 points!".green());
                answer.push_str(&format!("{} {} {}\n", i, 7, format!("{:?}", tour)));
            }
            (TestResult::Failed, tour) => {
                println!("Test {i} ACS: {}", "0 points!".red());
                answer.push_str(&format!("{} {} {}\n", i, 0, format!("{:?}", tour)));
            }
        }
    }

    let _ = fs::write(answer_filename, answer);
    let _ = fs::write(simple_answer_filename, simple_answer);
}
