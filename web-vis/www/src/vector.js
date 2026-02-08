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

// Remove stale entries from the color cache. Call after removing vectors.
export const pruneColors = (activeVectors) => {
  const live = new Set(activeVectors.flatMap((v) => [...v.rrbVecVis.nodeAddrs]));
  for (const addr of colors.keys()) {
    if (!live.has(addr)) colors.delete(addr);
  }
};
export class VectorVis {
  constructor(vector) {
    this.vector = vector;

    // Returns cached color for known addrs, otherwise assigns this vector's
    // color. Called with null for tail elements (always current vector's color).
    this.colorResolver = (node) => {
      if (!node) {
        return this.rrbVecVisColor ?? "none";
      }

      if (colors.has(node.data.addr)) {
        return colors.get(node.data.addr);
      }

      const newColor = this.rrbVecVisColor ?? "none";
      colors.set(node.data.addr, newColor);
      return newColor;
    };
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
    this.rrbVecVis.set(rrbVec);
  }

  concatenate(two) {
    this.vector.concatenate(two.vector);
    this.rrbVecVis.set(this.vector.json());
  }

  setSize(size) {
    this.vector.setSize(size);
    const rrbVec = this.vector.json();

    if (this.rrbVecVis === undefined) {
      this.rrbVecVis = new RrbVec(this.selector(), this.colorResolver);
      this.rrbVecVis.setOnMouseOverListener(this.listener);
      this.rrbVecVisColor = resolveColor(
        colorPalette[colorPicker++ % colorPalette.length]
      );
    }

    this.rrbVecVis.set(rrbVec);
  }

  size() {
    return this.vector.size();
  }
}
