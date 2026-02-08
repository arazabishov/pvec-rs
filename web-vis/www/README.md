# web-vis/www

Frontend for the RRB-Tree visualization. Uses D3.js for tree rendering, Tailwind CSS for styling, and Webpack for bundling.

The WASM package is imported from `../pkg` (built by `wasm-pack build` in the parent directory).

## Development

```bash
npm install
npm start        # webpack-dev-server at localhost:8080
npm run build    # production build to dist/
```
