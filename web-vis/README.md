# web-vis

Interactive visualization of the RRB-Tree structure used by [pvec](https://crates.io/crates/pvec). The Rust side is compiled to WebAssembly via [wasm-pack](https://github.com/rustwasm/wasm-pack), and the frontend uses D3.js to render the tree.

Live demo: [pvec-rs.abishov.com/web-vis/](https://pvec-rs.abishov.com/web-vis/)

## How it works

The WASM module (`src/lib.rs`) holds a `HashMap<VecId, RrbVec<usize>>` as global state and exposes operations to JavaScript:

- `push_vec` / `remove_vec` — create or remove vectors
- `set_vec_size` / `get_vec_size` — grow or shrink a vector by pushing elements or splitting
- `split_off_vec` — split a vector at an index, producing two vectors
- `clone_vec` — O(1) clone via structural sharing
- `concatenate` — append one vector into another via `append`
- `get` — serialize an RRB-Tree to JSON for rendering

The pvec crate is compiled with `small_branch` (branching factor 4) and `serde_serializer` so that tree structures are small enough to visualize and can be serialized to JSON.

## Note on Apple Silicon

When installing `wasm-pack` on Apple Silicon, use the Rosetta 2 translation layer to avoid compatibility issues.

## Build

```bash
# Build the WASM package
wasm-pack build

# Install frontend dependencies and start dev server
cd www
npm install
npm start
```

The dev server runs at `http://localhost:8080`.

## Deploy

The frontend is deployed to Cloudflare Pages. See the GitHub Actions workflow in `.github/workflows/` for details.
