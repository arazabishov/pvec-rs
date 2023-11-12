import { RrbVec } from "./rrbvec.js";
import * as wasm from "web-vis";

export class WasmDecorator {
  constructor(listener) {
    this.listener = listener;
  }

  add(initialSize) {
    const vecId = wasm.len();
    wasm.push_vec();

    const vector = new Vector(vecId, this);
    if (initialSize !== undefined) {
      vector.setSize(initialSize);
    }

    this.listener();

    return vector;
  }

  setSize(id, size) {
    wasm.set_vec_size(id, size);
    this.listener();
  }

  splitOffVec(id, index) {
    const other = wasm.split_off_vec(id, index);
    this.listener();

    return other;
  }

  getVecSize(id) {
    return wasm.get_vec_size(id);
  }

  get(id) {
    return wasm.get(id);
  }

  concatenatAll() {
    wasm.concatenat_all();
    this.listener();
  }

  len() {
    return wasm.len();
  }
}

export class Vector {
  constructor(id, wasmDecorator) {
    this._id = id;
    this.wasmDecorator = wasmDecorator;
  }

  id() {
    return this._id;
  }

  setSize(size) {
    this.wasmDecorator.setSize(this._id, size);
  }

  splitAt(index) {
    const newVecId = this.wasmDecorator.splitOffVec(this._id, index);
    return new Vector(newVecId, this.wasmDecorator);
  }

  size() {
    return this.wasmDecorator.getVecSize(this._id);
  }

  json() {
    return JSON.parse(this.wasmDecorator.get(this._id));
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
