import { ref, watch } from "vue";

type Theme = "system" | "light" | "dark";

const STORAGE_KEY = "fam-theme";

const theme = ref<Theme>(loadTheme());

function loadTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark") return stored;
  return "system";
}

function applyTheme(value: Theme) {
  const html = document.documentElement;
  if (value === "system") {
    html.removeAttribute("data-theme");
  } else {
    html.setAttribute("data-theme", value);
  }
}

applyTheme(theme.value);

watch(theme, (value) => {
  localStorage.setItem(STORAGE_KEY, value);
  applyTheme(value);
});

export function useTheme() {
  function cycle() {
    const order: Theme[] = ["system", "light", "dark"];
    const index = order.indexOf(theme.value);
    theme.value = order[(index + 1) % order.length];
  }

  return { theme, cycle };
}
