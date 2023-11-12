import "./styles.css";
import { WasmDecorator, VectorVis } from "./vector";

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

    this.slider = document.createElement("input");
    this.slider.addEventListener("change", () =>
      this.vectorVis.setSize(this.slider.value)
    );
    this.slider.type = "range";
    this.slider.min = 1;
    this.slider.max = 512;

    sliderContainer.appendChild(this.slider);
    sliderContainer.appendChild(sliderTooltip);

    const updateTooltip = () => {
      const offset =
        ((this.slider.value - this.slider.min) * 100) /
        (this.slider.max - this.slider.min);
      sliderTooltip.innerHTML = `<span>${this.slider.value}</span>`;

      // Kind of magic numbers based on size of the native UI thumb
      sliderTooltip.style.left = `calc(${offset}% + (${5 - offset * 0.1}px))`;
    };

    this.slider.addEventListener("input", updateTooltip);
    updateTooltip();

    this.appendChild(sliderContainer);

    if (this.vectorVis.size() > 0) {
      this.slider.value = this.vectorVis.size();
      this.slider.dispatchEvent(new Event("input"));
      this.slider.dispatchEvent(new Event("change"));
    }
  }

  update() {
    this.vectorVis.update();
    const vecSize = this.vectorVis.size();

    if (vecSize > 0) {
      if (this.slider.max < vecSize) {
        this.slider.max = vecSize;
      }

      this.slider.value = vecSize;
      this.slider.dispatchEvent(new Event("input"));
      this.slider.dispatchEvent(new Event("change"));
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

class ConcatenateVectorsButtonComponent extends HTMLElement {
  constructor() {
    super();
    this.onClick = null;
    this.classList.add(
      "transition-opacity",
      "duration-500",
      "ease-in-out",
      "opacity-0"
    );
  }

  connectedCallback() {
    const button = document.createElement("button");
    button.type = "button";
    button.innerHTML = "Concatenate";
    button.classList.add("button-concat-all");
    button.addEventListener("click", () => this.onClick(this));

    this.appendChild(button);
  }

  show() {
    this.classList.remove("opacity-0");
    this.classList.add("opacity-100");
  }

  hide() {
    this.classList.remove("opacity-100");
    this.classList.add("opacity-0");
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
customElements.define(
  "concatenate-vectors-button-component",
  ConcatenateVectorsButtonComponent
);

const concatenateVectorsButton = new ConcatenateVectorsButtonComponent();
concatenateVectorsButton.onClick = () => {
  // If we have only one vector, there is nothing to concatenate.
  if (wasmDecorator.len() > 1) {
    wasmDecorator.concatenatAll();

    // After concatenation, there will be only one vector left. Hence, we need to prune the rest.
    while (grid.children.length > 2) {
      grid.removeChild(grid.children[1]);
    }

    // Signal the first vector to update itself, as it now contains values from all other vectors.
    grid.firstElementChild.update();
  }
};

const wasmDecorator = new WasmDecorator(() => {
  if (wasmDecorator.len() > 1) {
    concatenateVectorsButton.show();
  } else {
    concatenateVectorsButton.hide();
  }
});
const grid = new GridComponent();
const addVectorButton = new AddVectorButtonComponent();

const createMouseEventsHandler = (vectorVis, vectorComponent) => {
  const onHoverEventDelay = 256;

  let vectorSplitControl = null;
  let vectorSplitControlEntered = false;
  let lastTargetHovered = null;

  return {
    onMouseOver: (onLeafOverEvent, index) => {
      if (vectorSplitControl) {
        vectorSplitControl.remove();
      }

      lastTargetHovered = onLeafOverEvent.target;
      setTimeout(() => {
        const target = onLeafOverEvent.target;
        const targetElementRect = target.getBoundingClientRect();

        if (target !== lastTargetHovered) {
          // If mouse has already moved to another element, don't show the split control
          return;
        }

        const vectorVisRoot = document.querySelector(vectorVis.selector());
        vectorSplitControl = document.createElement("div");
        vectorSplitControl.classList.add("tooltip-split");
        vectorSplitControl.innerHTML = "<span>Split</span>";
        vectorSplitControl.addEventListener("click", () => {
          const vector = vectorVis.vec();
          const other = vector.splitAt(index);

          vectorComponent.update();
          addVectorToGrid(other, vectorComponent.nextSibling);

          // Remove the control, otherwise it will be left hanging around
          vectorSplitControl.remove();
        });

        vectorSplitControlEntered = false;
        vectorSplitControl.addEventListener("mouseover", () => {
          vectorSplitControlEntered = true;
        });

        vectorSplitControl.addEventListener("mouseout", () => {
          vectorSplitControl.getBoundingClientRect();
          vectorSplitControl.style.opacity = 0;

          vectorSplitControl.addEventListener("transitionend", () => {
            vectorSplitControl.remove();
            vectorSplitControlEntered = false;
          });
        });

        vectorVisRoot.appendChild(vectorSplitControl);

        const vectorSplitControlRect =
          vectorSplitControl.firstChild.getBoundingClientRect();
        const offsetLeft =
          targetElementRect.left +
          targetElementRect.width / 2 -
          vectorSplitControlRect.width / 2;
        const offsetTop =
          targetElementRect.top -
          targetElementRect.height / 2 -
          vectorSplitControlRect.height;
        vectorSplitControl.style.left = `${offsetLeft}px`;
        vectorSplitControl.style.top = `${offsetTop}px`;

        // Trigger a reflow to force the browser to apply the opacity transition
        vectorSplitControl.getBoundingClientRect();
        vectorSplitControl.style.opacity = 1;
      }, onHoverEventDelay);
    },
    onMouseOut: (event) => {
      setTimeout(() => {
        if (event.target === lastTargetHovered && !vectorSplitControlEntered) {
          vectorSplitControl.getBoundingClientRect();
          vectorSplitControl.style.opacity = 0;

          vectorSplitControl.addEventListener("transitionend", () => {
            vectorSplitControl.remove();
          });
        }
      }, onHoverEventDelay);
    },
  };
};

const addVectorToGrid = (vector, nextSibling) => {
  const vectorVis = new VectorVis(vector);
  const vectorComponent = new VectorComponent(vectorVis);
  const mouseEventsHandler = createMouseEventsHandler(
    vectorVis,
    vectorComponent
  );

  vectorVis.setOnMouseOverListener(mouseEventsHandler);
  grid.insertBefore(vectorComponent, nextSibling);
};

const addVector = (button) => {
  const vector = wasmDecorator.add(64);
  addVectorToGrid(vector, button);
};
addVectorButton.onClick = addVector;

grid.appendChild(addVectorButton);
document.body.appendChild(grid);

addVector(addVectorButton);
document.body.appendChild(concatenateVectorsButton);
