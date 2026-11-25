"""Die-cut keepsakes: the outline follows the drawing, so no two read the same in the jar grid.

A slip's own things (a ferry and a cave, a tiffin and a chai glass) are drawn from MOTIFS and composed;
the piece is cut around them with a border, in one of five materials, each with its own moment:

    sticker  white kiss-cut border; touching it peels it up off the page
    magnet   thick, glossy, with a dark edge; it drops onto the fridge door and bounces
    patch    embroidered border with stitches; it presses on, the stitches catch the light
    pin      enamel in a gold rim; a shine runs over it
    charm    acrylic on a key ring; it swings

    cutout('elephanta', 'Ferry to Elephanta Caves', batch='b3', palette='sea', kind='magnet',
           name='…', parts=[('cave', 0, -40, 2.6), ('ferry', 0, 40, 2.4)], head='Elephanta', sub='first boat at 9')

`parts` are (motif, x, y, scale, *colours) around 0,0 in about ±150. `head`/`sub` go on a banner across
the bottom (banner=(x, y, rot) moves it). Check the silhouette sheet: build.py lists any two that match.
"""
from objects import MOTIFS, fit, text, place  # noqa: F401
from kit import board, g, swing, appear, slide, glow  # noqa: F401

BORDER = {'sticker': 9, 'magnet': 7, 'patch': 10, 'pin': 5, 'charm': 8}


def _banner(w, h, fill):
    n = 12
    return (f'<path d="M{-w / 2 - n} {-h / 2 + 4} H{w / 2 + n} L{w / 2 + 2} 2 L{w / 2 + n} {h / 2 + 4} H{-w / 2 - n} L{-w / 2 - 2} 2 Z" fill="{fill}" opacity=".85"></path>'
            f'<path d="M{-w / 2} {-h / 2} H{w / 2} V{h / 2} Q0 {h / 2 + 6} {-w / 2} {h / 2} Z" fill="{fill}"></path>')


def _edged(parts, cls, width, extra=''):
    """The parts again as a solid outline `width` wide (in the drawing's own units), for the cut border."""
    out = ''
    for m, x, y, sc, *cols in parts:
        out += f'<g transform="translate({x} {y}) scale({sc})" style="--w:{width / sc:.2f}px">{MOTIFS[m](*cols)}</g>'
    return f'<g class="{cls}">{out}{extra}</g>'


