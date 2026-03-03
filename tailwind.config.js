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
        signal: '#00f0ff', // Neon Cyan - Primary emphasis
        muted: '#71717A', // Zinc 500
        border: '#27272A', // Zinc 800

        // Neon colors for UI elements
        'neon-cyan': '#00f0ff', // Primary emphasis color
        'neon-red': '#ff003c', // Secondary emphasis/Danger color

        // Legacy Compatibility (Mapped to Elucid)
        'neon-blue': '#00f0ff', // Map to Neon Cyan
        'neon-purple': '#A1A1AA', // Zinc 400
        'neon-green': '#10B981', // Emerald 500
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

