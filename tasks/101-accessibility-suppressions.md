# Task 101: Remove Accessibility Suppressions

## Goal

Replace `<div onclick>` elements with semantic HTML and remove `svelte-ignore a11y_*` comments.

## Deliverables

### 1. Dashboard config dialog overlay — `Dashboard.svelte:288-300`

Two suppressions:
- `a11y_click_events_have_key_events` — div with `onclick` for chip input focus
- `a11y_no_static_element_interactions` — div serving as interactive container

**Fix:** Replace the outer `<div onclick>` with a `<button>` (styled as transparent/unstyled) or add `role="button"`, `tabindex="0"`, and a `onkeydown` handler for Enter/Space. Remove the `svelte-ignore` comments.

### 2. Dashboard config overlay backdrop — `Dashboard.svelte:288`

`svelte-ignore a11y_click_events_have_key_events` on the overlay that closes the dialog on click.

**Fix:** The overlay already has an Escape key handler elsewhere. Add `role="presentation"` or handle click-outside via a different pattern (e.g., a transparent `<button>` covering the backdrop). Remove the suppression.

### 3. ColumnMapping popover overlay — `ColumnMapping.svelte:185`

`svelte-ignore a11y_no_static_element_interactions` on the click-to-close overlay.

**Fix:** Same as #2. Use `role="presentation"` on the backdrop div, or restructure to avoid the interaction on a non-semantic element.

## Files to modify
- `src/lib/Dashboard.svelte`
- `src/lib/bulk-upload/ColumnMapping.svelte`

## Notes
- After removing all `svelte-ignore a11y_*` comments, run `npm run build` to verify no new a11y warnings appear.
- Don't over-engineer — the simplest fix is usually adding `role` + `tabindex` + keyboard handler to the existing div, rather than restructuring the DOM.
