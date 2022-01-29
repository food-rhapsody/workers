module.exports = {
  // `extends`를 생략해도 이 파일이 있는 위치까지만 부모 eslintrc를 찾도록 제한함
  root: true,

  env: {
    es6: true,
    node: true,
    browser: true,
    'shared-node-browser': true,
  },

  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaFeatures: { jsx: true },
    jsx: true,
    useJSXTextNode: true,
  },

  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended', 'eslint-config-prettier'],
  plugins: ['@typescript-eslint', 'import', 'prettier'],
  settings: { 'import/resolver': { typescript: {} }, react: { version: 'detect' } },

  rules: {
    'prettier/prettier': 'error',
    'no-implicit-coercion': 'error',
    'no-undef': 'off',
    indent: 'off',
    '@typescript-eslint/indent': 'off',
    semi: 'off',
    'getter-return': 'warn',
    '@typescript-eslint/explicit-function-return-type': 'off',
    '@typescript-eslint/no-use-before-define': 'off',
    '@typescript-eslint/no-empty-interface': 'off',
    '@typescript-eslint/no-parameter-properties': 'off',
    'no-restricted-imports': 'error',
    'no-async-promise-executor': 'error',
    '@typescript-eslint/prefer-as-const': 'warn',
    '@typescript-eslint/no-non-null-asserted-optional-chain': 'warn',
    '@typescript-eslint/ban-types': 'warn',
    '@typescript-eslint/no-inferrable-types': 'warn',
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/array-type': ['error', { default: 'array-simple' }],
    '@typescript-eslint/no-unused-vars': ['error', { ignoreRestSiblings: true }],
    'no-warning-comments': [
      'warn',
      {
        terms: ['TODO', 'FIXME', 'XXX', 'BUG'],
        location: 'anywhere',
      },
    ],
    'prefer-const': 'error',
    'no-var': 'error',
    curly: ['error', 'all'],
    eqeqeq: ['error', 'always', { null: 'ignore' }],
    'import/no-duplicates': 'error',
    '@typescript-eslint/no-var-requires': 'error',
  },
  overrides: [
    {
      files: '**/*.js',
      rules: {
        '@typescript-eslint/no-var-requires': 'off',
      },
    },
  ],
};
