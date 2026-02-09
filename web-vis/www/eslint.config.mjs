import js from "@eslint/js";
import globals from "globals";
import prettier from "eslint-config-prettier";

export default [
  js.configs.recommended,
  prettier,
  {
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
      },
    },
    rules: {
      curly: "error",
    },
  },
  {
    ignores: [
      "dist/",
      "node_modules/",
      "webpack.config.js",
      "postcss.config.js",
    ],
  },
];
