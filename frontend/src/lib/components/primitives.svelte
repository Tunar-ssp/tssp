<script lang="ts">
  /**
   * P0-FND-05: Primitive Components
   * Reusable UI building blocks. Build once, use everywhere.
   */

  // ===== Button Component =====
  export interface ButtonProps {
    kind?: 'primary' | 'accent' | 'ghost' | 'solid' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    icon?: any;
    iconRight?: any;
    disabled?: boolean;
    onClick?: () => void;
  }

  // ===== Pill Component =====
  export interface PillProps {
    tone?: 'green' | 'pink' | 'blue' | 'orange' | 'violet' | 'warn' | 'danger' | 'neutral';
  }

  // ===== Card Component =====
  export interface CardProps {
    head?: any;
    foot?: any;
    pad?: number;
    accent?: string;
  }

  // ===== StatusDot Component =====
  export interface StatusDotProps {
    tone?: 'ok' | 'warn' | 'err' | 'info';
  }

  // ===== Modal Component =====
  export interface ModalProps {
    title?: string;
    size?: 'sm' | 'md' | 'lg';
    onClose?: () => void;
    isOpen?: boolean;
  }

  // ===== Sheet Component (Side Panel) =====
  export interface SheetProps {
    side?: 'right' | 'bottom';
    isOpen?: boolean;
    onClose?: () => void;
    title?: string;
  }

  // ===== Tooltip Component =====
  export interface TooltipProps {
    content?: string;
    delay?: number;
    kbd?: string;
  }
</script>

<!--
  Primitive components are defined in separate Svelte files for composition.
  This file serves as the type definitions and documentation.

  Component inventory from P0-FND-05:
  - Btn (kind: primary/accent/ghost/solid/danger; size: sm/md/lg)
  - Pill (tone: green/pink/blue/orange/violet/warn/danger/neutral)
  - Kbd (displays keyboard shortcuts)
  - Card (head, foot, pad, accent)
  - Bar (value 0-100, segments[], tone)
  - Ring (value, tone, label, sub)
  - StatusDot (tone: ok/warn/err/info)
  - Toast (tone, title, body, action)
  - Modal (title, size, onClose)
  - Sheet (side: right/bottom, title)
  - Tooltip (content, delay, kbd)

  All components must:
  - Use only --color tokens from tokens.css
  - Support dark/light theme automatically
  - Have focus rings for a11y
  - Respect prefers-reduced-motion
  - Be tree-shakeable (no side effects)
-->