def cutout(sid, title, *, batch, palette, name, kind, parts, head, sub='', verbs=None, edge=None, ribbon='#a8321f', ink='#fbf8f1',
           banner=(0, 120, -2), back=None, sparkle=(), key=None, visit=1, rot=-4):
    """One die-cut keepsake. back=(rx, ry, cy) adds a backing oval so separate things become one piece."""
    w_ = BORDER[kind]
    edge = edge or {'sticker': '#fbf8f1', 'magnet': '#fbf8f1', 'patch': ribbon, 'pin': '#c9a14a', 'charm': '#eef3f6'}[kind]
    verbs = verbs or {'sticker': ('Peel it up', 'Press it down'), 'magnet': ('Stick it on the fridge', 'Take it off'),
                      'patch': ('Sew it on', 'Unpick it'), 'pin': ('Pin it on', 'Unpin it'), 'charm': ('Give it a swing', 'Let it hang')}[kind]
    bx, by, brot = banner
    hs = fit(head, 'serif', 24, 220)
    bw = min(250, max(110, len(head) * .64 * hs + 30))
    bh = 48 if sub else 36
    ban_shape = _banner(bw, bh, ribbon)
    ban = (f'<g transform="translate({bx} {by}) rotate({brot})">{ban_shape}'
           + text(0, 8 if not sub else 2, head, 'serif', 24, ink, bw - 16)
           + (text(0, 18, sub, 'mono', 10, ink, bw - 14) if sub else '') + '</g>')
    ban_edge = f'<g transform="translate({bx} {by}) rotate({brot})" style="--w:{w_}px">{_banner(bw, bh, "#000")}</g>'
    backing = f'<ellipse cy="{back[2]}" rx="{back[0]}" ry="{back[1]}" style="--w:{w_}px"></ellipse>' if back else ''
    back_art = f'<ellipse cy="{back[2]}" rx="{back[0]}" ry="{back[1]}" fill="{back[3] if len(back) > 3 else "#e9dcc2"}"></ellipse>' if back else ''
    art = back_art + place(parts)
    outline = _edged(parts, 'dc-e', w_ * 2, backing + ban_edge)
    css = ('.dc-e *{fill:%s!important;stroke:%s!important;stroke-width:var(--w)!important;stroke-linejoin:round!important;stroke-linecap:round!important;'
           'fill-opacity:1!important;stroke-opacity:1!important;opacity:1!important;stroke-dasharray:none!important}' % (edge, edge))
    under, over, motion = '', '', []
    if kind == 'magnet':
        under = _edged(parts, 'dc-d', w_ * 2, backing + ban_edge).replace('class="dc-d"', 'class="dc-d" transform="translate(4 7)"')
        css += '.dc-d *{fill:#2a2018!important;stroke:#2a2018!important;stroke-width:var(--w)!important;stroke-linejoin:round!important;fill-opacity:1!important;stroke-opacity:1!important;opacity:.55!important;stroke-dasharray:none!important}'
        css += '.pc{transform-box:fill-box}.open .pc{animation:mg .7s cubic-bezier(.3,0,.4,1)}@keyframes mg{0%{transform:translateY(0)}35%{transform:translateY(-34px) rotate(2deg)}70%{transform:translateY(0)}84%{transform:translateY(-8px)}100%{transform:none}}'
        over = '<ellipse class="gloss" cx="-40" cy="-60" rx="70" ry="22" fill="#fff" opacity=".18" transform="rotate(-24 -40 -60)"></ellipse>'
        motion = [('', '.pc'), glow('.gloss', .35, .5)]
    elif kind == 'patch':
        stitch = _edged(parts, 'dc-s', w_ * .7, backing + ban_edge.replace(f'--w:{w_}px', f'--w:{w_ * .35:.1f}px'))
        css += ('.dc-s *{fill:none!important;stroke:#fff8e8!important;stroke-width:var(--w)!important;stroke-dasharray:4 5!important;fill-opacity:1!important;stroke-opacity:.8!important;opacity:1!important}'
                '.pc{transform-box:fill-box;transform-origin:50% 50%}.open .pc{animation:pt .5s ease-out}@keyframes pt{40%{transform:scale(.95)}100%{transform:none}}')
        under = ''
        outline += stitch
        motion = [('', '.pc'), appear('.spk', .3)]
    elif kind == 'pin':
        over = '<path class="shine" d="M-160 -60 L-120 -160 L-96 -160 L-136 -60 Z" fill="#fff" opacity="0"></path>'
        css += ('.shine{opacity:0}.open .shine{animation:shn 1s ease-out .15s}@keyframes shn{0%{opacity:0;transform:translateX(0)}30%{opacity:.55}100%{opacity:0;transform:translateX(280px)}}')
        under = '<g opacity=".9"><path d="M-30 -8 H30" stroke="#8a8f96" stroke-width="4"></path></g>'
        motion = [('', '.shine'), swing('.pc', '50% 0%', 4)]
    elif kind == 'charm':
        top = min(y - 30 * sc for _, x, y, sc, *c in parts)
        under = (f'<g class="ring"><circle cx="0" cy="{top - 46}" r="24" fill="none" stroke="#a9adb4" stroke-width="6"></circle>'
                 f'<path d="M0 {top - 22} V{top + 90}" stroke="#a9adb4" stroke-width="5" stroke-dasharray="7 3"></path></g>')
        motion = [swing('.pc', f'50% 0%', 10)]
    else:  # sticker
        css += '.pc{transform-box:fill-box;transform-origin:0% 100%}.open .pc{transform:rotate(-7deg) translate(-4px,-14px);transition:transform .5s cubic-bezier(.3,1.4,.5,1)}.pc{transition:transform .3s}'
        over = '<path class="shine" d="M-160 -60 L-120 -160 L-96 -160 L-136 -60 Z" fill="#fff" opacity="0"></path>'
        css += '.shine{opacity:0}.open .shine{animation:shn 1s ease-out .2s}@keyframes shn{0%{opacity:0;transform:translateX(0)}30%{opacity:.45}100%{opacity:0;transform:translateX(280px)}}'
        motion = [('', '.pc,.shine')]
    spk = ''.join(f'<g class="spk" style="--d:{i * .12:.2f}s" transform="translate({x} {y})"><path d="M0 -10 L2.5 -2.5 L10 0 L2.5 2.5 L0 10 L-2.5 2.5 L-10 0 L-2.5 -2.5 Z" fill="#fff6d0"></path></g>'
                  for i, (x, y) in enumerate(sparkle))
    if spk and kind != 'patch':
        motion.append(appear('.spk', .35))
    body = under + outline + art + ban + over + spk
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, cls='pc', at=(300, 290), rot=rot), motion=motion)


# ── more motifs for the die-cuts (same contract as objects.MOTIFS: about 80 across, centred on 0,0) ──
def _mm(fn):
    MOTIFS[fn.__name__.rstrip('_')] = fn
    return fn


@_mm
def chai(c='#c98a3a', c2='#e9eef2'):
    """A cutting-chai glass, ridged."""
    return (f'<path d="M-16 -26 H16 L12 26 H-12 Z" fill="{c2}" fill-opacity=".8"></path><path d="M-14 -12 H14 L12 26 H-12 Z" fill="{c}"></path>'
            + ''.join(f'<path d="M{x} -24 L{x * .75:.0f} 24" stroke="#fff" stroke-opacity=".5" stroke-width="2"></path>' for x in (-10, -4, 2, 8)))


@_mm
def vadapav(c='#e0a64a', c2='#c47a2a'):
    return (f'<path d="M-34 -2 C-34 -30 34 -30 34 -2 Z" fill="{c}"></path><ellipse cy="2" rx="30" ry="9" fill="{c2}"></ellipse>'
            f'<path d="M-32 4 H32 C32 22 -32 22 -32 4 Z" fill="{c}"></path><path d="M14 -2 l14 -14 l3 3 Z" fill="#3f7d2c"></path>')


