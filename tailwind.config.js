/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        void: '#050505',
        surface: '#121214',
        'glass-border': 'rgba(255, 255, 255, 0.08)',
        'neon-blue': '#2DE2E6',
        'neon-purple': '#9D4EDD',
        'neon-red': '#FF3864',
        'neon-green': '#02C39A',
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      boxShadow: {
        'neon': '0 0 10px rgba(45, 226, 230, 0.5)', // 发光效果
      },
      animation: {
        'pulse-slow': 'pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite', // 呼吸灯
      }, 
    },
  },
  plugins: [],
}

