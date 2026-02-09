import { RrbVec } from "./rrbvec.js";
import * as d3 from "d3";

export class Vector {
  constructor(id, wasmDecorator) {
    this._id = id;
    this.wasmDecorator = wasmDecorator;
  }

  id() {
    return this._id;
  }

  setSize(size) {
    this.wasmDecorator.setVecSize(this._id, size);
  }

  splitAt(index) {
    const newVecId = this.wasmDecorator.splitOffVec(this._id, index);
    return new Vector(newVecId, this.wasmDecorator);
  }

  concatenate(other) {
    this.wasmDecorator.concatenate(this._id, other.id());
  }

  size() {
    return this.wasmDecorator.getVecSize(this._id);
  }

  json() {
    return JSON.parse(this.wasmDecorator.get(this._id));
  }
}

let colorPicker = 0;
const colorPalette = ["#dc2626", "#ea580c", "#65a30d", "#059669", "#0891b2"];
const resolveColor = (color) => {
  const rgb = d3.rgb(color);
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${0.6})`;
};

// Global addr(UUID)→color cache. Structurally shared nodes keep their UUID
// and thus their original color after concatenation.
const colors = new Map();

// Resolve and assign a color to each node in the raw tree data. Collects
// node addrs for later pruning.
const annotateColors = (node, color, nodeAddrs) => {
  if (!node) return;

  if (colors.has(node.addr)) {
    node.color = colors.get(node.addr);
  } else {
    node.color = color;
    colors.set(node.addr, color);
  }
  nodeAddrs.add(node.addr);

  const children = node.relaxedBranch || node.branch;
  if (children) {
    children.forEach((child) => annotateColors(child, color, nodeAddrs));
  }
};

export class VectorVis {
  constructor(vector) {
    this.vector = vector;
    this.nodeAddrs = new Set();
  }

  id() {
    return `vec${this.vector.id()}`;
  }

  vec() {
    return this.vector;
  }

  selector() {
    return `#${this.id()}`;
  }

  setOnMouseOverListener(listener) {
    this.listener = listener;
  }

  update() {
    const rrbVec = this.vector.json();
    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec, this.color);
  }

  concatenate(two) {
    this.vector.concatenate(two.vector);
    const rrbVec = this.vector.json();
    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec, this.color);
  }

  setSize(size) {
    this.vector.setSize(size);
    const rrbVec = this.vector.json();

    if (this.rrbVecVis === undefined) {
      this.rrbVecVis = new RrbVec(this.selector());
      this.rrbVecVis.setOnMouseOverListener(this.listener);
      this.color = resolveColor(
        colorPalette[colorPicker++ % colorPalette.length]
      );
    }

    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec, this.color);
  }

  size() {
    return this.vector.size();
  }

  // Clean up resources for removed vectors. Call with remaining active vectors.
  static prune(activeVectors) {
    const live = new Set(activeVectors.flatMap((v) => [...v.nodeAddrs]));
    for (const addr of colors.keys()) {
      if (!live.has(addr)) colors.delete(addr);
    }
  }

  #annotate(rrbVec) {
    this.nodeAddrs = new Set();
    if (rrbVec.tree.root_len > 0) {
      annotateColors(rrbVec.tree.root, this.color ?? "none", this.nodeAddrs);
    }
  }
}
