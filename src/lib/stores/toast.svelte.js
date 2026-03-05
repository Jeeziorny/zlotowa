let toasts = $state([]);
let nextId = 0;

export function addToast(message, type = "success", duration = 3500) {
  const id = nextId++;
  // Only one toast at a time — replace any existing
  toasts = [{ id, message, type, duration }];
  if (duration > 0) {
    setTimeout(() => removeToast(id), duration);
  }
  return id;
}

export function removeToast(id) {
  toasts = toasts.filter((t) => t.id !== id);
}

export function getToasts() {
  return toasts;
}
