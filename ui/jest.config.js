module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testTimeout: 30000,
  globals: {
    'ts-jest': {
      diagnostics: false,
    },
  },
};
