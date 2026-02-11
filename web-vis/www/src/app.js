import "./styles.css";
import { VectorVis } from "./vectorvis";
import {
  createElement,
  Plus,
  Minus,
  Crosshair,
  Copy,
  Trash2,
  Merge,
  Sun,
  Moon,
  Github,
  Linkedin,
  Twitter,
} from "lucide";
import * as theme from "./theme.js";

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

  const plusIcon = createIcon(Plus);
  plusIcon.classList.add("button-add-vector-icon");

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
  button.classList.add("button-concat-all");

  const icon = createIcon(Merge);
  icon.style.flexShrink = "0";
  icon.style.overflow = "visible";

  button.appendChild(icon);
  button.appendChild(document.createTextNode("Concat"));
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

function createSocialLinks() {
  const socials = [
    { icon: Github, href: "https://github.com/ArazAbishov", label: "GitHub" },
    { icon: Linkedin, href: "https://linkedin.com", label: "LinkedIn" },
    { icon: Twitter, href: "https://x.com", label: "X" },
  ];

  const container = document.createElement("nav");
  container.classList.add("social-links");

  for (const { icon, href, label } of socials) {
    const a = document.createElement("a");
    a.href = href;
    a.target = "_blank";
    a.rel = "noopener noreferrer";
    a.title = label;
    a.classList.add("social-link");
    a.appendChild(createIcon(icon));
    container.appendChild(a);
  }

  return container;
}

function createTopBar() {
  const bar = document.createElement("div");
  bar.classList.add("top-bar");

  const title = document.createElement("span");
  title.classList.add("top-bar-title");
  title.textContent = "pvec-rs";
  bar.appendChild(title);

  const toggle = document.createElement("button");
  toggle.type = "button";
  toggle.classList.add("theme-toggle");
  toggle.title = "Toggle dark mode";

  const track = document.createElement("span");
  track.classList.add("theme-toggle-track");

  const sunIcon = createIcon(Sun);
  sunIcon.classList.add("theme-toggle-icon");

  const moonIcon = createIcon(Moon);
  moonIcon.classList.add("theme-toggle-icon");

  const thumb = document.createElement("span");
  thumb.classList.add("theme-toggle-thumb");

  track.appendChild(sunIcon);
  track.appendChild(moonIcon);
  track.appendChild(thumb);
  toggle.appendChild(track);

  const setToggle = (dark) => {
    toggle.classList.toggle("dark", dark);
  };

  setToggle(theme.isDark());
  theme.onChange(setToggle);
  toggle.addEventListener("click", () => theme.toggle());

  const githubLink = document.createElement("a");
  githubLink.href = "https://github.com/ArazAbishov/pvec-rs";
  githubLink.target = "_blank";
  githubLink.rel = "noopener noreferrer";
  githubLink.title = "GitHub";
  githubLink.classList.add("top-bar-icon-link");
  githubLink.appendChild(createIcon(Github));

  const controls = document.createElement("div");
  controls.classList.add("top-bar-controls");
  controls.appendChild(githubLink);
  controls.appendChild(toggle);

  bar.appendChild(controls);
  return bar;
}

function createIcon(iconNode) {
  // prettier-ignore
  return createElement(iconNode, {
    "stroke-width": 1.5,
    "height": 18,
    "width": 18,
  });
}