@_mm
def cone(c='#e9d6b3', c2='#c98a3a'):
    """A newspaper cone of bhel or peanuts."""
    return (f'<path d="M-26 -18 L0 34 L26 -18 Z" fill="{c}"></path><path d="M-18 -6 H16 M-12 6 H10" stroke="#7a6a58" stroke-width="2"></path>'
            f'<path d="M-28 -18 C-20 -34 20 -34 28 -18 Z" fill="{c2}"></path><circle cx="-8" cy="-26" r="3" fill="#3f7d2c"></circle><circle cx="8" cy="-24" r="3" fill="#d8483a"></circle>')


@_mm
def bhutta(c='#f2c14e', c2='#6f8a3a'):
    return (f'<path d="M-6 -36 C10 -36 14 -10 10 20 C8 32 -8 32 -10 20 C-14 -10 -20 -36 -6 -36 Z" fill="{c}"></path>'
            + ''.join(f'<path d="M-10 {y} H10" stroke="#c98a2a" stroke-width="1.5"></path>' for y in range(-26, 24, 7))
            + f'<path d="M-10 10 C-30 20 -30 34 -20 40 L0 30 Z M10 10 C30 20 30 34 20 40 L0 30 Z" fill="{c2}"></path><circle cx="2" cy="-6" r="8" fill="#3a2414" opacity=".35"></circle>')


@_mm
def palm(c='#3f7d2c', c2='#8a5a2e'):
    return (f'<path d="M-2 40 C2 10 0 -10 4 -22" fill="none" stroke="{c2}" stroke-width="6" stroke-linecap="round"></path>'
            f'<path d="M4 -22 C-10 -36 -30 -30 -38 -18 C-24 -24 -12 -22 4 -22 C-4 -40 10 -50 24 -44 C12 -40 6 -32 4 -22 C20 -34 36 -26 40 -12 C28 -20 16 -22 4 -22 Z" fill="{c}"></path>'
            f'<circle cx="0" cy="-18" r="5" fill="{c2}"></circle><circle cx="8" cy="-16" r="5" fill="{c2}"></circle>')


@_mm
def tiffin(c='#c9ced3', c2='#8a8f96'):
    return (''.join(f'<rect x="-24" y="{y}" width="48" height="18" rx="5" fill="{c}" stroke="{c2}" stroke-width="2"></rect>' for y in (-18, 2, 22))
            + f'<path d="M-28 -10 V36 M28 -10 V36" stroke="{c2}" stroke-width="3"></path><path d="M-28 -10 C-28 -40 28 -40 28 -10" fill="none" stroke="{c2}" stroke-width="4"></path>')


@_mm
def steamer(c='#d9b27a', c2='#a8803f'):
    return (f'<ellipse cy="-8" rx="38" ry="12" fill="{c}"></ellipse><path d="M-38 -8 V16 C-38 28 38 28 38 16 V-8" fill="{c}"></path>'
            + ''.join(f'<path d="M{x} -6 V22" stroke="{c2}" stroke-width="1.5"></path>' for x in range(-32, 36, 8))
            + f'<path d="M-38 4 C-38 14 38 14 38 4" fill="none" stroke="{c2}" stroke-width="2"></path>')


@_mm
def sushi(c='#f6f3ec', c2='#e46f2a'):
    return f'<rect x="-28" y="-6" width="56" height="20" rx="9" fill="{c}"></rect><path d="M-30 -4 C-20 -18 20 -18 30 -4 C20 -8 -20 -8 -30 -4 Z" fill="{c2}"></path><rect x="-6" y="-12" width="12" height="26" fill="#1d2b22"></rect>'


@_mm
def skewer(c='#8c4a22', c2='#e46f2a'):
    return (f'<path d="M-6 -44 V40" stroke="#9aa1a4" stroke-width="3"></path>'
            + ''.join(f'<rect x="-14" y="{y}" width="28" height="14" rx="5" fill="{(c, c2, "#3f7d2c")[i % 3]}"></rect>' for i, y in enumerate(range(-34, 26, 16))))


@_mm
def beer(c='#f2b33d', c2='#fbf6ea'):
    return (f'<rect x="-20" y="-20" width="40" height="48" rx="4" fill="{c}"></rect><path d="M20 -8 C36 -8 36 18 20 18" fill="none" stroke="{c}" stroke-width="6"></path>'
            f'<path d="M-24 -18 C-24 -34 24 -34 24 -18 C16 -12 -16 -12 -24 -18 Z" fill="{c2}"></path>')


@_mm
def record(c='#1b1b1e', c2='#e0245e'):
    return (f'<circle r="38" fill="{c}"></circle>' + ''.join(f'<circle r="{r}" fill="none" stroke="#fff" stroke-opacity=".12"></circle>' for r in (32, 26, 20))
            + f'<circle r="13" fill="{c2}"></circle><circle r="2.5" fill="#f6f3ec"></circle>')


@_mm
def cassette(c='#2b2d33', c2='#f2c230'):
    return (f'<rect x="-40" y="-26" width="80" height="52" rx="5" fill="{c}"></rect><rect x="-32" y="-20" width="64" height="24" rx="3" fill="{c2}"></rect>'
            f'<circle cx="-16" cy="-8" r="7" fill="{c}"></circle><circle cx="16" cy="-8" r="7" fill="{c}"></circle><path d="M-24 26 L-18 12 H18 L24 26" fill="#555"></path>')


