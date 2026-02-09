import * as wasm from "web-vis";

export class Vector {
  constructor(id) {
    this._id = id;
  }

  static create() {
    return new Vector(wasm.push_vec());
  }

  id() {
    return this._id;
  }

  resize(size) {
    wasm.set_vec_size(this._id, size);
  }

  split(index) {
    return new Vector(wasm.split_off_vec(this._id, index));
  }

  concatenate(other) {
    wasm.concatenate(this._id, other.id());
  }

  size() {
    return wasm.get_vec_size(this._id);
  }

  json() {
    return JSON.parse(wasm.get(this._id));
  }
}
