import "./styles.css";
import * as wasm from "web-vis";
import { RrbVec } from "./rrbtree";

class Vector {
  constructor(id) {
    this._id = id;
  }

  id() {
    return this._id;
  }

  setSize(size) {
    wasm.set_vec_size(this._id, size);
  }

  json() {
    return JSON.parse(wasm.get(this._id));
  }
}

class VectorVis {
  constructor(vector) {
    this.vector = vector;
  }

  id() {
    return `tree${this.vector.id()}`;
  }

  setSize(size) {
    if (this.rrbVecVis === undefined) {
      this.rrbVecVis = new RrbVec(`#${this.id()}`);
    }

    this.vector.setSize(size);
    const rrbVec = this.vector.json();

    this.rrbVecVis.set(rrbVec);
  }
}

class VectorFactory {
  static create() {
    const vecId = wasm.len();

    wasm.push_vec();

    return new Vector(vecId);
  }
}

function createTree(vectorVis) {
  const tree = document.createElement("div");
  tree.id = vectorVis.id();
  tree.classList.add("tree");

  const sliderContainer = document.createElement("div");
  sliderContainer.classList.add("slider-container");

  const sliderTooltip = document.createElement("output");
  sliderTooltip.classList.add("tooltip-value");

  const slider = document.createElement("input");
  slider.addEventListener("change", () => vectorVis.setSize(slider.value));
  slider.type = "range";
  slider.min = 1;
  slider.max = 512;

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
  return { tree, slider };
}

function createAddTreeButton(onClick) {
  const container = document.createElement("div");
  container.classList.add("button-add-tree-container");

  const button = document.createElement("button");
  button.type = "button";
  button.classList.add("button-add-tree");

  const plusIcon = document.createElement("span");
  plusIcon.classList.add("button-add-tree-icon");
  plusIcon.innerHTML = "+";

  button.appendChild(plusIcon);
  button.addEventListener("click", () => onClick(container));

  container.appendChild(button);
  return container;
}

function createGrid() {
  const grid = document.createElement("div");
  grid.classList.add("grid-tree");

  return grid;
}
const grid = createGrid();

const addTree = (button, initialSize) => {
  const vector = VectorFactory.create();
  const vectorVis = new VectorVis(vector);
  const { tree, slider } = createTree(vectorVis);

  grid.insertBefore(tree, button);

  if (initialSize !== undefined) {
    slider.value = initialSize;
    slider.dispatchEvent(new Event("input"));
    slider.dispatchEvent(new Event("change"));
  }
};
const addTreeButton = createAddTreeButton(addTree);

grid.appendChild(addTreeButton);
document.body.appendChild(grid);

addTree(addTreeButton, 256);
