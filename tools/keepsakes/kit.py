"""The parts kit: shapes, words and moments a keepsake is made of, so a slip is a short description.

A slip module is a file named s_<batch>_<anything>.py that calls board() once per slip. build.py finds
it, registers it and does the rest (no list to edit). Example:

    from kit import board, paper, tag, words, swing, drawon, torn
    board('lonavala', 'Long drive to Lonavala', batch='b3', palette='ghat',
          name='An expressway toll receipt with the ghat road drawn up the side',
          verbs=('Take the ghat', 'Head home'),
          svg=paper(torn([(80, 90), (300, 90), (300, 500), (80, 500)], seed=3), '#fbfaf5', cls='rec')
              + words(190, 140, 'Mumbai–Pune Expressway', 'mono', 11, '#333', anchor='middle'),
          motion=[swing('.rec', '50% 0%'), drawon('.road', delay=.3)])

Shapes take a `cls` so a moment can move them. Every moment adds its own reduced-motion rule.
"""
import inspect, itertools, math
import frame
from frame import grain, torn  # noqa: F401  (re-exported for slip modules)

REGISTRY = {}  # board file -> (scene id, slip key, visit, batch, name); build.py merges it
KIND = {}  # scene id -> the kit object it was built with (stub, tag, …): build.py groups them

FONTS = 'family=Young+Serif&amp;family=Kalam:wght@400;700&amp;family=Space+Mono:wght@400;700&amp;family=Permanent+Marker&amp;family=Caveat:wght@600;700&amp;family=Gochi+Hand&amp;family=Fraunces:ital,opsz,wght@0,9..144,500..800;1,9..144,400..700&amp;family=Shippori+Mincho:wght@600;800'
FACE = {'serif': "'Young Serif', serif", 'hand': 'Kalam, cursive', 'mono': "'Space Mono', monospace", 'marker': "'Permanent Marker', cursive",
        'script': 'Caveat, cursive', 'kid': "'Gochi Hand', cursive", 'display': 'Fraunces, serif', 'mincho': "'Shippori Mincho', serif", 'ui': 'Figtree, sans-serif'}

# background stops, text, heading, button, button text, button hover
PALETTES = {
    'plum': (('#7a4a80', '#5d3564', '#43244a'), '#f6e8f1', '#fff7fb', '#ffd5e6', '#3a1430', '#f7b9cf'),
    'ghat': (('#6f8a64', '#57704f', '#3f553b'), '#eef2e6', '#fbfff4', '#f2e3b3', '#26301f', '#e6d394'),
    'night': (('#3f5d7d', '#2f4865', '#22354c'), '#e9f0f7', '#ffffff', '#f6efe3', '#22354c', '#e5dccb'),
    'ochre': (('#e3a35a', '#cc8840', '#a86a2c'), '#2a1a0a', '#1d1206', '#1e6a66', '#f4ecd8', '#17524f'),
    'brick': (('#c7663a', '#a94f2a', '#843b1f'), '#fbeee3', '#fff8f0', '#f6e2b3', '#4b1c0b', '#ecd193'),
    'ink': (('#4a3a5c', '#362a45', '#241c30'), '#ece6f2', '#ffffff', '#f2c230', '#1c1408', '#e0ae1c'),
    'sea': (('#5d97ac', '#467d92', '#326174'), '#eaf4f7', '#ffffff', '#fbe7a8', '#1d2b30', '#f1d880'),
    'lilac': (('#8a6a9c', '#6f5282', '#553d66'), '#f7eefb', '#ffffff', '#f7e2a2', '#2b1d33', '#ebd082'),
    'clay': (('#b9a58a', '#9d896d', '#7e6c53'), '#251a10', '#1c120a', '#5a2d16', '#f7eadc', '#47220f'),
    'paper': (('#e8dcc4', '#d4c6aa', '#b9a98a'), '#2a2238', '#1c1628', '#3a2a5a', '#f3efe2', '#2c1f46'),
    'moss': (('#6a7f5e', '#536849', '#3e5036'), '#eef3e8', '#ffffff', '#efe1c0', '#2b2b20', '#e2cfa4'),
    'arcade': (('#3a2d6e', '#2a2054', '#1c153c'), '#ece8fb', '#ffffff', '#9dffc4', '#1c153c', '#7ff0ad'),
    'pine': (('#2d4a3c', '#20372c', '#15261e'), '#e6f5ec', '#ffffff', '#7dffb2', '#0b2216', '#5cf09a'),
    'navy': (('#2b3a63', '#1f2b4d', '#141c35'), '#e8edfa', '#ffffff', '#ffd23c', '#1b1405', '#f0c11d'),
    'red': (('#d8483a', '#b9352a', '#92261e'), '#fdeeea', '#ffffff', '#16161a', '#fdeeea', '#000000'),
    'sand': (('#c9b48a', '#b09a6e', '#8f7b52'), '#2b1d0e', '#1c1206', '#5b3a1f', '#f6e4c0', '#472c16'),
    'olive': (('#5d6a58', '#4a5646', '#364033'), '#eef1ea', '#ffffff', '#efe8d6', '#1d211b', '#ddd3bb'),
    'slate': (('#4a4f57', '#383c43', '#26292e'), '#eceef1', '#ffffff', '#f2c230', '#16161a', '#e0ae1c'),
    'ice': (('#3c6f9a', '#2c5679', '#1f3f5a'), '#eaf4fb', '#ffffff', '#e9f7ff', '#123049', '#cfe9f8'),
    'kraft': (('#c99c6c', '#b3855a', '#9a6e47'), '#2a1c10', '#1f140a', '#2f4a33', '#f3ead8', '#243a28'),
}

