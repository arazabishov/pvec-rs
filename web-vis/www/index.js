import * as wasm from "web-vis";
import { RrbVec } from "./rrbtree";

wasm.push_vec();
// wasm.push_vec();

const rrbVecOne = new RrbVec("#tree1");
const rrbVecTwo = new RrbVec("#tree2");

const slider = document.getElementById("vectorSize");
slider.addEventListener("change", function () {
  wasm.set_vec_size(0, slider.value);
  // wasm.set_vec_size(1, slider.value);

  const vectors = wasm.get();

  const vecOne = JSON.parse(vectors[0]);
  console.log(vecOne)
  // const vecTwo = JSON.parse(vectors[1]);

  rrbVecOne.set(vecOne);
  // rrbVecTwo.draw(vecTwo);
});

const splitter = document.getElementById("vectorSplitter");
splitter.addEventListener("click", () => {
  wasm.split_off_vec(0, 167);  

  const vectors = wasm.get();

  const vecOne = JSON.parse(vectors[0]);
  const vecTwo = JSON.parse(vectors[1]);

  rrbVecOne.set(vecOne);
  rrbVecTwo.set(vecTwo);
});
