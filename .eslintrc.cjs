'use strict';

module.exports = {
  "env": {
    "mocha": true,
    "commonjs": true,
    "es2021": true
  },
  "extends": "eslint:recommended",
  "parserOptions": {
    "ecmaVersion": "latest"
  },
  overrides: [
    {
      files: ['pkg/**/*.{js,mjs}'],
      env: {
        commonjs: false,
        module: true,
      },

    }
  ]
}
