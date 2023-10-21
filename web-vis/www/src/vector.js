import { RrbVec } from "./rrbvec.js";
import * as wasm from "web-vis";

export class Vector {
  constructor(id) {
    this._id = id;
  }

  id() {
    return this._id;
  }

  setSize(size) {
    wasm.set_vec_size(this._id, size);
  }

  splitAt(index) {
    const newVecId = wasm.split_off_vec(this._id, index);
    return new Vector(newVecId);
  }

  size() {
    return wasm.get_vec_size(this._id);
  }

  json() {
    return JSON.parse(wasm.get(this._id));
  }
}

export class VectorFactory {
  static create(initialSize) {
    const vecId = wasm.len();

    wasm.push_vec();

    const vector = new Vector(vecId);
    if (initialSize !== undefined) {
      vector.setSize(initialSize);
    }
    return vector;
  }
}

export class VectorVis {
  constructor(vector) {
    this.vector = vector;
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

  setSize(size) {
    if (this.rrbVecVis === undefined) {
      this.rrbVecVis = new RrbVec(this.selector());
      this.rrbVecVis.setOnMouseOverListener(this.listener);
    }

    this.vector.setSize(size);
    const rrbVec = this.vector.json();

    this.rrbVecVis.set(rrbVec);
  }

  size() {
    return this.vector.size();
  }
}
