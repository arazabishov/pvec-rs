extern crate csv;

use csv::Writer;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const STD_VEC: &str = "std-vec";

const IM_RS_VECTOR_BALANCED: &str = "im-rs-vector-balanced";
const IM_RS_VECTOR_RELAXED: &str = "im-rs-vector-relaxed";

const PVEC_RRBVEC_BALANCED: &str = "pvec-rrbvec-balanced";
const PVEC_RRBVEC_RELAXED: &str = "pvec-rrbvec-relaxed";
const PVEC_STD: &str = "pvec-std";

const RRBVEC: &str = "rrbvec";
const RBVEC: &str = "rbvec";

struct Bench<'a> {
    name: &'a str,
    types: Vec<&'a str>,
    sizes: Vec<usize>,
}

struct BenchRunner {
    path: String,
}

impl BenchRunner {
    fn new(exe_path: PathBuf) -> Self {
        let benches_path = exe_path.join("benches").to_str().map(|it| it.to_string());

        BenchRunner {
            path: benches_path.unwrap(),
        }
    }

    fn execute(&self, bench: Option<&str>, vec: Option<&str>, n: Option<&usize>) -> usize {
        let mut command = Command::new("/usr/bin/time");
        command.arg("-l").arg(self.path.clone());

        if let Some(bench) = bench {
            command.arg(bench);
        }

        if let Some(vec) = vec {
            command.arg(vec);
        }

        if let Some(n) = n {
            command.arg(n.to_string());
        }

        // The output of the benches process is piped into /dev/null
        // to simplify parsing of the 'time' command output
        command.arg("> /dev/null");

        let output = command.output().expect("Failed to run benches.");

        // For some reason, even when executed successfully, results of the
        // child process are piped into the error stream.
        let output_str = std::str::from_utf8(&output.stderr)
            .expect("Couldn't read the output from a child process.");

        // The maximum resident set size is located at line 2.
        // Hence, we skip the first one.
        let maximum_resident_set_size_report_line = output_str.lines().skip(1).next().unwrap();

        // Tokenizing the line by splitting it using space as delimiter.
        let maximum_resident_set_size_tokens = maximum_resident_set_size_report_line
            .split_whitespace()
            .collect::<Vec<&str>>();

        // The first token is the actual size, then we take
        // it and parse to a number type.
        let maximum_resident_set_size = maximum_resident_set_size_tokens
            .first()
            .unwrap()
            .parse::<usize>()
            .unwrap();

        maximum_resident_set_size
    }

    // Runs the benchmark process without any arguments
    // to measure the memory footprint of the process.
    fn baseline(&self) -> usize {
        self.execute(None, None, None)
    }

    fn run(&self, bench: &str, vec: &str, n: &usize) -> usize {
        self.execute(Some(bench), Some(vec), Some(n))
    }
}

fn execute() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe().map(|mut path| {
        path.pop();
        path
    })?;

    let report_path = exe_path.clone().join("report");
    let bench_runner = BenchRunner::new(exe_path.clone());

    // Baseline is the memory footprint of the process
    // without running any benchmarks. Its value is subtracted
    // from the result of a benchmark to get a sense of
    // how much memory vector consumes
    let baseline = bench_runner.baseline();

    let update_clone = Bench {
        name: "update_clone",
        types: vec![
            STD_VEC,
            RBVEC,
            PVEC_STD,
            PVEC_RRBVEC_BALANCED,
            IM_RS_VECTOR_BALANCED,
        ],
        sizes: vec![
            20, 40, 60, 80, 100, 200, 400, 600, 800, 1_000, 2_000, 4_000, 6_000, 8_000, 10_000,
            20_000, 40_000, 60_000,
        ],
    };

    let push = Bench {
        name: "push",
        types: vec![
            STD_VEC,
            RBVEC,
            PVEC_STD,
            PVEC_RRBVEC_BALANCED,
            IM_RS_VECTOR_BALANCED,
        ],
        sizes: vec![
            20, 40, 60, 80, 100, 200, 400, 600, 800, 1_000, 2_000, 4_000, 6_000, 8_000, 10_000,
            20_000, 40_000, 60_000, 80_000, 100_000, 200_000, 400_000, 600_000, 800_000, 1_000_000,
        ],
    };

    let append = Bench {
        name: "append",
        types: vec![
            STD_VEC,
            RBVEC,
            RRBVEC,
            PVEC_STD,
            PVEC_RRBVEC_RELAXED,
            IM_RS_VECTOR_RELAXED,
        ],
        sizes: vec![
            20, 40, 60, 80, 100, 200, 400, 600, 800, 1_000, 2_000, 4_000, 6_000, 8_000, 10_000,
            20_000, 40_000, 60_000, 80_000, 100_000, 200_000, 400_000, 600_000, 800_000, 1_000_000,
        ],
    };

    let benchmarks = vec![update_clone, push, append];

    for bench in benchmarks.iter() {
        println!("Running \"{}\" bench", bench.name);

        for vec in bench.types.iter() {
            // Make sure that the directory for the report exists.
            let bench_report_dir = report_path.clone().join(bench.name);
            fs::create_dir_all(bench_report_dir.clone())?;

            let bench_report_path = bench_report_dir.join(format!("{}.csv", vec));
            let mut wtr = Writer::from_path(bench_report_path)?;

            for n in bench.sizes.iter() {
                let maximum_resident_set_size = bench_runner.run(&bench.name, vec, n);
                let mut maximum_resident_set_size_diff = 0;

                if maximum_resident_set_size > baseline {
                    maximum_resident_set_size_diff = maximum_resident_set_size - baseline;
                }

                let n_str: &str = &n.to_string();
                let maximum_resident_set_size_diff_str =
                    &maximum_resident_set_size_diff.to_string();

                wtr.write_record(&[n_str, maximum_resident_set_size_diff_str])?;
            }

            wtr.flush()?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = execute() {
        println!("Error running the benchmark suite: {}", err);
        std::process::exit(1);
    }
}
