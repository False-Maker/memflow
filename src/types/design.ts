/**
 * Design Token Type Definitions
 * 
 * This file contains TypeScript type definitions for design tokens used throughout the UI.
 * These types ensure consistency and type safety when using design tokens.
 */

/**
 * Design Token Type containing all color tokens
 */
export interface DesignToken {
  /** Main colors */
  void: '#050505'; // Deep Matte Black
  surface: '#18181B'; // Zinc 900
  signal: '#00f0ff'; // Neon Cyan - Primary emphasis
  muted: '#71717A'; // Zinc 500
  border: '#27272A'; // Zinc 800
  
  /** Neon colors for UI elements */
  'neon-cyan': '#00f0ff'; // Primary emphasis color
  'neon-red': '#ff003c'; // Secondary emphasis/Danger color
  
  /** Legacy Compatibility (Mapped to Elucid) */
  'neon-blue': '#00f0ff'; // Map to Neon Cyan
  'neon-purple': '#A1A1AA'; // Zinc 400
  'neon-green': '#10B981'; // Emerald 500
  'neon-pink': '#EC4899'; // Pink 500
  'glass-border': '#27272A'; // Map to Border
}

/**
 * Animation Duration Type
 * Defines standard animation durations for consistent motion
 */
export interface AnimationDuration {
  /** Fast animations (150ms) - for immediate feedback */
  fast: '150ms';
  
  /** Normal animations (250ms) - default for most animations */
  normal: '250ms';
  
  /** Slow animations (350ms) - for subtle transitions */
  slow: '350ms';
  
  /** Extra slow animations (500ms) - for dramatic effects */
  slower: '500ms';
}

/**
 * Spacing Scale Type
 * Defines standard spacing values using Tailwind's spacing scale
 */
export interface SpacingScale {
  /** Extra small spacing (4px) */
  'xs': '0.25rem'; // 4px
  
  /** Small spacing (8px) */
  'sm': '0.5rem'; // 8px
  
  /** Medium spacing (12px) */
  'md': '0.75rem'; // 12px
  
  /** Normal spacing (16px) */
  'normal': '1rem'; // 16px
  
  /** Large spacing (24px) */
  'lg': '1.5rem'; // 24px
  
  /** Extra large spacing (32px) */
  'xl': '2rem'; // 32px
  
  /** Extra extra large spacing (48px) */
  '2xl': '3rem'; // 48px
  
  /** Extra extra extra large spacing (64px) */
  '3xl': '4rem'; // 64px
}

/**
 * Border Radius Type
 * Defines border radius values for consistent corner styling
 */
export interface BorderRadius {
  /** No border radius */
  none: '0';
  
  /** Small border radius (2px) - for engineered look */
  sm: '2px';
}

/**
 * Font Family Type
 * Defines standard font families
 */
export interface FontFamily {
  /** Sans-serif font stack */
  sans: string[];
  
  /** Monospace font stack */
  mono: string[];
}

/**
 * Tailwind Theme Extension Type
 * Extends Tailwind theme configuration with design tokens
 */
export interface TailwindThemeExtension {
  colors: DesignToken;
  fontFamily: FontFamily;
  borderRadius: BorderRadius;
  animation: {
    scan: string;
    blink: string;
  };
  keyframes: {
    scan: {
      '0%': { backgroundPosition: string };
      '100%': { backgroundPosition: string };
    };
    blink: {
      '0%, 100%': { opacity: string };
      '50%': { opacity: string };
    };
  };
}

/**
 * Complete Design System Type
 * Combines all design token types for comprehensive type checking
 */
export interface DesignSystem {
  colors: DesignToken;
  spacing: SpacingScale;
  animation: AnimationDuration;
  theme: TailwindThemeExtension;
}

/**
 * Default design tokens
 * Provides default values that can be used throughout the application
 */
export const defaultDesignTokens: DesignToken = {
  void: '#050505',
  surface: '#18181B',
  signal: '#00f0ff',
  muted: '#71717A',
  border: '#27272A',
  'neon-cyan': '#00f0ff',
  'neon-red': '#ff003c',
  'neon-blue': '#00f0ff',
  'neon-purple': '#A1A1AA',
  'neon-green': '#10B981',
  'neon-pink': '#EC4899',
  'glass-border': '#27272A',
};

/**
 * Default animation durations
 */
export const defaultAnimationDurations: AnimationDuration = {
  fast: '150ms',
  normal: '250ms',
  slow: '350ms',
  slower: '500ms',
};

/**
 * Default spacing scale
 */
export const defaultSpacingScale: SpacingScale = {
  'xs': '0.25rem',
  'sm': '0.5rem',
  'md': '0.75rem',
  'normal': '1rem',
  'lg': '1.5rem',
  'xl': '2rem',
  '2xl': '3rem',
  '3xl': '4rem',
};

/**
 * Default border radius values
 */
export const defaultBorderRadius: BorderRadius = {
  none: '0',
  sm: '2px',
};

/**
 * Default font families
 */
export const defaultFontFamily: FontFamily = {
  sans: ['Inter', 'sans-serif'],
  mono: ['JetBrains Mono', 'monospace'],
};

/**
 * Complete default design system
 */
export const defaultDesignSystem: DesignSystem = {
  colors: defaultDesignTokens,
  spacing: defaultSpacingScale,
  animation: defaultAnimationDurations,
  theme: {
    colors: defaultDesignTokens,
    fontFamily: defaultFontFamily,
    borderRadius: defaultBorderRadius,
    animation: {
      scan: 'scan 2s linear infinite',
      blink: 'blink 1s steps(2, start) infinite',
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
};