_n = itertools.count()


def board(sid, title, *, batch, palette, name, verbs, svg, motion=(), key=None, visit=1, css='', defs=''):
    """One keepsake: registers it for the app and writes its canvas board."""
    stops, text, head, ctabg, ctafg, hover = PALETTES[palette]
    moving = [m[1] for m in motion]
    rules = ''.join(m[0] for m in motion) + css
    if moving:
        rules += '@media (prefers-reduced-motion:reduce){' + ','.join(moving) + '{transition-duration:.01s!important;transition-delay:0s!important;animation:none!important}}'
    file = f'{sid}.dc.html'
    KIND[sid] = next(('cutout' if f.filename.endswith('cutouts.py') else f.function for f in inspect.stack(0)[1:] if f.filename.endswith(('objects.py', 'cutouts.py'))), 'bespoke')
    REGISTRY[file] = (sid, (key or title).lower(), visit, batch, name)
    frame.write(file, title=f'{title} keepsake', fonts=FONTS, css=rules, svg=(f'<defs>{defs}</defs>' if defs else '') + svg,
                focus=ctabg, bg=f'radial-gradient(ellipse 55% 65% at 34% 48%, {stops[0]} 0%, {stops[1]} 58%, {stops[2]} 100%)',
                text=text, head=head, ctabg=ctabg, ctafg=ctafg, ctahover=hover, label=name + f', from {title}.',
                h1=title, p='', opened=verbs[0], closed=verbs[1])


# ── shapes ───────────────────────────────────────────────────────────────────────────────────
def g(inner, cls='', at=None, rot=0, scale=1, style=''):
    """A group, placed: at=(x, y), rotated, scaled."""
    t = (f'translate({at[0]} {at[1]}) ' if at else '') + (f'rotate({rot}) ' if rot else '') + (f'scale({scale})' if scale != 1 else '')
    return f'<g{f" class=\"{cls}\"" if cls else ""}{f" style=\"{style}\"" if style else ""}><g transform="{t.strip()}">{inner}</g></g>' if cls else f'<g transform="{t.strip()}">{inner}</g>'


def paper(d, fill, cls='', grain_id=None, edge=None):
    """Any shape as paper: its fill, a grain on top (pass grain_id after adding grain() to defs), an edge line."""
    out = f'<path d="{d}" fill="{fill}"></path>'
    if grain_id:
        out += f'<path d="{d}" filter="url(#{grain_id})"></path>'
    if edge:
        out += f'<path d="{d}" fill="none" stroke="{edge}" stroke-width="1.2"></path>'
    return f'<g class="{cls}">{out}</g>' if cls else out


def tag_path(w, h, point=34, r=12):
    """A luggage-tag outline: rounded at one end, pointed at the other (the hole goes in the point)."""
    return f'M{r} 0 H{w - point} L{w} {h / 2} L{w - point} {h} H{r} Q0 {h} 0 {h - r} V{r} Q0 0 {r} 0 Z'


def stub_path(w, h, notch=10, at=None):
    """A ticket: two half-circle bites where it tears (at x = `at`, default 70%)."""
    x = at if at is not None else w * .7
    return f'M0 0 H{x - notch} A{notch} {notch} 0 0 0 {x + notch} 0 H{w} V{h} H{x + notch} A{notch} {notch} 0 0 0 {x - notch} {h} H0 Z'


def arch_path(w, h, r=None):
    """A card with an arched top (a booth, a window, a gate)."""
    r = r or w / 2
    return f'M0 {r} A{w / 2} {r} 0 0 1 {w} {r} V{h - 14} Q{w} {h} {w - 14} {h} H14 Q0 {h} 0 {h - 14} Z'


def token(r, fill, rim, inner=''):
    return f'<circle r="{r}" fill="{fill}" stroke="{rim}" stroke-width="3"></circle><circle r="{r * .84}" fill="none" stroke="{rim}" stroke-opacity=".6" stroke-dasharray="3 5"></circle>{inner}'


