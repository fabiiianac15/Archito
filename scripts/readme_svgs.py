#!/usr/bin/env python3
# Genera los SVG animados del README (assets/readme/archito-hola.svg y
# archito-poses.svg) con las mismas piezas que el personaje de app/src/mascot.ts.
#
#   python3 scripts/readme_svgs.py assets/readme
import sys
D = sys.argv[1]
BODY="M60 7 L104 104 Q88 95 73 96 C73 79 68 71 60 71 C52 71 47 79 47 96 Q32 95 16 104 Z"
NOTCH="M49 31 Q57 37 64.5 42.5 Q55 40 47.6 34.2 Z"
BLUE="#1793d1"; INK="#14111a"

def mascot(uid, mouth="smile", eyes="open", extra=""):
    m = {
      "smile": f'<path d="M55.5 61.5 Q60 62.5 64.5 61.5 Q63.6 67.5 60 67.5 Q56.4 67.5 55.5 61.5 Z" fill="{INK}"/><path d="M57.4 65.8 Q60 64.2 62.6 65.8 Q61.6 67.4 60 67.4 Q58.4 67.4 57.4 65.8 Z" fill="#f27b84"/>',
      "o": f'<path d="M60 62.2 a2.2 2.6 0 1 0 0.01 0 Z" fill="{INK}"/>',
      "wavy": f'<path d="M55.5 64.5 Q57.7 62.5 60 64.5 Q62.3 66.5 64.5 64.5" class="ln"/>',
      "flat": f'<path d="M57 64.6 Q60 65.4 63 64.6" class="ln"/>',
    }[mouth]
    e = (f'<g class="eyes"><ellipse cx="49.5" cy="56" rx="3.1" ry="4.1" fill="{INK}"/><ellipse cx="70.5" cy="56" rx="3.1" ry="4.1" fill="{INK}"/>'
         f'<circle cx="48.4" cy="54.3" r="1.1" fill="#fff"/><circle cx="69.4" cy="54.3" r="1.1" fill="#fff"/></g>') if eyes=="open" else \
        '<path d="M46.5 56.5 Q49.5 58.6 52.5 56.5 M67.5 56.5 Q70.5 58.6 73.5 56.5" class="ln"/>'
    return f'''<defs><mask id="cut{uid}" maskUnits="userSpaceOnUse" x="0" y="0" width="120" height="124"><rect width="120" height="124" fill="#fff"/><path d="{NOTCH}"/></mask></defs>
<g class="bob">
<path class="limb" d="M52 95 L51 109"/><ellipse cx="48.5" cy="111" rx="5.4" ry="3" class="fill"/>
<path class="limb" d="M68 95 L69 109"/><ellipse cx="71.5" cy="111" rx="5.4" ry="3" class="fill"/>
<g class="arm-l"><path class="limb" d="M36 64 Q26 66 22 80"/><circle cx="21.6" cy="82.5" r="3.6" class="fill"/></g>
<g class="arm-r"><path class="limb" d="M84 64 Q94 66 98 80"/><circle cx="98.4" cy="82.5" r="3.6" class="fill"/></g>
<path d="{BODY}" class="fill" mask="url(#cut{uid})"/>
<g class="face"><g class="brows"><path class="ln brow-l" d="M45.5 49.5 Q49.5 47 53 49.5"/><path class="ln brow-r" d="M67 49.5 Q70.5 47 74.5 49.5"/></g>{e}
<ellipse cx="43.5" cy="62.5" rx="3.2" ry="2" fill="#ff8fa3" opacity=".5"/><ellipse cx="76.5" cy="62.5" rx="3.2" ry="2" fill="#ff8fa3" opacity=".5"/>{m}{extra}</g>
</g>'''

BASE = f'''.fill{{fill:{BLUE}}}.limb{{fill:none;stroke:{BLUE};stroke-width:3.4;stroke-linecap:round}}
.ln{{fill:none;stroke:{INK};stroke-width:1.4;stroke-linecap:round}}
g,path,ellipse{{transform-box:view-box}}
.bob{{transform-origin:60px 112px}}.arm-l{{transform-origin:35px 64px}}.arm-r{{transform-origin:85px 64px}}
.eyes{{transform-origin:60px 56px}}.brow-l{{transform-origin:49px 49px}}.brow-r{{transform-origin:71px 49px}}
text{{font-family:"Inter","Segoe UI",Helvetica,Arial,sans-serif}}'''

def pct_wave(start, end, step=3, lo=96, hi=134):
    out=[f"0%,{start-0.5}%{{transform:rotate(0)}}"]
    v=start; up=True
    while v<=end:
        out.append(f"{v}%{{transform:rotate({hi if up else lo}deg)}}"); up=not up; v+=step
    out.append(f"{end+2}%,100%{{transform:rotate(0)}}")
    return "".join(out)

