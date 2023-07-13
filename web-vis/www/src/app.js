import "./styles.css";
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

const grid = addGrid();

function addTree() {
  const vectorIndex = wasm.len();

  // push a vector so then a new instance becomes accessible
  // to the rest of the code below
  wasm.push_vec();

  const tree = document.createElement("div");
  tree.id = `tree${trees.length}`;
  tree.classList.add(
    "bg-white",
    "rounded-lg",
    "border",
    "border-gray-300",
    "p-4",
    "min-h-[600px]",
    "relative"
  );

  const sliderContainer = document.createElement("div");
  sliderContainer.classList.add("absolute", "w-1/4", "bottom-4", "right-12");

  const sliderTooltip = document.createElement("output");
  sliderTooltip.classList.add("tooltip-value");

  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = 1;
  slider.max = 512;
  slider.value = 50;
  slider.addEventListener("change", () => {
    wasm.set_vec_size(vectorIndex, slider.value);

    const rrbVecVis = trees[vectorIndex];
    const rrbVec = JSON.parse(wasm.get(vectorIndex));

    console.log(rrbVec);
    rrbVecVis.set(rrbVec);
  });

  sliderContainer.appendChild(slider);
  sliderContainer.appendChild(sliderTooltip);

  const updateTooltip = () => {
    const offset =
      ((slider.value - slider.min) * 100) / (slider.max - slider.min);
    sliderTooltip.innerHTML = `<span>${slider.value}</span>`;

    // Kind of magic numbers based on size of the native UI thumb
    sliderTooltip.style.left = `calc(${offset}% + (${5 - offset * 0.1}px))`;
  };

  slider.addEventListener("input", updateTooltip);
  updateTooltip();

  tree.appendChild(sliderContainer);
  grid.insertBefore(tree, newTreeButton);

  const rrbVec = new RrbVec(`#${tree.id}`);
  trees.push(rrbVec);
}

function addNewTreeButton() {
  const button = document.createElement("button");
  button.type = "button";
  button.classList.add(
    "bg-white",
    "rounded-lg",
    "border",
    "border-gray-300",
    "p-4",
    "min-h-[196px]",
    "min-w-[196px]",
    "focus:outline-none",
    "py-2.5",
    "px-5",
    "border",
    "hover:bg-gray-50",
    "flex",
    "items-center",
    "justify-center"
  );

  const plusIcon = document.createElement("span");
  plusIcon.classList.add("text-gray-500", "text-4xl");
  plusIcon.innerHTML = "+";

  button.appendChild(plusIcon);
  button.addEventListener("click", addTree);

  const container = document.createElement("div");
  container.classList.add(
    "flex",
    "justify-center",
    "items-center",
    "min-h-[600px]"
  );
  container.appendChild(button);

  grid.appendChild(container);

  return container;
}

function addGrid() {
  const grid = document.createElement("div");
  grid.classList.add(
    "grid",
    "grid-cols-1",
    "md:grid-cols-1",
    "lg:grid-cols-2",
    "gap-4",
    "m-4"
  );
  document.body.appendChild(grid);
  return grid;
}

const body = document.body;
body.style.background =
  "radial-gradient(#000 1px, transparent 0px) 0 0 / 24px 24px";

const newTreeButton = addNewTreeButton();

addTree();
