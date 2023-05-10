import * as wasm from "web-vis";
import { RrbVec } from "./rrbtree";

const trees = [];

// <button type="button" id="vectorSplitter">
//   Split vector in the middle
// </button>

// const splitter = document.getElementById("vectorSplitter");
// splitter.addEventListener("click", () => {
//   wasm.split_off_vec(0, 167);

//   const vectors = wasm.get();

//   const vecOne = JSON.parse(vectors[0]);
//   const vecTwo = JSON.parse(vectors[1]);

//   // rrbVecOne.set(vecOne);
//   // rrbVecTwo.set(vecTwo);
// });

function addTree() {
  const vectorIndex = wasm.len();

  // push a vector so then a new instance becomes accessible
  // to the rest of the code below
  wasm.push_vec();

  const treeId = `tree${trees.length}`;

  const div = document.createElement("div");
  div.id = treeId;

  const sizeSlider = document.createElement("input");
  sizeSlider.type = "range";
  sizeSlider.min = 1;
  sizeSlider.max = 512;
  sizeSlider.value = 50;
  sizeSlider.addEventListener("change", () => {
    wasm.set_vec_size(vectorIndex, sizeSlider.value);

    const rrbVecVis = trees[vectorIndex];
    const rrbVec = JSON.parse(wasm.get(vectorIndex));

    console.log(rrbVec);

    rrbVecVis.set(rrbVec);
  });

  div.appendChild(sizeSlider);

  document.body.appendChild(div);

  const rrbVec = new RrbVec(`#${treeId}`);
  trees.push(rrbVec);
}

function addButton() {
  const button = document.createElement("button");
  button.innerHTML = "Add Tree";
  button.addEventListener("click", addTree);

  document.body.appendChild(button);
}

addButton();

addTree();
