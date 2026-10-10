// Utilidades de DOM. Todo el texto entra por `textContent`: nada de lo que
// devuelve el sistema (salidas de comandos, nombres de paquetes) se interpreta
// como HTML.

type Child = Node | string | null | undefined | false;

/** Añade hijos saltando los vacíos, para poder escribir `cond ? nodo : null`. */
export function add<T extends Element>(parent: T, ...children: Child[]): T {
  for (const child of children) {
    if (child === null || child === undefined || child === false) continue;
    parent.append(child);
  }
  return parent;
}

export function el<K extends keyof HTMLElementTagNameMap>(tag: K, className = "", ...children: Child[]): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return add(node, ...children);
}

export function button(label: string, className: string, onClick: (btn: HTMLButtonElement) => void): HTMLButtonElement {
  const btn = el("button", className, label);
  btn.type = "button";
  btn.addEventListener("click", () => onClick(btn));
  return btn;
}

/** **negrita** y `código` dentro de una línea. */
function inline(text: string): Node[] {
  const nodes: Node[] = [];
  const pattern = /\*\*(.+?)\*\*|`([^`]+)`/g;
  let last = 0;
  for (const match of text.matchAll(pattern)) {
    const at = match.index ?? 0;
    if (at > last) nodes.push(document.createTextNode(text.slice(last, at)));
    nodes.push(match[1] !== undefined ? el("strong", "", match[1]) : el("code", "", match[2]));
    last = at + match[0].length;
  }
  if (last < text.length) nodes.push(document.createTextNode(text.slice(last)));
  return nodes;
}

/** Texto con el formato ligero que usa el núcleo: párrafos, citas `> ` y listas. */
export function rich(text: string): DocumentFragment {
  const out = document.createDocumentFragment();
  let paragraph: HTMLElement | null = null;
  let quote: HTMLElement | null = null;
  for (const line of text.split("\n")) {
    if (line.trim() === "") {
      paragraph = null;
      quote = null;
      continue;
    }
    if (line.startsWith("> ")) {
      paragraph = null;
      if (!quote) {
        quote = el("blockquote");
        out.append(quote);
      } else {
        quote.append(" ");
      }
      quote.append(...inline(line.slice(2)));
      continue;
    }
    quote = null;
    const isItem = /^(•|\d+\.)\s/.test(line);
    if (isItem || !paragraph) {
      paragraph = el("p", isItem ? "item" : "");
      out.append(paragraph);
    } else {
      paragraph.append(el("br"));
    }
    paragraph.append(...inline(line));
    if (isItem) paragraph = null;
  }
  return out;
}

export async function copy(text: string, btn: HTMLButtonElement): Promise<void> {
  const original = btn.textContent;
  try {
    await navigator.clipboard.writeText(text);
    btn.textContent = "Copiado";
  } catch {
    btn.textContent = "No se pudo copiar";
  }
  window.setTimeout(() => (btn.textContent = original), 1400);
}
