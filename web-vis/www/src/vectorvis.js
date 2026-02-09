import { Vector } from "./vector.js";
import { RrbVec } from "./rrbvec.js";
import * as d3 from "d3";

let colorPicker = 0;
const colorPalette = ["#dc2626", "#ea580c", "#65a30d", "#059669", "#0891b2"];
const resolveColor = (color) => {
  const rgb = d3.rgb(color);
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${0.6})`;
};

// Global addr(UUID)->color cache. Structurally shared nodes keep their UUID
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
  static #instances = new Set();
  static #onChangeCallback = null;

  static onChange(callback) {
    VectorVis.#onChangeCallback = callback;
  }

  static count() {
    return VectorVis.#instances.size;
  }

  static #prune() {
    const live = new Set(
      [...VectorVis.#instances].flatMap((v) => [...v.nodeAddrs]),
    );
    for (const addr of colors.keys()) {
      if (!live.has(addr)) colors.delete(addr);
    }
  }

  static #notify() {
    if (VectorVis.#onChangeCallback) {
      VectorVis.#onChangeCallback(VectorVis.#instances.size);
    }
  }

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
    this.nodeAddrs = new Set();
  }

  id() {
    return `vec${this.vector.id()}`;
  }

  selector() {
    return `#${this.id()}`;
  }

  onMouseOver(listener) {
    this.listener = listener;
  }

  resize(size) {
    this.vector.resize(size);
    const rrbVec = this.vector.json();

    if (this.rrbVecVis === undefined) {
      this.rrbVecVis = new RrbVec(this.selector());
      this.rrbVecVis.onMouseOver(this.listener);
      this.color = resolveColor(
        colorPalette[colorPicker++ % colorPalette.length],
      );
    }

    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec, this.color);
  }

  update() {
    const rrbVec = this.vector.json();
    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec, this.color);
  }

  split(index) {
    const otherVector = this.vector.split(index);
    const otherVis = new VectorVis(otherVector);
    VectorVis.#instances.add(otherVis);
    VectorVis.#notify();
    return otherVis;
  }

  concatenate(other) {
    this.vector.concatenate(other.vector);
    other.dispose();

    const rrbVec = this.vector.json();
    this.#annotate(rrbVec);
    this.rrbVecVis.set(rrbVec, this.color);

    VectorVis.#prune();
    VectorVis.#notify();
  }

  size() {
    return this.vector.size();
  }

  dispose() {
    VectorVis.#instances.delete(this);
  }

  #annotate(rrbVec) {
    this.nodeAddrs = new Set();
    if (rrbVec.tree.root_len > 0) {
      annotateColors(rrbVec.tree.root, this.color ?? "none", this.nodeAddrs);
    }
  }
}