@_mm
def headphones(c='#2b2d33', c2='#e0245e'):
    return f'<path d="M-30 10 C-30 -40 30 -40 30 10" fill="none" stroke="{c}" stroke-width="7"></path><rect x="-38" y="0" width="16" height="30" rx="6" fill="{c2}"></rect><rect x="22" y="0" width="16" height="30" rx="6" fill="{c2}"></rect>'


@_mm
def gamepad(c='#2b2d33', c2='#e0245e'):
    return (f'<path d="M-30 -14 H30 C44 -14 50 24 38 28 C28 32 22 16 14 14 H-14 C-22 16 -28 32 -38 28 C-50 24 -44 -14 -30 -14 Z" fill="{c}"></path>'
            f'<path d="M-26 -2 h12 M-20 -8 v12" stroke="#f6f3ec" stroke-width="4"></path><circle cx="18" cy="-4" r="4" fill="{c2}"></circle><circle cx="26" cy="2" r="4" fill="#f2c230"></circle>')


@_mm
def knight(c='#2b2b30'):
    return f'<path d="M-18 30 H22 V20 H16 C16 6 20 -6 14 -20 C8 -32 -4 -36 -14 -30 L-12 -24 L-24 -10 C-26 -4 -22 0 -16 -2 L-6 -8 C-4 2 -16 10 -14 20 H-18 Z" fill="{c}"></path><circle cx="2" cy="-22" r="2.5" fill="#f6f3ec"></circle>'


@_mm
def card(c='#fbf8f1', c2='#c0392b'):
    return (f'<rect x="-22" y="-32" width="44" height="64" rx="5" fill="{c}" stroke="#2b2b30" stroke-width="2"></rect>'
            f'<path d="M0 12 C-18 0 -14 -14 -6 -14 C-2 -14 0 -10 0 -8 C0 -10 2 -14 6 -14 C14 -14 18 0 0 12 Z" fill="{c2}"></path><path d="M-16 -22 h6 M10 22 h6" stroke="{c2}" stroke-width="3"></path>')


@_mm
def plane(c='#f6f3ec', c2='#9aa0a6'):
    return f'<path d="M-40 6 L40 -20 L-6 14 Z" fill="{c}"></path><path d="M-6 14 L40 -20 L4 28 Z" fill="{c2}"></path><path d="M-40 6 L-6 14 L-14 20 Z" fill="{c2}"></path>'


@_mm
def rocket(c='#f6f3ec', c2='#d8483a'):
    return (f'<path d="M0 -40 C16 -24 18 0 14 20 H-14 C-18 0 -16 -24 0 -40 Z" fill="{c}"></path><circle cy="-10" r="7" fill="#5aa4d6" stroke="{c2}" stroke-width="3"></circle>'
            f'<path d="M-14 6 L-26 26 L-12 22 Z M14 6 L26 26 L12 22 Z" fill="{c2}"></path><path d="M-8 20 L0 38 L8 20 Z" fill="#f2b33d"></path>')


@_mm
def torch(c='#2b2d33', c2='#ffe08a'):
    return f'<path d="M-30 30 L30 -30" stroke="{c2}" stroke-width="0"></path><g transform="rotate(-35)"><rect x="-8" y="-6" width="16" height="44" rx="4" fill="{c}"></rect><path d="M-14 -24 H14 L8 -6 H-8 Z" fill="{c}"></path><ellipse cy="-24" rx="14" ry="4" fill="{c2}"></ellipse><path d="M-14 -26 L-40 -90 H40 L14 -26 Z" fill="{c2}" opacity=".35"></path></g>'


@_mm
def bunny(c='#2b2b30'):
    """A hand-shadow rabbit."""
    return f'<path d="M-30 20 C-30 0 -14 -6 0 -6 L-6 -40 C-4 -46 4 -44 6 -38 L10 -8 L18 -40 C20 -46 28 -44 26 -36 L20 -4 C30 2 34 14 30 24 C20 34 -20 34 -30 20 Z" fill="{c}"></path><circle cx="14" cy="8" r="3" fill="#f6f3ec"></circle>'


@_mm
def sock(c='#c0392b', c2='#f6f3ec'):
    return (f'<path d="M-16 -40 H16 V10 C16 20 24 22 32 24 C42 28 38 42 26 40 L-6 36 C-16 34 -16 24 -16 14 Z" fill="{c}"></path>'
            f'<rect x="-16" y="-40" width="32" height="10" fill="{c2}"></rect><circle cx="-2" cy="-14" r="5" fill="{c2}"></circle><circle cx="8" cy="-14" r="5" fill="{c2}"></circle><circle cx="-2" cy="-14" r="2" fill="#222"></circle><circle cx="8" cy="-14" r="2" fill="#222"></circle>')


@_mm
def brick(c='#d8483a'):
    """A toy building brick."""
    return f'<rect x="-34" y="-10" width="68" height="34" rx="3" fill="{c}"></rect>' + ''.join(f'<rect x="{x}" y="-20" width="14" height="10" rx="2" fill="{c}"></rect>' for x in (-28, -7, 14))


