/** @type {import('tailwindcss').Config} */
import { colors, fontFamily, } from "./styles/theme.js";

module.exports = {
	content: {
		files: ["*.html", "./src/**/*.rs"],
	},
	theme: {
		extend: {
			colors,
			fontFamily,
			transitionPropety: {
				"width": "width"
			}
		},
	},
	plugins: [
		require('@tailwindcss/forms'),
		require('tailwind-scrollbar'),
	],
}
