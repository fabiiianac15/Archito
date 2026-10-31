// La isla: una sola pieza que cambia de forma con un resorte.
//
// La ventana es fija y transparente; la isla cuelga del borde superior, al centro.
// Cada cambio de forma anima ancho, alto y radio con el mismo resorte, y al
// backend solo se le dice qué rectángulo recibe clics (ver layer.rs).

import { api } from "./api";

export type IslandState = "hidden" | "pill" | "open";

interface Shape {
  w: number;
  h: number;
  r: number;
  /** Desplazamiento vertical: negativa, la isla queda escondida arriba. */
  y: number;
}

/**
 * Tamaño de la isla abierta. Todas las vistas caben aquí: nunca crece. La
 * ventana (tauri.conf.json) es algo más grande para que quepan la sombra y la
 * entrada del personaje.
 */
export const OPEN = { w: 700, h: 360 };

function shapeOf(state: IslandState): Shape {
  switch (state) {
    case "hidden":
      return { w: 40, h: 30, r: 15, y: -80 };
    case "pill":
      return { w: 176, h: 36, r: 18, y: 0 };
    case "open":
      return { w: OPEN.w, h: OPEN.h, r: 28, y: 0 };
  }
}

export const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

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
  state: IslandState = "hidden";
  private current: Shape = shapeOf("hidden");
  private stop: () => void = () => {};

  constructor(readonly root: HTMLElement) {
    this.paint(this.current);
  }

  get isOpen(): boolean {
    return this.state === "open";
  }

  /**
   * Todo lo que mueve la isla pasa por aquí: tamaño, radio y posición. Nada de
   * `translate`, `scale` ni transiciones CSS sobre la isla: WebKitGTK no
   * recalcula el recorte de `overflow: hidden` cuando se mezclan con un
   * cambio de tamaño, y el contenido de la píldora dejaba de pintarse.
   */
  private paint(shape: Shape) {
    this.current = shape;
    const style = this.root.style;
    style.width = `${shape.w}px`;
    style.height = `${shape.h}px`;
    style.marginLeft = `${-shape.w / 2}px`;
    style.top = `${shape.y}px`;
    style.borderRadius = `0 0 ${shape.r}px ${shape.r}px`;
  }

  /** Rectángulo que recibe clics: la isla con un par de píxeles de margen. */
  private region(shape: Shape) {
    const w = shape.w + 6;
    void api.setRegion((window.innerWidth - w) / 2, 0, w, shape.h + 3);
  }

  /** Nada recibe clics: durante la entrada y al despedirse. */
  clearRegion() {
    void api.setRegion(0, 0, 0, 0);
  }

  /** Cambia de forma partiendo de donde esté, aunque haya otra animación en curso. */
  go(state: IslandState): Promise<void> {
    this.stop();
    const from = { ...this.current };
    const to = shapeOf(state);
    const opening = to.h > from.h;
    this.state = state;
    this.root.dataset.state = state;
    this.root.classList.remove("is-settled");
    // Durante el movimiento los clics caen en la forma más grande de las dos.
    if (state === "hidden") this.clearRegion();
    else this.region({ w: Math.max(from.w, to.w), h: Math.max(from.h, to.h), r: 0, y: 0 });
    if (state === "open") void api.setExpanded(true);
    return new Promise((resolve) => {
      // Abrir rebota un poco; recoger es más seco para no "latir" dos veces.
      this.stop = spring(opening ? 210 : 300, opening ? 21 : 30, (t) => {
        const mix = (a: number, b: number) => a + (b - a) * t;
        this.paint({ w: Math.max(36, mix(from.w, to.w)), h: Math.max(30, mix(from.h, to.h)), r: Math.max(8, mix(from.r, to.r)), y: mix(from.y, to.y) });
      }, () => {
        this.paint(to);
        this.root.classList.add("is-settled");
        if (state !== "hidden") this.region(to);
        if (state !== "open") void api.setExpanded(false);
        resolve();
      });
    });
  }

  /** Asoma desde el borde de arriba ya como píldora. */
  appear(): Promise<void> {
    return this.go("pill");
  }

  /** Se esconde por el borde de arriba (al cerrar el programa). */
  vanish(): Promise<void> {
    return this.go("hidden");
  }

  /** Un saltito sin cambiar de forma: algo bueno pasó. */
  bump() {
    if (reducedMotion) return;
    const base = { ...this.current };
    const start = performance.now();
    const frame = (now: number) => {
      const t = Math.min(1, (now - start) / 420);
      const k = 1 + 0.08 * Math.sin(Math.PI * t);
      this.paint({ ...base, w: base.w * k, h: base.h * k });
      if (t < 1) requestAnimationFrame(frame);
      else this.paint(base);
    };
    requestAnimationFrame(frame);
  }
}
