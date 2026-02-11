const HtmlWebpackPlugin = require("html-webpack-plugin");

const path = require("path");

module.exports = (_, argv) => {
  const isProduction = argv.mode === "production";

  return {
    entry: "./index.js",
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: isProduction ? "[name].[contenthash:8].js" : "[name].bundle.js",
      chunkFilename: isProduction
        ? "[id].[contenthash:8].js"
        : "[id].bundle.js",
      clean: true,
    },
    mode: argv.mode || "development",
    devtool: isProduction ? "source-map" : "eval-source-map",
    experiments: {
      asyncWebAssembly: true,
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          use: ["style-loader", "css-loader", "postcss-loader"],
        },
      ],
    },
    plugins: [
      new HtmlWebpackPlugin({
        template: "./index.html",
      }),
    ],
  };
};
