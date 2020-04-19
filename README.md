# pvec-rs

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ArazAbishov/pvec-rs/build)](https://github.com/ArazAbishov/pvec-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/pvec)](https://crates.io/crates/pvec)
[![API](https://docs.rs/pvec/badge.svg?version=0.2.0)](https://docs.rs/pvec/0.2.0/pvec/)

A persistent vector implementation based on RRB-Tree for Rust, inspired by the blog post of Niko Matsakis - [In Rust, ordinary vectors are values](http://smallcultfollowing.com/babysteps/blog/2018/02/01/in-rust-ordinary-vectors-are-values/). This project offers a general-purpose, persistent vector with good performance across all operations, including efficient clone, concatenation, and splitting.

One of the vector types - **PVec**, explores an idea of starting out as the standard vector and spills to the tree representation only when cloned to offer the best possible performance. The API of methods provided by pvec-rs is identical to the standard vector, reducing the friction of using the library. Another notable feature is the out of the box support for [Rayon](https://github.com/rayon-rs/rayon).

The performance evaluation of the library is provided in the [technical report](https://abishov.com/thesis). PVec is available on [crates.io](https://crates.io/crates/pvec), and API documentation is available on [docs.rs](https://docs.rs/pvec/0.2.0/pvec/).

## Examples

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