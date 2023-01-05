import * as wasm from "web-vis";
import { RrbVec } from "./rrbtree";

wasm.push_vec();
// wasm.push_vec();

const rrbVecOne = new RrbVec("#tree1");
// const rrbVecTwo = new RrbVec("#tree2");

// const svg = d3.select("#branch");

// const cellWidth = 50;
// const cellHeight = 50;

// const array = svg.append("g")
//   .style("stroke-width", "2px")
//   .style("stroke-linecap", "butt")
//   .style("vector-effect", "non-scaling-stroke");

// const cells = array.selectAll("g")
//   .data([1, 2, 3, 4])
//   .enter()
//   .append("g")
//   .attr("transform", (d, i) => `translate(${i * cellWidth}, 0)`);

// cells.append("rect")
//   .attr("width", cellWidth)
//   .attr("height", cellHeight)
//   .style("stroke", "black")
//   .style("fill", "none");

// cells.append("text")
//   .attr("x", cellWidth / 2)
//   .attr("y", cellHeight / 2)
//   .attr("text-anchor", "middle")
//   .attr("dominant-baseline", "central")
//   .text((d, i) => i);

const slider = document.getElementById("vectorSize");
slider.addEventListener("change", function () {
  wasm.set_vec_size(0, slider.value);
  // wasm.set_vec_size(1, slider.value);

  const vectors = wasm.get();

  const vecOne = JSON.parse(vectors[0]);
  console.log(vecOne)
  // const vecTwo = JSON.parse(vectors[1]);

  rrbVecOne.set(vecOne);
  // rrbVecTwo.draw(vecTwo);
});

const splitter = document.getElementById("vectorSplitter");
splitter.addEventListener("click", () => {
  wasm.split_off_vec(0, 167);
  // wasm.split_off_vec(1, 167);

  const vectors = wasm.get();

  const vecOne = JSON.parse(vectors[0]);
  // const vecTwo = JSON.parse(vectors[1]);

  rrbVecOne.set(vecOne);
  // rrbVecTwo.draw(vecTwo);
});
