/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ["./templates/*.html", "./templates/**/*.html"],
  theme: {
    extend: {
            colors: {
                'nord':'#2E3440',
            }
        },
  },
  plugins: [],
}

