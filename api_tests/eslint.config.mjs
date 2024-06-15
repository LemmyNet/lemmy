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
    ignores: ["putTypesInIndex.js", "dist/*", "docs/*", ".yalc", "jest.config.js"],
  },
  {
    files: ["src/**/*"],
    rules: {
      "@typescript-eslint/no-empty-interface": 0,
      "@typescript-eslint/no-empty-function": 0,
      "@typescript-eslint/ban-ts-comment": 0,
      "@typescript-eslint/no-explicit-any": 0,
      "@typescript-eslint/explicit-module-boundary-types": 0,
      "@typescript-eslint/no-var-requires": 0,
      "arrow-body-style": 0,
      curly: 0,
      "eol-last": 0,
      eqeqeq: 0,
      "func-style": 0,
      "import/no-duplicates": 0,
      "max-statements": 0,
      "max-params": 0,
      "new-cap": 0,
      "no-console": 0,
      "no-duplicate-imports": 0,
      "no-extra-parens": 0,
      "no-return-assign": 0,
      "no-throw-literal": 0,
      "no-trailing-spaces": 0,
      "no-unused-expressions": 0,
      "no-useless-constructor": 0,
      "no-useless-escape": 0,
      "no-var": 0,
      "prefer-const": 0,
      "prefer-rest-params": 0,
      "quote-props": 0,
      "unicorn/filename-case": 0,
    },
  },
];
