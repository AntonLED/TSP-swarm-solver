use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Clone)]
pub struct TspData {
    pub coords: Vec<(f64, f64)>,
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

        Ok(TspData { coords, n })
    }

    #[inline(always)]
    pub fn dist(&self, i: usize, j: usize) -> f64 {
        let (x1, y1) = self.coords[i];
        let (x2, y2) = self.coords[j];
        let dx = x1 - x2;
        let dy = y1 - y2;
        (dx * dx + dy * dy).sqrt()
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
