module.exports = {
  extends: [
    // add more generic rule sets here, such as:
    // 'eslint:recommended',
    "plugin:svelte/recommended",
  ],
  rules: {
    "@typescript-eslint/no-unused-vars": [
      "warn",
      {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^$$(Props|Events|Slots|Generic)$",
      },
    ],
  },
};
