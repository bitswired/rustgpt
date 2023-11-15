/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./**/*.{html,js}"],
  plugins: [require("@tailwindcss/forms"), require("@tailwindcss/typography")],
};
