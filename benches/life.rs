use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::iter::repeat;
use std::num::Wrapping;
use std::sync::Arc;

use rayon::iter::ParallelBridge;
use rayon::prelude::*;

use criterion::*;

mod stdvec {
    use super::*;
    #[derive(PartialEq, Eq, Clone, Debug)]
    pub struct Board {
        board: Vec<bool>,
        survive: Arc<Vec<usize>>,
        born: Arc<Vec<usize>>,
        rows: usize,
        cols: usize,
    }

    impl Board {
        pub fn new(rows: usize, cols: usize) -> Board {
            let born = vec![3];
            let survive = vec![2, 3];

            Board::new_with_custom_rules(rows, cols, born, survive)
        }

        fn new_with_custom_rules(
            rows: usize,
            cols: usize,
            born: Vec<usize>,
            survive: Vec<usize>,
        ) -> Board {
            let new_board = repeat(false).take(rows * cols).collect();

            Board {
                board: new_board,
                born: Arc::new(born),
                survive: Arc::new(survive),
                rows: rows,
                cols: cols,
            }
        }

        fn len(&self) -> usize {
            self.rows * self.cols
        }

        fn next_board(&self, new_board: Vec<bool>) -> Board {
            assert!(new_board.len() == self.len());

            Board {
                board: new_board,
                born: self.born.clone(),
                survive: self.survive.clone(),
                rows: self.rows,
                cols: self.cols,
            }
        }

        pub fn random(&self) -> Board {
            let new_brd = thread_rng()
                .sample_iter(&Standard)
                .take(self.len())
                .collect();

            self.next_board(new_brd)
        }

        pub fn next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        pub fn parallel_next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .into_par_iter()
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        pub fn par_bridge_next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .par_bridge()
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        fn cell_live(&self, x: usize, y: usize) -> bool {
            !(x >= self.cols || y >= self.rows) && self.board[y * self.cols + x]
        }

        fn living_neighbors(&self, x: usize, y: usize) -> usize {
            let Wrapping(x_1) = Wrapping(x) - Wrapping(1);
            let Wrapping(y_1) = Wrapping(y) - Wrapping(1);
            let neighbors = [
                self.cell_live(x_1, y_1),
                self.cell_live(x, y_1),
                self.cell_live(x + 1, y_1),
                self.cell_live(x_1, y + 0),
                self.cell_live(x + 1, y + 0),
                self.cell_live(x_1, y + 1),
                self.cell_live(x, y + 1),
                self.cell_live(x + 1, y + 1),
            ];
            neighbors.iter().filter(|&x| *x).count()
        }

        fn successor_cell(&self, cell: usize) -> bool {
            self.successor(cell % self.cols, cell / self.cols)
        }

        fn successor(&self, x: usize, y: usize) -> bool {
            let neighbors = self.living_neighbors(x, y);
            if self.cell_live(x, y) {
                self.survive.contains(&neighbors)
            } else {
                self.born.contains(&neighbors)
            }
        }
    }

    pub fn generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.next_generation();
        }
    }

    pub fn parallel_generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.parallel_next_generation();
        }
    }

    pub fn par_bridge_generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.par_bridge_next_generation();
        }
    }

    #[test]
    fn test_life() {
        let mut brd1 = Board::new(200, 200).random();
        let mut brd2 = brd1.clone();

        for _ in 0..100 {
            brd1 = brd1.next_generation();
            brd2 = brd2.parallel_next_generation();

            assert_eq!(brd1, brd2);
        }
    }
}

mod rrbvec {
    use super::*;
    use crate::pvec::core::RrbVec;

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub struct Board {
        board: RrbVec<bool>,
        survive: RrbVec<usize>,
        born: RrbVec<usize>,
        rows: usize,
        cols: usize,
    }

    impl Board {
        pub fn new(rows: usize, cols: usize) -> Board {
            let mut born = RrbVec::new();
            born.push(3);

            let mut survive = RrbVec::new();
            survive.push(2);
            survive.push(3);

            Board::new_with_custom_rules(rows, cols, born, survive)
        }

        fn new_with_custom_rules(
            rows: usize,
            cols: usize,
            born: RrbVec<usize>,
            survive: RrbVec<usize>,
        ) -> Board {
            let mut new_board = RrbVec::new();

            for i in repeat(false).take(rows * cols) {
                new_board.push(i);
            }

            Board {
                board: new_board,
                born: born,
                survive: survive,
                rows: rows,
                cols: cols,
            }
        }

