const listeners = [];
const darkMedia = window.matchMedia("(prefers-color-scheme: dark)");

function apply(dark) {
  document.documentElement.classList.toggle("dark", dark);
}

function current() {
  return document.documentElement.classList.contains("dark");
}

// Stored preference > system preference
const stored = localStorage.getItem("theme");
apply(stored ? stored === "dark" : darkMedia.matches);

// Track live system preference changes (only when no manual override)
darkMedia.addEventListener("change", (e) => {
  if (!localStorage.getItem("theme")) {
    apply(e.matches);
    listeners.forEach((fn) => fn(current()));
  }
});

export function isDark() {
  return current();
}

export function toggle() {
  const dark = !current();
  apply(dark);

  // Persist the choice so it survives page reloads.
  localStorage.setItem("theme", dark ? "dark" : "light");
  listeners.forEach((fn) => fn(dark));
}

export function onChange(fn) {
  listeners.push(fn);
}
