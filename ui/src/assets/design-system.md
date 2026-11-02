# SnapCord Design System

## Color Palette

### Backgrounds
- `bg-void`: `#050505` (OLED black)
- `bg-glass`: `rgba(10, 10, 10, 0.8)` (glass panels)
- `bg-glass-light`: `rgba(255, 255, 255, 0.03)` (subtle surfaces)

### Borders
- `border-hairline`: `rgba(255, 255, 255, 0.1)` (subtle borders)
- `border-glow`: `rgba(168, 85, 247, 0.5)` (purple glow on hover)

### Accents
- `accent-purple`: `#A855F7` (primary accent)
- `accent-emerald`: `#10B981` (secondary accent)
- `gradient-primary`: `linear-gradient(135deg, #A855F7 0%, #10B981 100%)`

### Text
- `text-primary`: `#FFFFFF` (main text)
- `text-secondary`: `rgba(255, 255, 255, 0.7)` (subtle text)
- `text-muted`: `rgba(255, 255, 255, 0.5)` (disabled/muted)

## Typography

### Font Family
- Primary: `'Geist', 'Inter', system-ui, sans-serif`
- Display: `'Clash Display', 'Geist', sans-serif` (for headings)

### Scale
- `text-xs`: 12px (metadata, timestamps)
- `text-sm`: 14px (body, labels)
- `text-base`: 16px (default)
- `text-lg`: 18px (subheadings)
- `text-xl`: 20px (section titles)
- `text-2xl`: 24px (modal titles)
- `text-4xl`: 36px (hero headings)

### Weights
- `font-light`: 300 (elegant text)
- `font-normal`: 400 (body)
- `font-medium`: 500 (labels, buttons)
- `font-semibold`: 600 (headings)
- `font-bold`: 700 (emphasis)

## Spacing

### Scale (8px base)
- `space-1`: 4px
- `space-2`: 8px
- `space-3`: 12px
- `space-4`: 16px
- `space-6`: 24px
- `space-8`: 32px
- `space-12`: 48px
- `space-16`: 64px
- `space-24`: 96px

### Section Padding
- Vertical: `py-24` to `py-32` (96px - 128px)
- Horizontal: `px-8` to `px-16` (32px - 64px)

## Border Radius

- `rounded-sm`: 8px (small elements)
- `rounded-md`: 12px (buttons, inputs)
- `rounded-lg`: 16px (cards)
- `rounded-xl`: 20px (panels)
- `rounded-2xl`: 24px (large panels)
- `rounded-3xl`: 32px (image cards)
- `rounded-full`: 9999px (pills, avatars)

## Shadows

### Depth System
- `shadow-sm`: `0 2px 8px rgba(0, 0, 0, 0.3)` (subtle lift)
- `shadow-md`: `0 8px 24px rgba(0, 0, 0, 0.4)` (card hover)
- `shadow-lg`: `0 16px 48px rgba(0, 0, 0, 0.5)` (modal, floating)
- `shadow-glow`: `0 0 24px rgba(168, 85, 247, 0.3)` (accent glow)

### Glass Effect
```css
backdrop-filter: blur(24px);
background: rgba(10, 10, 10, 0.8);
border: 1px solid rgba(255, 255, 255, 0.1);
```

## Motion

### Easing
- `ease-out-expo`: `cubic-bezier(0.16, 1, 0.3, 1)` (smooth deceleration)
- `ease-in-out-quart`: `cubic-bezier(0.76, 0, 0.24, 1)` (balanced)
- `ease-spring`: `cubic-bezier(0.34, 1.56, 0.64, 1)` (bouncy)

### Duration
- `duration-fast`: 150ms (micro-interactions)
- `duration-normal`: 300ms (standard transitions)
- `duration-slow`: 500ms (major state changes)
- `duration-slower`: 800ms (entrance animations)

## Components

### Double-Bezel Card
```css
/* Outer shell */
background: rgba(255, 255, 255, 0.03);
border: 1px solid rgba(255, 255, 255, 0.05);
padding: 6px;
border-radius: 32px;

/* Inner core */
background: rgba(20, 20, 20, 0.9);
border-radius: 26px;
box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.1);
```

### Glass Button
```css
background: rgba(255, 255, 255, 0.05);
border: 1px solid rgba(255, 255, 255, 0.1);
border-radius: 9999px;
padding: 12px 24px;
backdrop-filter: blur(12px);
```

### Primary Button
```css
background: linear-gradient(135deg, #A855F7 0%, #10B981 100%);
border-radius: 9999px;
padding: 12px 24px;
box-shadow: 0 4px 16px rgba(168, 85, 247, 0.3);
```

## Breakpoints

- `sm`: 640px
- `md`: 768px
- `lg`: 1024px
- `xl`: 1280px
- `2xl`: 1536px

## Z-Index Layers

- `z-base`: 0 (default)
- `z-dropdown`: 100
- `z-sticky`: 200
- `z-overlay`: 300
- `z-modal`: 400
- `z-popover`: 500
- `z-tooltip`: 600