        fn len(&self) -> usize {
            self.rows * self.cols
        }

        fn next_board(&self, new_board: RrbVec<bool>) -> Board {
            assert!(new_board.len() == self.len());

            Board {
                board: new_board,
                born: self.born.clone(),
                survive: self.survive.clone(),
                rows: self.rows,
                cols: self.cols,
            }
        }

        pub fn random(&self) -> Board {
            let mut new_brd = RrbVec::new();

            for i in thread_rng().sample_iter(&Standard).take(self.len()) {
                new_brd.push(i);
            }

            self.next_board(new_brd)
        }

        pub fn next_generation(&self) -> Board {
            let mut new_brd = RrbVec::new();
            for i in (0..self.len()).map(|cell| self.successor_cell(cell)) {
                new_brd.push(i);
            }

            self.next_board(new_brd)
        }

        pub fn parallel_next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .into_par_iter()
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        pub fn par_bridge_next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .par_bridge()
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        fn cell_live(&self, x: usize, y: usize) -> bool {
            !(x >= self.cols || y >= self.rows) && self.board[y * self.cols + x]
        }

        fn living_neighbors(&self, x: usize, y: usize) -> usize {
            let Wrapping(x_1) = Wrapping(x) - Wrapping(1);
            let Wrapping(y_1) = Wrapping(y) - Wrapping(1);
            let neighbors = [
                self.cell_live(x_1, y_1),
                self.cell_live(x, y_1),
                self.cell_live(x + 1, y_1),
                self.cell_live(x_1, y + 0),
                self.cell_live(x + 1, y + 0),
                self.cell_live(x_1, y + 1),
                self.cell_live(x, y + 1),
                self.cell_live(x + 1, y + 1),
            ];
            neighbors.iter().filter(|&x| *x).count()
        }

        fn successor_cell(&self, cell: usize) -> bool {
            self.successor(cell % self.cols, cell / self.cols)
        }

        fn successor(&self, x: usize, y: usize) -> bool {
            let neighbors = self.living_neighbors(x, y);
            if self.cell_live(x, y) {
                for i in 0..self.survive.len() {
                    if self
                        .survive
                        .get(i)
                        .map(|val| *val == neighbors)
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }

                false
            } else {
                for i in 0..self.survive.len() {
                    if self
                        .born
                        .get(i)
                        .map(|val| *val == neighbors)
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.next_generation();
        }
    }

    pub fn parallel_generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.parallel_next_generation();
        }
    }

    pub fn par_bridge_generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.par_bridge_next_generation();
        }
    }

    #[test]
    fn test_life() {
        let mut brd1 = Board::new(200, 200).random();
        let mut brd2 = brd1.clone();

        for _ in 0..100 {
            brd1 = brd1.next_generation();
            brd2 = brd2.parallel_next_generation();

            assert_eq!(brd1, brd2);
        }
    }
}

mod pvec {
    use super::*;
    use crate::pvec::PVec;

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub struct Board {
        board: PVec<bool>,
        survive: PVec<usize>,
        born: PVec<usize>,
        rows: usize,
        cols: usize,
    }

    impl Board {
        pub fn new(rows: usize, cols: usize) -> Board {
            let mut born = PVec::new();
            born.push(3);

            let mut survive = PVec::new();
            survive.push(2);
            survive.push(3);

            Board::new_with_custom_rules(rows, cols, born, survive)
        }

        fn new_with_custom_rules(
            rows: usize,
            cols: usize,
            born: PVec<usize>,
            survive: PVec<usize>,
        ) -> Board {
            let mut new_board = PVec::new();

            for i in repeat(false).take(rows * cols) {
                new_board.push(i);
            }

            Board {
                board: new_board,
                born: born,
                survive: survive,
                rows: rows,
                cols: cols,
            }
        }

        fn len(&self) -> usize {
            self.rows * self.cols
        }

        fn next_board(&self, new_board: PVec<bool>) -> Board {
            assert!(new_board.len() == self.len());

            Board {
                board: new_board,
                born: self.born.clone(),
                survive: self.survive.clone(),
                rows: self.rows,
                cols: self.cols,
            }
        }