def hole(x, y, r=7, fill='#2a2018'):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="{fill}"></circle>'


def string(x1, y1, x2, y2, sag=30, color='#efe4cf'):
    return f'<path d="M{x1} {y1} Q{(x1 + x2) / 2} {max(y1, y2) + sag} {x2} {y2}" fill="none" stroke="{color}" stroke-width="2"></path>'


def words(x, y, s, face='hand', size=16, fill='#2b2b30', anchor='start', rot=0, cls='', weight=None):
    t = f' transform="rotate({rot} {x} {y})"' if rot else ''
    w = f' font-weight="{weight}"' if weight else ''
    c = f' class="{cls}"' if cls else ''
    return f'<text{c} x="{x}" y="{y}" font-family="{FACE[face]}" font-size="{size}"{w} fill="{fill}" text-anchor="{anchor}"{t}>{s}</text>'


def lines(x, y, rows, face='hand', size=16, fill='#2b2b30', gap=None, anchor='start'):
    gap = gap or size * 1.35
    return ''.join(words(x, y + i * gap, r, face, size, fill, anchor) for i, r in enumerate(rows))


def stamp(word, sub='', ink='#a8321f', r=34, shape='circle', cls=''):
    """A rubber stamp, unevenly inked (needs grain-free ink: draw it last)."""
    ring = {'circle': f'<circle r="{r}" fill="none" stroke="{ink}" stroke-width="2.6"></circle><circle r="{r * .8}" fill="none" stroke="{ink}" stroke-width="1"></circle>',
            'box': f'<rect x="{-r * 1.4}" y="{-r * .6}" width="{r * 2.8}" height="{r * 1.2}" rx="4" fill="none" stroke="{ink}" stroke-width="2.6"></rect>'}[shape]
    body = ring + words(0, 5 if not sub else 0, word, 'mono', r * .42, ink, 'middle', weight=700) + (words(0, r * .42, sub, 'mono', r * .26, ink, 'middle') if sub else '')
    return f'<g class="{cls}" opacity=".88">{body}</g>' if cls else f'<g opacity=".88">{body}</g>'


def flake(r, color='#fff', w=3):
    arms = ''.join(f'<g transform="rotate({a})"><path d="M0 0 V{-r} M0 {-r * .55} l{-r * .22} {-r * .22} M0 {-r * .55} l{r * .22} {-r * .22}"></path></g>' for a in range(0, 360, 60))
    return f'<g stroke="{color}" stroke-width="{w}" stroke-linecap="round" fill="none">{arms}</g>'


def scatter(n, box, r=(2, 4), fill='#fff', cls='bit', seed=1):
    """Small dots spread over a box (snow, sparks, crumbs), each with its own delay for fall()/pop()."""
    x0, y0, x1, y1 = box
    out = ''
    for i in range(n):
        x = x0 + ((i * 97 + seed * 31) % 1000) / 1000 * (x1 - x0)
        y = y0 + ((i * 61 + seed * 17) % 1000) / 1000 * (y1 - y0)
        out += f'<circle class="{cls}" style="--d:{(i * .37) % 1.6:.2f}s" cx="{x:.0f}" cy="{y:.0f}" r="{r[0] + i % (r[1] - r[0] + 1)}" fill="{fill}"></circle>'
    return out


# ── moments: each returns (css, selector) ───────────────────────────────────────────────────
def _k():
    return f'k{next(_n)}'


def swing(sel, origin='50% 0%', deg=9, delay=0, times=1):
    k = _k()
    return (f'{sel}{{transform-box:fill-box;transform-origin:{origin}}}.open {sel}{{animation:{k} 2.2s ease-in-out {delay}s {times}}}'
            f'@keyframes {k}{{0%,100%{{transform:rotate(0)}}20%{{transform:rotate({deg}deg)}}45%{{transform:rotate({-deg * .8}deg)}}70%{{transform:rotate({deg * .45}deg)}}88%{{transform:rotate({-deg * .2}deg)}}}}', sel)


def drawon(sel, delay=0, dur=.8):
    """Paths with pathLength="1" draw themselves; a --d on each element adds to the delay."""
    return (f'{sel}{{stroke-dasharray:1;stroke-dashoffset:1;transition:stroke-dashoffset .2s}}'
            f'.open {sel}{{stroke-dashoffset:0;transition:stroke-dashoffset {dur}s ease-out calc({delay}s + var(--d, 0s))}}', sel)


def appear(sel, delay=0, dur=.3):
    """Hidden until opened; --d per element staggers them."""
    return (f'{sel}{{opacity:0;transition:opacity .15s}}.open {sel}{{opacity:1;transition:opacity {dur}s ease-out calc({delay}s + var(--d, 0s))}}', sel)


