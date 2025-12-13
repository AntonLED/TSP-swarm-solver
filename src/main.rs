mod tsp_data;
mod tsp_solvers;

use std::path::Path;
use tsp_data::TspData;
use tsp_solvers::AcsTspSolver;

fn main() {
    // let filename = "./data/tsp_51_1";
    // let filename = "./data/tsp_100_3";
    // let filename = "./data/tsp_200_2";
    // let filename = "./data/tsp_574_1";
    // let filename = "./data/tsp_1889_1";
    let filename = "./data/tsp_33810_1";

    // Проверка наличия файла перед запуском
    if !Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found.", filename);
        eprintln!(
            "Please make sure you have a 'data' folder next to Cargo.toml containing the test file."
        );
        return;
    }

    // Загрузка данных
    match TspData::new(filename) {
        Ok(data) => {
            println!("Loaded {} cities from {}", data.n, filename);

            let mut solver = AcsTspSolver::new(data.clone(), 40, 64, 0.9, 2.0);

            solver.run();

            println!("Final Best Tour Length: {:.2}", solver.best_score);
            // println!("Tour: {:?}", solver.best_tour);
        }
        Err(e) => eprintln!("Error loading data: {}", e),
    }
}
