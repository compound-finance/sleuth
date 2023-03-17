module.exports = {
  transform: {"^.+\\.(t|j)sx?$": ["@swc/jest"]},
  testEnvironment: 'node',
  testRegex: 'cli/test/.*\\.(test|spec)?\\.(ts|tsx)$',
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
  transformIgnorePatterns: ["node_modules/.*"],
};
