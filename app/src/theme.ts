// Viste la isla con los colores del escritorio. El núcleo (theme.rs) los lee
// y avisa cuando cambian; aquí se convierten en variables de CSS.
//
// Cada color va dos veces: tal cual (`--accent`) y como trío r, g, b
// (`--accent-rgb`), para poder escribir `rgba(var(--accent-rgb), 0.2)` sin
// depender de color-mix, que WebKitGTK antiguo no tiene.

import { listen } from "@tauri-apps/api/event";

import { api, type Palette } from "./api";

function rgb(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1, 7), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function luminance(hex: string): number {
  const lin = (v: number) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  const [r, g, b] = rgb(hex);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function mix(a: string, b: string, t: number): string {
  const [ar, ag, ab] = rgb(a);
  const [br, bg, bb] = rgb(b);
  const c = (x: number, y: number) => Math.round(x + (y - x) * t).toString(16).padStart(2, "0");
  return `#${c(ar, br)}${c(ag, bg)}${c(ab, bb)}`;
}

export function apply(p: Palette) {
  const root = document.documentElement.style;
  const set = (name: string, hex: string) => {
    root.setProperty(`--${name}`, hex);
    root.setProperty(`--${name}-rgb`, rgb(hex).join(", "));
  };
  set("bg", p.bg);
  set("surface", p.surface);
  set("surface-high", p.surface_high);
  set("text", p.text);
  set("dim", p.dim);
  set("line", p.line);
  set("accent", p.accent);
  set("on-accent", p.on_accent);
  set("accent2", p.accent2);
  set("ok", p.ok);
  set("warn", p.warn);
  set("danger", p.danger);
  // Archito toma el acento, pero un acento muy oscuro (temas claros) se
  // aclara: si no, sus ojos oscuros no se verían.
  const body = luminance(p.accent) < 0.14 ? mix(p.accent, "#ffffff", 0.32) : p.accent;
  set("mascot", body);
  root.setProperty("--mascot-ink", luminance(body) > 0.5 ? "#2a1f24" : "#14111a");
  document.documentElement.dataset.scheme = p.dark ? "dark" : "light";
  document.documentElement.dataset.source = p.source;
}

/** Aplica los colores del escritorio y los sigue cuando cambian. */
export async function followDesktop() {
  try {
    apply(await api.palette());
  } catch {
    // Sin paleta se queda el tema de style.css.
  }
  void listen<Palette>("theme", (event) => apply(event.payload));
}
