# pvec-rs

[![GitHub Workflow Status](https://github.com/ArazAbishov/pvec-rs/workflows/build/badge.svg)](https://github.com/ArazAbishov/pvec-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/pvec)](https://crates.io/crates/pvec)
[![API](https://docs.rs/pvec/badge.svg?version=0.2.1)](https://docs.rs/pvec/0.2.1/pvec/)

A persistent vector implementation based on RRB-Tree for Rust, inspired by the blog post of Niko Matsakis - [In Rust, ordinary vectors are values](http://smallcultfollowing.com/babysteps/blog/2018/02/01/in-rust-ordinary-vectors-are-values/). This project offers a general-purpose, persistent vector with good performance across all operations, including efficient clone, concatenation, and splitting.

One of the vector types - **PVec**, explores an idea of starting out as the standard vector and spills to the tree representation only when cloned to offer the best possible performance. The API of methods provided by pvec-rs is identical to the standard vector, reducing the friction of using the library. Another notable feature is the out of the box support for [Rayon](https://github.com/rayon-rs/rayon).

The performance evaluation of the library is provided in the [technical report](https://abishov.com/thesis). PVec is available on [crates.io](https://crates.io/crates/pvec), and API documentation is available on [docs.rs](https://docs.rs/pvec/0.2.1/pvec/).

## Example

A demonstration of using **PVec**:

```rust
extern crate pvec;

use pvec::PVec;

fn example_pvec(size: usize) {
    let mut vec = PVec::new();
    // ^ backed by the standard vector internally

    for i in 0..size {
        vec.push(i);
    }

    let cln = vec.clone();
    // ^ transitions to RrbVec internally,
    // with consequent clones that cost O(1)

    let res: PVec<usize> = vec.into_par_iter()
        .map(|it| it + 1)
        .collect();
    // ^ processing vector in parallel and
    // collecting results
}
```

## Benchmarks

### Runtime

Runtime benchmarks are subdivided into sequential and parallel groups. The framework used to run benchmark is [criterion](https://github.com/bheisler/criterion.rs), which can generate an HTML report with charts if you have gnuplot pre-installed.

```bash
# running all sequential benches
cargo bench

# running parallel benches
cargo bench --features=arc,rayon_iter
```

The report can be found at `target/criterion/report/index.html`. To avoid running benchmarks for hours, pass the `--sample-size=10` option to reduce the sample count.

### Memory

Memory footprint is measured using a custom binary crate - **benches-mem**. This binary runs benchmarks from the **benches** crate through the [time](https://www.freebsd.org/cgi/man.cgi?query=time) util, capturing the peak memory usage of the process. The report is placed at `target/release/report`. Note, these benchmarks can be executed only on macOS at the moment, as `time` behaves differently on mac and linux.

```bash
cd benches-mem && sh bench.sh
```

## License

```
MIT License

Copyright (c) 2020 Araz Abishov

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```