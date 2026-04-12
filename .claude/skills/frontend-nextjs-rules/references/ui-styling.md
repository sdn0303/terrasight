# UI Styling and Accessibility

## Contents

- [UI Styling and Accessibility](#ui-styling-and-accessibility)
  - [Contents](#contents)
  - [Tailwind CSS v4](#tailwind-css-v4)
  - [shadcn/ui](#shadcnui)
  - [Design Tokens](#design-tokens)
  - [Accessibility](#accessibility)
  - [Web Interface Guidelines](#web-interface-guidelines)

---

## Tailwind CSS v4

CSS-first configuration — no `tailwind.config.js`:

```css
@import "tailwindcss";

@theme {
  --font-display: "Geist Sans", sans-serif;
  --font-mono: "Geist Mono", monospace;
  --color-accent-500: oklch(0.84 0.18 117.33);
  --spacing-sidebar: 280px;
}
```

Key features:

- Native cascade layers for style precedence control
- `color-mix()` for opacity adjustment of any color value
- `@property` for animatable custom properties
- Container queries with `@container` (no plugin needed)
- `@starting-style` variant for enter/exit transitions

## shadcn/ui

- Style variant: `new-york`
- RSC-compatible — import from `@/components/ui/`
- Use Radix primitives for accessible base components
- Customize via CSS variables, not component props

## Design Tokens

Defined in `globals.css` `:root` variables with Tailwind `@theme`:

- Prefix: `ds-*` for project design tokens
- Dark mode: CSS variables toggled via `[data-theme="dark"]`
- Geist Sans for UI text, Geist Mono for data/metrics
- zinc/neutral tokens with one accent color

## Accessibility

- Semantic HTML elements (`nav`, `main`, `article`, `section`, `button`)
- ARIA attributes when semantics alone are insufficient
- Keyboard navigation for all interactive elements
- Focus-visible styles (never remove outline without replacement)
- Color contrast: WCAG AA minimum (4.5:1 text, 3:1 large text)
- Touch targets: minimum 44x44px
- Reduced motion: respect `prefers-reduced-motion`

## Web Interface Guidelines

For comprehensive UI review, fetch the latest Vercel Web Interface Guidelines:

```text
https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md
```

Apply all rules from the fetched guidelines to the files being reviewed.
