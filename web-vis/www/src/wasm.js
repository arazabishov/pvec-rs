import * as wasm from "web-vis";

export class WasmDecorator {
  constructor(listener) {
    this.listener = listener;
  }

  pushVec() {
    const vecId = wasm.len();

    wasm.push_vec();
    this.listener();

    return vecId;
  }

  setVecSize(id, size) {
    wasm.set_vec_size(id, size);
    this.listener();
  }

  splitOffVec(id, index) {
    const other = wasm.split_off_vec(id, index);
    this.listener();

    return other;
  }

  concatenate(one, two) {
    wasm.concatenate(one, two);
    this.listener();
  }

  concatenatAll() {
    wasm.concatenat_all();
    this.listener();
  }

  getVecSize(id) {
    return wasm.get_vec_size(id);
  }

  get(id) {
    return wasm.get(id);
  }

  len() {
    return wasm.len();
  }
}