# ── Portada: baja colgado del hilo, saluda, vuelve a subir. Ciclo de 7 s. ──
hero = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 230" width="400" height="230" role="img" aria-label="Archito baja colgado de un hilo y saluda">
<title>Archito saluda</title>
<style>{BASE}
.drop{{animation:drop 7s cubic-bezier(.3,.7,.4,1) infinite}}
@keyframes drop{{0%{{transform:translateY(-240px)}}11%{{transform:translateY(14px)}}14%{{transform:translateY(-6px)}}17%,86%{{transform:translateY(0)}}100%{{transform:translateY(-240px)}}}}
.hero .arm-l{{animation:wave 7s ease-in-out infinite}}
@keyframes wave{{{pct_wave(20, 68)}}}
.hero .eyes{{animation:blink 7s infinite}}
@keyframes blink{{0%,44%,46.5%,76%,78.5%,100%{{transform:scaleY(1)}}45%,77%{{transform:scaleY(.1)}}}}
.bubble{{animation:pop 7s infinite;transform-origin:248px 112px;transform-box:view-box}}
@keyframes pop{{0%,19%{{opacity:0;transform:scale(.6)}}23%,70%{{opacity:1;transform:scale(1)}}74%,100%{{opacity:0;transform:scale(.9)}}}}
</style>
<g class="drop">
  <line x1="170" y1="-260" x2="170" y2="58" stroke="#8b949e" stroke-width="1.5"/>
  <g class="hero" transform="translate(95 49) scale(1.25)">{mascot("h")}</g>
  <g class="bubble">
    <rect x="246" y="78" width="146" height="34" rx="14" fill="#ffffff" stroke="#d0d7de"/>
    <path d="M250 104 L238 118 L262 108 Z" fill="#ffffff" stroke="#d0d7de" stroke-linejoin="round"/>
    <rect x="251" y="98" width="16" height="12" fill="#ffffff"/>
    <text x="319" y="100" text-anchor="middle" font-size="14" font-weight="600" fill="#1f2328">¡Hola! Soy Archito</text>
  </g>
</g>
<ellipse cx="170" cy="198" rx="40" ry="4" fill="#8b949e" opacity=".25"/>
</svg>'''
open(f"{D}/archito-hola.svg","w").write(hero)

# ── Poses: cada una en bucle. ──
poses = [
  ("idle",   "en reposo",   mascot("p0"),
   ".p0 .bob{animation:breathe 3.4s ease-in-out infinite}.p0 .eyes{animation:blink2 4s infinite}"),
  ("wave",   "saluda",      mascot("p1"),
   ".p1 .arm-l{animation:wv .42s ease-in-out infinite alternate}"),
  ("think",  "piensa",      mascot("p2", mouth="o"),
   ".p2 .face{transform:translate(1.4px,-1.6px)}.p2 .arm-r{transform:rotate(-38deg)}.p2 .bob{animation:ponder 1.8s ease-in-out infinite}"),
  ("happy",  "celebra",     mascot("p3"),
   ".p3 .arm-l{transform:rotate(112deg)}.p3 .arm-r{transform:rotate(-112deg)}.p3 .bob{animation:hop .9s cubic-bezier(.3,.9,.4,1) infinite}"),
  ("worried","se preocupa", mascot("p4", mouth="wavy", extra='<path d="M79 44 Q81.6 48.6 79 50 Q76.4 48.6 79 44 Z" fill="#9fdcff"/>'),
   ".p4 .brow-l{transform:rotate(-16deg)}.p4 .brow-r{transform:rotate(16deg)}.p4 .bob{animation:shiver 1.6s ease-in-out infinite}"),
  ("sleep",  "duerme",      mascot("p5", mouth="flat", eyes="closed",
                              extra='<g class="zzz"><text x="80" y="36" font-size="9" font-weight="700" fill="#8b949e">z</text><text x="88" y="26" font-size="9" font-weight="700" fill="#8b949e">z</text></g>'),
   ".p5 .bob{animation:breathe 5s ease-in-out infinite}.p5 .zzz text{animation:drift 3s ease-in-out infinite}.p5 .zzz text:nth-child(2){animation-delay:1.5s}"),
]
cells=[]; css=[]
for i,(key,label,svg,c) in enumerate(poses):
    x=i*130
    cells.append(f'<g class="p{i}" transform="translate({x+5} 4)">{svg}</g><text x="{x+65}" y="150" text-anchor="middle" font-size="13" fill="#8b949e">{label}</text>')
    css.append(c)
strip=f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 780 160" width="780" height="160" role="img" aria-label="Las poses de Archito">
<title>Las poses de Archito</title>
<style>{BASE}
{"".join(css)}
@keyframes breathe{{50%{{transform:translateY(-1.8px) scaleY(1.012)}}}}
@keyframes blink2{{0%,90%,96%,100%{{transform:scaleY(1)}}93%{{transform:scaleY(.1)}}}}
@keyframes wv{{from{{transform:rotate(92deg)}}to{{transform:rotate(134deg)}}}}
@keyframes ponder{{50%{{transform:rotate(-3deg) translateY(-1px)}}}}
@keyframes hop{{30%{{transform:translateY(-11px)}}55%{{transform:translateY(0) scaleY(.93) scaleX(1.04)}}72%{{transform:translateY(-3px)}}}}
@keyframes shiver{{0%,70%,100%{{transform:translateX(0)}}76%,88%{{transform:translateX(-1.2px)}}82%,94%{{transform:translateX(1.2px)}}}}
@keyframes drift{{0%{{opacity:0;transform:translate(0,4px)}}30%{{opacity:1}}100%{{opacity:0;transform:translate(5px,-8px)}}}}
</style>
{"".join(cells)}
</svg>'''
open(f"{D}/archito-poses.svg","w").write(strip)