@_mm
def block(c='#d9b27a', c2='#a8803f'):
    """A wooden block from a tower game."""
    return f'<path d="M-40 -6 L30 -16 L40 -8 L-30 2 Z" fill="#ead2a0"></path><path d="M-40 -6 L-30 2 V16 L-40 8 Z" fill="{c2}"></path><path d="M-30 2 L40 -8 V6 L-30 16 Z" fill="{c}"></path>'


@_mm
def paper(c='#f6f3ec', c2='#2b2b30'):
    """A newspaper, folded."""
    return (f'<path d="M-34 -30 H34 V30 H-34 Z" fill="{c}" stroke="#c9c4b8" stroke-width="1.5"></path><rect x="-28" y="-24" width="56" height="10" fill="{c2}"></rect>'
            + ''.join(f'<path d="M{x} {y} h22" stroke="#9aa0a6" stroke-width="2"></path>' for x in (-28, 4) for y in (-6, 2, 10, 18, 24)))


@_mm
def tv(c='#2b2d33', c2='#9fd3f2'):
    return f'<rect x="-36" y="-26" width="72" height="46" rx="6" fill="{c}"></rect><rect x="-30" y="-20" width="60" height="34" rx="3" fill="{c2}"></rect><path d="M-12 28 H12 M0 20 V28 M-14 -38 L0 -26 L14 -38" stroke="{c}" stroke-width="3" fill="none"></path>'


@_mm
def laptop(c='#9aa0a6', c2='#9fd3f2'):
    return f'<rect x="-30" y="-26" width="60" height="38" rx="3" fill="{c}"></rect><rect x="-26" y="-22" width="52" height="30" fill="{c2}"></rect><path d="M-40 12 H40 L34 20 H-34 Z" fill="{c}"></path>'


@_mm
def dumbbell(c='#2b2d33', c2='#9aa0a6'):
    return f'<rect x="-24" y="-3" width="48" height="6" fill="{c2}"></rect><rect x="-38" y="-16" width="12" height="32" rx="3" fill="{c}"></rect><rect x="26" y="-16" width="12" height="32" rx="3" fill="{c}"></rect><rect x="-26" y="-11" width="6" height="22" rx="2" fill="{c}"></rect><rect x="20" y="-11" width="6" height="22" rx="2" fill="{c}"></rect>'


@_mm
def dhol(c='#c0392b', c2='#f2c230'):
    return (f'<rect x="-34" y="-20" width="68" height="40" rx="6" fill="{c}"></rect><ellipse cx="-34" rx="8" ry="20" fill="#f3e3c0"></ellipse><ellipse cx="34" rx="8" ry="20" fill="#f3e3c0"></ellipse>'
            + ''.join(f'<path d="M{x} -20 L{x + 12} 20" stroke="{c2}" stroke-width="2"></path>' for x in range(-28, 28, 12)))


@_mm
def mask(c='#2f6db5', c2='#1d2b30'):
    """A dive mask and snorkel."""
    return f'<rect x="-30" y="-14" width="60" height="30" rx="12" fill="{c2}"></rect><rect x="-24" y="-9" width="22" height="20" rx="7" fill="#bfe3f2"></rect><rect x="2" y="-9" width="22" height="20" rx="7" fill="#bfe3f2"></rect><path d="M34 20 V-34 H40" fill="none" stroke="{c}" stroke-width="6" stroke-linecap="round"></path>'


@_mm
def cookie(c='#d9963a', c2='#4a2a14'):
    return f'<circle r="28" fill="{c}"></circle>' + ''.join(f'<circle cx="{x}" cy="{y}" r="4" fill="{c2}"></circle>' for x, y in [(-12, -10), (10, -14), (2, 4), (-14, 12), (14, 12)])


@_mm
def cupcake(c='#f7c9d4', c2='#c98a3a'):
    return f'<path d="M-22 2 H22 L16 30 H-16 Z" fill="{c2}"></path><path d="M-24 4 C-30 -14 -12 -30 0 -30 C12 -30 30 -14 24 4 Z" fill="{c}"></path><circle cy="-32" r="6" fill="#c0392b"></circle>'


@_mm
def chocolate(c='#5a3216', c2='#c0392b'):
    return f'<rect x="-22" y="-34" width="44" height="68" rx="3" fill="{c}"></rect>' + ''.join(f'<rect x="{x}" y="{y}" width="16" height="14" rx="2" fill="#6b3d1e"></rect>' for x in (-18, 2) for y in (-30, -12)) + f'<path d="M-22 4 H22 V34 H-22 Z" fill="{c2}"></path>'


@_mm
def gelato(c='#f6f3ec', c2='#9ccc4a'):
    return f'<path d="M-20 0 H20 L14 28 H-14 Z" fill="{c}"></path><path d="M-20 0 C-22 -18 -6 -24 0 -16 C8 -26 24 -16 20 0 Z" fill="{c2}"></path><path d="M6 -10 L22 -34" stroke="#c0392b" stroke-width="5" stroke-linecap="round"></path>'


@_mm
def slice_(c='#f2e3b3', c2='#5a3216'):
    return f'<path d="M-34 20 L30 -20 V20 Z" fill="{c}"></path><path d="M-34 20 L30 -20 V-8 L-30 26 Z" fill="{c2}"></path><path d="M-34 6 L30 -34 V-20 L-34 20 Z" fill="{c2}" opacity=".8"></path><circle cx="20" cy="-34" r="6" fill="#d6334a"></circle>'


