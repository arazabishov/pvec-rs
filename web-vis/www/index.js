import * as wasm from "web-vis";
import * as d3 from "d3";

const margin = { top: 32, right: 120, bottom: 10, left: 512 };
const width = 1400 + 512 - margin.left - margin.right;

const dy = width / 12;
const dx = 12;

const diagonal = d3
  .linkVertical()
  .x((d) => d.x)
  .y((d) => d.y);
const tree = d3.tree().nodeSize([dx, dy]);

wasm.add_vec(427);

let vec = JSON.parse(wasm.get()[0]);
console.log(wasm.get()[0]);

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
  } else {
    return node;
  }
};

function drawCollapsibleTree() {
  const root = d3.hierarchy(vec.tree.root, getDescendants);

  root.x0 = dy / 2;
  root.y0 = 0;

  let descendants = root.descendants();
  
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
        next_node_to_expand = children ? children[next_node_to_expand_i++] : null;
      }
    } else {
      d.children = null;
    }
  });

  // TODO: the first two lines have to change. This function should
  // create an SVG and then return it to the caller, so then
  // the caller can place it on the page appropriately.
  const svg = d3
    .select("body")
    .append("svg")
    .attr("viewBox", [-margin.left, -margin.top, width, dx])
    .style("font", "10px sans-serif")
    .style("user-select", "none");

  const gLink = svg
    .append("g")
    .attr("fill", "none")
    .attr("stroke", "#555")
    .attr("stroke-opacity", 0.4)
    .attr("stroke-width", 1.5);

  const gNode = svg
    .append("g")
    .attr("cursor", "pointer")
    .attr("pointer-events", "all");

  function update(source) {
    const duration = d3.event && d3.event.altKey ? 2500 : 250;
    const nodes = root.descendants().reverse();
    const links = root.links();

    // Compute the new tree layout.
    tree(root);

    let top = root;
    let bottom = root;
    root.eachBefore((node) => {
      if (node.y < top.y) top = node;
      if (node.y > bottom.y) bottom = node;
    });

    // +32 is a hack
    const height = bottom.y - top.y + margin.top + margin.bottom + 32;

    const transition = svg
      .transition()
      .duration(duration)
      .attr("viewBox", [-margin.left, -margin.top, width, height])
      .tween(
        "resize",
        window.ResizeObserver ? null : () => () => svg.dispatch("toggle")
      );

    // Update the nodes…
    const node = gNode.selectAll("g").data(nodes, (d) => d.id);

    // Enter any new nodes at the parent's previous position.
    const nodeEnter = node
      .enter()
      .append("g")
      .attr("transform", (d) => `translate(${source.x0},${source.y0})`)
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0)
      .on("click", (event, d) => {
        d.children = d.children ? null : d._children;
        update(d);
      });

    nodeEnter
      .append("circle")
      .attr("r", 2.5)
      .attr("fill", (d) => (d._children ? "#555" : "#999"))
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
    const nodeUpdate = node
      .merge(nodeEnter)
      .transition(transition)
      .attr("transform", (d) => `translate(${d.x},${d.y})`)
      .attr("fill-opacity", 1)
      .attr("stroke-opacity", 1);

    // Transition exiting nodes to the parent's new position.
    const nodeExit = node
      .exit()
      .transition(transition)
      .remove()
      .attr("transform", (d) => `translate(${source.x},${source.y})`)
      .attr("fill-opacity", 0)
      .attr("stroke-opacity", 0);

    // Update the links…
    const link = gLink.selectAll("path").data(links, (d) => d.target.id);

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
    root.eachBefore((d) => {
      d.x0 = d.x;
      d.y0 = d.y;
    });
  }

  update(root);
}

drawCollapsibleTree();

var slider = document.getElementById("vectorSize");
slider.addEventListener("change", function () {
  // TODO: remove this hack
  d3.select("svg").remove();

  // drawTree(slider.value);
  drawCollapsibleTree();
});

var splitter = document.getElementById("vectorSplitter");
splitter.addEventListener("click", () => {
  wasm.split_vec(0, 29);

  // TODO: remove this hack
  d3.select("svg").remove();
  drawCollapsibleTree();
});
