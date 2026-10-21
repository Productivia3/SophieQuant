// Tailwind mapping — SophieQuant design tokens
// tailwind.config.ts (snippet). Referencia de tema vía CSS vars de tokens.css.
export default {
  darkMode: 'class',
  content: ['./src/**/*.{ts,tsx}', './index.html'],
  theme: {
    extend: {
      colors: {
        bg: 'var(--color-bg)',
        surface: { DEFAULT: 'var(--color-surface)', raised: 'var(--color-surface-raised)' },
        border: 'var(--color-border)',
        'border-input': 'var(--color-border-input)',
        text: 'var(--color-text)',
        muted: 'var(--color-text-muted)',
        accent: 'var(--color-accent)',
        'on-accent': 'var(--color-on-accent)',
        danger: 'var(--color-danger)',
        success: 'var(--color-success)',
        warning: 'var(--color-warning)',
        'chart-grid': 'var(--chart-grid)',
        'chart-axis': 'var(--chart-axis)',
      },
      font: {
        sans: 'var(--font-sans)',
        mono: 'var(--font-mono)',
      },
      fontSize: {
        xs: ['var(--text-xs)', { lineHeight: 'var(--lxs)' }],
        sm: ['var(--text-sm)', { lineHeight: 'var(--lsm)' }],
        base: ['var(--text-base)', { lineHeight: 'var(--lbase)' }],
        lg: ['var(--text-lg)', { lineHeight: 'var(--llg)' }],
        xl: ['var(--text-xl)', { lineHeight: 'var(--lxl)' }],
        display: ['var(--text-display)', { lineHeight: 'var(--ldisplay)' }],
      },
      spacing: {
        // escala de 4px: 1=4 ... 10=40 (space tokens)
      },
      borderRadius: {
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
        pill: 'var(--radius-pill)',
      },
      boxShadow: {
        card: 'var(--shadow-card)',
        pop: 'var(--shadow-pop)',
      },
      transition: {
        fast: 'var(--motion-fast) var(--motion-ease)',
        base: 'var(--motion-base) var(--motion-ease)',
      },
    },
  },
}