// La isla: una sola pieza que cambia de forma con un resorte.
//
// La ventana es fija y transparente; la isla cuelga del borde superior, al centro.
// Cada cambio de forma anima ancho, alto y radio con el mismo resorte, y al
// backend solo se le dice qué rectángulo recibe clics (ver layer.rs).

import { api } from "./api";

export type IslandState = "boot" | "dot" | "hello" | "pill" | "open";

interface Shape {
  w: number;
  h: number;
  r: number;
}

/** Tamaño de la isla abierta. Todas las vistas caben aquí: nunca crece. */
export const OPEN = { w: 664, h: 232 };

function shapeOf(state: IslandState): Shape {
  switch (state) {
    case "boot":
    case "dot":
      return { w: 46, h: 34, r: 17 };
    case "hello":
      return { w: 262, h: 38, r: 19 };
    case "pill":
      return { w: 168, h: 34, r: 17 };
    case "open":
      return { w: OPEN.w, h: OPEN.h, r: 26 };
  }
}

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/**
 * Resorte amortiguado de 0 a 1. Puede pasarse un poco de 1 antes de asentarse:
 * ese rebote es lo que hace que el cambio de forma se sienta vivo.
 */
function spring(stiffness: number, damping: number, onUpdate: (t: number) => void, onDone: () => void): () => void {
  if (reducedMotion) {
    onUpdate(1);
    onDone();
    return () => {};
  }
  let x = 0;
  let v = 0;
  let last = performance.now();
  let raf = 0;
  const step = (now: number) => {
    // Pasos pequeños y fijos: un cuadro largo no hace saltar el resorte.
    let dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    while (dt > 0) {
      const h = Math.min(dt, 0.004);
      v += (stiffness * (1 - x) - damping * v) * h;
      x += v * h;
      dt -= h;
    }
    if (Math.abs(1 - x) < 0.0015 && Math.abs(v) < 0.02) {
      onUpdate(1);
      onDone();
      return;
    }
    onUpdate(x);
    raf = requestAnimationFrame(step);
  };
  raf = requestAnimationFrame(step);
  return () => cancelAnimationFrame(raf);
}

export class Island {
  state: IslandState = "boot";
  private current: Shape = shapeOf("boot");
  private stop: () => void = () => {};
  private timers: number[] = [];

  constructor(readonly root: HTMLElement) {
    this.paint(this.current);
  }

  get isOpen(): boolean {
    return this.state === "open";
  }

  private paint(shape: Shape) {
    this.current = shape;
    this.root.style.width = `${shape.w}px`;
    this.root.style.height = `${shape.h}px`;
    this.root.style.borderRadius = `0 0 ${shape.r}px ${shape.r}px`;
  }

  /** Rectángulo que recibe clics: la isla con un par de píxeles de margen. */
  private region(shape: Shape) {
    const w = shape.w + 6;
    void api.setRegion((window.innerWidth - w) / 2, 0, w, shape.h + 3);
  }

  /** Cambia de forma partiendo de donde esté, aunque haya otra animación en curso. */
  go(state: IslandState): Promise<void> {
    this.stop();
    for (const t of this.timers.splice(0)) window.clearTimeout(t);
    const from = { ...this.current };
    const to = shapeOf(state);
    const opening = to.h > from.h;
    this.state = state;
    this.root.dataset.state = state;
    // Durante el movimiento los clics caen en la forma más grande de las dos.
    this.region({ w: Math.max(from.w, to.w), h: Math.max(from.h, to.h), r: 0 });
    if (state === "open") void api.setExpanded(true);
    return new Promise((resolve) => {
      // Abrir rebota un poco; recoger es más seco para no "latir" dos veces.
      this.stop = spring(opening ? 210 : 300, opening ? 21 : 30, (t) => {
        const mix = (a: number, b: number) => a + (b - a) * t;
        this.paint({ w: Math.max(36, mix(from.w, to.w)), h: Math.max(30, mix(from.h, to.h)), r: Math.max(8, mix(from.r, to.r)) });
      }, () => {
        this.paint(to);
        this.region(to);
        if (state !== "open") void api.setExpanded(false);
        resolve();
      });
    });
  }

  private after(ms: number, run: () => void) {
    this.timers.push(window.setTimeout(run, ms));
  }

  /** Entrada: cae desde arriba como un punto, saluda y se queda en píldora. */
  greet() {
    this.root.classList.add("is-in");
    this.state = "dot";
    this.root.dataset.state = "dot";
    this.region(shapeOf("hello"));
    this.after(520, () => {
      if (this.state !== "dot") return;
      void this.go("hello");
      this.after(2300, () => this.state === "hello" && void this.go("pill"));
    });
  }

  /** Un saltito sin cambiar de forma: algo bueno pasó. */
  bump() {
    this.root.classList.remove("is-bump");
    void this.root.offsetWidth;
    this.root.classList.add("is-bump");
  }
}