class VectorCard {
  constructor(vector, letter) {
    this.vector = vector;
    this.letter = letter;

    this.el = document.createElement("div");
    this.el.classList.add("vector");

    // --- Card header ---
    const header = document.createElement("div");
    header.classList.add("card-header");

    const headerLeft = document.createElement("div");
    headerLeft.classList.add("card-header-left");

    this.swatch = document.createElement("span");
    this.swatch.classList.add("color-swatch");

    this.label = document.createElement("span");
    this.label.classList.add("card-label");
    this.label.textContent = `Vector ${letter}`;

    headerLeft.appendChild(this.swatch);
    headerLeft.appendChild(this.label);

    const headerRight = document.createElement("div");
    headerRight.classList.add("card-header-right");

    const cloneBtn = document.createElement("button");
    cloneBtn.type = "button";
    cloneBtn.classList.add("card-action-btn");
    cloneBtn.title = "Clone";
    cloneBtn.appendChild(createIcon(Copy));
    cloneBtn.addEventListener("click", () => {
      this.el.dispatchEvent(
        new CustomEvent("vector-clone", {
          bubbles: true,
          detail: { card: this },
        })
      );
    });

    this.deleteBtn = document.createElement("button");
    this.deleteBtn.type = "button";
    this.deleteBtn.classList.add("card-action-btn", "card-action-btn-delete");
    this.deleteBtn.title = "Delete";
    this.deleteBtn.appendChild(createIcon(Trash2));
    this.deleteBtn.addEventListener("click", () => {
      this.el.dispatchEvent(
        new CustomEvent("vector-delete", {
          bubbles: true,
          detail: { card: this },
        })
      );
    });

    headerRight.appendChild(cloneBtn);
    headerRight.appendChild(this.deleteBtn);

    header.appendChild(headerLeft);
    header.appendChild(headerRight);

    // --- Card body (where d3 SVG renders) ---
    this.body = document.createElement("div");
    this.body.id = vector.id();
    this.body.classList.add("card-body");

    const sliderContainer = document.createElement("div");
    sliderContainer.classList.add("slider-container");

    const sliderTooltip = document.createElement("output");
    sliderTooltip.classList.add("tooltip-value");

    this.slider = document.createElement("input");
    this.slider.addEventListener("change", () => {
      this.vector.resize(this.slider.value);
      this.#updateLabel();
    });
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
    zoomIn.appendChild(createIcon(Plus));
    zoomIn.classList.add("zoom-btn", "zoom-btn-top");
    zoomIn.addEventListener("click", () => this.vector.zoomIn());

    const zoomOut = document.createElement("button");
    zoomOut.type = "button";
    zoomOut.appendChild(createIcon(Minus));
    zoomOut.classList.add("zoom-btn", "zoom-btn-mid");
    zoomOut.addEventListener("click", () => this.vector.zoomOut());

    const fitBtn = document.createElement("button");
    fitBtn.type = "button";
    fitBtn.appendChild(createIcon(Crosshair));
    fitBtn.classList.add("zoom-btn", "zoom-btn-bottom");
    fitBtn.addEventListener("click", () => this.vector.fit());

    zoomControls.appendChild(zoomIn);
    zoomControls.appendChild(zoomOut);
    zoomControls.appendChild(fitBtn);

    this.body.appendChild(zoomControls);
    this.body.appendChild(sliderContainer);

    // Persistent split tooltip
    this.tooltip = document.createElement("div");
    this.tooltip.classList.add("tooltip-split");
    this.tooltip.innerHTML = "<span>Split</span>";
    this.body.appendChild(this.tooltip);

    this.el.appendChild(header);
    this.el.appendChild(this.body);

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
      this.#syncSlider(this.vector.size());
    }
  }

  #syncSlider(size) {
    if (this.slider.max < size) {
      this.slider.max = size;
    }
    this.slider.value = size;
    this.slider.dispatchEvent(new Event("input"));
    this.slider.dispatchEvent(new Event("change"));
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

  updateDeleteButton(cardCount) {
    this.deleteBtn.disabled = cardCount <= 1;
  }

  update() {
    this.vector.update();
    const vecSize = this.vector.size();

    if (vecSize > 0) {
      this.#syncSlider(vecSize);
    }

    this.#updateLabel();
    this.swatch.style.backgroundColor = this.vector.color ?? "transparent";
  }

  #updateLabel() {
    this.label.textContent = `Vector ${this.letter} \u00b7 ${this.vector.size().toLocaleString()}`;
  }
}

function init() {
  let letterCounter = 0;

  // Assigns successive labels A–Z (wrapping) to each new vector card.
  const nextLetter = () => {
    return String.fromCharCode(65 + (letterCounter++ % 26));
  };

  const cards = new WeakMap();
  const allCards = new Set();
  const grid = createGrid();

  const updateDeleteButtons = () => {
    for (const card of allCards) {
      card.updateDeleteButton(allCards.size);
    }
  };

  const addVectorToGrid = (vector, nextSibling, letter) => {
    const cardLetter = letter ?? nextLetter();
    const card = new VectorCard(vector, cardLetter);
    cards.set(card.el, card);
    allCards.add(card);

    grid.insertBefore(card.el, nextSibling);
    card.mount();
    card.update();
    updateDeleteButtons();
  };

  const removeCard = (card) => {
    card.el.remove();
    card.vector.dispose();
    allCards.delete(card);
    updateDeleteButtons();
  };

  grid.addEventListener("vector-split", (e) => {
    addVectorToGrid(e.detail.vector, e.detail.nextSibling);
  });

  grid.addEventListener("vector-clone", (e) => {
    const sourceCard = e.detail.card;
    const clonedVis = sourceCard.vector.clone();
    addVectorToGrid(clonedVis, sourceCard.el.nextSibling);
  });

  grid.addEventListener("vector-delete", (e) => {
    if (allCards.size <= 1) {
      return;
    }
    removeCard(e.detail.card);
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

      allCards.delete(lastCard);
      lastEl.remove();
    }
    updateDeleteButtons();
  });

  VectorVis.onChange((count) => {
    count > 1 ? concatenateButton.show() : concatenateButton.hide();
  });

  const addButton = createAddButton((button) => {
    addVectorToGrid(VectorVis.create(64), button);
  });

  grid.appendChild(addButton);
  document.body.appendChild(createTopBar());
  document.body.appendChild(grid);

  // Add the first vector
  addVectorToGrid(VectorVis.create(64), addButton);

  document.body.appendChild(concatenateButton);

  const footer = document.createElement("footer");
  footer.classList.add("page-footer");

  const attribution = document.createElement("span");
  attribution.classList.add("footer-attribution");
  attribution.textContent = "made by ";

  const authorLink = document.createElement("a");
  authorLink.href = "https://abishov.com";
  authorLink.target = "_blank";
  authorLink.rel = "noopener noreferrer";
  authorLink.classList.add("footer-author-link");
  authorLink.textContent = "arazabishov";

  attribution.appendChild(authorLink);

  const separator = document.createElement("span");
  separator.classList.add("footer-separator");
  separator.textContent = "\u00b7";

  footer.appendChild(attribution);
  footer.appendChild(separator);
  footer.appendChild(createSocialLinks());
  document.body.appendChild(footer);
}

init();
