// @ts-check

import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';

export default tseslint.config(
	js.configs.recommended,
	...tseslint.configs.strictTypeChecked,
	reactHooks.configs.recommended,
	reactRefresh.configs.vite,
	{ ignores: ['dist', 'cloudflare', 'eslint.config.js'] },
	{
		files: ['**/*.{ts,tsx}'],
	},
	{
		languageOptions: {
			globals: globals.browser,
			parserOptions: {
				projectService: true,
				tsconfigRootDir: import.meta.dirname,
			},
		},
		rules: {
			'react-hooks/exhaustive-deps': 'error',
			'react-hooks/react-compiler': 'error',
			'@typescript-eslint/promise-function-async': 'error',
		},
	},
);