@_mm
def waffle(c='#d9963a', c2='#b5702a'):
    return f'<path d="M-34 0 C-34 -26 34 -26 34 0 Z" fill="{c}"></path>' + ''.join(f'<path d="M{x} -20 V0" stroke="{c2}" stroke-width="3"></path>' for x in (-20, -6, 8, 22)) + f'<path d="M-30 -10 H30" stroke="{c2}" stroke-width="3"></path>'


@_mm
def shaker(c='#f6f3ec', c2='#9aa0a6'):
    return f'<path d="M-14 -14 H14 L18 30 H-18 Z" fill="{c}" stroke="{c2}" stroke-width="2"></path><path d="M-14 -14 C-14 -34 14 -34 14 -14 Z" fill="{c2}"></path><circle cx="-4" cy="-24" r="2" fill="#2b2b30"></circle><circle cx="4" cy="-24" r="2" fill="#2b2b30"></circle><circle cy="-30" r="2" fill="#2b2b30"></circle>'


@_mm
def olive(c='#6f8a3a', c2='#3f553b'):
    return f'<path d="M-40 20 C-10 10 20 -10 40 -30" fill="none" stroke="{c2}" stroke-width="3"></path>' + ''.join(f'<ellipse cx="{x}" cy="{y}" rx="12" ry="5" fill="{c}" transform="rotate({r} {x} {y})"></ellipse>' for x, y, r in [(-24, 6, -40), (-10, 0, 30), (6, -8, -40), (20, -18, 30)]) + f'<ellipse cx="-4" cy="14" rx="6" ry="8" fill="{c2}"></ellipse>'


@_mm
def gate(c='#c9b48a', c2='#7a5c40'):
    """An arched stone gateway."""
    return (f'<path d="M-40 30 V-16 H-30 V-30 H30 V-16 H40 V30 Z" fill="{c}"></path><path d="M-14 30 V0 C-14 -16 14 -16 14 0 V30 Z" fill="{c2}"></path>'
            f'<path d="M-36 30 V8 C-36 0 -26 0 -26 8 V30 Z M26 30 V8 C26 0 36 0 36 8 V30 Z" fill="{c2}"></path><path d="M-30 -30 L-24 -40 L-18 -30 M18 -30 L24 -40 L30 -30" fill="{c}"></path>')


@_mm
def deco(c='#f2e3cc', c2='#7a9a8a'):
    """An Art Deco facade."""
    return (f'<path d="M-36 30 V-10 H-20 V-24 H-8 V-40 H8 V-24 H20 V-10 H36 V30 Z" fill="{c}"></path>'
            + ''.join(f'<rect x="{x}" y="{y}" width="8" height="10" fill="{c2}"></rect>' for x in (-30, -18, 10, 22) for y in (-2, 14))
            + f'<path d="M-4 -34 V30 M4 -34 V30" stroke="{c2}" stroke-width="2"></path>')


@_mm
def church(c='#f6f3ec', c2='#7a5c40'):
    return (f'<path d="M-30 30 V-4 L-20 -14 L-10 -4 V30 Z M10 30 V-4 L20 -14 L30 -4 V30 Z" fill="{c}"></path><path d="M-10 30 V-10 L0 -20 L10 -10 V30 Z" fill="{c}"></path>'
            f'<path d="M-20 -14 V-34 M-26 -26 H-14 M20 -14 V-34 M14 -26 H26" stroke="{c2}" stroke-width="3"></path><path d="M-5 30 V14 a5 5 0 0 1 10 0 V30 Z" fill="{c2}"></path>')


@_mm
def shirt(c='#5aa4d6', c2='#efe4cf'):
    """A shirt on a washing line."""
    return f'<path d="M-50 -30 H50" stroke="{c2}" stroke-width="2"></path><path d="M-14 -28 L-34 -18 L-26 -4 L-18 -8 V30 H18 V-8 L26 -4 L34 -18 L14 -28 C8 -20 -8 -20 -14 -28 Z" fill="{c}"></path><rect x="-16" y="-34" width="5" height="12" fill="#d9b27a"></rect><rect x="11" y="-34" width="5" height="12" fill="#d9b27a"></rect>'


@_mm
def peak(c='#7a8a6a', c2='#f6f3ec'):
    """A summit with a flag."""
    return f'<path d="M-46 30 L-10 -24 L6 -6 L18 -18 L46 30 Z" fill="{c}"></path><path d="M-10 -24 L-2 -12 L-14 -10 Z" fill="{c2}"></path><path d="M18 -18 V-44" stroke="#3a2a1a" stroke-width="2"></path><path d="M18 -44 L34 -38 L18 -32 Z" fill="#d8483a"></path>'


@_mm
def ladder(c='#a8803f'):
    return f'<path d="M-14 -40 L-18 40 M14 -40 L18 40" stroke="{c}" stroke-width="5" stroke-linecap="round"></path>' + ''.join(f'<path d="M{-14 - i * .5} {-30 + i * 14} H{14 + i * .5}" stroke="{c}" stroke-width="4"></path>' for i in range(6))


