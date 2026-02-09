import * as d3 from "d3";

const transitionDuration = 256;

const margin = { top: 32, right: 120, bottom: 42, left: 512 };
const width = 1512 + 512 - margin.left - margin.right;

const arrayCellWidth = 16;
const arrayCellHeight = 20;

const dy = width / 28;
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

const getDescendants = (node) => {
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
    .text(textFn)
    .clone(true)
    .lower()
    .attr("stroke-linejoin", "round")
    .attr("stroke-width", 3)
    .attr("stroke", "white");
};

export class RrbVec {
  constructor(selector) {
    this.svgTree = d3
      .select(selector)
      .append("svg")
      .attr("viewBox", [-margin.left, -margin.top, width, dx])
      .style("font", "10px sans-serif")
      .style("user-select", "none");

    this.gLink = this.svgTree
      .append("g")
      .attr("fill", "none")
      .attr("stroke", "#555")
      .attr("stroke-opacity", 0.4)
      .attr("stroke-width", 1.5);

    this.gNode = this.svgTree
      .append("g")
      .attr("cursor", "pointer")
      .attr("pointer-events", "all");

    this.gNodeTail = this.svgTree
      .append("g")
      .attr("transform", () => `translate(${arrayCellWidth * 8}, 0)`);
  }

  onMouseOver(listener) {
    this.listener = listener;
  }

  set(vec) {
    this.#updateTail(vec.tail);

    if (vec.tree.root_len > 0) {
      this.root = d3.hierarchy(vec.tree.root, getDescendants);

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
          const children = getDescendants(data);
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

    let top = this.root;
    let bottom = this.root;

    this.root.eachBefore((node) => {
      if (node.y < top.y) {
        top = node;
      }

      if (node.y > bottom.y) {
        bottom = node;
      }
    });

    const height = bottom.y - top.y + margin.top + margin.bottom;

    const transition = this.svgTree
      .transition()
      .duration(transitionDuration)
      .attr("viewBox", [-margin.left, -margin.top, width, height]);

    // Update the nodes…
    const node = this.gNode.selectAll("g").data(nodes, (d) => d.id);

    // Enter any new nodes at the parent's previous position.
    const nodeEnter = node
      .enter()
      .append("g")
      .attr("transform", () => `translate(${source.x0},${source.y0})`)
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0)
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
      .style("stroke", "black")
      .style("fill", (d) => d.data.color ?? "none")
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
      .attr("fill-opacity", 1)
      .attr("stroke-opacity", 1);

    // Transition exiting nodes to the parent's new position.
    node
      .exit()
      .transition(transition)
      .remove()
      .attr("transform", () => `translate(${source.x},${source.y})`)
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0);

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
        const children = getDescendants(d.source.data);
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

  #updateTail(tail) {
    const tailElements = tail.elements.filter(
      (d) => d !== null && d !== undefined
    );
    const node = this.gNodeTail
      .selectAll("g")
      .data(tailElements, (d) => `${d}:tail`);
    const nodeEnter = node.enter().append("g");

    nodeEnter
      .append("rect")
      .style("stroke-width", "1px")
      .style("stroke", "black")
      .style("fill", tail.color ?? "none")
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
      .attr("fill-opacity", 1)
      .attr("stroke-opacity", 1);

    node
      .exit()
      .transition()
      .remove()
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0);
  }
}
