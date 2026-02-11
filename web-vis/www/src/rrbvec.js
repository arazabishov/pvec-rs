import * as d3 from "d3";

const transitionDuration = 256;
const fitPadding = 60;

const arrayCellWidth = 16;
const arrayCellHeight = 20;

// Vertical spacing between tree levels in px.
const dy = 54;
const dx = arrayCellWidth * 5;

const diagonal = d3
  .linkVertical()
  .x((d) => d.x)
  .y((d) => d.y);

const tree = d3
  .tree()
  .nodeSize([dx, dy])
  .separation((a, b) => {
    if (a.parent && b.parent) {
      if (a.parent.data.leaf && b.parent.data.leaf) {
        return a.parent == b.parent ? 0.3 : 0.8;
      }
    }

    return a.parent == b.parent ? 1 : 2;
  });

const getChildren = (node) => {
  if (!node) {
    return null;
  }

  if (node.relaxedBranch) {
    return node.relaxedBranch.filter((node) => node);
  } else if (node.branch) {
    return node.branch.filter((node) => node);
  } else if (node.leaf) {
    return node;
  }

  return null;
};

// Appends a text element with a white stroke outline for readability.
// `selection` is a d3 enter selection, `transformFn` computes the transform
// attribute, and `textFn` extracts the display text from each datum.
const appendOutlinedText = (selection, transformFn, textFn) => {
  selection
    .append("text")
    .attr("transform", transformFn)
    .attr("dy", "0.31em")
    .attr("x", 8)
    .attr("text-anchor", "end")
    .style("fill", "var(--text-label-fill)")
    .text(textFn)
    .clone(true)
    .lower()
    .attr("stroke-linejoin", "round")
    .attr("stroke-width", 3)
    .style("stroke", "var(--text-outline-stroke)");
};

export class RrbVec {
  constructor(selector) {
    this.container = document.querySelector(selector);

    // Content box dimensions for the initial viewBox (before ResizeObserver fires).
    // clientWidth/clientHeight include padding, so subtract it to match the SVG's
    // CSS-assigned size (width/height: 100% resolves against the content box).
    const cs = getComputedStyle(this.container);
    this.width =
      this.container.clientWidth -
      parseFloat(cs.paddingLeft) -
      parseFloat(cs.paddingRight);
    this.height =
      this.container.clientHeight -
      parseFloat(cs.paddingTop) -
      parseFloat(cs.paddingBottom);

    this.svgTree = d3
      .select(selector)
      .append("svg")
      .attr("viewBox", [0, 0, this.width, this.height])
      .style("font", "10px sans-serif")
      .style("user-select", "none");

    this.gCanvas = this.svgTree.append("g");

    this.zoom = d3
      .zoom()
      .scaleExtent([0.1, 4])
      .on("start", () => this.svgTree.classed("grabbing", true))
      .on("zoom", (event) => {
        this.gCanvas.attr("transform", event.transform);
      })
      .on("end", () => this.svgTree.classed("grabbing", false));

    this.svgTree.call(this.zoom);

    this.gLink = this.gCanvas
      .append("g")
      .attr("fill", "none")
      .style("stroke", "var(--tree-link-stroke)")
      .style("stroke-opacity", "var(--tree-link-opacity)")
      .attr("stroke-width", 1.5);

    this.gNode = this.gCanvas
      .append("g")
      .attr("cursor", "pointer")
      .attr("pointer-events", "all");

    this.gNodeTail = this.gCanvas
      .append("g")
      .attr("transform", () => `translate(${arrayCellWidth * 8}, 0)`);

    this.tailLabel = this.gNodeTail
      .append("text")
      .attr("x", 0)
      .attr("y", -6)
      .style("fill", "var(--tail-label-fill)")
      .attr("font-size", "10px")
      .attr("visibility", "hidden")
      .text("tail");

    // Callback fires once per observed element that changed size; the entries
    // array is guaranteed non-empty. contentBoxSize is a FrozenArray with one
    // entry per fragment (always one for a non-fragmented div).
    this.resizeObserver = new ResizeObserver((entries) => {
      const { inlineSize, blockSize } = entries[0].contentBoxSize[0];
      this.width = inlineSize;
      this.height = blockSize;
      this.svgTree.attr("viewBox", [0, 0, this.width, this.height]);

      if (this.root) {
        const { left, top, right, bottom } = this.#computeBounds();
        this.#fit(left, top, right, bottom, 0);
      }
    });
    this.resizeObserver.observe(this.container);
  }

