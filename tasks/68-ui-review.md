1. No visual branding beyond the "4ccountant" text — no logo, no distinctive visual identity
2. Chart visualizations (monthly trend, spending by category) are basic CSS bars — fine for MVP but feel flat compared to the rest of the UI
3. The sidebar version label "v0.1.0" sits alone at the bottom with no other context — minor but slightly awkward
4. Category autocomplete uses a fragile blur-timeout pattern (150ms delay to let mousedown fire before blur closes the dropdown) — this is a known UX anti-pattern that can fail on slower
  devices or with keyboard navigation
5. Inline edit in tables replaces the entire row with inputs, but there's no visual indication of which row is editable until you click. Discoverability is low
6. No drag-and-drop for widget reorder — only left/right arrow buttons in edit mode, which is clunky for more than a few widgets
7. No empty-state illustrations — empty states are plain text ("No expenses yet"), which feels bare. Even simple icons or hints toward next actions would help
8. Filter state is invisible — when filters are active on the expense list, only a small "Clear filters" link appears. There's no persistent filter chip bar showing active filters at a
  glance
9. Three different modal components (DeleteConfirmModal, BatchDeleteModal, RuleDeleteModal) that are nearly identical — a generic ConfirmModal would enforce consistency and reduce drift
10. Table styling varies — expense table has hover-reveal action buttons (opacity-0 group-hover:opacity-100), while rules table uses similar but slightly different patterns. Categories table
   has yet another variation with click-to-rename
11. Color-only status indicators — budget progress (red/amber/green) and confidence labels rely on color alone with no icon or text fallback for colorblind users
12. Icon-only buttons (edit pencil, delete trash, move arrows) lack aria-label in several places — screen readers would announce them as empty buttons
13. Focus trapping in modals is missing — Tab key can escape the modal and reach elements behind the overlay
14. Table sort headers use aria-sort but the sort toggle isn't announced to screen readers
15. No keyboard shortcuts — no Cmd+N for new expense, no Cmd+S to save, no Cmd+F for search. For a desktop app, this is a missed opportunity
16. Adding a single expense requires 3 clicks (sidebar → Expenses → + Add) — there's no quick-add from the dashboard or global shortcut
17. CSV upload doesn't remember column mappings — if you upload from the same bank every month, you re-map columns each time
18. No validation on expense amount — can enter 0 or negative amounts for expenses
19. No character limit feedback on text inputs — title could theoretically be very long
20. Restoring a backup overwrites everything with only a single confirmation modal — this is high-stakes and deserves a more prominent warning or preview of what will change