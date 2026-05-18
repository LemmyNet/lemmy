import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";

export default [
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  {
    languageOptions: {
      parser: tseslint.parser,
    },
  },
  // For some reason this has to be in its own block
  {
    ignores: [
      "putTypesInIndex.js",
      "dist/*",
      "docs/*",
      ".yalc",
      "jest.config.js",
    ],
  },
  {
    files: ["src/**/*"],
  },
];
