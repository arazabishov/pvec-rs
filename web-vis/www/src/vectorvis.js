import { Vector } from "./vector.js";
import { RrbVec } from "./rrbvec.js";
import * as d3 from "d3";

// Pairs a Vector (WASM data) with an RrbVec (d3 rendering) and manages
// per-node color assignment for structurally shared trees.
export class VectorVis {
  static #palette = ["#dc2626", "#ea580c", "#65a30d", "#059669", "#0891b2"];

  // Live instances, used by #prune to determine which node addresses are still reachable.
  static #instances = new Set();
  static #onChangeCallback = null;

  // Monotonic counter into #palette; never resets, so colors cycle predictably
  // even as instances are disposed.
  static #colorCursor = 0;

  // Maps node addr (UUID) to its assigned rgba color string. Nodes retain their
  // UUID across clone/concatenate, so shared structure keeps its original color.
  static #colors = new Map();

  // Registers a callback fired whenever the instance count changes (create, split, concatenate).
  static onChange(callback) {
    VectorVis.#onChangeCallback = callback;
  }

  static count() {
    return VectorVis.#instances.size;
  }

  // Factory. Sets the WASM size eagerly but defers d3 initialization
  // until resize() is called after the DOM element exists.
  static create(initialSize) {
    const vector = Vector.create();

    if (initialSize !== undefined) {
      vector.resize(initialSize);
    }

    const vis = new VectorVis(vector);
    VectorVis.#instances.add(vis);
    VectorVis.#notify();
    return vis;
  }

  constructor(vector) {
    this.vector = vector;
    // Node addresses seen during the last #annotate pass, fed to #prune.
    this.addresses = new Set();
  }

  // DOM element id, e.g. "vec<uuid>".
  id() {
    return `vec${this.vector.id()}`;
  }

  // CSS selector targeting this instance's DOM element.
  selector() {
    return `#${this.id()}`;
  }

  // Sets the mouse event handler forwarded to RrbVec's leaf nodes.
  onMouseOver(listener) {
    this.listener = listener;
  }

  // Resizes the underlying WASM vector and re-renders. On first call,
  // lazily creates the RrbVec (d3 SVG) and assigns a color from the palette.
  resize(size) {
    this.vector.resize(size);
    const rrbVec = this.vector.json();

    if (this.rrbVecVis === undefined) {
      this.rrbVecVis = new RrbVec(this.selector());
      this.rrbVecVis.onMouseOver(this.listener);
      this.color = VectorVis.#nextColor();
    }

    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec);
  }

  // Advances the palette cursor and returns an rgba string with 0.6 opacity.
  static #nextColor() {
    const colorIndex = VectorVis.#colorCursor++ % VectorVis.#palette.length;
    const colorHex = VectorVis.#palette[colorIndex];
    const color = d3.rgb(colorHex);

    return `rgba(${color.r}, ${color.g}, ${color.b}, ${0.6})`;
  }

  // Re-fetches tree JSON from WASM and re-renders without changing size.
  update() {
    const rrbVec = this.vector.json();
    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec);
  }

  // Splits the WASM vector at index. Returns a new registered VectorVis
  // for the right half. Does NOT re-render this instance (caller should call update).
  split(index) {
    const otherVector = this.vector.split(index);
    const otherVis = new VectorVis(otherVector);
    VectorVis.#instances.add(otherVis);
    VectorVis.#notify();
    return otherVis;
  }

  // Appends other's data into this vector, disposes other, re-renders,
  // and prunes stale color cache entries.
  concatenate(other) {
    this.vector.concatenate(other.vector);
    other.dispose();

    const rrbVec = this.vector.json();
    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec);

    VectorVis.#prune();
    VectorVis.#notify();
  }

  // Removes stale entries from #colors by collecting all addresses still
  // reachable from live instances and deleting the rest.
  static #prune() {
    const live = new Set(
      [...VectorVis.#instances].flatMap((v) => [...v.addresses])
    );
    for (const addr of VectorVis.#colors.keys()) {
      if (!live.has(addr)) {
        VectorVis.#colors.delete(addr);
      }
    }
  }

  static #notify() {
    if (VectorVis.#onChangeCallback) {
      VectorVis.#onChangeCallback(VectorVis.#instances.size);
    }
  }

  size() {
    return this.vector.size();
  }

  // Removes this instance from the live set. Does not touch WASM or DOM.
  dispose() {
    VectorVis.#instances.delete(this);
  }

  // Stamps a color on every node in the tree JSON and on the tail node.
  // Existing nodes (by addr) keep their cached color; new nodes get this
  // instance's color. Collects all visited addresses for later pruning.
  #annotate(rrbVec) {
    this.addresses = new Set();
    const color = this.color ?? "none";

    if (rrbVec.tree.root_len > 0) {
      VectorVis.#annotateColors(rrbVec.tree.root, color, this.addresses);
    }

    rrbVec.tail.color = color;
  }

  // Iterative tree walk that assigns colors to nodes. Nodes already in the
  // #colors cache (structurally shared from another vector) keep their original
  // color; unseen nodes are assigned the provided color and cached.
  static #annotateColors(root, color, addresses) {
    const stack = [root];

    while (stack.length > 0) {
      const node = stack.pop();

      if (!node) {
        continue;
      }

      if (VectorVis.#colors.has(node.addr)) {
        node.color = VectorVis.#colors.get(node.addr);
      } else {
        node.color = color;
        VectorVis.#colors.set(node.addr, color);
      }

      addresses.add(node.addr);

      const children = node.relaxedBranch || node.branch;
      if (children) {
        for (const child of children) {
          stack.push(child);
        }
      }
    }
  }
}