def hide(sel, delay=0, dur=.25):
    return (f'{sel}{{transition:opacity .2s}}.open {sel}{{opacity:0;transition:opacity {dur}s calc({delay}s + var(--d, 0s))}}', sel)


def pop(sel, delay=0):
    k = _k()
    return (f'{sel}{{opacity:0;transform-box:fill-box;transform-origin:50% 50%}}.open {sel}{{animation:{k} .5s cubic-bezier(.3,1.6,.5,1) calc({delay}s + var(--d, 0s)) both}}'
            f'@keyframes {k}{{0%{{opacity:0;transform:scale(.2) rotate(-14deg)}}100%{{opacity:1;transform:none}}}}', sel)


def stampin(sel, delay=0):
    k = _k()
    return (f'{sel}{{opacity:0;transform-box:fill-box;transform-origin:50% 50%}}.open {sel}{{animation:{k} .25s ease-out calc({delay}s + var(--d, 0s)) both}}'
            f'@keyframes {k}{{0%{{opacity:0;transform:scale(1.8)}}100%{{opacity:1;transform:none}}}}', sel)


def slide(sel, dx, dy, rot=0, delay=0, dur=.7):
    return (f'{sel}{{transform-box:fill-box;transition:transform .35s ease-in}}'
            f'.open {sel}{{transform:translate({dx}px,{dy}px) rotate({rot}deg);transition:transform {dur}s cubic-bezier(.3,1.2,.5,1) {delay}s}}', sel)


def fall(sel, dist=520, dur=2.6, times=2):
    k = _k()
    return (f'{sel}{{opacity:0}}.open {sel}{{animation:{k} {dur}s linear var(--d, 0s) {times}}}'
            f'@keyframes {k}{{0%{{opacity:0;transform:translateY(-30px)}}15%{{opacity:1}}100%{{opacity:.2;transform:translateY({dist}px)}}}}', sel)


def rise(sel, dist=60, dur=2, times=2):
    """Steam, bubbles, notes: drift up and fade."""
    k = _k()
    return (f'{sel}{{opacity:0;transform-box:fill-box}}.open {sel}{{animation:{k} {dur}s ease-out var(--d, 0s) {times}}}'
            f'@keyframes {k}{{0%{{opacity:0;transform:translateY(10px)}}30%{{opacity:.9}}100%{{opacity:0;transform:translate(8px,{-dist}px)}}}}', sel)


def spin(sel, turns=3, dur=1.2, delay=0):
    k = _k()
    return (f'{sel}{{transform-box:fill-box;transform-origin:50% 50%}}.open {sel}{{animation:{k} {dur}s cubic-bezier(.3,0,.3,1) {delay}s}}'
            f'@keyframes {k}{{to{{transform:rotate({360 * turns}deg)}}}}', sel)


def flip(sel, face_a, face_b, times=4):
    """A coin or card turning over: `sel` spins, face_a/face_b swap halfway through each turn."""
    k1, k2, k3 = _k(), _k(), _k()
    return (f'{sel}{{transform-box:fill-box;transform-origin:50% 50%}}{face_b}{{opacity:0}}'
            f'.open {sel}{{animation:{k1} .28s linear {times}}}.open {face_a}{{animation:{k2} .56s step-end {times // 2} both}}.open {face_b}{{animation:{k3} .56s step-end {times // 2} both}}'
            f'@keyframes {k1}{{0%{{transform:scaleY(1)}}25%{{transform:scaleY(0)}}50%{{transform:scaleY(-1)}}75%{{transform:scaleY(0)}}100%{{transform:scaleY(1)}}}}'
            f'@keyframes {k2}{{0%{{opacity:1}}25%{{opacity:0}}75%{{opacity:1}}}}@keyframes {k3}{{0%{{opacity:0}}25%{{opacity:1}}75%{{opacity:0}}}}', f'{sel},{face_a},{face_b}')


def shake(sel, times=8, delay=0):
    k = _k()
    return (f'{sel}{{transform-box:fill-box}}.open {sel}{{animation:{k} .09s linear {delay}s {times}}}'
            f'@keyframes {k}{{25%{{transform:translate(2px,-1px)}}50%{{transform:translate(-2px,1px)}}75%{{transform:translate(1px,2px)}}}}', sel)


def glow(sel, to=1, delay=0, dur=.8):
    return (f'{sel}{{opacity:0;transition:opacity .3s}}.open {sel}{{opacity:{to};transition:opacity {dur}s {delay}s}}', sel)


def arc_points(cx, cy, r, n, start=-90, step=None):
    step = step if step is not None else 360 / n
    return [(cx + r * math.cos(math.radians(start + i * step)), cy + r * math.sin(math.radians(start + i * step))) for i in range(n)]
