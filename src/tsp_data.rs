use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Clone)]
pub struct TspData {
    pub coords: Vec<(f64, f64)>,
    pub dist_matrix: Vec<f64>,
    pub n: usize,
}

impl TspData {
    pub fn new(filename: &str) -> io::Result<Self> {
        let path = Path::new(filename);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut n = 0;
        let mut coords = Vec::new();

        if let Some(Ok(line)) = lines.next() {
            if let Ok(val) = line.trim().parse::<usize>() {
                n = val;
            }
        }

        for line in lines {
            if let Ok(l) = line {
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 2 {
                    let x = parts[0].parse::<f64>().unwrap_or(0.0);
                    let y = parts[1].parse::<f64>().unwrap_or(0.0);
                    coords.push((x, y));
                }
            }
        }

        if coords.len() != n {
            eprintln!("Warning: Expected {} points, found {}", n, coords.len());
            n = coords.len();
        }

        // Вычисляем матрицу расстояний
        let mut dist_matrix = vec![0.0; n * n];
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = coords[i].0 - coords[j].0;
                let dy = coords[i].1 - coords[j].1;
                let dist = (dx * dx + dy * dy).sqrt();
                dist_matrix[i * n + j] = dist;
                dist_matrix[j * n + i] = dist;
            }
        }

        Ok(TspData {
            coords,
            dist_matrix,
            n,
        })
    }

    #[inline(always)]
    pub fn dist(&self, i: usize, j: usize) -> f64 {
        unsafe { *self.dist_matrix.get_unchecked(i * self.n + j) }
    }

    pub fn calculate_tour_length(&self, tour: &[usize]) -> f64 {
        let mut length = 0.0;
        for i in 0..tour.len() {
            let u = tour[i];
            let v = tour[(i + 1) % tour.len()];
            length += self.dist(u, v);
        }
        length
    }
}
