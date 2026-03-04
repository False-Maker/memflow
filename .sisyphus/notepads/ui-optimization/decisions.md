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

## ContextSidebar Color Update

**Date**: 2026-03-04  
**Task**: Update ContextSidebar.tsx to use neon-cyan instead of signal

### Changes Made

1. **Status indicator**: Updated pulse indicator from `bg-signal` to `bg-neon-cyan` with cyan glow `shadow-[0_0_8px_rgba(0,240,255,0.5)]`
2. **Expand hint**: Changed hover text from `text-signal` to `text-neon-cyan`
3. **Active status**: Updated status text from `text-signal` to `text-neon-cyan`
4. **Deep Automation button**: Updated hover border and text from `border-signal/50` and `text-signal` to `border-neon-cyan/50` and `text-neon-cyan`
5. **Waiting state indicator**: Updated pulse indicator from `bg-signal` to `bg-neon-cyan`
6. **Suggested action buttons**: Updated hover border from `border-signal/50` to `border-neon-cyan/50`
7. **Related memories**: Updated hover border and indicator from `border-signal/50` and `bg-signal` to `border-neon-cyan/50` and `bg-neon-cyan`

### Design Rationale

- Consistency with Wave 1 updates: neon-cyan (#00f0ff) is now the unified primary accent color
- Cyan glow effect emphasizes active state with subtle animation
- Maintains hover interactions and transitions unchanged
- All signal references replaced with neon-cyan for visual consistency

### Files Modified

- `src/components/ContextSidebar.tsx`: Updated all signal color references to neon-cyan

### Verification

- ✅ grep confirms zero occurrences of `text-signal`, `bg-signal`, or `border-signal` in ContextSidebar.tsx
- ✅ All status indicators use cyan glow effect
- ✅ All hover states updated to neon-cyan
- ✅ Expand/collapse logic unchanged
- ✅ Event listeners unchanged
- ✅ Tauri invoke calls unchanged

## GalleryView Color Update

**Date**: 2026-03-04  
**Task**: Update GalleryView.tsx to use neon-cyan instead of neon-blue

### Changes Made

1. **Sidebar borders**: Changed `border-glass-border` → `border-white/10` (2 instances)
2. **Sidebar selected state**: Changed `bg-neon-blue/*`, `text-neon-blue`, `border-neon-blue/*` → `neon-cyan` equivalents
3. **Main icons**: Changed `text-neon-blue` → `text-neon-cyan` in headers
4. **Grid border**: Changed `border-glass-border/30` → `border-white/10`
5. **GalleryItem hover**: 
   - Changed `border-neon-blue` → `border-neon-cyan`
   - Updated shadow from `rgba(0,243,255,0.3)` → `rgba(0,240,255,0.3)`
   - Added hover scale `hover:scale-[1.02]`
6. **Loading spinner**: Changed `border-neon-blue` → `border-neon-cyan`
7. **Metadata text**: Changed `text-neon-blue` → `text-neon-cyan`
8. **Preserved OCR tag**: Kept `text-neon-green` for success semantic (as per spec)

### Design Rationale

- Consistency with Wave 1 UI updates: neon-cyan (#00f0ff) is the unified primary accent
- Border transparency updated to match glass effect spec (white/10)
- Added micro-interaction: card hover scale for better tactile feedback
- OCR tag retains neon-green to preserve success semantic meaning
- Grid layout, filtering, and image loading logic unchanged

### Files Modified

- `src/components/GalleryView.tsx`: Updated all neon-blue references to neon-cyan

### Verification

- ✅ `grep -c "neon-blue"` returns 0 in GalleryView.tsx
- ✅ 8 occurrences of neon-cyan found
- ✅ OCR tag maintains neon-green
- ✅ Grid layout logic unchanged
- ✅ Filtering functionality unchanged

## KnowledgeGraph Color Update

**Date**: 2026-03-04  
**Task**: Update KnowledgeGraph.tsx to use neon-cyan instead of neon-purple

### Changes Made

1. **Loading spinner**: Changed `text-neon-purple` → `text-neon-cyan`
2. **Error retry button**: Changed `bg-neon-purple/20`, `text-neon-purple`, `hover:bg-neon-purple/30` → `neon-cyan` equivalents
3. **Header title**: Changed `text-neon-purple` → `text-neon-cyan`
4. **Rebuild button**: Changed `bg-neon-purple/20`, `text-neon-purple`, `hover:bg-neon-purple/30` → `neon-cyan` equivalents
5. **Notice banner**: Changed `text-neon-purple`, `bg-neon-purple/5` → `neon-cyan` equivalents

### Design Rationale

- Consistency with Wave 1-2 UI updates: neon-cyan (#00f0ff) is the unified primary accent
- Zinc color scheme preserved for graph nodes (data visualization best practice)
- Interactive elements use neon-cyan for visual consistency
- Force-directed layout, D3 configuration, and graph data structure unchanged

### Files Modified

- `src/components/KnowledgeGraph.tsx`: Updated all neon-purple references to neon-cyan

### Verification

- ✅ `grep -c "neon-purple"` returns 0 in KnowledgeGraph.tsx
- ✅ All interactive elements use neon-cyan
- ✅ Graph node colors remain zinc-based (app: zinc-100, doc: zinc-400, time: zinc-700)
- ✅ Force layout logic unchanged

## QnA Component Color Update

**Date**: 2026-03-04  
**Task**: Update QnA.tsx to use neon-cyan instead of neon-blue

### Changes Made

1. **Header icon & title**: Changed `text-neon-blue` → `text-neon-cyan` (Sparkles icon, h2)
2. **Loading spinner**: Changed `text-neon-blue` → `text-neon-cyan`
3. **User message bubble**: Changed `bg-neon-blue/10 border-neon-blue/20` → `bg-neon-cyan/10 border-neon-cyan/20`
4. **Thinking indicator**: Changed `text-neon-blue` → `text-neon-cyan`
5. **Input hover/focus**: Changed `border-neon-blue/30` and `ring-neon-blue/30` → `neon-cyan` equivalents
6. **Send button**: Changed `bg-neon-blue/20 text-neon-blue hover:bg-neon-blue/30` → `bg-neon-cyan/20 text-neon-cyan hover:bg-neon-cyan/30`

### Design Rationale

- Consistency with Wave 1-2 UI updates: neon-cyan (#00f0ff) is the unified primary accent
- User bubble now uses neon-cyan for clear contrast with assistant bubble (surface/glass)
- Input field focus ring updated to cyan for consistent interaction feedback
- Send button maintains its prominence with cyan background
- Message rendering, streaming logic, and Tauri event handling unchanged

### Files Modified

- `src/components/QnA.tsx`: Updated all neon-blue references to neon-cyan

### Verification

- ✅ `grep -c "neon-blue"` returns 0 in QnA.tsx
- ✅ 9 occurrences of neon-cyan found
- ✅ Message bubble styles updated (user: cyan bg, assistant: glass bg)
- ✅ Input field focus ring updated
- ✅ Send button updated
- ✅ Streaming listener unchanged
- ✅ Tauri event handling unchanged