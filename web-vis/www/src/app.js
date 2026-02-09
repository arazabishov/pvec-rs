import "./styles.css";
import { VectorVis } from "./vectorvis";

function createGrid() {
  const el = document.createElement("div");
  el.classList.add("grid-vector");
  return el;
}

function createAddButton(onClick) {
  const container = document.createElement("div");
  container.classList.add("button-add-vector-container");

  const button = document.createElement("button");
  button.type = "button";
  button.classList.add("button-add-vector");

  const plusIcon = document.createElement("span");
  plusIcon.classList.add("button-add-vector-icon");
  plusIcon.innerHTML = "+";

  button.appendChild(plusIcon);
  button.addEventListener("click", () => onClick(container));

  container.appendChild(button);
  return container;
}

function createConcatenateButton(onClick) {
  const container = document.createElement("div");
  container.classList.add("button-concat-container");

  const button = document.createElement("button");
  button.type = "button";
  button.innerHTML = "Concatenate";
  button.classList.add("button-concat-all");
  button.addEventListener("click", () => onClick(container));

  container.appendChild(button);

  container.show = () => {
    container.classList.add("visible");
  };
  container.hide = () => {
    container.classList.remove("visible");
  };

  return container;
}

class VectorCard {
  constructor(vector) {
    this.vector = vector;

    this.el = document.createElement("div");
    this.el.id = vector.id();
    this.el.classList.add("vector");

    const sliderContainer = document.createElement("div");
    sliderContainer.classList.add("slider-container");

    const sliderTooltip = document.createElement("output");
    sliderTooltip.classList.add("tooltip-value");

    this.slider = document.createElement("input");
    this.slider.addEventListener("change", () =>
      this.vector.resize(this.slider.value)
    );
    this.slider.type = "range";
    this.slider.min = 1;
    this.slider.max = 4096;

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

    const zoomControls = document.createElement("div");
    zoomControls.classList.add("zoom-controls");

    const zoomIn = document.createElement("button");
    zoomIn.type = "button";
    zoomIn.textContent = "+";
    zoomIn.classList.add("zoom-btn", "zoom-btn-top");
    zoomIn.addEventListener("click", () => this.vector.zoomIn());

    const zoomOut = document.createElement("button");
    zoomOut.type = "button";
    zoomOut.textContent = "\u2212";
    zoomOut.classList.add("zoom-btn", "zoom-btn-mid");
    zoomOut.addEventListener("click", () => this.vector.zoomOut());

    const fitBtn = document.createElement("button");
    fitBtn.type = "button";
    fitBtn.textContent = "\u2316";
    fitBtn.classList.add("zoom-btn", "zoom-btn-bottom");
    fitBtn.addEventListener("click", () => this.vector.fit());

    zoomControls.appendChild(zoomIn);
    zoomControls.appendChild(zoomOut);
    zoomControls.appendChild(fitBtn);

    this.el.appendChild(zoomControls);
    this.el.appendChild(sliderContainer);

    // Persistent split tooltip
    this.tooltip = document.createElement("div");
    this.tooltip.classList.add("tooltip-split");
    this.tooltip.innerHTML = "<span>Split</span>";
    this.el.appendChild(this.tooltip);

    this.showTimeoutId = null;
    this.hideTimeoutId = null;
    this.currentIndex = null;

    this.#setupTooltipEvents();
  }

  // Must be called after this.el is inserted into the DOM, since
  // RrbVec looks up the element by selector via d3.select().
  mount() {
    this.vector.onMouseOver({
      onMouseOver: (event, index) => this.#showTooltip(event, index),
      onMouseOut: () => this.#hideTooltip(),
    });

    if (this.vector.size() > 0) {
      this.slider.value = this.vector.size();
      this.slider.dispatchEvent(new Event("input"));
      this.slider.dispatchEvent(new Event("change"));
    }
  }

  #setupTooltipEvents() {
    this.tooltip.addEventListener("mouseover", () => {
      clearTimeout(this.hideTimeoutId);
      this.hideTimeoutId = null;
    });

    this.tooltip.addEventListener("mouseout", () => {
      this.#hideTooltip();
    });

    this.tooltip.addEventListener("click", () => {
      if (this.currentIndex === null) {
        return;
      }

      const otherVector = this.vector.split(this.currentIndex);
      this.update();

      this.el.dispatchEvent(
        new CustomEvent("vector-split", {
          bubbles: true,
          detail: {
            vector: otherVector,
            nextSibling: this.el.nextSibling,
          },
        })
      );

      this.tooltip.classList.remove("visible");
      this.currentIndex = null;
    });
  }

  // Debounce: avoid flashing the tooltip when the mouse sweeps across cells.
  #showTooltip(event, index) {
    clearTimeout(this.showTimeoutId);
    clearTimeout(this.hideTimeoutId);
    this.hideTimeoutId = null;

    const target = event.target;

    this.showTimeoutId = setTimeout(() => {
      this.currentIndex = index;
      const targetRect = target.getBoundingClientRect();
      const spanRect = this.tooltip.firstChild.getBoundingClientRect();

      const left = targetRect.left + targetRect.width / 2 - spanRect.width / 2;
      const top = targetRect.top - targetRect.height / 2 - spanRect.height;

      this.tooltip.style.left = `${left}px`;
      this.tooltip.style.top = `${top}px`;

      requestAnimationFrame(() => {
        this.tooltip.classList.add("visible");
      });
    }, 256);
  }

  // Delay: let the user move from the leaf cell to the tooltip before hiding it.
  #hideTooltip() {
    clearTimeout(this.showTimeoutId);
    this.showTimeoutId = null;

    this.hideTimeoutId = setTimeout(() => {
      this.tooltip.classList.remove("visible");
      this.currentIndex = null;
    }, 256);
  }

  update() {
    this.vector.update();
    const vecSize = this.vector.size();

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

function init() {
  const cards = new WeakMap();
  const grid = createGrid();

  const addVectorToGrid = (vector, nextSibling) => {
    const card = new VectorCard(vector);
    cards.set(card.el, card);

    grid.insertBefore(card.el, nextSibling);
    card.mount();
  };

  grid.addEventListener("vector-split", (e) => {
    addVectorToGrid(e.detail.vector, e.detail.nextSibling);
  });

  const concatenateButton = createConcatenateButton(() => {
    // If we have only one vector, there is nothing to concatenate.
    while (grid.children.length > 2) {
      const lastEl = addButton.previousSibling;
      const prevEl = lastEl.previousSibling;

      const lastCard = cards.get(lastEl);
      const prevCard = cards.get(prevEl);

      prevCard.vector.concatenate(lastCard.vector);
      prevCard.update();

      lastEl.remove();
    }
  });

  VectorVis.onChange((count) => {
    if (count > 1) {
      concatenateButton.show();
    } else {
      concatenateButton.hide();
    }
  });

  const addButton = createAddButton((button) => {
    addVectorToGrid(VectorVis.create(64), button);
  });

  grid.appendChild(addButton);
  document.body.appendChild(grid);

  // Add the first vector
  addVectorToGrid(VectorVis.create(64), addButton);

  document.body.appendChild(concatenateButton);
}

init();
