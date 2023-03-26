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

  set(vec) {
    this.root = d3.hierarchy(vec.tree.root, getDescendants);

    this.root.x0 = dy / 2;
    this.root.y0 = 0;

    let descendants = this.root.descendants();
    let next_node_to_expand = descendants ? descendants[0].data : null;

    // descendants are sorted in topological order
    descendants.forEach((d) => {
      const { children, data } = d;

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

    this.updateTree(this.root);

    // TODO: pass something else to be similar to updateTree?
    this.updateTail(vec.tail);
  }

  updateTree(source) {    
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
      .attr("viewBox", [-margin.left, -margin.top, width, height])
      .tween(
        "resize",
        window.ResizeObserver ? null : () => () => svg.dispatch("toggle")
      );

    // Update the nodes…
    const node = this.gNode.selectAll("g").data(nodes, (d) => d.id);

    // Enter any new nodes at the parent's previous position.
    const nodeEnter = node
      .enter()
      .append("g")
      .attr("transform", () => `translate(${source.x0},${source.y0})`)
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0)
      .on("click", (event, d) => {
        // TODO: this function has to be assigned/updated properly?
        // otherwise it can capture variables/references and that can lead to a bug
        console.log("about to do something with this node", d);
        d.children = d.children ? null : d._children;
        this.updateTree(d);
      });

    nodeEnter
      .selectAll("rect")
      .data((d) => Array.from({ length: d.data.len }, () => d.data.len))
      .enter()
      .append("rect")
      .style("stroke-width", "1px")
      .style("stroke", "black")
      .style("fill", "none")
      .attr("width", arrayCellWidth)
      .attr("height", arrayCellHeight)
      .attr(
        "transform",
        (len, i) => `translate(${(i - len / 2) * arrayCellWidth}, 0)`
      );

    nodeEnter
      .selectAll("text")
      .data((d) =>
        Array.from(d.data.leaf || [], (item) => ({ item, len: d.data.len }))
      )
      .enter()
      .append("text")
      .attr(
        "transform",
        (d, i) =>
          `translate(${
            (i - d.len / 2) * arrayCellWidth + arrayCellWidth / 2
          }, ${arrayCellHeight + arrayCellHeight * 0.6}) rotate(270)`
      )
      .attr("dy", "0.31em")
      .attr("x", 8)
      .attr("text-anchor", "end")
      .text((d) => d.item)
      .clone(true)
      .lower()
      .attr("stroke-linejoin", "round")
      .attr("stroke-width", 3)
      .attr("stroke", "white");

    nodeEnter
      .selectAll("text")
      .data((d) =>
        Array.from(d.data.sizes || [], (item) => ({ item, len: d.data.len }))
      )
      .enter()
      .append("text")
      .attr(
        "transform",
        (d, i) =>
          `translate(${
            (i - d.len / 2) * arrayCellWidth + arrayCellWidth / 2
          }, ${-0.6 * arrayCellHeight}) rotate(315)`
      )
      .attr("dy", "0.31em")
      .attr("x", 8)
      .attr("text-anchor", "end")
      .text((d) => d.item)
      .clone(true)
      .lower()
      .attr("stroke-linejoin", "round")
      .attr("stroke-width", 3)
      .attr("stroke", "white");

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
      .attr("transform", (d) => `translate(${source.x},${source.y})`)
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

    // TODO: de-dupe this code
    const getDescendants = (node) => {
      if (node.branch) {
        return node.branch;
      } else if (node.relaxedBranch) {
        return node.relaxedBranch;
      }
    };

    // Transition links to their new position.
    link
      .merge(linkEnter)
      .transition(transition)
      .attr("d", (d) => {
        const childNodePosition = getDescendants(d.source.data).indexOf(
          d.target.data
        );
        const halfCellWidth = arrayCellWidth / 2;

        // TODO: making assumptions about branching factor is not great + array is ugly.
        const offsets = {
          1: [0],
          2: [-halfCellWidth, halfCellWidth],
          3: [-arrayCellWidth, 0, arrayCellWidth],
          4: [
            -arrayCellWidth - halfCellWidth,
            -halfCellWidth,
            halfCellWidth,
            arrayCellWidth + halfCellWidth,
          ],
        };

        let sourceX =
          d.source.x + offsets[d.source.data.len][childNodePosition];

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
      .attr("d", (d) => {
        const o = { x: source.x, y: source.y };
        return diagonal({ source: o, target: o });
      });

    // Stash the old positions for transition.
    this.root.eachBefore((node) => {
      node.x0 = node.x;
      node.y0 = node.y;
    });
  }

  updateTail(tail) {
    // TODO: this is ugly
    tail = tail.filter((d) => d !== null);

    const node = this.gNodeTail.selectAll("g").data(tail, (d) => `${d}:tail`);
    const nodeEnter = node.enter().append("g");

    nodeEnter
      .append("rect")
      .style("stroke-width", "1px")
      .style("stroke", "black")
      .style("fill", "none")
      .attr("width", arrayCellWidth)
      .attr("height", arrayCellHeight)
      .attr("transform", (_val, i) => `translate(${i * arrayCellWidth}, 0)`);

    // TODO: the problem here is that the underlayer contrast text is being kept, while the top level text gets removed.
    // TODO: somehow both have to stay in sync
    nodeEnter
      .append("text")
      .attr(
        "transform",
        (_d, i) =>
          `translate(${i * arrayCellWidth + arrayCellWidth / 2}, ${
            arrayCellHeight + arrayCellHeight * 0.6
          }) rotate(270)`
      )
      .attr("dy", "0.31em")
      .attr("x", 8)
      .attr("text-anchor", "end")
      .text((d) => d)
      .clone(true)
      .lower() // TODO: what the hell is underlayer for?
      .attr("stroke-linejoin", "round")
      .attr("stroke-width", 3)
      .attr("stroke", "white");

    node
      .merge(nodeEnter)
      .transition() // TODO: its interesting that transition works out of the box
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
