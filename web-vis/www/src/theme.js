const listeners = [];

function apply(dark) {
  document.documentElement.classList.toggle("dark", dark);
}

function current() {
  return document.documentElement.classList.contains("dark");
}

// Initialise: stored preference > system preference
const stored = localStorage.getItem("theme");
if (stored) {
  apply(stored === "dark");
} else {
  apply(window.matchMedia("(prefers-color-scheme: dark)").matches);
}

// Live system preference tracking (only when no manual override)
window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", (e) => {
    if (!localStorage.getItem("theme")) {
      apply(e.matches);
      listeners.forEach((fn) => fn(current()));
    }
  });

export function isDark() {
  return current();
}

export function toggle() {
  // Flip the current theme.
  const dark = !current();
  apply(dark);

  // Persist the choice so it survives page reloads.
  localStorage.setItem("theme", dark ? "dark" : "light");

  // Notify subscribers (e.g. the toggle button) of the change.
  listeners.forEach((fn) => fn(dark));
}

export function onChange(fn) {
  listeners.push(fn);
}
