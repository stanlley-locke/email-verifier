/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        brand: {
          dark: '#165039',
          DEFAULT: '#1E6446',
          light: '#E8F5E9',
          accent: '#AEE4C4',
        },
        bg: {
          main: '#F4F7F5',
          card: '#FFFFFF',
          sidebar: '#F8F9FA'
        },
        text: {
          main: '#1A1D1F',
          muted: '#6F767E'
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
      }
    },
  },
  plugins: [],
}
