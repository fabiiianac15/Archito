// Archito, el personaje: un SVG por piezas (cuerpo, cejas, ojos, bocas,
// brazos, piernas) para poder animar cada una por separado.
//
// La pose se elige con `data-pose` y la anima style.css; aquí solo se arma el
// dibujo. El color del cuerpo es `--mascot`, que pone theme.ts a partir del
// acento del escritorio.

export type Pose = "idle" | "wave" | "think" | "happy" | "worried" | "error" | "sleep" | "bye";

const NS = "http://www.w3.org/2000/svg";
let serial = 0;

/** Cuerpo con la forma del logo de Arch: punta arriba y el arco entre las piernas. */
const BODY = "M60 7 L104 104 Q88 95 73 96 C73 79 68 71 60 71 C52 71 47 79 47 96 Q32 95 16 104 Z";
/** La muesca del logo, recortada del cuerpo con una máscara. */
const NOTCH = "M49 31 Q57 37 64.5 42.5 Q55 40 47.6 34.2 Z";

function markup(id: string): string {
  return `
  <defs>
    <mask id="${id}-cut" maskUnits="userSpaceOnUse" x="0" y="0" width="120" height="124">
      <rect width="120" height="124" fill="#fff"/>
      <path d="${NOTCH}" fill="#000"/>
    </mask>
  </defs>
  <g class="m-all">
    <g class="m-shadow"><ellipse cx="60" cy="116" rx="30" ry="3.2"/></g>
    <g class="m-bob">
      <g class="m-legs">
        <path class="m-limb" d="M52 95 L51 109"/><ellipse class="m-fill" cx="48.5" cy="111" rx="5.4" ry="3"/>
        <path class="m-limb" d="M68 95 L69 109"/><ellipse class="m-fill" cx="71.5" cy="111" rx="5.4" ry="3"/>
      </g>
      <g class="m-arm m-arm-l"><path class="m-limb" d="M36 64 Q26 66 22 80"/><circle class="m-fill" cx="21.6" cy="82.5" r="3.6"/></g>
      <g class="m-arm m-arm-r"><path class="m-limb" d="M84 64 Q94 66 98 80"/><circle class="m-fill" cx="98.4" cy="82.5" r="3.6"/></g>
      <path class="m-body" d="${BODY}" mask="url(#${id}-cut)"/>
      <g class="m-face">
        <g class="m-brows">
          <path class="m-brow m-brow-l" d="M45.5 49.5 Q49.5 47 53 49.5"/>
          <path class="m-brow m-brow-r" d="M67 49.5 Q70.5 47 74.5 49.5"/>
        </g>
        <g class="m-eyes">
          <g class="m-eye"><ellipse cx="49.5" cy="56" rx="3.1" ry="4.1"/><circle class="m-glint" cx="48.4" cy="54.3" r="1.1"/></g>
          <g class="m-eye"><ellipse cx="70.5" cy="56" rx="3.1" ry="4.1"/><circle class="m-glint" cx="69.4" cy="54.3" r="1.1"/></g>
        </g>
        <path class="m-closed" d="M46.5 56.5 Q49.5 58.6 52.5 56.5 M67.5 56.5 Q70.5 58.6 73.5 56.5"/>
        <ellipse class="m-cheek" cx="43.5" cy="62.5" rx="3.2" ry="2"/>
        <ellipse class="m-cheek" cx="76.5" cy="62.5" rx="3.2" ry="2"/>
        <g class="m-mouth m-mouth-smile">
          <path d="M55.5 61.5 Q60 62.5 64.5 61.5 Q63.6 67.5 60 67.5 Q56.4 67.5 55.5 61.5 Z"/>
          <path class="m-tongue" d="M57.4 65.8 Q60 64.2 62.6 65.8 Q61.6 67.4 60 67.4 Q58.4 67.4 57.4 65.8 Z"/>
        </g>
        <path class="m-mouth m-mouth-o" d="M60 62.2 a2.2 2.6 0 1 0 0.01 0 Z"/>
        <path class="m-mouth m-mouth-wavy" d="M55.5 64.5 Q57.7 62.5 60 64.5 Q62.3 66.5 64.5 64.5"/>
        <path class="m-mouth m-mouth-flat" d="M57 64.6 Q60 65.4 63 64.6"/>
        <path class="m-sweat" d="M79 44 Q81.6 48.6 79 50 Q76.4 48.6 79 44 Z"/>
      </g>
      <g class="m-zzz"><text x="80" y="36">z</text><text x="88" y="26">z</text></g>
    </g>
  </g>`;
}

/** Un Archito nuevo. `face`: encuadre de solo la cara, para la píldora. */
export function createMascot(className: string, face = false): SVGSVGElement {
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("viewBox", face ? "26 22 68 66" : "0 0 120 124");
  svg.setAttribute("class", `mascot ${className}`);
  svg.setAttribute("aria-hidden", "true");
  svg.dataset.pose = "idle";
  serial += 1;
  svg.innerHTML = markup(`archito-${serial}`);
  return svg;
}

/**
 * Cambia de pose. Las poses de un momento (saludar, celebrar) vuelven solas a
 * `back` cuando terminan, para que el personaje no se quede congelado.
 */
export function pose(svg: SVGSVGElement, next: Pose, back?: Pose, ms = 0) {
  window.clearTimeout(Number(svg.dataset.timer ?? 0));
  // Reiniciar la animación aunque la pose sea la misma (otro salto seguido).
  svg.dataset.pose = "";
  void svg.getBoundingClientRect();
  svg.dataset.pose = next;
  if (back && ms > 0) {
    svg.dataset.timer = String(window.setTimeout(() => (svg.dataset.pose = back), ms));
  }
}

/**
 * Pequeños gestos de vida: parpadear y, a veces, mirar a un lado. Van con un
 * temporizador y una clase que dura un instante, no con animaciones
 * infinitas: WebKitGTK redibuja por software y una animación que nunca para
 * le cuesta un núcleo entero. Solo se mueven los Archito que se ven.
 */
export function startLife(visible: () => SVGSVGElement[]) {
  const flash = (svg: SVGSVGElement, cls: string, ms: number) => {
    svg.classList.add(cls);
    window.setTimeout(() => svg.classList.remove(cls), ms);
  };
  const tick = () => {
    for (const svg of visible()) {
      const p = svg.dataset.pose;
      if (p === "sleep") continue;
      flash(svg, "is-blink", 170);
      if (p === "idle" && Math.random() < 0.3) flash(svg, Math.random() < 0.5 ? "look-l" : "look-r", 1400);
    }
    window.setTimeout(tick, 3200 + Math.random() * 3200);
  };
  window.setTimeout(tick, 2500);
}
