// Theme handling: auto / light / dark. Dark is the default (no attribute);
// light sets data-theme="light". The palette lives in app.css.

const KEY = "blogwriter-theme";

export function getTheme() {
  return localStorage.getItem(KEY) || "auto";
}

export function applyTheme(theme) {
  localStorage.setItem(KEY, theme);
  const root = document.documentElement;
  const wantLight =
    theme === "light" ||
    (theme === "auto" &&
      window.matchMedia("(prefers-color-scheme: light)").matches);
  if (wantLight) root.setAttribute("data-theme", "light");
  else root.removeAttribute("data-theme");
}

export function initTheme() {
  applyTheme(getTheme());
  window
    .matchMedia("(prefers-color-scheme: light)")
    .addEventListener("change", () => {
      if (getTheme() === "auto") applyTheme("auto");
    });
}