  onMouseOver(listener) {
    this.listener = listener;
  }

  set(vec) {
    this.#updateTail(vec.tail);

    if (vec.tree.root_len > 0) {
      this.root = d3.hierarchy(vec.tree.root, getChildren);

      this.root.x0 = dy / 2;
      this.root.y0 = 0;

      let descendants = this.root.descendants();
      let next_node_to_expand = descendants ? descendants[0].data : null;

      // Here we store subtree length within leaves to simplify
      // vector splitting in outer layers.
      let lenSubTree = 0;

      // Sorting descendants in topological order.
      descendants.forEach((d) => {
        const { children, data } = d;

        if (data.leaf) {
          data.lenSubTree = lenSubTree;
          lenSubTree += data.len;
        }

        d.id = `${data.addr}:${data.len}`;
        d._children = children;

        // keep only the right-most branches expanded to save space
        if (next_node_to_expand === data || (data && data.leaf)) {
          const children = getChildren(data);
          next_node_to_expand = children ? children[data.len - 1] : undefined;
        } else {
          d.children = null;
        }
      });

      this.#updateTree(this.root);
    } else {
      this.root = null;

      // Clear out all nodes and paths when the root node is empty.
      this.gNode.selectAll("g").remove();
      this.gLink.selectAll("path").remove();
    }
  }

  #onMouseEvent(entering, event, d) {
    if (!d.data.leaf || !this.listener) {
      return;
    }

    // Infer the vector index so outer layers don't deal with nodes directly.
    const index = d.data.lenSubTree + d.position;

