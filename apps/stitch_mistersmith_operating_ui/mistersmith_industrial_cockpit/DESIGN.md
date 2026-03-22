# Midnight Terminal Design System

### 1. Overview & Creative North Star
**Creative North Star: The Sovereign Operator**
Midnight Terminal is a high-density, utility-first design system inspired by mission-critical command centers and industrial monitoring tools. It eschews the "consumer-friendly" soft edges of modern web design in favor of a brutalist, technical aesthetic that prioritizes information density and state-awareness over decorative elements. 

The system utilizes an ultra-tight grid, monospaced data injections, and a "HUD" (Heads-Up Display) philosophy where every pixel of screen real estate is treated as a functional asset.

### 2. Colors
The palette is built on a "True Dark" foundation, using varied levels of charcoal and obsidian to separate functional zones.

- **Primary Role:** The Brand Blue (`#3b82f6`) is used sparingly for active states, terminal cursors, and highlights.
- **The "No-Line" Rule:** Sectioning is primarily achieved through background shifts (e.g., `#171717` for headers against `#121212` for the root). While subtle borders (`#2e2e2e`) are permitted for high-density tables, structural separation should first be attempted with tonal nesting.
- **Surface Hierarchy:**
    - `surface-root` (#121212): The base layer for workspaces.
    - `surface-header` (#171717): Used for toolbars and metadata strips.
    - `surface-elevated` (#1c1c1c): Used for cards, evidence containers, and interactive hover states.
- **Signature Textures:** Use of `backdrop-blur` (sm) on sticky headers with 95% opacity creates a "glass-cockpit" feel that maintains context while scrolling.

### 3. Typography
The system employs a dual-font strategy: Inter for structural navigation and JetBrains Mono for all data, logs, and interactive commands.

**Typography Scale:**
- **Display/Icons:** 1.1rem (Material Symbols)
- **Headlines/Titles:** 1rem to 0.9rem (Semi-bold, often Uppercase)
- **Primary Data:** 0.75rem (The workhorse size for body and logs)
- **Secondary Metadata:** 0.65rem (XXS) for system labels and status timestamps.
- **Micro-Data:** 0.55rem for low-priority timestamps and encryption labels.

The typographic rhythm is intentionally small to allow for maximum data visibility without scrolling. Inter should be used for human-readable labels, while JetBrains Mono is strictly for system-generated output.

### 4. Elevation & Depth
Midnight Terminal avoids traditional drop shadows in favor of "Tonal Stacking."

- **The Layering Principle:** Depth is communicated by the brightness of the surface. A "higher" element is simply a lighter shade of grey (`#1c1c1c`) compared to the base (`#121212`).
- **Ambient Shadows:** Shadows are absent from this system. Focus is instead placed on `border-left` accents (2px solid primary) to indicate the active focus area.
- **The Ghost Border:** Where contrast is needed, a 1px border of `#2e2e2e` is used to define the perimeter of floating panels.
- **Active State:** Active rows or selections use a subtle translucent tint of the primary color: `rgba(59, 130, 246, 0.05)`.

### 5. Components
- **Terminal Input:** Unstyled, borderless text fields with a monospaced font and a pulsating primary-colored cursor (`#3b82f6`).
- **Status Pills:** Small, circular indicators (6px x 6px) using a traffic-light color system (Success: `#10b981`, Error: `#ef4444`, Warning: `#f59e0b`).
- **Data Tables:** Dense, no-padding-cell layouts. Headers are uppercase, tracking-wide, and sticky.
- **Evidence Shelf:** Modular cards with `surface-elevated` backgrounds, using icons to denote file types (code, image, description).
- **Navigation Rail:** A slim, 48px left-aligned rail with high-contrast active icons and low-contrast inactive icons.

### 6. Do's and Don'ts
- **Do:** Use uppercase for all labels and metadata to maintain the technical "instrument" feel.
- **Do:** Use `mono` fonts for any numerical value or timestamp to ensure vertical alignment.
- **Don't:** Use rounded corners. This system is strictly `rounded-none` or `rounded-sm` (max 2px) to maintain its industrial edge.
- **Don't:** Add large margins or paddings. Space is a luxury; use it only to separate distinct functional modules.
- **Do:** Use `animate-pulse` on critical system status icons to draw operator attention without using invasive pop-ups.