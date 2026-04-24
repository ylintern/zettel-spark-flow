# Design System Document

## 1. Creative North Star: "The Ethereal Stage"
This design system is built upon the concept of **The Ethereal Stage**. In ballet, the stage is a place of immense physical power masked by effortless grace. We translate this to the digital screen through high-end editorial layouts that favor "breathing room" over density. 

The design must feel "fluid" rather than "fixed." We break the standard rigid grid by utilizing intentional asymmetry—placing typography off-center or allowing imagery to bleed off the edges of containers—to mimic the movement of a dancer across a floor. The interface is not a container; it is a performance space.

---

## 2. Color & Tonal Depth
Our palette transitions from the ethereal (Blush) to the grounded (Deep Burgundy), anchored by the prestige of Soft Gold.

### The "No-Line" Rule
To maintain the airiness of dance, **1px solid borders are strictly prohibited** for sectioning or containment. Boundaries between content must be defined exclusively through:
*   **Background Shifts:** Transitioning from `surface` to `surface-container-low`.
*   **Tonal Transitions:** Using `surface-container-highest` to subtly ground a specific functional area.
*   **Whitespace:** Utilizing the Spacing Scale to create "invisible" partitions.

### Surface Hierarchy & Nesting
Treat the UI as layered sheets of fine vellum. Instead of a flat grid, use the surface-container tiers to create depth:
*   **Base Layer:** `surface` (#fbf9f8) for the main canvas.
*   **Sub-Sections:** `surface-container-low` (#f5f3f2) for secondary content blocks.
*   **Elevated Elements:** `surface-container-lowest` (#ffffff) for primary cards to create a "pop" against the off-white background.

### The "Glass & Gradient" Rule
To capture the "Projected" feeling of the brand, use Glassmorphism for floating navigation and overlays.
*   **Tokens:** Use `surface` or `secondary-container` at 70% opacity with a `24px` backdrop-blur.
*   **Signature Textures:** Apply subtle linear gradients (e.g., `primary` to `primary-container`) for hero CTAs to add "soul" and depth, reflecting the passion of the performance.

---

## 3. Typography: Editorial Grace
The typography scale is designed to feel like a high-fashion program. It balances the dramatic flair of a serif with the functional clarity of a modern sans-serif.

*   **Display & Headlines (Noto Serif):** These are our "Principal Dancers." Use `display-lg` for hero moments with generous letter-spacing (-0.02em) to evoke elegance. Headlines should often be center-aligned or dramatically oversized to command the stage.
*   **Body & Titles (Manrope):** Our "Corps de Ballet." Manrope provides a clean, contemporary rhythm. Ensure `body-lg` has a line-height of at least 1.6 to prevent the text from feeling "heavy."
*   **Labels:** Use `label-md` in all-caps with `0.1em` letter-spacing when referencing `tertiary` (Gold) accents for a touch of prestige.

---

## 4. Elevation & Depth: Tonal Layering
We avoid traditional "material" shadows in favor of ambient light.

*   **The Layering Principle:** Depth is achieved by stacking. A `surface-container-lowest` card placed on a `surface-container-low` background creates a natural lift without a single pixel of shadow.
*   **Ambient Shadows:** For floating elements (Modals, Dropdowns), use extra-diffused shadows.
    *   *Specification:* `0px 20px 40px rgba(88, 65, 65, 0.06)`. The shadow uses a tint of `on-surface-variant` rather than black to ensure it feels organic.
*   **The Ghost Border Fallback:** If a boundary is functionally required, use a "Ghost Border": `outline-variant` (#e0bfbf) at **15% opacity**.
*   **Fluid Motion:** All elevation changes must be accompanied by a `300ms` "Ease-in-out" transition to reflect the fluidity of movement.

---

## 5. Components

### Buttons: The "Grand Plie"
*   **Primary:** Background: `primary` (#570013); Text: `on-primary` (#ffffff). Shape: `full` (pill-shaped) to reflect fluidity.
*   **Secondary:** Background: `secondary-container`; Text: `on-secondary-container`. No border.
*   **Tertiary:** Text: `on-primary-fixed-variant`. No background. Use for low-priority actions.

### Cards: The Stage
*   **Styling:** Forbid divider lines. Use `xl` (1.5rem) rounded corners.
*   **Layout:** Use vertical white space to separate the header from the body. Background should be `surface-container-lowest` against a `surface-container-low` section.

### Input Fields: Minimalist Forms
*   **Style:** Underline-only or subtle `surface-variant` backgrounds. 
*   **Focus State:** Transition the underline to `primary` (Burgundy) with a soft `surface-tint` glow.
*   **Error:** Use `error` (#ba1a1a) with `error-container` as a soft background highlight.

### Specialized Components
*   **The "Motion Progress Bar":** A thin, `primary` line that moves across the top of the screen during page transitions.
*   **Curated Image Frames:** Images should use `lg` (1rem) rounded corners and, where possible, be paired with an overlapping `display-sm` serif title for a "layered" editorial look.

---

## 6. Do’s and Don’ts

### Do
*   **Do** use asymmetrical margins (e.g., more padding on the left than the right) to create a sense of movement.
*   **Do** use the `tertiary` (Gold) color sparingly—only for "prestige" moments like awards, elite levels, or gold-tier memberships.
*   **Do** prioritize high-quality imagery of movement with plenty of background "air."

### Don’t
*   **Don’t** use 100% black (#000000). Use `on-background` (#1b1c1b) to keep the contrast sophisticated rather than harsh.
*   **Don’t** use sharp 90-degree corners. Everything must feel soft to the touch (minimum `sm` roundedness).
*   **Don’t** crowd the interface. If a screen feels "busy," increase the whitespace by 20% before removing content.
*   **Don’t** use standard "drop shadows" with high opacity. They break the ethereal illusion.