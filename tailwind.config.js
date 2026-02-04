/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        void: '#050505', // Deep Matte Black
        surface: '#18181B', // Zinc 900
        signal: '#F59E0B', // Amber 500
        muted: '#71717A', // Zinc 500
        border: '#27272A', // Zinc 800

        // Legacy Compatibility (Mapped to Elucid)
        'neon-blue': '#F59E0B', // Map to Signal
        'neon-purple': '#A1A1AA', // Zinc 400
        'neon-green': '#10B981', // Emerald 500
        'neon-red': '#EF4444', // Red 500
        'neon-pink': '#EC4899', // Pink 500
        'glass-border': '#27272A', // Map to Border
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      borderRadius: {
        'none': '0',
        'sm': '2px', // Very slight radius for "engineered" look
      },
      animation: {
        'scan': 'scan 2s linear infinite',
        'blink': 'blink 1s steps(2, start) infinite',
      },
      keyframes: {
        scan: {
          '0%': { backgroundPosition: '0% 0%' },
          '100%': { backgroundPosition: '0% 100%' },
        },
        blink: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0' },
        }
      }
    },
  },
  plugins: [],
}