    if (entering) {
      this.listener.onMouseOver?.(event, index);
    } else {
      this.listener.onMouseOut?.(event, index);
    }
  }

  #updateTree(source) {
    const nodes = this.root.descendants().reverse();
    const links = this.root.links();

    // Compute the new tree layout.
    tree(this.root);

    const { left, top, right, bottom } = this.#computeBounds();

    const transition = this.svgTree.transition().duration(transitionDuration);

    this.#fit(left, top, right, bottom);

    // Update the nodes…
    const node = this.gNode.selectAll("g").data(nodes, (d) => d.id);

    // Enter any new nodes at the parent's previous position.
    const nodeEnter = node
      .enter()
      .append("g")
      .attr("transform", () => `translate(${source.x0},${source.y0})`)
      .attr("opacity", 0)
      .on("click", (_event, d) => {
        d.children = d.children ? null : d._children;
        this.#updateTree(d);
      });

    nodeEnter
      .selectAll("rect")
      .data((d) =>
        Array.from({ length: d.data.len }, (_v, i) => ({
          parent: d?.parent,
          data: d.data,
          position: i,
        }))
      )
      .enter()
      .append("rect")
      .style("stroke-width", "1px")
      .style("stroke", "var(--node-border-stroke)")
      .style("fill", (d) => d.data.color ?? "none")
      .style("fill-opacity", "var(--node-fill-opacity)")
      .attr("width", arrayCellWidth)
      .attr("height", arrayCellHeight)
      .attr(
        "transform",
        (d, i) => `translate(${(i - d.data.len / 2) * arrayCellWidth}, 0)`
      )
      .on("mouseover", (event, d) => this.#onMouseEvent(true, event, d))
      .on("mouseout", (event, d) => this.#onMouseEvent(false, event, d));

    appendOutlinedText(
      nodeEnter
        .selectAll("text")
        .data((d) =>
          Array.from(d.data.leaf || [], (item) => ({ item, len: d.data.len }))
        )
        .enter(),
      (d, i) =>
        `translate(${
          (i - d.len / 2) * arrayCellWidth + arrayCellWidth / 2
        }, ${arrayCellHeight + arrayCellHeight * 0.6}) rotate(270)`,
      (d) => d.item
    );

    appendOutlinedText(
      nodeEnter
        .selectAll("text")
        .data((d) =>
          Array.from(d.data.sizes || [], (item) => ({ item, len: d.data.len }))
        )
        .enter(),
      (d, i) =>
        `translate(${
          (i - d.len / 2) * arrayCellWidth + arrayCellWidth / 2
        }, ${-0.6 * arrayCellHeight}) rotate(315)`,
      (d) => d.item
    );

    // Transition nodes to their new position.
    node
      .merge(nodeEnter)
      .transition(transition)
      .attr("transform", (d) => `translate(${d.x},${d.y})`)
      .attr("opacity", 1);

    // Transition exiting nodes to the parent's new position.
    node
      .exit()
      .transition(transition)
      .remove()
      .attr("transform", () => `translate(${source.x},${source.y})`)
      .attr("opacity", 0);

    // Update the links…
    const link = this.gLink.selectAll("path").data(links, (d) => d.target.id);

    // Enter any new links at the parent's previous position.
    const linkEnter = link
      .enter()
      .append("path")
      .attr("d", () => {
        const o = { x: source.x0, y: source.y0 };
        return diagonal({ source: o, target: o });
      });

    // Transition links to their new position.
    link
      .merge(linkEnter)
      .transition(transition)
      .attr("d", (d) => {
        const children = getChildren(d.source.data);
        const childNodePosition = children.indexOf(d.target.data);

        const sourceX =
          d.source.x +
          (childNodePosition - (d.source.data.len - 1) / 2) * arrayCellWidth;

        return diagonal({
          source: { x: sourceX, y: d.source.y + arrayCellHeight },
          target: { x: d.target.x, y: d.target.y },
        });
      });

    // Transition exiting nodes to the parent's new position.
    link
      .exit()
      .transition(transition)
      .remove()
      .attr("d", () => {
        const o = { x: source.x, y: source.y };
        return diagonal({ source: o, target: o });
      });

    // Stash the old positions for transition.
    this.root.eachBefore((node) => {
      node.x0 = node.x;
      node.y0 = node.y;
    });
  }

  #computeBounds() {
    let left = Infinity;
    let right = -Infinity;
    let top = Infinity;
    let bottom = -Infinity;

    this.root.eachBefore((node) => {
      const halfWidth = (node.data.len / 2) * arrayCellWidth;

      if (node.x - halfWidth < left) {
        left = node.x - halfWidth;
      }
      if (node.x + halfWidth > right) {
        right = node.x + halfWidth;
      }
      if (node.y < top) {
        top = node.y;
      }
      if (node.y > bottom) {
        bottom = node.y;
      }
    });

    return { left, top, right, bottom: bottom + arrayCellHeight };
  }

  #fit(left, top, right, bottom, duration = transitionDuration) {
    const treeW = right - left + fitPadding * 2;
    const treeH = bottom - top + fitPadding * 2;

    const scale = Math.min(this.width / treeW, this.height / treeH, 1);

    const tx = (this.width - (right - left) * scale) / 2 - left * scale;
    const ty = (this.height - (bottom - top) * scale) / 2 - top * scale;

    const transform = d3.zoomIdentity.translate(tx, ty).scale(scale);

    this.svgTree
      .transition()
      .duration(duration)
      .call(this.zoom.transform, transform);
  }

  fit() {
    if (!this.root) {
      return;
    }
    const { left, top, right, bottom } = this.#computeBounds();
    this.#fit(left, top, right, bottom);
  }

  zoomIn() {
    this.svgTree
      .transition()
      .duration(transitionDuration)
      .call(this.zoom.scaleBy, 1.5);
  }

  zoomOut() {
    this.svgTree
      .transition()
      .duration(transitionDuration)
      .call(this.zoom.scaleBy, 1 / 1.5);
  }

  #updateTail(tail) {
    const tailElements = tail.elements.filter(
      (d) => d !== null && d !== undefined
    );

    this.tailLabel.attr(
      "visibility",
      tailElements.length > 0 ? "visible" : "hidden"
    );
    const node = this.gNodeTail
      .selectAll("g")
      .data(tailElements, (d) => `${d}:tail`);
    const nodeEnter = node.enter().append("g");

    nodeEnter
      .append("rect")
      .style("stroke-width", "1px")
      .style("stroke", "var(--node-border-stroke)")
      .style("fill", tail.color ?? "none")
      .style("fill-opacity", "var(--node-fill-opacity)")
      .attr("width", arrayCellWidth)
      .attr("height", arrayCellHeight)
      .attr("transform", (_val, i) => `translate(${i * arrayCellWidth}, 0)`);

    appendOutlinedText(
      nodeEnter,
      (_d, i) =>
        `translate(${i * arrayCellWidth + arrayCellWidth / 2}, ${
          arrayCellHeight + arrayCellHeight * 0.6
        }) rotate(270)`,
      (d) => d
    );

    node
      .merge(nodeEnter)
      .transition()
      .attr("opacity", 1);

    node
      .exit()
      .transition()
      .remove()
      .attr("opacity", 0);
  }
}
