import * as wasm from "web-vis";
import * as d3 from "d3";

var margin = { top: 100, right: 10, bottom: 240, left: 10 };
var height = 1080 - margin.top - margin.bottom;
var width = 1820 - margin.left - margin.right;

function redrawTree(size) {
    d3.select("svg").remove(); 

    let vec = JSON.parse(wasm.gen_vec(size));
    console.log(vec);

    var orientations = {
        "bottom-to-top": {
            size: [width, height],
            x: function(d) { return d.x; },
            y: function(d) { return d.y; }
        }
    };

    var svg = d3.select("body")
        .selectAll("svg")
        .data(d3.entries(orientations))
        .enter().append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom)
        .append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    svg.each(function(orientation) {

        var svg = d3.select(this);
        var o = orientation.value;

        // Compute the layout.
        var treemap = d3.tree().size(o.size);
            
        var nodes = d3.hierarchy(vec.tree.root, node => {        
            if (node == null) {
                return null;
            }

            // console.log(node);
            if (node.branch != null) {            
                return node.branch;
            } else if (node.leaf != null) {
                return node.leaf;
            } else {
                return node;
            }
        });
        nodes = treemap(nodes);

        var links = nodes.descendants().slice(1);

        // Create the link lines.
        svg.selectAll(".link")
            .data(links)
            .enter().append("path")
            .attr("class", "link")
            .attr("d", function(d) {
            return "M" + d.x + "," + o.y(d)
                + "C" + d.x + "," + (o.y(d) + o.y(d.parent)) / 2
                + " " + d.parent.x + "," +  (o.y(d) + o.y(d.parent)) / 2
                + " " + d.parent.x + "," + o.y(d.parent);
            });

        // Create the node circles.
        var node = svg.selectAll(".node")
            .data(nodes.descendants())
            .enter()
            .append("g");

        node.append("circle")
            .attr("class", "node")
            .attr("r", 4.5)
            .attr("cx", o.x)
            .attr("cy", o.y);

        node.append("text")
            .text(function (d) {
                console.log(d);

                if (Number.isInteger(d.data)) {
                    return d.data;
                } 
                            
                return ""
            })
            .attr("x", o.x)
            .attr("dx", 5)
            .attr("y", o.y);
    });
}

var slider = document.getElementById("vectorSize");
slider.addEventListener("change", function() {
    console.log(slider.value);
    redrawTree(slider.value);
});
