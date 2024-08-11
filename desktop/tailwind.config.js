/** @type {import('tailwindcss').Config} */
import { colors, fonts } from "./styles/theme.js";

module.exports = {
	content: {
		files: ["*.html", "./src/**/*.rs"],
	},
	theme: {
		extend: {
			colors,
			fonts,
		},
	},
	plugins: [],
}
