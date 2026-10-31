// La entrada de Archito: baja colgado de un hilo desde el borde de arriba,
// saluda, y cuando aparece la isla se encoge y vuela hasta su sitio en la
// píldora. Mientras dura no hay nada que reciba clics: la ventana deja pasar
// el ratón a lo que haya debajo.

import { type Island, reducedMotion } from "./island";
import { createMascot, pose } from "./mascot";

const $ = (id: string) => document.getElementById(id) as HTMLElement;
const wait = (ms: number) => new Promise((done) => window.setTimeout(done, reducedMotion ? 0 : ms));

/**
 * Una animación hecha a mano, cuadro a cuadro. No se usan animaciones CSS ni
 * la Web Animations API para moverse: WebKitGTK las pasa a su compositor y,
 * mientras la ventana no tiene el foco (justo al arrancar), a veces las deja
 * congeladas en el primer cuadro. Lo que se mueve desde JavaScript, como el
 * resorte de la isla, se pinta siempre.
 */
function tween(ms: number, step: (t: number) => void): Promise<void> {
  return new Promise((done) => {
    const start = performance.now();
    const frame = (now: number) => {
      const t = Math.min(1, (now - start) / ms);
      step(t);
      if (t < 1) requestAnimationFrame(frame);
      else done();
    };
    requestAnimationFrame(frame);
  });
}

/** Llega y rebota un par de veces, como colgado de un muelle. */
const bounce = (t: number) => 1 - Math.exp(-6 * t) * Math.cos(2.6 * Math.PI * t);
const easeOut = (t: number) => 1 - (1 - t) ** 3;
const easeInOut = (t: number) => (t < 0.5 ? 4 * t ** 3 : 1 - (-2 * t + 2) ** 3 / 2);

export async function introduce(island: Island, landing: HTMLElement, greeting: string) {
  const intro = $("intro");
  const holder = $("intro-buddy");
  const bubble = $("intro-bubble");
  const thread = intro.querySelector<HTMLElement>(".intro-thread")!;
  bubble.textContent = greeting;
  island.clearRegion();

  if (reducedMotion) {
    intro.remove();
    await island.appear();
    return;
  }

  const buddy = createMascot("intro-mascot");
  holder.append(buddy);
  intro.classList.add("is-on");
  const DROP = 190;

  // 1. Baja colgado del hilo, con un rebote de muelle al llegar.
  pose(buddy, "idle");
  await tween(1100, (t) => {
    holder.style.transform = `translateY(${-DROP * (1 - bounce(t))}px)`;
    thread.style.transform = `scaleY(${Math.min(1, easeOut(Math.min(1, t / 0.7)))})`;
  });

  // 2. Saluda.
  pose(buddy, "wave");
  bubble.classList.add("is-on");
  await wait(1900);
  bubble.classList.remove("is-on");

  // 3. Aparece la isla y Archito vuela hasta la cara de la píldora.
  pose(buddy, "happy");
  void tween(380, (t) => (thread.style.transform = `scaleY(${1 - easeOut(t)})`));
  island.root.classList.add("is-landing");
  await island.appear();
  // La píldora enseña solo la cara (createMascot con `face`), así que lo que
  // tiene que caer en su sitio es la cara: el giro se hace sobre ella.
  const FACE = { x: 0.5, y: 55 / 124, share: 68 / 120 };
  holder.style.transformOrigin = `${FACE.x * 100}% ${FACE.y * 100}%`;
  holder.style.transform = "none";
  const from = holder.getBoundingClientRect();
  const to = landing.getBoundingClientRect();
  const scale = to.width / (from.width * FACE.share);
  const dx = to.left + to.width / 2 - (from.left + from.width * FACE.x);
  const dy = to.top + to.height / 2 - (from.top + from.height * FACE.y);
  // Una curva: sube un poco antes de meterse en la píldora.
  await tween(640, (raw) => {
    const t = easeInOut(raw);
    const lift = -34 * 4 * t * (1 - t);
    holder.style.transform = `translate(${dx * t}px, ${dy * t + lift}px) scale(${1 + (scale - 1) * t})`;
  });
  island.root.classList.remove("is-landing");
  intro.remove();
}
/** La despedida: saluda desde la píldora, la isla sube y el programa termina. */
export async function farewell(island: Island, buddy: SVGSVGElement) {
  pose(buddy, "bye");
  await wait(1300);
  await island.vanish();
}