@_mm
def carabiner(c='#e46f2a'):
    return f'<path d="M-10 -36 C-24 -36 -26 -24 -24 -10 L-18 26 C-16 40 14 40 16 26 L22 -14 C24 -30 16 -36 4 -36 Z" fill="none" stroke="{c}" stroke-width="7"></path><path d="M22 -14 L4 -36" stroke="#9aa0a6" stroke-width="4"></path>'


@_mm
def shell(c='#f3d9c4', c2='#c98a6a'):
    return f'<path d="M0 26 L-30 -6 C-26 -28 26 -28 30 -6 Z" fill="{c}"></path>' + ''.join(f'<path d="M0 26 L{x} -20" stroke="{c2}" stroke-width="2"></path>' for x in (-20, -10, 0, 10, 20)) + f'<path d="M-8 26 H8 L4 32 H-4 Z" fill="{c2}"></path>'


@_mm
def butterfly(c='#f2b33d', c2='#2b2b30'):
    return f'<path d="M0 0 C-10 -30 -40 -30 -36 -6 C-34 6 -14 6 0 0 C-14 6 -30 20 -20 28 C-10 34 -2 16 0 0 Z" fill="{c}"></path><path d="M0 0 C10 -30 40 -30 36 -6 C34 6 14 6 0 0 C14 6 30 20 20 28 C10 34 2 16 0 0 Z" fill="{c}"></path><path d="M0 -14 V22" stroke="{c2}" stroke-width="4" stroke-linecap="round"></path>'


@_mm
def lion(c='#d9963a', c2='#8a5a2e'):
    return (''.join(f'<ellipse cx="{24 * __import__("math").cos(a / 57.3):.1f}" cy="{24 * __import__("math").sin(a / 57.3):.1f}" rx="12" ry="9" fill="{c2}" transform="rotate({a} {24 * __import__("math").cos(a / 57.3):.1f} {24 * __import__("math").sin(a / 57.3):.1f})"></ellipse>' for a in range(0, 360, 30))
            + f'<circle r="22" fill="{c}"></circle><circle cx="-8" cy="-4" r="2.5" fill="#2b2b30"></circle><circle cx="8" cy="-4" r="2.5" fill="#2b2b30"></circle><path d="M-5 6 L5 6 L0 11 Z" fill="#2b2b30"></path>')


@_mm
def garland(c='#f29a1f', c2='#e8701a'):
    return ''.join(f'<circle cx="{-40 + i * 10}" cy="{-20 + 30 * __import__("math").sin(__import__("math").pi * i / 8):.1f}" r="7" fill="{c if i % 2 else c2}"></circle>' for i in range(9))


@_mm
def diya(c='#b5683a', c2='#ffb347'):
    return f'<path d="M-30 4 C-26 24 26 24 30 4 C14 12 -14 12 -30 4 Z" fill="{c}"></path><path d="M20 2 C14 -10 18 -22 22 -30 C28 -20 30 -8 20 2 Z" fill="{c2}"></path>'


@_mm
def umbrella(c='#e0245e', c2='#f2c230'):
    """A cocktail umbrella."""
    return f'<path d="M-34 0 C-30 -30 30 -30 34 0 Z" fill="{c}"></path><path d="M-17 0 C-14 -26 0 -28 0 -28 M17 0 C14 -26 0 -28 0 -28" stroke="{c2}" stroke-width="2" fill="none"></path><path d="M0 0 V40" stroke="#d9b27a" stroke-width="3"></path>'


@_mm
def float_(c='#e46f2a', c2='#f6efe0'):
    """A net float on rope."""
    return f'<path d="M-46 -20 C-20 0 20 0 46 -20" fill="none" stroke="#c9b48a" stroke-width="3"></path><ellipse cy="4" rx="18" ry="24" fill="{c}"></ellipse><path d="M-18 4 H18" stroke="{c2}" stroke-width="5"></path>'


@_mm
def tag(c='#f6efe0', c2='#2b2b30'):
    """A plain luggage label shape (a small one, for composing)."""
    return f'<path d="M-30 -18 H18 L32 0 L18 18 H-30 Z" fill="{c}"></path><circle cx="20" cy="0" r="4" fill="{c2}"></circle>'


@_mm
def pot(c='#b5683a', c2='#8a4a22'):
    """A little clay pot or creature from the wheel."""
    return f'<path d="M-24 -14 C-34 10 -20 30 0 30 C20 30 34 10 24 -14 Z" fill="{c}"></path><ellipse cy="-14" rx="24" ry="6" fill="{c2}"></ellipse><path d="M-24 4 C-10 10 10 10 24 4" fill="none" stroke="{c2}" stroke-width="2"></path>'


@_mm
def brush(c='#c47a3a', c2='#2f6db5'):
    return f'<path d="M-36 30 L18 -24" stroke="{c}" stroke-width="7" stroke-linecap="round"></path><path d="M14 -20 L22 -28 L30 -20 L22 -12 Z" fill="#9aa0a6"></path><path d="M22 -28 C30 -40 42 -44 44 -42 C46 -40 42 -28 30 -20 Z" fill="{c2}"></path>'


