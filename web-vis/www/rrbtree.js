import * as d3 from "d3";

const margin = { top: 32, right: 120, bottom: 42, left: 512 };
const width = 3400 + 512 - margin.left - margin.right;

const arrayCellWidth = 20;
const arrayCellHeight = 20;

const dy = width / 24;
const dx = arrayCellWidth * 1.5;

const diagonal = d3
  .linkVertical()
  .x((d) => d.x)
  .y((d) => d.y);
const tree = d3.tree().nodeSize([dx, dy]);

const getDescendants = (node) => {
  if (!node) {
    return null;
  }

  if (node.relaxedBranch) {
    return node.relaxedBranch;
  } else if (node.branch) {
    return node.branch;
  } else if (node.leaf) {
    return node.leaf;
  }

  return null;
};

export class RrbVec {
  constructor(selector) {
    this.svg = d3
      .select(selector)
      .append("svg")
      .attr("viewBox", [-margin.left, -margin.top, width, dx])
      .style("font", "10px sans-serif")
      .style("user-select", "none");

    this.gLink = this.svg
      .append("g")
      .attr("fill", "none")
      .attr("stroke", "#555")
      .attr("stroke-opacity", 0.4)
      .attr("stroke-width", 1.5);

    this.gNode = this.svg
      .append("g")
      .attr("cursor", "pointer")
      .attr("pointer-events", "all");
  }

  set(vec) {
    this.root = d3.hierarchy(vec.tree.root, getDescendants);

    this.root.x0 = dy / 2;
    this.root.y0 = 0;

    let descendants = this.root.descendants();

    let next_node_to_expand = descendants ? descendants[0].data : null;
    let next_node_to_expand_i = 0;

    // descendants are sorted in topological order
    descendants.forEach((d, i) => {
      const { children, data, parent } = d;

      d.id = i;
      d._children = children;

      // keep only the right-most branches expanded to save space
      if (next_node_to_expand === data) {
        // if we have not exhausted all children of the parent, keep increasing the index
        if (parent && next_node_to_expand_i < parent.data.len) {
          const children = getDescendants(parent.data);
          next_node_to_expand = children[next_node_to_expand_i++];
        } else {
          const children = getDescendants(data);

          next_node_to_expand_i = 0;
          next_node_to_expand = children
            ? children[next_node_to_expand_i++]
            : null;
        }
      } else {
        d.children = null;
      }
    });

    this.update(this.root);
  }

  update(source) {
    const duration = d3.event && d3.event.altKey ? 2500 : 2500;
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

    const transition = this.svg
      .transition()
      .duration(duration)
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
      .attr("transform", (d) => `translate(${source.x0},${source.y0})`)
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0)
      .on("click", (event, d) => {
        // TODO: this function has to be assigned/updated properly?
        // otherwise it can capture variables/references and that can lead to a bug
        console.log("about to do something with this node", d);
        d.children = d.children ? null : d._children;
        this.update(d);
      });

    nodeEnter
      .append("circle")
      .attr("r", 2.5)
      .attr("fill", (_d) => "#999")
      .attr("stroke-width", 10);

    nodeEnter
      .append("text")
      .attr("transform", (d) => `rotate(90)`)
      .attr("dy", "0.31em")
      .attr("x", (d) => (d._children ? -8 : 8))
      .attr("text-anchor", (d) => (d._children ? "end" : "start"))
      .text((d) => {
        if (Number.isInteger(d.data)) {
          return d.data;
        }

        return null;
      })
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
      .attr("d", (d) => {
        const o = { x: source.x0, y: source.y0 };
        return diagonal({ source: o, target: o });
      });

    // Transition links to their new position.
    link.merge(linkEnter).transition(transition).attr("d", diagonal);

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
}