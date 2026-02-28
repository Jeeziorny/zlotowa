const FOCUSABLE = 'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function focusTrap(node) {
  const previouslyFocused = document.activeElement;

  function getFocusable() {
    return [...node.querySelectorAll(FOCUSABLE)];
  }

  // Focus first focusable element (or the node itself)
  const els = getFocusable();
  if (els.length > 0) {
    els[0].focus();
  } else {
    node.setAttribute("tabindex", "-1");
    node.focus();
  }

  function handleKeydown(e) {
    if (e.key !== "Tab") return;

    const focusable = getFocusable();
    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault();
        last.focus();
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }

  node.addEventListener("keydown", handleKeydown);

  return {
    destroy() {
      node.removeEventListener("keydown", handleKeydown);
      if (previouslyFocused && typeof previouslyFocused.focus === "function") {
        previouslyFocused.focus();
      }
    },
  };
}
