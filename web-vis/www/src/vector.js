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

const colors = new Map();
export class VectorVis {
  constructor(vector) {
    this.vector = vector;

    this.colorResolver = (node) => {
      if (colors.has(node.data.addr)) {
        return colors.get(node.data.addr);
      }

      // Traverse nodes all the way to parent until we find a painted node
      //   let next = node;
      //   while (next && !this.colors.has(next.data.addr)) {
      //     next = next.parent;
      //   }

      //   if (next && this.colors.has(next.data.addr)) {
      //     const parentNodeColor = this.colors.get(next.data.addr);
      //     this.colors.set(node.data.addr, parentNodeColor);

      //     return resolveColor(parentNodeColor);
      //  }

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
    // TODO: you will have to programatically paint nodes, otherwise this information will be lost
    // TODO: altenatively, you can implement concatenation by drag and drug for individual vectors
    this.vector.concatenate(two.vector);
    this.rrbVecVis.set(this.vector.json());
  }

  setSize(size) {
    this.vector.setSize(size);
    const rrbVec = this.vector.json();

    if (this.rrbVecVis === undefined) {
      // const root = rrbVec?.tree?.root;
      // if (root) {
      //   // If this root node was painted before, we should reuse the color.
      //   const rootColor = this.colors.get(root.addr);
      //   if (!rootColor) {
      //     console.log("::: setting color for the root", root.addr);
      //     this.colors.set(root.addr, "#4338ca");
      //   }
      // }

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
