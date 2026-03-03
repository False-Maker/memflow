# UI Decisions

## Neon Color System Update

**Date**: 2026-03-04  
**Task**: Update tailwind.config.js design tokens

### Changes Made

1. **Updated Signal Color**: Changed from `#F59E0B` (amber) to `#00f0ff` (neon-cyan) for primary emphasis
2. **Added Neon Colors**: 
   - `neon-cyan: '#00f0ff'` - Primary emphasis color
   - `neon-red: '#ff003c'` - Secondary emphasis/danger color
3. **Updated Aliases**:
   - `neon-blue` now points to `#00f0ff` (neon-cyan) instead of amber
   - Maintained backward compatibility with existing tokens

### Design Rationale

- Adopting Digital Horizon's color scheme for consistency
- Neon-cyan (#00f0ff) serves as the primary signal/emphasis color
- Neon-red (#ff003c) provides a strong secondary color for danger/warning states
- Maintained legacy token compatibility to prevent breaking changes

### Files Modified

- `tailwind.config.js`: Updated colors section with new definitions and aliases

### Verification

- ✅ New colors defined in config
- ✅ Signal and neon-blue aliases updated to point to neon-cyan
- ✅ Existing amber token preserved in legacy mapping (commented out)

## Glass Effect Restoration

**Date**: 2026-03-04  
**Task**: Restore glass effect in src/index.css

### Changes Made

1. **Restored .glass class**: Changed from solid bg-void to glass effect with `bg-white/5 backdrop-blur-md border border-white/10`
2. **Added glass variants**:
   - `.glass-strong`: `bg-white/10 backdrop-blur-lg border border-white/20` - stronger blur effect
   - `.glass-subtle`: `bg-white/3 backdrop-blur-sm border border-white/5` - subtle blur effect
3. **Restored neon-glow**: Added `shadow-[0_0_15px_rgba(0,240,255,0.3)]` for neon glow effect
4. **Preserved compatibility**: Kept existing .elucid-panel and .elucid-btn classes unchanged

### Design Rationale

- Adopted Digital Horizon's glass effect styling: `bg-white/5 backdrop-blur-md border border-white/10`
- Added multiple glass variants for different intensity needs
- Restored neon glow effect for better visual hierarchy
- Maintained backward compatibility with existing UI components

### Files Modified

- `src/index.css`: Updated @layer utilities section with restored glass effects

### Verification

- ✅ Glass effect restored with proper backdrop-blur-md
- ✅ New glass variants added for different intensity levels
- ✅ Neon glow effect restored
- ✅ grep verification shows correct glass class definitions
- ✅ Existing classes preserved for compatibility