# pvec-rs

Persistent vector library for Rust based on RRB-Trees (Relaxed Radix Balanced Trees). Published on [crates.io](https://crates.io/crates/pvec) as `pvec`. The design and performance evaluation are described in the [technical report](https://abishov.com/thesis).

## Architecture

The crate provides three vector types, all in `src/`:

- **RbVec** (`core/rbtree/`): Strictly balanced radix tree. Baseline for benchmarks, not for production use.
- **RrbVec** (`core/rrbtree/`): RRB-Tree with relaxed nodes carrying cumulative size tables. Supports O(log n) append, split, and clone.
- **PVec** (`lib.rs`): The public API. Starts as a standard `Vec<T>` ("Flat"), transitions to `RrbVec<T>` ("Tree") on first clone. Identical API to `std::Vec`.

Tree nodes use `SharedPtr<T>` (`core/sharedptr.rs`) which is `Rc` by default or `Arc` with the `arc` feature, enabling copy-on-write via `SharedPtr::make_mut()`.

Default branching factor is 32 (5 bits per level). The `small_branch` feature switches to 4 (2 bits per level) for testing.

## Invariants

These must be preserved in any change:

- **Balanced nodes**: All nodes except the rightmost at each level must be completely full (exactly `m` children).
- **Relaxed nodes**: May hold 1 to `m` children. Must carry a cumulative `sizes` array where `sizes[i]` = total elements in subtrees 0..=i.
- **Tail**: The rightmost leaf is stored separately as a tail buffer. When full (`m` elements), it is pushed into the tree. On pop with an empty tail, the rightmost tree leaf becomes the new tail.
- **Structural sharing**: Mutation uses path copying — only nodes on the root-to-leaf path are cloned (via `SharedPtr::make_mut()`). Nodes with refcount 1 are mutated in-place.
- **No gaps**: Within a node's children array, all `None` entries must be to the right of all `Some` entries.
- **Node enum size**: Node variants wrap their data in `SharedPtr` (not inline) so the `Node` enum stays pointer-sized. This is intentional — `RelaxedBranch` is much larger than `Branch`/`Leaf`.

## Workspace members

- `pvec` (root) — the library
- `web-vis` — WASM-based interactive tree visualization
- `benches-mem` — memory footprint measurement

## Feature flags

- `arc` — use `Arc` instead of `Rc` (required for thread safety)
- `rayon_iter` — parallel iterator support via Rayon (requires `arc`)
- `small_branch` — branching factor 4 instead of 32
- `serde_serializer` — JSON serialization of tree structure

## web-vis

WASM-based interactive RRB-Tree visualization. Two-part build: Rust→WASM backend + webpack frontend.

### Structure

- `web-vis/src/lib.rs` — Rust WASM bindings (uses `wasm-bindgen`, exposes push/split/append/serialize to JS)
- `web-vis/pkg/` — generated WASM package (output of `wasm-pack build`)
- `web-vis/www/` — webpack frontend (d3 visualization, Tailwind 4 CSS)

### Frontend stack

- **Bundler**: webpack 5 with `style-loader → css-loader → postcss-loader` chain
- **CSS**: Tailwind CSS 4 (CSS-first config via `@tailwindcss/postcss`, no `tailwind.config.js`)
- **Visualization**: d3 v7
- **Formatting**: Prettier 3 (config in `.prettierrc.json`)

### Building web-vis

```bash
cd web-vis && wasm-pack build           # build WASM package
cd www && npm install                   # install frontend deps
npm run build                           # production webpack build
npm start                               # dev server
```

### Rust toolchain

Keep Rust up to date (`rustup update`). `wasm-pack build` installs `wasm-bindgen-cli` via `cargo install`, which resolves latest transitive deps — these may require a newer Rust than what's installed.

### CSS conventions

- Custom utility classes used with `@apply` must be wrapped in `@utility` blocks (Tailwind 4 requirement)
- Regular CSS classes applied via `classList.add()` in JS stay as normal CSS rules
- Tailwind color references use CSS variables: `var(--color-gray-300)`, not `theme("colors.gray.300")`

## Building and testing

```bash
cargo test                              # run tests
cargo test --features=small_branch      # test with branching factor 4
cargo bench                             # sequential benchmarks (criterion)
cargo bench --features=arc,rayon_iter   # parallel benchmarks
```

## Key conventions

- Macro-driven code generation (`impl_vec!`, `clone_arr!`) to reduce duplication across vector types.
- Hot paths are annotated with `#[inline(always)]`.
- Index arithmetic uses bit manipulation: `(index >> shift) & MASK`.
- Relaxed radix search uses linear scan over the `sizes` array (not binary search) because `m` is small and linear scan is more cache-friendly.

## Complexity reference

| Operation | Vec | RbVec | RrbVec/PVec (tree) |
|-----------|-----|-------|--------------------|
| push/pop  | O(1) | O(log n) | O(1) amortized (tail) |
| get       | O(1) | O(log n) | O(log n) |
| clone     | O(n) | O(1) | O(1) |
| append    | O(n) | O(n) | O(m² log n) |
| split_off | O(n) | O(n) | O(m log n) |

With m=32, tree height ≤ 7 for 32-bit indices, so log-factor operations are effectively constant.
