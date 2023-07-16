import "./styles.css";
import { VectorFactory, VectorVis } from "./vector";

class VectorComponent extends HTMLElement {
  constructor(vectorVis) {
    super();
    this.vectorVis = vectorVis;
  }

  connectedCallback() {
    this.id = this.vectorVis.id();
    this.classList.add("vector");

    const sliderContainer = document.createElement("div");
    sliderContainer.classList.add("slider-container");

    const sliderTooltip = document.createElement("output");
    sliderTooltip.classList.add("tooltip-value");

    const slider = document.createElement("input");
    slider.addEventListener("change", () =>
      this.vectorVis.setSize(slider.value)
    );
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

    this.appendChild(sliderContainer);

    if (this.vectorVis.size() > 0) {
      slider.value = this.vectorVis.size();
      slider.dispatchEvent(new Event("input"));
      slider.dispatchEvent(new Event("change"));
    }
  }
}

class AddVectorButtonComponent extends HTMLElement {
  constructor() {
    super();
    this.onClick = null;
  }

  connectedCallback() {
    const button = document.createElement("button");
    button.type = "button";
    button.classList.add("button-add-vector");

    const plusIcon = document.createElement("span");
    plusIcon.classList.add("button-add-vector-icon");
    plusIcon.innerHTML = "+";

    button.appendChild(plusIcon);
    button.addEventListener("click", () => this.onClick(this));

    this.classList.add("button-add-vector-container");
    this.appendChild(button);
  }
}

class GridComponent extends HTMLElement {
  connectedCallback() {
    this.classList.add("grid-vector");
  }
}

customElements.define("vector-component", VectorComponent);
customElements.define("add-vector-button-component", AddVectorButtonComponent);
customElements.define("grid-component", GridComponent);

const grid = new GridComponent();
const addVectorButton = new AddVectorButtonComponent();
const addVector = (button) => {
  const vectorVis = new VectorVis(VectorFactory.create(64));
  const vectorComponent = new VectorComponent(vectorVis);

  grid.insertBefore(vectorComponent, button);
};
addVectorButton.onClick = addVector;

grid.appendChild(addVectorButton);
document.body.appendChild(grid);

addVector(addVectorButton);
