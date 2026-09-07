import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";

export default [
  pluginJs.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
      },
    },
  },
  // For some reason this has to be in its own block
  {
    ignores: [
      "eslint.config.mjs",
      "putTypesInIndex.js",
      "dist/*",
      "docs/*",
      ".yalc",
      "jest.config.js",
    ],
  },
  {
    files: ["src/**/*"],
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_" },
      ],
    },
  },
];