@_mm
def pencil(c='#f2c230', c2='#2b2b30'):
    return f'<g transform="rotate(-35)"><rect x="-6" y="-36" width="12" height="56" fill="{c}"></rect><path d="M-6 20 L0 36 L6 20 Z" fill="#e9c9a0"></path><path d="M-2 32 L0 36 L2 32 Z" fill="{c2}"></path><rect x="-6" y="-42" width="12" height="7" fill="#e0245e"></rect></g>'


@_mm
def spicetin(c='#c9ced3', c2='#c0392b'):
    return f'<circle r="34" fill="{c}"></circle>' + ''.join(f'<circle cx="{20 * __import__("math").cos(a / 57.3):.1f}" cy="{20 * __import__("math").sin(a / 57.3):.1f}" r="9" fill="{col}"></circle>' for a, col in zip(range(0, 360, 60), (c2, '#f2b33d', '#8a5a2e', '#3f7d2c', '#e46f2a', '#f6efe0'))) + '<circle r="9" fill="#7a3a1a"></circle>'


@_mm
def jar(c='#bfe3f2', c2='#c9a14a'):
    return f'<path d="M-22 -22 H22 C30 -14 30 30 22 34 H-22 C-30 30 -30 -14 -22 -22 Z" fill="{c}" fill-opacity=".75"></path><rect x="-20" y="-34" width="40" height="12" rx="3" fill="{c2}"></rect>'


@_mm
def bulb(c='#ffe08a', c2='#9aa0a6'):
    return f'<circle cy="-8" r="22" fill="{c}"></circle><rect x="-10" y="12" width="20" height="16" rx="3" fill="{c2}"></rect><path d="M-6 -6 L0 4 L6 -6" fill="none" stroke="#c98a2a" stroke-width="2"></path>'


@_mm
def neon(c='#e0245e', c2='#2b2d33'):
    """A neon sign: a heart in a frame."""
    return f'<rect x="-40" y="-28" width="80" height="56" rx="8" fill="{c2}"></rect><path d="M0 16 C-24 0 -20 -18 -8 -18 C-3 -18 0 -14 0 -10 C0 -14 3 -18 8 -18 C20 -18 24 0 0 16 Z" fill="none" stroke="{c}" stroke-width="4"></path>'


@_mm
def calendar(c='#f6f3ec', c2='#c0392b'):
    return f'<rect x="-28" y="-24" width="56" height="52" rx="4" fill="{c}"></rect><rect x="-28" y="-24" width="56" height="14" rx="4" fill="{c2}"></rect><path d="M-16 -30 V-18 M16 -30 V-18" stroke="#2b2b30" stroke-width="4" stroke-linecap="round"></path>' + ''.join(f'<rect x="{x}" y="{y}" width="8" height="6" fill="#c9c4b8"></rect>' for x in (-20, -6, 8) for y in (-2, 8, 18))


@_mm
def footprints(c='#2b2b30'):
    return ''.join(f'<g transform="translate({x} {y}) rotate({r})"><ellipse rx="8" ry="13" fill="{c}"></ellipse><circle cy="-18" r="4" fill="{c}"></circle></g>' for x, y, r in [(-24, 16, -15), (-4, -6, 10), (16, 14, -10), (30, -16, 15)])


@_mm
def wheel(c='#2b2b30', c2='#e0245e'):
    """A spinner wheel."""
    return ''.join(f'<path d="M0 0 L{36 * __import__("math").cos(a / 57.3):.1f} {36 * __import__("math").sin(a / 57.3):.1f} A36 36 0 0 1 {36 * __import__("math").cos((a + 60) / 57.3):.1f} {36 * __import__("math").sin((a + 60) / 57.3):.1f} Z" fill="{col}"></path>' for a, col in zip(range(0, 360, 60), (c2, '#f2c230', '#2f6db5', '#3f7d2c', '#e46f2a', '#f6efe0'))) + f'<circle r="5" fill="{c}"></circle><path d="M0 -44 L-6 -34 H6 Z" fill="{c}"></path>'


@_mm
def thali(c='#c9ced3', c2='#e0a64a'):
    return (f'<ellipse rx="44" ry="18" fill="{c}"></ellipse><ellipse rx="36" ry="13" fill="#b9bec4"></ellipse>'
            + ''.join(f'<ellipse cx="{x}" cy="{y}" rx="8" ry="4" fill="{col}"></ellipse>' for x, y, col in [(-22, -4, c2), (-6, -8, '#d8483a'), (12, -6, '#3f7d2c'), (24, 2, '#f6f3ec')]) + '<ellipse cx="-4" cy="6" rx="14" ry="5" fill="#f6efe0"></ellipse>')


@_mm
def mappin(c='#d8483a', c2='#fbf8f1'):
    return f'<path d="M0 36 C-10 18 -26 4 -26 -12 A26 26 0 0 1 26 -12 C26 4 10 18 0 36 Z" fill="{c}"></path><circle cy="-12" r="9" fill="{c2}"></circle>'


@_mm
def parcel(c='#d9b27a', c2='#2f4a33'):
    return f'<path d="M-34 -10 L0 -26 L34 -10 V22 L0 38 L-34 22 Z" fill="{c}"></path><path d="M-34 -10 L0 6 L34 -10 M0 6 V38" stroke="#a8803f" stroke-width="2" fill="none"></path><path d="M-17 -18 L17 -2 V10" stroke="{c2}" stroke-width="5" fill="none"></path>'
