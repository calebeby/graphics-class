module.exports = {
  extends: [],
  parser: "@typescript-eslint/parser",
  plugins: ["@typescript-eslint"],
  root: true,
  overrides: [
    {
      files: ["*.tsx", "*.ts"], // Tell eslint about more exts without the --exts CLI flag
    },
  ],
  rules: {
    "@typescript-eslint/naming-convention": [
      "error",
      {
        selector: "default",
        format: ["snake_case"],
      },
      {
        selector: "objectLiteralProperty",
        modifiers: ["requiresQuotes"],
        format: null,
      },
      {
        selector: ["variable", "function"],
        format: ["snake_case", "UPPER_CASE", "PascalCase"],
      },
      {
        selector: "typeLike",
        format: ["PascalCase"],
      },
    ],
  },
};
