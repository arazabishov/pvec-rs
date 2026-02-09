import * as wasm from "web-vis";

export class Vector {
  #id;

  constructor(id) {
    this.#id = id;
  }

  static create() {
    return new Vector(wasm.push_vec());
  }

  id() {
    return this.#id;
  }

  resize(size) {
    wasm.set_vec_size(this.#id, size);
  }

  split(index) {
    return new Vector(wasm.split_off_vec(this.#id, index));
  }

  concatenate(other) {
    wasm.concatenate(this.#id, other.id());
  }

  size() {
    return wasm.get_vec_size(this.#id);
  }

  json() {
    return wasm.get(this.#id);
  }
}