        pub fn random(&self) -> Board {
            let mut new_brd = PVec::new();

            for i in thread_rng().sample_iter(&Standard).take(self.len()) {
                new_brd.push(i);
            }

            self.next_board(new_brd)
        }

        pub fn next_generation(&self) -> Board {
            let mut new_brd = PVec::new();
            for i in (0..self.len()).map(|cell| self.successor_cell(cell)) {
                new_brd.push(i);
            }

            self.next_board(new_brd)
        }

        pub fn parallel_next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .into_par_iter()
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        pub fn par_bridge_next_generation(&self) -> Board {
            let new_brd = (0..self.len())
                .par_bridge()
                .map(|cell| self.successor_cell(cell))
                .collect();

            self.next_board(new_brd)
        }

        fn cell_live(&self, x: usize, y: usize) -> bool {
            !(x >= self.cols || y >= self.rows) && self.board[y * self.cols + x]
        }

        fn living_neighbors(&self, x: usize, y: usize) -> usize {
            let Wrapping(x_1) = Wrapping(x) - Wrapping(1);
            let Wrapping(y_1) = Wrapping(y) - Wrapping(1);
            let neighbors = [
                self.cell_live(x_1, y_1),
                self.cell_live(x, y_1),
                self.cell_live(x + 1, y_1),
                self.cell_live(x_1, y + 0),
                self.cell_live(x + 1, y + 0),
                self.cell_live(x_1, y + 1),
                self.cell_live(x, y + 1),
                self.cell_live(x + 1, y + 1),
            ];
            neighbors.iter().filter(|&x| *x).count()
        }

        fn successor_cell(&self, cell: usize) -> bool {
            self.successor(cell % self.cols, cell / self.cols)
        }

        fn successor(&self, x: usize, y: usize) -> bool {
            let neighbors = self.living_neighbors(x, y);
            if self.cell_live(x, y) {
                for i in 0..self.survive.len() {
                    if self
                        .survive
                        .get(i)
                        .map(|val| *val == neighbors)
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }

                false
            } else {
                for i in 0..self.survive.len() {
                    if self
                        .born
                        .get(i)
                        .map(|val| *val == neighbors)
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.next_generation();
        }
    }

    pub fn parallel_generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.parallel_next_generation();
        }
    }

    pub fn par_bridge_generations(board: Board, gens: usize) {
        let mut brd = board;
        for _ in 0..gens {
            brd = brd.par_bridge_next_generation();
        }
    }

    #[test]
    fn test_life() {
        let mut brd1 = Board::new(200, 200).random();
        let mut brd2 = brd1.clone();

        for _ in 0..100 {
            brd1 = brd1.next_generation();
            brd2 = brd2.parallel_next_generation();

            assert_eq!(brd1, brd2);
        }
    }
}

fn generations(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $module:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| $module::generations($module::Board::new(200, 200).random(), *n));
            });
        };
    }

    let mut group = criterion.benchmark_group("generations");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![10, 50, 100];
    for p in params.iter() {
        make_bench!(group, p, stdvec, "std");
        make_bench!(group, p, rrbvec, "rrbvec");
        make_bench!(group, p, pvec, "pvec");
    }

    group.finish();
}

fn parallel_generations(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $module:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| {
                    $module::parallel_generations($module::Board::new(200, 200).random(), *n)
                });
            });
        };
    }

    let mut group = criterion.benchmark_group("parallel_generations");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![10, 50, 100];
    for p in params.iter() {
        make_bench!(group, p, stdvec, "std");
        make_bench!(group, p, rrbvec, "rrbvec");
        make_bench!(group, p, pvec, "pvec");
    }

    group.finish();
}

fn as_parallel_generations(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $module:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| {
                    $module::par_bridge_generations($module::Board::new(200, 200).random(), *n)
                });
            });
        };
    }

    let mut group = criterion.benchmark_group("as_parallel_generations");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![10, 50, 100];
    for p in params.iter() {
        make_bench!(group, p, stdvec, "std");
        make_bench!(group, p, rrbvec, "rrbvec");
        make_bench!(group, p, pvec, "pvec");
    }

    group.finish();
}

criterion_group!(
    benches,
    generations,
    parallel_generations,
    as_parallel_generations
);
