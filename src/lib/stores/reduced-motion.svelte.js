let _prefersReducedMotion = $state(false);

function applyClass(reduced) {
  if (typeof document !== "undefined") {
    document.documentElement.classList.toggle("reduce-motion", reduced);
  }
}

if (typeof window !== "undefined") {
  const mql = window.matchMedia("(prefers-reduced-motion: reduce)");
  _prefersReducedMotion = mql.matches;
  applyClass(_prefersReducedMotion);
  mql.addEventListener("change", (e) => {
    _prefersReducedMotion = e.matches;
    applyClass(_prefersReducedMotion);
  });
}

export function getPrefersReducedMotion() {
  return _prefersReducedMotion;
}
