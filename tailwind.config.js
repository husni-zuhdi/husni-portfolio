/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ["./internal/templates/*.html", "./internal/templates/**/*.html"],
  theme: {
    extend: {
            colors: {
                'nord':'#2E3440',
            }
        },
  },
  plugins: [],
}

