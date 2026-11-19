"""Whole keepsake objects, each with its own moment, built from kit.py. A slip is then one call:

    stub('prithvi', 'A play at Prithvi Theatre', batch='b3', palette='brick', ...)

Every type takes: sid, title, batch, palette, name (what it is, for a screen reader), verbs (open, close),
plus its own words, colours and `art` (a list of motif calls from MOTIFS, placed on the object).
Text is shrunk to fit its object, so nothing runs off the edge.
"""
import math
from kit import (board, paper, words, lines, stamp, hole, string, tag_path, stub_path, arch_path, token, flake, scatter, g, torn, grain,
                 swing, drawon, appear, hide, pop, stampin, slide, fall, rise, spin, flip, shake, glow, FACE)

W = {'serif': .64, 'hand': .5, 'mono': .61, 'marker': .6, 'script': .42, 'kid': .48, 'display': .55, 'mincho': .62, 'ui': .52}


def fit(s, face, size, maxw):
    """The font size at which `s` fits in `maxw` (never bigger than `size`)."""
    est = len(s) * W[face] * size
    return round(size if est <= maxw else max(8, size * maxw / est), 1)


def text(x, y, s, face, size, fill, maxw, anchor='middle', rot=0, cls='', weight=None):
    return words(x, y, s, face, fit(s, face, size, maxw), fill, anchor, rot, cls, weight)


def place(art):
    """art: list of (motif name, x, y, scale, *colours) placed on the object."""
    out = ''
    for a in art or []:
        m, x, y, sc, *cols = a
        out += f'<g transform="translate({x} {y}) scale({sc})">{MOTIFS[m](*cols)}</g>'
    return out


# ── motifs: small drawings, about 80 units across, centred on 0,0 ─────────────────────────────
def _m(fn):
    MOTIFS[fn.__name__.rstrip('_')] = fn
    return fn


MOTIFS = {}


@_m
def sun(c='#f2b33d'):
    return f'<circle r="18" fill="{c}"></circle>' + ''.join(f'<path d="M0 -26 V-36" stroke="{c}" stroke-width="4" stroke-linecap="round" transform="rotate({a})"></path>' for a in range(0, 360, 45))


@_m
def moon(c='#f5edc8'):
    return f'<path d="M10 -24 A26 26 0 1 0 10 24 A20 20 0 1 1 10 -24 Z" fill="{c}"></path>'


@_m
def star(c='#f1c84b'):
    return f'<path d="M0 -22 L6 -7 L22 -7 L9 3 L14 20 L0 10 L-14 20 L-9 3 L-22 -7 L-6 -7 Z" fill="{c}"></path>'


@_m
def stars(c='#fff6d0'):
    return ''.join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="{c}"></circle>' for x, y, r in [(-30, -10, 2.4), (-6, -26, 1.8), (20, -8, 2.8), (34, -28, 1.6), (6, 12, 2), (-24, 20, 1.6)])


@_m
def waves(c='#2f6db5'):
    return ''.join(f'<path d="M-44 {y} q11 -8 22 0 t22 0 t22 0 t22 0" fill="none" stroke="{c}" stroke-width="3" stroke-linecap="round"></path>' for y in (-8, 6, 20))


@_m
def hills(c='#5d7d4f', c2='#3f5d36'):
    return f'<path d="M-50 20 C-30 -20 -10 -24 6 4 C18 -30 36 -32 50 20 Z" fill="{c}"></path><path d="M-50 20 C-24 0 0 6 20 20 Z" fill="{c2}"></path>'


@_m
def ghat(c='#3c3c3c', c2='#5d7d4f'):
    return (f'<path d="M-50 28 C-30 -10 -10 -26 10 -6 C24 -30 40 -34 50 -10 V28 Z" fill="{c2}"></path>'
            f'<path d="M-30 28 C-10 16 20 18 4 6 C-12 -4 26 -8 10 -18 C0 -24 20 -28 30 -30" fill="none" stroke="{c}" stroke-width="4" stroke-linecap="round"></path>'
            f'<path d="M-30 28 C-10 16 20 18 4 6 C-12 -4 26 -8 10 -18 C0 -24 20 -28 30 -30" fill="none" stroke="#fff" stroke-width=".8" stroke-dasharray="3 3"></path>')


@_m
def mist(c='#ffffff'):
    return ''.join(f'<path d="M{x} {y} h{w}" stroke="{c}" stroke-opacity=".7" stroke-width="5" stroke-linecap="round"></path>' for x, y, w in [(-40, -8, 50), (-10, 4, 46), (-34, 16, 40)])


@_m
def skyline(c='#2b3a63'):
    return f'<path d="M-50 24 V2 H-40 V-14 H-30 V4 H-22 V-30 H-12 V-6 H-4 V-20 H8 V-40 L12 -48 L16 -40 V0 H24 V-12 H34 V6 H42 V-8 H50 V24 Z" fill="{c}"></path>'


@_m
def necklace(c='#f6d36b', c2='#1f2b4d'):
    """Marine Drive at night: the curve of lights."""
    return f'<path d="M-50 -6 C-20 22 20 22 50 -6" fill="none" stroke="{c2}" stroke-width="6"></path>' + ''.join(f'<circle cx="{-50 + i * 10}" cy="{-6 + 28 * math.sin(math.pi * i / 10) * .62:.1f}" r="2.6" fill="{c}"></circle>' for i in range(11))


@_m
def boat(c='#c8362a', c2='#f4ecd8'):
    return f'<path d="M-34 6 H34 L24 22 H-24 Z" fill="{c}"></path><path d="M0 4 V-34 L22 0 Z" fill="{c2}"></path><path d="M0 4 V-34" stroke="#3a2a1a" stroke-width="2"></path>'


@_m
def ferry(c='#f4ecd8', c2='#2f6db5'):
    return f'<path d="M-44 8 H44 L36 24 H-36 Z" fill="{c2}"></path><rect x="-30" y="-10" width="56" height="18" rx="3" fill="{c}"></rect><rect x="-12" y="-22" width="22" height="12" rx="2" fill="{c}"></rect><g fill="{c2}"><rect x="-24" y="-6" width="8" height="7"></rect><rect x="-10" y="-6" width="8" height="7"></rect><rect x="4" y="-6" width="8" height="7"></rect></g>'


@_m
def train(c='#c8362a', c2='#f2c230'):
    return f'<rect x="-40" y="-16" width="80" height="30" rx="8" fill="{c}"></rect><rect x="-40" y="2" width="80" height="6" fill="{c2}"></rect><g fill="#bfe1ee"><rect x="-32" y="-10" width="12" height="9" rx="2"></rect><rect x="-14" y="-10" width="12" height="9" rx="2"></rect><rect x="4" y="-10" width="12" height="9" rx="2"></rect><rect x="22" y="-10" width="12" height="9" rx="2"></rect></g><circle cx="-24" cy="16" r="5" fill="#222"></circle><circle cx="24" cy="16" r="5" fill="#222"></circle>'


@_m
def temple(c='#e8b923', c2='#b5452c'):
    return f'<path d="M-30 24 V0 H30 V24 Z" fill="{c2}"></path><path d="M-18 0 C-18 -26 18 -26 18 0 Z" fill="{c}"></path><path d="M0 -26 V-40 M-4 -36 h8" stroke="{c}" stroke-width="3"></path><path d="M-8 24 V10 a8 8 0 0 1 16 0 V24" fill="#3a1a0c"></path>'


@_m
def dome(c='#f1e7d0', c2='#2f6d57'):
    return f'<path d="M-40 26 V4 H40 V26 Z" fill="{c}"></path><path d="M-22 4 C-22 -30 22 -30 22 4 Z" fill="{c2}"></path><path d="M0 -26 V-38" stroke="{c2}" stroke-width="3"></path><g fill="{c2}"><path d="M-30 26 V12 a5 5 0 0 1 10 0 V26 Z"></path><path d="M-5 26 V12 a5 5 0 0 1 10 0 V26 Z"></path><path d="M20 26 V12 a5 5 0 0 1 10 0 V26 Z"></path></g>'


@_m
def fort(c='#a08060', c2='#7a5c40'):
    return f'<path d="M-46 26 V-4 H-38 V-12 H-30 V-4 H-22 V-12 H-14 V-4 H-6 V-12 H2 V-4 H10 V-12 H18 V-4 H26 V-12 H34 V-4 H46 V26 Z" fill="{c}"></path><path d="M-8 26 V10 a8 8 0 0 1 16 0 V26 Z" fill="{c2}"></path>'


@_m
def cave(c='#7a6a58', c2='#2a221a'):
    return f'<path d="M-50 26 C-46 -20 -20 -36 0 -36 C20 -36 46 -20 50 26 Z" fill="{c}"></path><path d="M-20 26 C-20 0 -10 -12 0 -12 C10 -12 20 0 20 26 Z" fill="{c2}"></path><path d="M-36 0 h10 M28 -6 h10 M-30 14 h8" stroke="{c2}" stroke-width="2" opacity=".5"></path>'


@_m
def waterfall(c='#bfe3f2', c2='#5d7d4f'):
    return f'<path d="M-50 26 V-30 C-30 -36 -16 -20 -10 -30 V26 Z M10 26 V-30 C20 -20 34 -36 50 -30 V26 Z" fill="{c2}"></path><path d="M-10 -30 C-4 -34 4 -34 10 -30 V20 H-10 Z" fill="{c}"></path><path d="M-4 -26 V14 M4 -26 V16" stroke="#fff" stroke-width="2"></path><ellipse cy="22" rx="22" ry="6" fill="#fff" opacity=".8"></ellipse>'


@_m
def flamingo(c='#f48aa6', c2='#3a2a2a'):
    return f'<path d="M-4 30 V2 M4 30 L0 2" stroke="{c2}" stroke-width="2"></path><ellipse cx="0" cy="-4" rx="20" ry="12" fill="{c}"></ellipse><path d="M14 -10 C24 -24 20 -40 10 -40 C2 -40 2 -30 10 -30" fill="none" stroke="{c}" stroke-width="5" stroke-linecap="round"></path><path d="M10 -40 l-8 6 l6 2 Z" fill="{c2}"></path>'


@_m
def bird(c='#3a3a3a'):
    return f'<path d="M-20 0 C-12 -10 -4 -10 0 0 C4 -10 12 -10 20 0" fill="none" stroke="{c}" stroke-width="3" stroke-linecap="round"></path>'


@_m
def penguin(c='#1d1d22', c2='#f6f3ec'):
    return f'<ellipse rx="20" ry="28" fill="{c}"></ellipse><ellipse cy="6" rx="13" ry="20" fill="{c2}"></ellipse><circle cx="-6" cy="-14" r="2.4" fill="{c2}"></circle><circle cx="6" cy="-14" r="2.4" fill="{c2}"></circle><path d="M-4 -8 L4 -8 L0 -2 Z" fill="#f2a12e"></path><path d="M-8 28 h6 M2 28 h6" stroke="#f2a12e" stroke-width="3"></path>'


@_m
def fish(c='#e46f2a', c2='#fff'):
    return f'<path d="M-30 0 C-14 -16 14 -16 24 0 C14 16 -14 16 -30 0 Z" fill="{c}"></path><path d="M24 0 L40 -12 V12 Z" fill="{c}"></path><circle cx="-16" cy="-3" r="3" fill="{c2}"></circle>'


@_m
def crab(c='#d8483a'):
    return f'<ellipse rx="24" ry="16" fill="{c}"></ellipse><path d="M-22 -8 C-36 -18 -36 -30 -26 -32 M22 -8 C36 -18 36 -30 26 -32" fill="none" stroke="{c}" stroke-width="5" stroke-linecap="round"></path><path d="M-30 -36 l-6 -6 M-26 -36 l2 -8 M30 -36 l6 -6 M26 -36 l-2 -8" stroke="{c}" stroke-width="4" stroke-linecap="round"></path>' + ''.join(f'<path d="M{s * 18} {6 + i * 6} l{s * 14} {4 + i * 2}" stroke="{c}" stroke-width="3" stroke-linecap="round"></path>' for s in (-1, 1) for i in range(3))


@_m
def leaf(c='#4f8a3a'):
    return f'<path d="M-30 20 C-30 -10 0 -30 30 -24 C24 6 0 24 -30 20 Z" fill="{c}"></path><path d="M-26 18 C-6 4 10 -8 26 -20" fill="none" stroke="#fff" stroke-opacity=".5" stroke-width="2"></path>'


@_m
def flower(c='#e8b923', c2='#c2501a'):
    return ''.join(f'<ellipse cy="-14" rx="8" ry="14" fill="{c}" transform="rotate({a})"></ellipse>' for a in range(0, 360, 45)) + f'<circle r="8" fill="{c2}"></circle>'


@_m
def marigold(c='#f29a1f', c2='#e8701a'):
    return ''.join(f'<circle cx="{14 * math.cos(math.radians(a)):.1f}" cy="{14 * math.sin(math.radians(a)):.1f}" r="10" fill="{c if a % 60 else c2}"></circle>' for a in range(0, 360, 30)) + f'<circle r="11" fill="{c2}"></circle>'


@_m
def cup(c='#f6f3ec', c2='#6b3a1f'):
    return f'<path d="M-20 -14 H20 L16 18 H-16 Z" fill="{c}"></path><ellipse cy="-14" rx="20" ry="5" fill="{c2}"></ellipse><path d="M20 -6 C32 -6 32 10 18 10" fill="none" stroke="{c}" stroke-width="4"></path><ellipse cy="20" rx="26" ry="5" fill="{c}" opacity=".9"></ellipse>'


@_m
def tumbler(c='#c9ced3', c2='#8a5a2e'):
    """South Indian filter coffee: tumbler in its davara."""
    return f'<path d="M-14 -24 H14 L11 10 H-11 Z" fill="{c}"></path><ellipse cy="-24" rx="14" ry="4" fill="{c2}"></ellipse><path d="M-30 10 H30 L24 24 H-24 Z" fill="{c}"></path><ellipse cy="10" rx="30" ry="5" fill="#9aa0a6"></ellipse>'


@_m
def glass(c='#f2b33d', c2='#ffffff'):
    return f'<path d="M-18 -26 H18 L14 22 H-14 Z" fill="{c2}" fill-opacity=".35" stroke="{c2}" stroke-width="2"></path><path d="M-15 -6 H15 L13 20 H-13 Z" fill="{c}"></path>'


@_m
def cocktail(c='#e0245e', c2='#ffffff'):
    return f'<path d="M-24 -24 H24 L0 4 Z" fill="{c}"></path><path d="M-24 -24 H24 L0 4 Z" fill="none" stroke="{c2}" stroke-width="2"></path><path d="M0 4 V24 M-12 26 H12" stroke="{c2}" stroke-width="3"></path><circle cx="16" cy="-26" r="6" fill="#9ccc4a"></circle>'


@_m
def wine(c='#7a1a2e', c2='#ffffff'):
    return f'<path d="M-16 -28 H16 C16 0 6 6 0 6 C-6 6 -16 0 -16 -28 Z" fill="{c2}" fill-opacity=".3" stroke="{c2}" stroke-width="2"></path><path d="M-15 -12 H15 C13 2 6 5 0 5 C-6 5 -13 2 -15 -12 Z" fill="{c}"></path><path d="M0 6 V24 M-10 26 H10" stroke="{c2}" stroke-width="3"></path>'


@_m
def bottle(c='#2f6d57', c2='#f6efe0'):
    return f'<path d="M-6 -36 H6 V-22 C16 -16 16 -8 16 0 V30 H-16 V0 C-16 -8 -16 -16 -6 -22 Z" fill="{c}"></path><rect x="-14" y="2" width="28" height="18" fill="{c2}"></rect>'


@_m
def plate(c='#f6f3ec', c2='#d9a85c'):
    return f'<ellipse rx="40" ry="16" fill="{c}"></ellipse><ellipse rx="28" ry="10" fill="none" stroke="#c9c4b8"></ellipse><ellipse cy="-3" rx="18" ry="7" fill="{c2}"></ellipse>'


@_m
def dosa(c='#e0a64a', c2='#f6f3ec'):
    return f'<ellipse cy="10" rx="44" ry="12" fill="{c2}"></ellipse><path d="M-40 6 C-20 -14 20 -14 40 6 C20 0 -20 0 -40 6 Z" fill="{c}"></path><ellipse cx="22" cy="10" rx="8" ry="4" fill="#f3e6a0"></ellipse>'


@_m
def kebab(c='#8c4a22', c2='#9aa1a4'):
    return f'<path d="M-44 0 H44" stroke="{c2}" stroke-width="3"></path><path d="M-30 0 H30" stroke="{c}" stroke-width="12" stroke-linecap="round"></path><g stroke="#3d1a0a" stroke-width="2" opacity=".6"><path d="M-18 -5 l-4 10 M-4 -5 l-4 10 M10 -5 l-4 10"></path></g>'


@_m
def croissant(c='#d9963a', c2='#b5702a'):
    return f'<path d="M-36 8 C-30 -20 30 -20 36 8 C26 0 -26 0 -36 8 Z" fill="{c}"></path><path d="M-18 -8 L-12 6 M0 -12 V4 M18 -8 L12 6" stroke="{c2}" stroke-width="3" stroke-linecap="round"></path>'


@_m
def cake(c='#f7c9d4', c2='#c2185b'):
    return f'<path d="M-26 20 V-6 H26 V20 Z" fill="{c}"></path><path d="M-26 -6 C-20 2 -12 -4 -6 2 C0 -4 8 2 14 -4 C18 2 22 -2 26 -6" fill="none" stroke="#fff" stroke-width="4"></path><circle cy="-14" r="6" fill="{c2}"></circle>'


@_m
def scoop(c='#f2a3b9', c2='#d9a85c'):
    return f'<path d="M-14 0 L0 34 L14 0 Z" fill="{c2}"></path><circle cy="-6" r="16" fill="{c}"></circle>'


@_m
def pizza(c='#f2c14e', c2='#c0392b'):
    return f'<path d="M0 30 L-26 -24 C-10 -32 10 -32 26 -24 Z" fill="{c}"></path><path d="M-26 -24 C-10 -32 10 -32 26 -24" fill="none" stroke="#c9852a" stroke-width="6"></path><g fill="{c2}"><circle cx="-6" cy="-12" r="5"></circle><circle cx="8" cy="0" r="5"></circle><circle cx="-2" cy="12" r="4"></circle></g>'


@_m
def dumpling(c='#f3ece0', c2='#d8cdb9'):
    return f'<path d="M-30 12 C-30 -16 30 -16 30 12 Z" fill="{c}"></path>' + ''.join(f'<path d="M{x} -12 q2 8 0 16" stroke="{c2}" stroke-width="2" fill="none"></path>' for x in (-14, -6, 2, 10))


@_m
def handi(c='#7a3a1a', c2='#c9a14a'):
    return f'<path d="M-30 -6 C-36 24 36 24 30 -6 Z" fill="{c}"></path><ellipse cy="-6" rx="30" ry="7" fill="{c2}"></ellipse><path d="M-20 -14 C-10 -22 10 -22 20 -14" fill="none" stroke="{c2}" stroke-width="4"></path>'


@_m
def chili(c='#c0392b'):
    return f'<path d="M-24 -10 C-10 0 14 4 26 22 C6 22 -18 8 -24 -10 Z" fill="{c}"></path><path d="M-24 -10 l-6 -8" stroke="#3f7d2c" stroke-width="4" stroke-linecap="round"></path>'


@_m
def mic(c='#a9adb4', c2='#1f2026'):
    return f'<rect x="-6" y="-2" width="12" height="38" rx="5" fill="{c2}"></rect><circle cy="-14" r="14" fill="{c}"></circle>'


@_m
def note(c='#ffffff'):
    return f'<path d="M-6 14 V-20 L18 -26 V8" fill="none" stroke="{c}" stroke-width="3"></path><ellipse cx="-11" cy="14" rx="7" ry="5" fill="{c}"></ellipse><ellipse cx="13" cy="8" rx="7" ry="5" fill="{c}"></ellipse>'


@_m
def guitar(c='#c47a3a', c2='#3a2414'):
    return f'<path d="M-6 -40 H6 V-6 H-6 Z" fill="{c2}"></path><circle cy="6" r="18" fill="{c}"></circle><circle cy="-8" r="13" fill="{c}"></circle><circle cy="2" r="5" fill="{c2}"></circle>'


@_m
def heart(c='#e0245e'):
    return f'<path d="M0 22 C-30 0 -26 -24 -10 -24 C-4 -24 0 -18 0 -14 C0 -18 4 -24 10 -24 C26 -24 30 0 0 22 Z" fill="{c}"></path>'


@_m
def camera(c='#2b2d33', c2='#9aa0a6'):
    return f'<rect x="-30" y="-16" width="60" height="38" rx="6" fill="{c}"></rect><rect x="-14" y="-24" width="20" height="10" rx="2" fill="{c}"></rect><circle cy="3" r="12" fill="{c2}"></circle><circle cy="3" r="6" fill="{c}"></circle>'


@_m
def book(c='#2f6db5', c2='#f6efe0'):
    return f'<path d="M0 -20 C-14 -26 -30 -26 -40 -20 V22 C-30 16 -14 16 0 22 Z" fill="{c2}" stroke="{c}" stroke-width="3"></path><path d="M0 -20 C14 -26 30 -26 40 -20 V22 C30 16 14 16 0 22 Z" fill="{c2}" stroke="{c}" stroke-width="3"></path>'


@_m
def phone(c='#1f2026', c2='#9fd3f2'):
    return f'<rect x="-16" y="-30" width="32" height="60" rx="6" fill="{c}"></rect><rect x="-12" y="-24" width="24" height="44" rx="2" fill="{c2}"></rect>'


@_m
def screens(c='#1f2026', c2='#9fd3f2'):
    """Two phones on a video call, side by side."""
    return f'<g transform="translate(-20 0)">{phone(c, c2)}</g><g transform="translate(20 0)">{phone(c, "#f7c9d4")}</g><path d="M-8 0 h16" stroke="#fff" stroke-width="2" stroke-dasharray="2 3"></path>'


@_m
def pawn(c='#2b2b30'):
    return f'<circle cy="-18" r="10" fill="{c}"></circle><path d="M-8 -8 H8 L12 18 H-12 Z" fill="{c}"></path><rect x="-18" y="16" width="36" height="8" rx="3" fill="{c}"></rect>'


@_m
def dice(c='#f6f3ec', c2='#2b2b30'):
    return f'<rect x="-20" y="-20" width="40" height="40" rx="8" fill="{c}" stroke="{c2}" stroke-width="2"></rect>' + ''.join(f'<circle cx="{x}" cy="{y}" r="4" fill="{c2}"></circle>' for x, y in [(-9, -9), (9, 9), (0, 0)])


@_m
def puzzle(c='#2f6db5'):
    return f'<path d="M-24 -24 H-6 a8 8 0 1 1 12 0 H24 V-6 a8 8 0 1 0 0 12 V24 H6 a8 8 0 1 0 -12 0 H-24 Z" fill="{c}"></path>'


@_m
def kite(c='#e0245e', c2='#f2c230'):
    return f'<path d="M0 -34 L24 0 L0 34 L-24 0 Z" fill="{c}"></path><path d="M0 -34 V34 M-24 0 H24" stroke="{c2}" stroke-width="2"></path><path d="M0 34 C10 44 -6 52 6 62" fill="none" stroke="{c2}" stroke-width="2"></path>'


@_m
def crane(c='#e46f2a'):
    return f'<path d="M-36 6 L0 -6 L36 6 L6 2 L0 22 L-6 2 Z" fill="{c}"></path><path d="M0 -6 L-10 -30 L-4 -10 Z" fill="{c}" opacity=".8"></path>'


@_m
def candle(c='#f6efe0', c2='#ffb347'):
    return f'<rect x="-8" y="-6" width="16" height="36" rx="2" fill="{c}"></rect><path d="M0 -8 C-7 -18 -2 -26 0 -32 C4 -24 7 -18 0 -8 Z" fill="{c2}"></path>'


@_m
def lantern(c='#f2c230', c2='#3a2a1a'):
    return f'<path d="M-12 -24 H12 L16 20 H-16 Z" fill="{c}" fill-opacity=".9"></path><path d="M-14 -24 H14 M-18 20 H18 M0 -24 V-34" stroke="{c2}" stroke-width="3"></path><path d="M-8 -30 a8 6 0 0 1 16 0" fill="none" stroke="{c2}" stroke-width="2"></path>'


@_m
def lights(c='#ffe08a', c2='#3a3a3a'):
    return f'<path d="M-50 -10 C-25 10 25 10 50 -10" fill="none" stroke="{c2}" stroke-width="1.5"></path>' + ''.join(f'<circle cx="{x}" cy="{-10 + 20 * (1 - (x / 50) ** 2) * .5:.1f}" r="4" fill="{c}"></circle>' for x in range(-40, 50, 16))


@_m
def plant(c='#4f8a3a', c2='#b5683a'):
    return f'<path d="M-16 6 H16 L12 30 H-12 Z" fill="{c2}"></path><path d="M0 6 C0 -10 -14 -18 -22 -20 C-18 -8 -10 0 0 6 C0 -14 10 -24 22 -28 C20 -12 10 0 0 6" fill="{c}"></path>'


@_m
def cricket(c='#c8a46e', c2='#c0392b'):
    return f'<path d="M-6 -40 H6 V-14 L10 30 H-10 L-6 -14 Z" fill="{c}"></path><circle cx="26" cy="20" r="9" fill="{c2}"></circle><path d="M18 20 a8 8 0 0 0 16 0" fill="none" stroke="#fff" stroke-width="1.4" stroke-dasharray="2 2"></path>'


@_m
def scooter(c='#2aa3a8', c2='#222'):
    return f'<path d="M-30 10 C-30 -6 -10 -8 0 -8 H16 L24 -30 H34" fill="none" stroke="{c}" stroke-width="7" stroke-linecap="round"></path><path d="M-36 10 H20 C26 10 28 4 26 0 H-20 C-30 0 -36 4 -36 10 Z" fill="{c}"></path><circle cx="-26" cy="16" r="9" fill="{c2}"></circle><circle cx="26" cy="16" r="9" fill="{c2}"></circle>'


@_m
def bicycle(c='#c8362a', c2='#222'):
    return f'<circle cx="-24" cy="10" r="16" fill="none" stroke="{c2}" stroke-width="3"></circle><circle cx="24" cy="10" r="16" fill="none" stroke="{c2}" stroke-width="3"></circle><path d="M-24 10 L-6 -14 H16 L24 10 M-6 -14 L2 10 L16 -14 M-10 -20 H0 M16 -14 L12 -26 H20" fill="none" stroke="{c}" stroke-width="3.4" stroke-linecap="round" stroke-linejoin="round"></path>'


@_m
def strawberry(c='#d6334a', c2='#3f7d2c'):
    return f'<path d="M0 26 C-24 10 -24 -14 -10 -16 C-4 -18 4 -18 10 -16 C24 -14 24 10 0 26 Z" fill="{c}"></path><path d="M-12 -16 L0 -26 L12 -16 L4 -14 L0 -20 L-4 -14 Z" fill="{c2}"></path>' + ''.join(f'<circle cx="{x}" cy="{y}" r="1.4" fill="#f6e08a"></circle>' for x, y in [(-8, -4), (6, -2), (-2, 8), (10, 8), (-10, 10), (2, 16)])


@_m
def mango(c='#f2b33d', c2='#e46f2a'):
    return f'<path d="M-24 6 C-28 -20 10 -34 26 -12 C36 4 20 30 -2 28 C-16 26 -22 18 -24 6 Z" fill="{c}"></path><path d="M10 -26 C14 -34 20 -36 24 -36" stroke="#3f7d2c" stroke-width="3" fill="none"></path><path d="M-10 18 C0 24 14 20 20 8" fill="none" stroke="{c2}" stroke-width="4" opacity=".6"></path>'


@_m
def coffeebean(c='#5a3216'):
    return f'<ellipse rx="14" ry="20" fill="{c}" transform="rotate(20)"></ellipse><path d="M-4 -18 C6 -6 -6 6 4 18" fill="none" stroke="#2a1608" stroke-width="2.4" transform="rotate(20)"></path>'


@_m
def firefly(c='#f6f08a'):
    return ''.join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="{c}" opacity=".9"></circle><circle cx="{x}" cy="{y}" r="{r * 3}" fill="{c}" opacity=".18"></circle>' for x, y, r in [(-24, -10, 2.4), (6, -24, 2), (20, 8, 2.6), (-6, 18, 1.8)])


@_m
def drop(c='#5aa4d6'):
    return f'<path d="M0 -24 C-14 -4 -16 6 -16 12 a16 16 0 0 0 32 0 C16 6 14 -4 0 -24 Z" fill="{c}"></path>'


@_m
def spray(c='#e0245e', c2='#2b2b30'):
    return f'<rect x="-12" y="-14" width="24" height="46" rx="5" fill="{c}"></rect><rect x="-6" y="-24" width="12" height="10" rx="2" fill="{c2}"></rect>' + ''.join(f'<circle cx="{14 + i * 5}" cy="{-26 - (i % 2) * 4}" r="2" fill="{c}"></circle>' for i in range(4))


@_m
def lotus(c='#f29ab8', c2='#c2507a'):
    return f'<path d="M0 14 C-8 0 -8 -16 0 -26 C8 -16 8 0 0 14 Z" fill="{c}"></path><path d="M0 14 C-16 6 -26 -6 -26 -16 C-12 -12 -4 -2 0 14 Z M0 14 C16 6 26 -6 26 -16 C12 -12 4 -2 0 14 Z" fill="{c2}"></path>'


@_m
def pagoda(c='#e8b923', c2='#b88a1a'):
    return f'<path d="M-44 24 H44 L36 14 H-36 Z" fill="{c2}"></path><path d="M-32 14 C-32 -30 32 -30 32 14 Z" fill="{c}"></path><path d="M0 -24 L-4 -40 L0 -50 L4 -40 Z" fill="{c}"></path>'


@_m
def anchor(c='#2b3a63'):
    return f'<circle cy="-26" r="6" fill="none" stroke="{c}" stroke-width="3"></circle><path d="M0 -20 V26 M-14 -10 H14 M-26 8 C-24 24 24 24 26 8" fill="none" stroke="{c}" stroke-width="4" stroke-linecap="round"></path>'


@_m
def clock(c='#f6f3ec', c2='#2b2b30'):
    return f'<circle r="26" fill="{c}" stroke="{c2}" stroke-width="3"></circle><path d="M0 0 V-16 M0 0 L10 6" stroke="{c2}" stroke-width="3" stroke-linecap="round"></path>'


@_m
def envelope(c='#f6efe0', c2='#c0392b'):
    return f'<rect x="-30" y="-18" width="60" height="38" rx="3" fill="{c}"></rect><path d="M-30 -18 L0 4 L30 -18" fill="none" stroke="#c9b48a" stroke-width="2"></path><circle cy="4" r="6" fill="{c2}"></circle>'


@_m
def globe(c='#5aa4d6', c2='#4f8a3a'):
    return f'<circle r="26" fill="{c}"></circle><path d="M-14 -16 C-4 -20 6 -10 0 -2 C-8 4 -18 -4 -14 -16 Z M6 6 C16 2 22 10 14 20 C8 22 2 14 6 6 Z" fill="{c2}"></path><ellipse rx="26" ry="10" fill="none" stroke="#fff" stroke-opacity=".5"></ellipse>'


@_m
def house(c='#f6efe0', c2='#c0392b'):
    return f'<path d="M-26 24 V-4 L0 -26 L26 -4 V24 Z" fill="{c}"></path><path d="M-32 0 L0 -30 L32 0" fill="none" stroke="{c2}" stroke-width="5" stroke-linecap="round"></path><rect x="-6" y="6" width="12" height="18" fill="{c2}"></rect>'


@_m
def tomato(c='#d8483a', c2='#3f7d2c'):
    return f'<circle r="24" fill="{c}"></circle><path d="M-10 -22 L0 -16 L10 -22 L4 -14 Z" fill="{c2}"></path>'


@_m
def bell(c='#d9a84a'):
    return f'<path d="M-22 16 C-22 -24 22 -24 22 16 Z" fill="{c}"></path><rect x="-26" y="14" width="52" height="6" rx="3" fill="{c}"></rect><circle cy="24" r="5" fill="{c}"></circle>'


@_m
def spoon(c='#c9ced3'):
    return f'<ellipse cy="-20" rx="10" ry="14" fill="{c}"></ellipse><path d="M0 -6 V34" stroke="{c}" stroke-width="5" stroke-linecap="round"></path>'


@_m
def steam(c='#ffffff'):
    return ''.join(f'<path class="steam" style="--d:{i * .5}s" d="M{x} 20 c-8 -10 8 -16 0 -28 c-6 -8 6 -12 0 -20" fill="none" stroke="{c}" stroke-width="3" stroke-linecap="round"></path>' for i, x in enumerate((-12, 0, 12)))


# ── the objects ─────────────────────────────────────────────────────────────────────────────────
STEAM = rise('.steam', 30, 1.8, 3)


def stub(sid, title, *, batch, palette, name, verbs=('Tear the stub', 'Keep it whole'), head, sub='', rows=(), stub_text='ADMIT 2',
         paper_='#fbf8f1', band='#a84a3a', ink='#24303a', art=(), note='', shape='plain', key=None, visit=1):
    """A ticket with a tear-off stub; touching it tears the stub away and a punch goes through."""
    w, h = 400, 210
    d = stub_path(w, h, 12, 300) if shape == 'plain' else (f'M18 0 H{w - 18} A18 18 0 0 0 {w} 18 V{h - 18} A18 18 0 0 0 {w - 18} {h} H18 A18 18 0 0 0 0 {h - 18} V18 A18 18 0 0 0 18 0 Z' if shape == 'scallop' else torn([(0, 0), (w, 0), (w, h), (0, h)], seed=len(head), amp=2.2, step=8))
    body = (paper(d, paper_, edge='#00000018') + f'<path d="M0 0 H288 V44 H0 Z" fill="{band}"></path>'
            + text(150, 30, sub or ' ', 'mono', 12, '#fff', 260, weight=700)
            + text(150, 92, head, 'display', 30, ink, 260, weight=700)
            + ''.join(text(20, 126 + i * 22, r, 'mono', 12, ink, 260, 'start') for i, r in enumerate(rows[:3]))
            + place(art) + '<path d="M300 14 V196" stroke="#5a4a3a" stroke-opacity=".45" stroke-width="2.5" stroke-dasharray="3 5"></path>')
    tear = (f'<g class="tear"><path d="M312 0 H{w} V{h} H312 A12 12 0 0 0 312 {h} Z" fill="{paper_}"></path><path d="M312 0 H{w} V44 H312 Z" fill="{band}"></path>'
            f'<g transform="translate(356 120) rotate(-90)">{text(0, 6, stub_text, "mono", 16, ink, 150, weight=700)}</g>'
            f'<path class="punch" d="M356 54 l3 7 l8 1 l-6 5 l2 8 l-7 -4 l-7 4 l2 -8 l-6 -5 l8 -1 Z" fill="#1d1a16"></path></g>')
    extra = text(200, 268, note, 'hand', 22, '#fff8ea', 380, rot=-3, cls='note') if note else ''
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body + tear + extra, cls='tk', at=(100, 190), rot=-5),
          motion=[slide('.tear', 46, 22, 9, .2), appear('.punch', .1), swing('.tk', '0% 50%', 3, .2), appear('.note', .7)])


def receipt(sid, title, *, batch, palette, name, verbs=('Read the bill', 'Fold it'), head, sub='', rows=(), total=('', ''), note='', art=(),
            ink='#2b2b30', pen='#2a54a6', key=None, visit=1):
    """A thermal receipt, torn off; touching it circles what mattered and the doodle appears."""
    w = 250
    h = 150 + 24 * len(rows) + (60 if note else 20)
    d = torn([(0, 0), (w, 0), (w, h), (0, h)], seed=len(head), amp=3, step=10)
    body = paper(d, '#fbfaf5', grain_id='r-g') + text(w / 2, 44, head, 'serif', 28, ink, w - 30) + text(w / 2, 66, sub, 'mono', 10, ink, w - 24)
    y = 98
    body += f'<path d="M14 {y - 12} H{w - 14}" stroke="#8a8a8a" stroke-dasharray="3 3"></path>'
    for left, right in rows:
        body += text(16, y, left, 'mono', 11.5, ink, 160, 'start') + text(w - 16, y, right, 'mono', 11.5, ink, 70, 'end')
        y += 24
    if total[0]:
        body += f'<path d="M14 {y - 12} H{w - 14}" stroke="#8a8a8a" stroke-dasharray="3 3"></path>' + text(16, y + 8, total[0], 'mono', 13, ink, 150, 'start', weight=700) + text(w - 16, y + 8, total[1], 'mono', 13, ink, 80, 'end', weight=700)
        body += f'<path class="circ" pathLength="1" d="M{w - 92} {y + 12} C{w - 96} {y - 14} {w - 4} {y - 14} {w - 6} {y + 6} C{w - 8} {y + 24} {w - 90} {y + 26} {w - 96} {y + 4}" fill="none" stroke="{pen}" stroke-width="2.2"></path>'
        y += 40
    if note:
        body += text(w / 2, y + 6, note, 'hand', 20, pen, w - 20, rot=-3)
    doodle = f'<g class="doodle">{place(art)}</g>' if art else ''
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, defs=grain('r-g', .2, .18, .12, .1),
          svg=g(body, at=(300 - w / 2 - 40, max(40, 300 - h / 2)), rot=-3) + doodle,
          motion=[drawon('.circ', .2, .7), pop('.doodle', .5)] + ([STEAM] if any(a[0] == 'steam' for a in art) else []))


def tag(sid, title, *, batch, palette, name, verbs=('Give it a swing', 'Let it hang'), head, rows=(), stamp_word='', stamp_sub='',
        paper_='#efe2c6', ink='#3b2618', ink2='#b5452c', art=(), w=320, h=170, key=None, visit=1):
    """A luggage tag on a string: it swings, and a stamp lands."""
    d = tag_path(w, h)
    body = (paper(d, paper_, grain_id='t-g', edge='#00000022') + hole(w - 30, h / 2, 9) + string(w - 30, h / 2, w + 80, h / 2 - 90, 20, '#c9b48a')
            + text(26, 54, head, 'serif', 28, ink, w - 110, 'start') + ''.join(text(26, 86 + i * 24, r, 'hand', 18, ink, w - 90, 'start') for i, r in enumerate(rows[:3]))
            + place(art) + (g(stamp(stamp_word, stamp_sub, ink2, 30), cls='stp', at=(w - 110, h - 46), rot=-10) if stamp_word else ''))
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, defs=grain('t-g', .25, .16, .08, .14),
          svg=g(body, cls='tg', at=(90, 300 - h / 2), rot=-6), motion=[swing('.tg', '100% 50%', 7), stampin('.stp', .6)])


def coin(sid, title, *, batch, palette, name, verbs=('Flip it', 'Pocket it'), top, big, bottom='', back_big='', back_small='',
         face='#e9c46a', rim='#9c7a2a', ink='#5a3a10', art=(), key=None, visit=1):
    """A token or coin: touching it tosses it and it lands on its other face."""
    fa = text(0, -40, top, 'mono', 13, ink, 150) + text(0, 16, big, 'display', 40, ink, 160, weight=700) + text(0, 46, bottom, 'mono', 11, ink, 150) + place(art)
    fb = text(0, 8, back_big or big, 'display', 34, ink, 160, weight=700) + text(0, 36, back_small, 'hand', 18, ink, 160)
    body = f'<g class="cf">{token(110, face, rim)}<g class="fa">{fa}</g><g class="fb">{fb}</g></g>'
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=f'<ellipse cx="300" cy="440" rx="100" ry="14" fill="#000" opacity=".22"></ellipse><g class="toss">{g(body, at=(300, 300))}</g>',
          motion=[('.cf{transform-box:fill-box;transform-origin:50% 50%}.fb{opacity:0}.open .cf{animation:cfs .275s linear 5}'
                  '.open .fa{animation:cfa 1.375s step-end both}.open .fb{animation:cfb 1.375s step-end both}'
                  '@keyframes cfs{0%{transform:scaleY(1)}25%{transform:scaleY(0)}50%{transform:scaleY(-1)}75%{transform:scaleY(0)}100%{transform:scaleY(1)}}'
                  '@keyframes cfa{0%{opacity:1}5%{opacity:0}15%{opacity:1}25%{opacity:0}35%{opacity:1}45%{opacity:0}55%{opacity:1}65%{opacity:0}75%{opacity:1}85%,100%{opacity:0}}'
                  '@keyframes cfb{0%{opacity:0}5%{opacity:1}15%{opacity:0}25%{opacity:1}35%{opacity:0}45%{opacity:1}55%{opacity:0}65%{opacity:1}75%{opacity:0}85%,100%{opacity:1}}', '.cf,.fa,.fb'),
                 (f'.open .toss{{animation:tossup 1.375s cubic-bezier(.3,0,.7,1)}}@keyframes tossup{{50%{{transform:translateY(-150px)}}}}', '.toss')])


def postcard(sid, title, *, batch, palette, name, verbs=('Turn it over', 'Turn it back'), scene, sky=('#f6c27a', '#f29a6b'), ground='#3f5d7d',
             caption='', message=(), to='', stamp_art=('star',), key=None, visit=1):
    """A postcard: a little scene on the front; touching it flips it to our message on the back.
    scene: motif placements on a 360 × 230 card (sky above, ground below y=160)."""
    w, h = 360, 230
    clip = f'<clipPath id="pc-c"><rect width="{w}" height="{h}" rx="6"></rect></clipPath>'
    front = (f'<g clip-path="url(#pc-c)"><rect width="{w}" height="{h}" fill="url(#pc-sky)"></rect><path d="M0 160 C90 150 270 170 {w} 156 V{h} H0 Z" fill="{ground}"></path>{place(scene)}</g>'
             f'<rect x="8" y="8" width="{w - 16}" height="{h - 16}" rx="4" fill="none" stroke="#fff" stroke-width="6"></rect>'
             + (f'<path d="M14 {h - 52} H{min(w - 20, 40 + len(caption) * 13)} L{min(w - 12, 48 + len(caption) * 13)} {h - 34} L{min(w - 20, 40 + len(caption) * 13)} {h - 16} H14 Z" fill="#fbf6ea"></path>' + text(24, h - 26, caption, 'script', 28, '#2a3f7a', w - 60, 'start', weight=700) if caption else ''))
    back = (f'<rect width="{w}" height="{h}" rx="6" fill="#fbf6ea"></rect><path d="M{w / 2 + 10} 24 V{h - 24}" stroke="#c9b48a"></path>'
            + ''.join(text(20, 60 + i * 30, m, 'hand', 20, '#2a3f7a', w / 2 - 30, 'start') for i, m in enumerate(message[:5]))
            + f'<rect x="{w - 70}" y="18" width="52" height="62" fill="#f2e3b3" stroke="#c0392b" stroke-dasharray="3 2"></rect>'
            + f'<g transform="translate({w - 44} 49) scale(.5)">{MOTIFS[stamp_art[0]](*stamp_art[1:])}</g>'
            + ''.join(f'<path d="M{w / 2 + 24} {120 + i * 30} H{w - 24}" stroke="#c9b48a"></path>' for i in range(3))
            + text(w / 2 + 28, 114, to, 'hand', 18, '#2a3f7a', w / 2 - 50, 'start'))
    frank = (f'<g transform="translate({w - 70} 40) rotate(-12)"><g class="frank"><circle r="26" fill="none" stroke="#2a2a2a" stroke-width="2.4"></circle>'
             + ''.join(f'<path d="M30 {-12 + k * 8} q8 -5 16 0 t16 0 t16 0" fill="none" stroke="#2a2a2a" stroke-width="2"></path>' for k in range(4)) + '</g></g>')
    body = f'<g class="pf">{front}{frank}</g><g class="pb" transform="translate({w} 0) scale(-1 1)">{back}</g>'
    sky_grad = f'<linearGradient id="pc-sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{sky[0]}"></stop><stop offset="1" stop-color="{sky[1]}"></stop></linearGradient>'
    css = ('.card{transform-box:fill-box;transform-origin:50% 50%;transition:transform .7s cubic-bezier(.4,0,.2,1)}.pb{opacity:0;transition:opacity 0s .35s}.pf{transition:opacity 0s .35s}'
           '.open .card{transform:scaleX(-1) rotate(-3deg);transition:transform .7s cubic-bezier(.4,0,.2,1) .7s}.open .pb{opacity:1;transition:opacity 0s 1.05s}.open .pf{opacity:0;transition:opacity 0s 1.05s}'
           '.frank{opacity:0;transform-box:fill-box;transform-origin:50% 50%}.open .frank{animation:frk .25s ease-out .15s both}@keyframes frk{0%{opacity:0;transform:scale(1.7)}100%{opacity:.85;transform:none}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, defs=clip + sky_grad, css=css,
          svg=g(f'<g class="card">{body}</g>', at=(120, 186), rot=-4),
          motion=[('', '.card,.pb,.pf,.frank')])


def scorecard(sid, title, *, batch, palette, name, verbs=('Fill it in', 'Rub it out'), head, sub='', rows=(), cols=('',), marks=(), circle=None,
              paper_='#fbf8ef', ink='#2b2b30', inks=('#1f4bb5', '#c0392b'), art=(), shape='clip', key=None, visit=1):
    """A card of rows (a scorecard, a checklist, a passport page); touching it fills the marks in.
    marks: per row, a tuple of strings for each column ('✓' draws a tick). circle: index of the row to ring."""
    w = 330
    h = 120 + 40 * len(rows) + 30
    body = ''
    if shape == 'clip':
        body += f'<rect x="-18" y="-24" width="{w + 36}" height="{h + 44}" rx="14" fill="#b07c45"></rect><g transform="translate({w / 2 - 50} -38)"><rect width="100" height="40" rx="8" fill="#c9ccd1"></rect></g>'
    card = f'M12 0 H{w - 34} L{w} 34 V{h - 12} Q{w} {h} {w - 12} {h} H12 Q0 {h} 0 {h - 12} V12 Q0 0 12 0 Z' if shape != 'clip' else f'M0 0 H{w} V{h} H0 Z'
    body += paper(card, paper_) + ('' if shape == 'clip' else hole(20, h / 2 - 30, 6, '#00000055') + hole(20, h / 2 + 30, 6, '#00000055'))
    body += text(20, 46, head, 'serif', 24, ink, w - 40, 'start') + text(20, 70, sub, 'mono', 10, '#6b6658', w - 40, 'start')
    for j, c in enumerate(cols):
        body += text(w - 40 - (len(cols) - 1 - j) * 50, 96, c, 'mono', 10, inks[j % 2], 46)
    for i, r in enumerate(rows):
        y = 130 + i * 40
        body += text(34 if shape != 'clip' else 20, y, r, 'hand', 18, ink, w - 74 - 50 * len(cols), 'start') + f'<path d="M16 {y + 13} H{w - 16}" stroke="#c9c2b0"></path>'
        for j, m in enumerate(marks[i] if i < len(marks) else ()):
            x = w - 40 - (len(cols) - 1 - j) * 50
            col = inks[j % 2]
            if m == '✓':
                body += f'<path class="mk" style="--d:{.15 + i * .2 + j * .1:.2f}s" pathLength="1" d="M{x - 9} {y - 6} l6 7 l12 -16" fill="none" stroke="{col}" stroke-width="3" stroke-linecap="round"></path>'
            else:
                body += text(x, y, m, 'hand', 20, col, 46, cls='mw', weight=700).replace('<text', f'<text style="--d:{.15 + i * .2 + j * .1:.2f}s"', 1)
    if circle is not None:
        y = 130 + circle * 40
        body += f'<path class="ring" pathLength="1" d="M12 {y + 4} C40 {y - 22} {w - 30} {y - 24} {w - 8} {y - 2} C{w} {y + 20} 40 {y + 22} 10 {y + 6}" fill="none" stroke="#2c7a3f" stroke-width="2.4"></path>'
    body += place(art)
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, at=(300 - w / 2 - 20, max(60, 300 - h / 2)), rot=-3),
          motion=[drawon('.mk', 0, .3), appear('.mw', 0, .15), drawon('.ring', .3 + .2 * len(rows), .6)])


def label(sid, title, *, batch, palette, name, verbs=('Open it', 'Close it'), kind='bottle', head, rows=(), glass_='#2f6d57', paper_='#f6efe0',
          ink='#3a2a20', fx='bubbles', art=(), key=None, visit=1):
    """A bottle, jar, packet or bag with its label; touching it lets out what's inside (bubbles, steam, sparkle, petals)."""
    shapes = {
        'bottle': ('M-30 -170 H30 V-120 C70 -100 80 -70 80 -30 V170 Q80 190 60 190 H-60 Q-80 190 -80 170 V-30 C-80 -70 -70 -100 -30 -120 Z', (-62, -10, 124, 150)),
        'jar': ('M-70 -120 H70 V-96 C96 -90 104 -70 104 -40 V160 Q104 186 78 186 H-78 Q-104 186 -104 160 V-40 C-104 -70 -96 -90 -70 -96 Z', (-84, -40, 168, 160)),
        'packet': ('M-100 -170 L-80 -150 L-60 -170 L-40 -150 L-20 -170 L0 -150 L20 -170 L40 -150 L60 -170 L80 -150 L100 -170 V170 H-100 Z', (-84, -110, 168, 230)),
        'bag': ('M-110 -150 H110 L120 180 H-120 Z', (-86, -90, 172, 200)),
        'box': ('M-130 -100 L-100 -140 H100 L130 -100 V150 H-130 Z', (-110, -86, 220, 200)),
    }
    d, (lx, ly, lw, lh) = shapes[kind]
    body = f'<path d="{d}" fill="{glass_}"></path>'
    if kind in ('bottle', 'jar'):
        body += f'<path d="{d}" fill="url(#lb-shine)"></path><rect x="{-36 if kind == "bottle" else -78}" y="{-196 if kind == "bottle" else -150}" width="{72 if kind == "bottle" else 156}" height="{30 if kind == "bottle" else 34}" rx="6" fill="#3a2a20"></rect>'
    body += f'<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" rx="6" fill="{paper_}"></rect><rect x="{lx + 6}" y="{ly + 6}" width="{lw - 12}" height="{lh - 12}" rx="4" fill="none" stroke="{ink}" stroke-opacity=".4"></rect>'
    body += text(0, ly + 40, head, 'serif', 24, ink, lw - 24) + ''.join(text(0, ly + 70 + i * 22, r, 'hand', 16, ink, lw - 24) for i, r in enumerate(rows[:4]))
    body += place(art)
    fxsvg, fxmotion = {
        'bubbles': (scatter(14, (-60, -330, 60, -180), (3, 7), '#ffffff', 'fx'), rise('.fx', 120, 1.6, 2)),
        'steam': (''.join(f'<path class="fx" style="--d:{i * .4}s" d="M{x} -200 c-12 -16 12 -24 0 -42 c-8 -12 8 -18 0 -30" fill="none" stroke="#fff" stroke-width="4" stroke-linecap="round"></path>' for i, x in enumerate((-30, 0, 30))), rise('.fx', 60, 1.8, 3)),
        'sparkle': (''.join(f'<g transform="translate({x} {y})"><g class="fx" style="--d:{i * .2:.1f}s">{MOTIFS["star"]("#fff3b0")}</g></g>' for i, (x, y) in enumerate([(-110, -150), (120, -120), (-130, 40), (130, 70), (0, -240)])), pop('.fx', .2)),
        'petals': (scatter(16, (-160, -320, 160, -200), (4, 7), '#f29ab8', 'fx'), fall('.fx', 480, 2.4, 1)),
    }[fx]
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          defs='<linearGradient id="lb-shine" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff" stop-opacity=".35"></stop><stop offset=".25" stop-color="#fff" stop-opacity="0"></stop><stop offset=".85" stop-color="#000" stop-opacity="0"></stop><stop offset="1" stop-color="#000" stop-opacity=".2"></stop></linearGradient>',
          svg=g(body + fxsvg, cls='lb', at=(290, 330), rot=-4), motion=[fxmotion, shake('.lb', 4)])


def envelope_(sid, title, *, batch, palette, name, verbs=('Open it', 'Seal it'), front='', letter=(), seal='#a8321f', paper_='#f6efe0', ink='#2a3f7a',
              art=(), key=None, visit=1):
    """An envelope with a wax seal: touching it lifts the flap and the letter slides up."""
    w, h = 360, 230
    letter_svg = (f'<g class="lt"><rect x="20" y="10" width="{w - 40}" height="{h + 40}" rx="4" fill="#fffdf6"></rect>'
                  + ''.join(text(40, 50 + i * 30, l, 'hand', 20, ink, w - 80, 'start') for i, l in enumerate(letter[:6])) + '</g>')
    body = (f'<clipPath id="ev-c"><rect x="-10" y="-400" width="{w + 20}" height="{h + 400}"></rect></clipPath>'
            f'<rect width="{w}" height="{h}" rx="12" fill="{paper_}" filter="url(#e-sh)"></rect>'
            f'<g clip-path="url(#ev-c)">{letter_svg}</g>'
            f'<path d="M0 {h} L{w / 2} 110 L{w} {h} Z" fill="{paper_}" stroke="#d8c8a6"></path><path d="M0 0 L{w / 2} 130 L0 {h} Z M{w} 0 L{w / 2} 130 L{w} {h} Z" fill="{paper_}" stroke="#d8c8a6"></path>'
            f'<g class="flap"><path d="M0 0 H{w} L{w / 2} 140 Z" fill="#efe4cc" stroke="#d8c8a6"></path></g>'
            f'<g class="seal"><circle cx="{w / 2}" cy="136" r="24" fill="{seal}"></circle><circle cx="{w / 2}" cy="136" r="17" fill="none" stroke="#fff" stroke-opacity=".35" stroke-width="2"></circle>{place(art)}</g>'
            + text(w / 2, h - 26, front, 'hand', 22, ink, w - 60))
    css = '.flap{transform-box:fill-box;transform-origin:50% 0%;transition:transform .4s .3s,opacity 0s .3s}.open .flap{transform:scaleY(-1);opacity:0;transition:transform .4s,opacity .25s .4s}'
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          defs='<filter id="e-sh" x="-10%" y="-10%" width="120%" height="130%"><feDropShadow dx="0" dy="3" stdDeviation="3" flood-opacity=".2"></feDropShadow></filter>',
          svg=g(body, at=(110, 240), rot=-3), motion=[('', '.flap'), hide('.seal', 0, .15), slide('.lt', 0, -250, 0, .4, .8)])


def badge(sid, title, *, batch, palette, name, verbs=('Pin it on', 'Take it off'), head, sub='', big='', ribbon=('#c0392b', '#2f6db5'), face='#f6efe0',
          ink='#2b2b30', scallop=True, art=(), key=None, visit=1):
    """A rosette, badge or sticker: touching it shines and the ribbons flutter."""
    r = 120
    edge = (''.join(f'<circle cx="{(r + 4) * math.cos(math.radians(a)):.1f}" cy="{(r + 4) * math.sin(math.radians(a)):.1f}" r="14" fill="{ribbon[0]}"></circle>' for a in range(0, 360, 15))
            if scallop else f'<circle r="{r + 10}" fill="{ribbon[0]}"></circle>')
    tails = f'<g class="tails"><path d="M-60 80 L-90 260 L-60 236 L-36 266 L-10 96 Z" fill="{ribbon[1]}"></path><path d="M60 80 L90 260 L60 236 L36 266 L10 96 Z" fill="{ribbon[0]}"></path></g>'
    body = (tails + edge + f'<circle r="{r}" fill="{face}"></circle><circle r="{r - 12}" fill="none" stroke="{ink}" stroke-opacity=".4" stroke-dasharray="3 5"></circle>'
            + text(0, -44, head, 'serif', 26, ink, 200) + text(0, 20, big, 'display', 52, ink, 190, weight=800) + text(0, 56, sub, 'mono', 11, ink, 180) + place(art)
            + f'<path class="shine" d="M-90 -40 L-40 -110 L-20 -110 L-70 -40 Z" fill="#fff" opacity="0"></path>')
    css = '.tails{transform-box:fill-box;transform-origin:50% 0%}.open .tails{animation:flut .3s ease-in-out 5 alternate}@keyframes flut{to{transform:skewX(5deg)}}'
    shine = ('.shine{opacity:0}.open .shine{animation:shn .9s ease-out .2s}@keyframes shn{0%{opacity:0;transform:translateX(0)}30%{opacity:.6}100%{opacity:0;transform:translateX(160px)}}', '.shine')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, cls='bd', at=(300, 250), rot=-6), motion=[('', '.tails'), shine, swing('.bd', '50% 0%', 5)])


def meal(sid, title, *, batch, palette, name, verbs=('Serve it', 'Clear it'), base='plate', items=(), tag_head='', tag_sub='', base_col=('#f6f3ec', '#c9c4b8'),
         key=None, visit=1):
    """Food on a plate, leaf, paper or tray, with a little name card: touching it sends the steam up and the garnish in."""
    bases = {
        'plate': f'<ellipse cx="300" cy="320" rx="220" ry="150" fill="{base_col[0]}"></ellipse><ellipse cx="300" cy="320" rx="170" ry="112" fill="none" stroke="{base_col[1]}" stroke-width="2"></ellipse>',
        'leaf': f'<path d="M60 340 C60 230 160 150 320 140 C440 132 540 190 580 280 C540 280 530 300 560 330 C520 430 420 480 300 488 C160 496 60 440 60 340 Z" fill="{base_col[0]}"></path><path d="M64 340 C220 300 420 290 576 284" stroke="{base_col[1]}" stroke-width="4" fill="none"></path>',
        'paper': f'<path d="M110 180 L470 150 L500 470 L120 500 Z" fill="{base_col[0]}"></path><path d="M110 180 L470 150 L500 470 L120 500 Z" fill="none" stroke="{base_col[1]}" stroke-dasharray="4 6"></path>',
        'tray': f'<rect x="90" y="170" width="420" height="300" rx="26" fill="{base_col[0]}"></rect><rect x="108" y="188" width="384" height="264" rx="18" fill="none" stroke="{base_col[1]}" stroke-width="3"></rect>',
        'thali': f'<circle cx="300" cy="320" r="210" fill="#c9ced3"></circle><circle cx="300" cy="320" r="190" fill="#e6e9ec"></circle>',
    }
    body = bases[base] + place(items)
    card = (f'<g transform="translate(70 470) rotate(-6)"><path d="M0 0 H210 V60 H0 Z" fill="#fbf6ec"></path>'
            + text(105, 28, tag_head, 'serif', 18, '#5a1d0c', 190) + text(105, 46, tag_sub, 'mono', 9, '#4a3a30', 190) + '</g>') if tag_head else ''
    steam = ''.join(f'<path class="sm" style="--d:{i * .45:.2f}s" d="M{x} 250 c-10 -14 10 -22 0 -38 c-8 -12 8 -18 0 -30" fill="none" stroke="#fff" stroke-width="4" stroke-linecap="round"></path>' for i, x in enumerate((240, 300, 360)))
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=body + steam + card, motion=[rise('.sm', 60, 1.8, 3), pop('.gn', .3)])


def strip(sid, title, *, batch, palette, name, verbs=('Develop it', 'Fade it'), frames=(), caption='', film=False, key=None, visit=1):
    """A photo-booth strip or film frames: touching it develops each frame in turn.
    frames: list of (sky, ground, motif list) per frame."""
    n = len(frames)
    fw, fh = (150, 120) if film else (150, 112)
    body = ''
    if film:
        fwid = n * (fw + 16) + 24
        body += paper(torn([(-20, -10), (fwid - 20, -10), (fwid - 20, fh + 34), (-20, fh + 34)], seed=n + 7, amp=5, step=8), '#1b1b1e') + ''.join(f'<rect x="{-12 + i * 22}" y="-4" width="12" height="9" rx="2" fill="#f6f3ec"></rect><rect x="{-12 + i * 22}" y="{fh + 20}" width="12" height="9" rx="2" fill="#f6f3ec"></rect>' for i in range((n * (fw + 16)) // 22 + 1))
    else:
        sh = n * (fw * .8 + 14) + 70
        body += paper(torn([(-14, -14), (fw + 14, -14), (fw + 14, sh - 14), (-14, sh - 14)], seed=n, amp=3, step=9), '#fbfaf6')
    for i, (sky, ground, mot) in enumerate(frames):
        x, y = (i * (fw + 16), 12) if film else (0, i * (fw * .8 + 14))
        fhh = fh if film else fw * .8
        body += f'<g transform="translate({x} {y})"><clipPath id="sf{i}"><rect width="{fw}" height="{fhh}"></rect></clipPath><g clip-path="url(#sf{i})"><rect width="{fw}" height="{fhh}" fill="{sky}"></rect><path d="M0 {fhh * .7} H{fw} V{fhh} H0 Z" fill="{ground}"></path>{place(mot)}</g>'
        body += f'<rect class="dev" style="--d:{.2 + i * .35:.2f}s" width="{fw}" height="{fhh}" fill="#3b3226"></rect></g>'
    if caption and not film:
        body += text(fw / 2, n * (fw * .8 + 14) + 34, caption, 'script', 26, '#2b2b30', fw + 10)
    w = n * (fw + 16) if film else fw
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, at=(300 - w / 2, 120 if not film else 230), rot=-4 if not film else -3), motion=[hide('.dev', 0, .7)])


def media(sid, title, *, batch, palette, name, verbs=('Press play', 'Stop'), kind='cassette', head, rows=(), shell='#2b2d33', paper_='#f6efe0', ink='#2b2b30',
          accent='#e0245e', key=None, visit=1):
    """A cassette with its J-card, a record in its sleeve, or a game cartridge: touching it plays (reels and records spin, notes rise)."""
    notes = ''.join(f'<g transform="translate({420 + i * 30} {150 - (i % 2) * 30}) scale(.7)"><g class="nt" style="--d:{i * .4:.1f}s">{MOTIFS["note"]("#fff")}</g></g>' for i in range(3))
    if kind == 'cassette':
        body = (f'<rect x="-170" y="-110" width="340" height="220" rx="14" fill="{shell}"></rect><rect x="-150" y="-92" width="300" height="110" rx="6" fill="{paper_}"></rect>'
                + text(0, -60, head, 'marker', 24, ink, 270) + text(0, -36, rows[0] if rows else '', 'hand', 16, ink, 270)
                + f'<rect x="-90" y="-20" width="180" height="40" rx="20" fill="#1b1b1e"></rect>'
                + ''.join(f'<g transform="translate({x} 0)"><g class="reel"><circle r="16" fill="#f6f3ec"></circle>' + ''.join(f'<rect x="-2" y="-14" width="4" height="8" fill="#1b1b1e" transform="rotate({a})"></rect>' for a in (0, 60, 120, 180, 240, 300)) + '</g></g>' for x in (-56, 56))
                + f'<path d="M-110 110 L-90 60 H90 L110 110 Z" fill="#00000033"></path>' + f'<rect x="-150" y="40" width="300" height="10" fill="{accent}"></rect>')
        spinner = spin('.reel', 4, 2.4)
    elif kind == 'record':
        body = (f'<g transform="translate(70 0)"><g class="disc"><circle r="150" fill="#141416"></circle>' + ''.join(f'<circle r="{r}" fill="none" stroke="#2a2a2e" stroke-width="1.4"></circle>' for r in range(60, 148, 10))
                + f'<circle r="54" fill="{accent}"></circle>' + text(0, -6, head, 'serif', 14, '#fff', 90) + text(0, 14, rows[0] if rows else '', 'mono', 8, '#fff', 90) + '<circle r="5" fill="#141416"></circle></g></g>'
                + f'<path d="M-182 -160 H-50 A20 20 0 0 0 -10 -160 H122 Q130 -160 130 -152 V152 Q130 160 122 160 H-182 Q-190 160 -190 152 V-152 Q-190 -160 -182 -160 Z" fill="{paper_}"></path>' + text(-30, -100, head, 'display', 34, ink, 280, weight=700)
                + ''.join(text(-170, -50 + i * 26, r, 'hand', 18, ink, 280, 'start') for i, r in enumerate(rows[1:7])))
        spinner = slide('.disc', 90, 0, 0, 0, .8)
        spinner = (spinner[0] + spin('.disc', 3, 2.2, .6)[0].replace('.open .disc{', '.open .disc{transform:translateX(90px);'), '.disc')
    else:  # cartridge
        body = (f'<path d="M-130 -150 H110 L130 -130 V150 H-130 Z" fill="{shell}"></path><rect x="-110" y="-120" width="220" height="170" rx="6" fill="{paper_}"></rect>'
                + text(0, -80, head, 'display', 28, ink, 200, weight=800) + ''.join(text(0, -46 + i * 24, r, 'mono', 12, ink, 200) for i, r in enumerate(rows[:5]))
                + ''.join(f'<rect x="{-110 + i * 18}" y="120" width="10" height="30" fill="#c9a14a"></rect>' for i in range(13)))
        spinner = shake('.md', 6, .2)
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, cls='md', at=(270, 300), rot=-6) + notes, motion=[spinner, rise('.nt', 90, 2, 2)])


def mapcard(sid, title, *, batch, palette, name, verbs=('Follow the route', 'Fold the map'), head, route, pins=(), labels=(), land='#efe6d0', water=None,
            art=(), key=None, visit=1):
    """A torn bit of map: touching it draws our route and drops the pins.
    route: an SVG path on a 420 × 360 sheet; pins: (x, y, word); labels: (x, y, word, rot); water: path for sea or lake."""
    w, h = 420, 360
    d = torn([(0, 0), (w, 0), (w, h), (0, h)], seed=len(head) + 3, amp=4, step=12)
    body = (f'<clipPath id="mc-c"><path d="{d}"></path></clipPath>' + paper(d, land)
            + f'<g clip-path="url(#mc-c)">' + (f'<path d="{water}" fill="#b9d3df"></path>' if water else '')
            + ''.join(f'<path d="M{x} -10 V{h + 10}" stroke="#d8cdb2" stroke-width="10"></path>' for x in (90, 240, 350))
            + ''.join(f'<path d="M-10 {y} H{w + 10}" stroke="#d8cdb2" stroke-width="10"></path>' for y in (110, 240))
            + ''.join(f'<rect x="{x}" y="{y}" width="{bw}" height="{bh}" rx="3" fill="#e3d7bb"></rect>' for x, y, bw, bh in [(12, 128, 60, 90), (110, 20, 110, 70), (258, 128, 74, 96), (110, 262, 110, 80), (366, 20, 48, 70)])
            + ''.join(f'<path d="M{x} {y} q14 -10 28 0" fill="none" stroke="#7d9c57" stroke-width="8" stroke-linecap="round"></path>' for x, y in [(20, 60), (300, 40), (40, 300), (370, 310)])
            + place(art) + ''.join(words(x, y, s, 'mono', 9, '#8f8268', 'middle', r) for x, y, s, r in labels) + '</g>'
            + f'<path class="rt" pathLength="1" d="{route}" fill="none" stroke="#c23a2b" stroke-width="3.4" stroke-linecap="round" stroke-linejoin="round"></path>'
            + ''.join(f'<g transform="translate({x} {y})"><g class="pin" style="--d:{.5 + i * .35:.2f}s"><path d="M0 0 C-12 -16 -12 -30 0 -30 C12 -30 12 -16 0 0 Z" fill="#c23a2b"></path><circle cy="-21" r="4" fill="#fff"></circle>'
                      + words(14, -16, s, 'hand', 16, '#c23a2b') + '</g></g>' for i, (x, y, s) in enumerate(pins))
            + text(20, 34, head, 'serif', 22, '#3a3328', w - 40, 'start'))
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, at=(80, 130), rot=-3), motion=[drawon('.rt', .1, 1.6), pop('.pin', 0)])


def lanyard(sid, title, *, batch, palette, name, verbs=('Wear it', 'Hang it up'), head, sub='', big='', card='#2f6db5', ink='#ffffff', strap='#111111',
            art=(), key=None, visit=1):
    """A lanyard pass or festival wristband on a strap: touching it swings and lights up."""
    d = 'M-130 30 Q-130 0 -100 0 H100 Q130 0 130 30 V300 Q130 330 100 330 H30 L0 352 L-30 330 H-100 Q-130 330 -130 300 Z'
    body = (f'<path d="M-40 -150 C-60 -80 -60 -30 -50 0 M40 -150 C60 -80 60 -30 50 0" fill="none" stroke="{strap}" stroke-width="12"></path><rect x="-16" y="-10" width="32" height="24" rx="4" fill="#9aa0a6"></rect>'
            + f'<path d="{d}" fill="{card}"></path><rect x="-30" y="16" width="60" height="10" rx="5" fill="#00000044"></rect>'
            + text(0, 90, head, 'display', 30, ink, 230, weight=800) + text(0, 116, sub, 'mono', 11, ink, 230) + text(0, 220, big, 'marker', 44, ink, 230)
            + place(art) + f'<path class="glow" d="{d}" fill="#fff"></path>')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, cls='ly', at=(300, 160), rot=-3), motion=[swing('.ly', '50% 0%', 6), (f'.glow{{opacity:0}}.open .glow{{animation:gl 1.2s ease-out .3s}}@keyframes gl{{30%{{opacity:.35}}100%{{opacity:0}}}}', '.glow')])


# ── more objects (variety pass) ───────────────────────────────────────────────────────────────
def matchbook(sid, title, *, batch, palette, name, verbs=('Strike one', 'Close it'), head, sub='', cover='#1b1b1e', ink='#f2c230', inside='',
              art=(), key=None, visit=1):
    """A bar's matchbook: touching it flips the cover up, one match lifts and lights."""
    w, h = 220, 300
    matches = ''.join(f'<g transform="translate({30 + i * 20} 70)"><rect x="-6" y="0" width="12" height="90" rx="3" fill="#efe1c0"></rect><circle r="8" fill="#b5452c"></circle></g>' for i in range(9) if i != 4)
    one = (f'<g class="mt"><g transform="translate(110 70)"><rect x="-6" y="0" width="12" height="90" rx="3" fill="#efe1c0"></rect><circle r="8" fill="#b5452c"></circle>'
           f'<g class="fl"><path d="M0 -6 C-14 -22 -6 -40 0 -52 C8 -38 14 -22 0 -6 Z" fill="#ffb347"></path><path d="M0 -8 C-6 -18 -2 -28 0 -34 C4 -26 6 -18 0 -8 Z" fill="#fff1b8"></path></g></g></g>')
    body = (f'<rect width="{w}" height="{h}" rx="10" fill="{cover}"></rect><rect y="{h - 70}" width="{w}" height="18" fill="#6b5a4a"></rect>'
            + f'<rect x="10" y="40" width="{w - 20}" height="{h - 120}" rx="4" fill="#3a2a20"></rect>{matches}{one}'
            + text(w / 2, h - 18, inside, 'hand', 15, ink, w - 30)
            + f'<g class="cv"><rect width="{w}" height="{h - 70}" rx="10" fill="{cover}"></rect>'
            + text(w / 2, 80, head, 'display', 34, ink, w - 30, weight=800) + text(w / 2, 108, sub, 'mono', 11, ink, w - 30) + place(art) + '</g>')
    css = ('.cv{transform-box:fill-box;transform-origin:50% 0%;transition:transform .45s .2s}.open .cv{transform:scaleY(-.35) translateY(40px);transition:transform .45s}'
           '.mt{transform-box:fill-box;transition:transform .3s}.open .mt{transform:translateY(-60px) rotate(8deg);transition:transform .5s cubic-bezier(.3,1.3,.5,1) .4s}'
           '.fl{opacity:0;transform-box:fill-box;transform-origin:50% 100%}.open .fl{opacity:1;animation:flk .5s ease-in-out .9s infinite alternate;transition:opacity .2s .9s}@keyframes flk{to{transform:scale(.88,1.12)}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, at=(190, 170), rot=-6), motion=[('', '.cv,.mt,.fl')])


def coaster(sid, title, *, batch, palette, name, verbs=('Put a glass down', 'Pick it up'), head, sub='', shape='round', board_='#e9d6b3', ink='#3a2414',
            drink='glass', drink_col=('#f2b33d', '#ffffff'), art=(), key=None, visit=1):
    """A coaster (round cardboard, square cork, or scalloped): touching it sets a glass down, leaves a ring, and the coaster spins a little."""
    shp = {'round': '<circle r="140" fill="{c}"></circle><circle r="124" fill="none" stroke="{i}" stroke-opacity=".35" stroke-width="2"></circle>',
           'square': '<rect x="-130" y="-130" width="260" height="260" rx="22" fill="{c}"></rect><rect x="-116" y="-116" width="232" height="232" rx="14" fill="none" stroke="{i}" stroke-opacity=".35" stroke-width="2"></rect>',
           'scallop': ''.join(f'<circle cx="{132 * math.cos(math.radians(a)):.1f}" cy="{132 * math.sin(math.radians(a)):.1f}" r="16" fill="{{c}}"></circle>' for a in range(0, 360, 20)) + '<circle r="134" fill="{c}"></circle>'}[shape]
    body = (shp.replace('{c}', board_).replace('{i}', ink) + text(0, -30, head, 'display', 36, ink, 220, weight=800) + text(0, 0, sub, 'mono', 12, ink, 210) + place(art)
            + f'<circle class="ring" r="48" cx="56" cy="74" fill="none" stroke="#6b4a2a" stroke-opacity=".45" stroke-width="7"></circle>')
    glass_ = f'<g class="gl"><g transform="translate(340 320) scale(2.4)">{MOTIFS[drink](*drink_col)}</g></g>'
    css = ('.ring{opacity:0}.open .ring{opacity:1;transition:opacity .4s 1.1s}.gl{transform-box:fill-box;opacity:0}'
           '.open .gl{animation:gld 1.6s ease-in-out both}@keyframes gld{0%{opacity:0;transform:translate(60px,-160px)}35%{opacity:1;transform:translate(0,0)}70%{opacity:1;transform:translate(0,0)}100%{opacity:0;transform:translate(80px,-140px)}}'
           '.cs{transform-box:fill-box;transform-origin:50% 50%}.open .cs{animation:csp .7s ease-out .9s}@keyframes csp{40%{transform:rotate(9deg)}100%{transform:rotate(0)}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, cls='cs', at=(290, 300), rot=-8) + glass_, motion=[('', '.ring,.gl,.cs')])


def menu(sid, title, *, batch, palette, name, verbs=('Open the menu', 'Fold it'), head, sub='', panels=(), card='#f6efe0', ink='#2b2b30', accent='#7a1a16',
         ticks=(), art=(), key=None, visit=1):
    """A folded menu: touching it opens the side flaps and ticks what we ordered.
    panels: (left lines, middle lines, right lines); ticks: (panel, line) pairs."""
    pw, h = 150, 300
    def panel(lines, x):
        return ''.join(text(x + pw / 2, 70 + i * 30, l, 'serif', 15, ink, pw - 20) for i, l in enumerate(lines))
    left, mid, right = (list(panels) + [[], [], []])[:3]
    inner = (f'<path d="M0 8 L{pw} 0 L{pw * 2} 8 L{pw * 3} 0 V{h} L{pw * 2} {h - 8} L{pw} {h} L0 {h - 8} Z" fill="{card}"></path>' + ''.join(f'<path d="M{pw * k} 10 V{h - 10}" stroke="{ink}" stroke-opacity=".2"></path>' for k in (1, 2))
             + panel(left, 0) + panel(mid, pw) + panel(right, pw * 2)
             + ''.join(f'<path class="mk" style="--d:{.7 + i * .25:.2f}s" pathLength="1" d="M{p * pw + 10} {62 + l * 30} l6 7 l12 -16" fill="none" stroke="{accent}" stroke-width="2.6" stroke-linecap="round"></path>' for i, (p, l) in enumerate(ticks)))
    cover = lambda x, cls, org: (f'<g class="{cls}" style="transform-origin:{org}"><rect x="{x}" width="{pw}" height="{h}" fill="{accent}"></rect>'
                                 f'<rect x="{x + 8}" y="8" width="{pw - 16}" height="{h - 16}" fill="none" stroke="#fff" stroke-opacity=".5"></rect></g>')
    front = (f'<g class="frontface">' + text(pw * 1.5, 120, head, 'display', 30, '#fff', pw - 16, weight=700) + text(pw * 1.5, 150, sub, 'mono', 10, '#fff', pw - 16) + place(art) + '</g>')
    css = ('.fl1,.fl2,.mid,.frontface{transition:opacity .3s .1s}.open .fl1,.open .fl2,.open .mid,.open .frontface{opacity:0;transition:opacity .35s}'
           '.inner{transform-box:fill-box;transform-origin:50% 50%;transform:scaleX(.34);transition:transform .45s}.open .inner{transform:none;transition:transform .55s cubic-bezier(.3,1.1,.4,1)}')
    body = f'<g class="inner">{inner}<g class="fl1">{cover(0, "", "0 0")}</g><g class="fl2">{cover(pw * 2, "", "0 0")}</g><rect x="{pw}" width="{pw}" height="{h}" fill="{accent}" class="mid"></rect>{front}</g>'
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, at=(75, 150), rot=-3), motion=[('', '.fl1,.fl2,.mid,.frontface,.inner'), drawon('.mk', 0, .3)])


def cuppa(sid, title, *, batch, palette, name, verbs=('Lift the lid', 'Put it back'), kind='takeaway', head='', sub='', cup_col='#f6f3ec', band='#c99d69',
          ink='#2b2b30', drink='#6b3a1f', marker='', art=(), key=None, visit=1):
    """Something to drink: a takeaway cup with a sleeve, a cutting-chai glass, a teacup on its saucer, or a mug.
    Touching it lifts the lid (or tips it a little) and the steam rises."""
    if kind == 'takeaway':
        body = (f'<path d="M-90 -120 H90 L74 160 H-74 Z" fill="{cup_col}"></path><path d="M-84 -10 H84 L78 80 H-78 Z" fill="{band}"></path>'
                + text(0, 30, head, 'serif', 26, ink, 150) + text(0, 56, sub, 'mono', 10, ink, 150) + text(0, -60, marker, 'marker', 26, '#2b2b30', 160, rot=-4)
                + f'<g class="lid"><path d="M-100 -122 H100 L94 -146 H-94 Z" fill="#e2ded3"></path><path d="M-80 -146 C-80 -170 80 -170 80 -146 Z" fill="#efebe2"></path><ellipse cx="36" cy="-160" rx="14" ry="5" fill="#333"></ellipse></g>'
                + f'<ellipse cy="-122" rx="88" ry="10" fill="{drink}"></ellipse>')
        mv = slide('.lid', 70, -90, 24, 0, .6)
        top = -150
    elif kind == 'cutting':
        body = (f'<path d="M-50 -90 H50 L40 90 H-40 Z" fill="#ffffff" fill-opacity=".25" stroke="#fff" stroke-width="2"></path><path d="M-46 -40 H46 L40 88 H-40 Z" fill="{drink}"></path>'
                + ''.join(f'<path d="M{x} -86 L{x * .8:.0f} 86" stroke="#fff" stroke-opacity=".35" stroke-width="3"></path>' for x in (-30, -10, 10, 30))
                + f'<ellipse cy="-40" rx="46" ry="6" fill="#c98a4a"></ellipse>' + f'<g transform="translate(0 150)">' + text(0, 0, head, 'hand', 26, '#fff', 220) + text(0, 26, sub, 'mono', 10, '#fff', 220) + '</g>')
        mv = shake('.cu', 6, .1)
        top = -96
    elif kind == 'teacup':
        body = (f'<ellipse cy="76" rx="150" ry="30" fill="{cup_col}" stroke="{band}" stroke-width="3"></ellipse><path d="M-100 -40 H100 C100 50 60 76 0 76 C-60 76 -100 50 -100 -40 Z" fill="{cup_col}"></path>'
                + f'<path d="M100 -20 C150 -20 150 40 90 40" fill="none" stroke="{cup_col}" stroke-width="14"></path><ellipse cy="-40" rx="100" ry="16" fill="{drink}"></ellipse>'
                + f'<path d="M-96 -10 C-60 10 60 10 96 -10" fill="none" stroke="{band}" stroke-width="5"></path>' + place(art)
                + f'<g transform="translate(0 140)">' + text(0, 0, head, 'script', 30, '#fff', 260, weight=700) + text(0, 24, sub, 'mono', 10, '#fff', 260) + '</g>')
        mv = shake('.cu', 4, .1)
        top = -60
    else:  # mug
        body = (f'<rect x="-90" y="-100" width="180" height="200" rx="24" fill="{cup_col}"></rect><path d="M90 -50 C150 -50 150 50 90 50" fill="none" stroke="{cup_col}" stroke-width="22"></path>'
                + f'<ellipse cy="-100" rx="90" ry="14" fill="{drink}"></ellipse>' + text(0, 0, head, 'display', 30, ink, 160, weight=800) + text(0, 30, sub, 'mono', 10, ink, 160) + place(art))
        mv = shake('.cu', 4, .1)
        top = -110
    steam = ''.join(f'<path class="sm" style="--d:{i * .45:.2f}s" d="M{x} {top} c-12 -16 12 -24 0 -42 c-8 -12 8 -18 0 -30" fill="none" stroke="#fff" stroke-width="5" stroke-linecap="round"></path>' for i, x in enumerate((-30, 0, 30)))
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body + steam, cls='cu', at=(300, 320), rot=-3), motion=[mv, rise('.sm', 70, 1.8, 3)])


def cakebox(sid, title, *, batch, palette, name, verbs=('Untie the ribbon', 'Box it up'), head, sub='', box='#f7c9d4', ribbon='#c2185b', ink='#3a1a28',
            treat=('cake',), art=(), key=None, visit=1):
    """A bakery box tied with ribbon: touching it slips the bow, lifts the lid and shows what's inside."""
    inside = f'<g transform="translate(0 -40) scale(2.6)">{MOTIFS[treat[0]](*treat[1:])}</g>'
    body = (f'<path d="M-150 -60 H150 L138 120 Q136 128 128 128 H-128 Q-136 128 -138 120 Z" fill="{box}" stroke="#00000022"></path><path d="M-150 -60 L-138 120 M150 -60 L138 120" stroke="#00000018" stroke-width="6"></path>{inside}'
            + f'<path d="M-150 -60 H150 V-20 H-150 Z" fill="{box}" opacity=".0"></path>'
            + f'<g class="lid"><path d="M-160 -96 Q-160 -104 -152 -104 H152 Q160 -104 160 -96 L156 -50 H-156 Z" fill="{box}" stroke="#00000022"></path>' + text(0, -68, head, 'script', 34, ink, 280, weight=700) + '</g>'
            + f'<g class="rb"><path d="M-160 -76 H160" stroke="{ribbon}" stroke-width="10"></path><path d="M0 -100 V120" stroke="{ribbon}" stroke-width="10"></path>'
            + f'<path d="M0 -100 C-40 -150 -70 -110 0 -100 C70 -110 40 -150 0 -100 Z" fill="{ribbon}"></path></g>'
            + f'<g transform="translate(0 70)">' + text(0, 0, sub, 'mono', 11, ink, 260) + place(art) + '</g>')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, at=(300, 330), rot=-4), motion=[slide('.rb', 0, -40, 0, 0, .3), hide('.rb', .15, .2), slide('.lid', 40, -150, -12, .35, .6)])


def polaroid(sid, title, *, batch, palette, name, verbs=('Shake it', 'Fade it'), scene, sky=('#9fc7e0', '#e9d6b3'), ground='#5d7d4f', caption='', key=None, visit=1):
    """An instant photo: touching it gives it a shake and the picture develops."""
    w, h = 300, 360
    pic = (f'<clipPath id="pl-c"><rect x="20" y="20" width="{w - 40}" height="{w - 40}"></rect></clipPath>'
           f'<linearGradient id="pl-s" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{sky[0]}"></stop><stop offset="1" stop-color="{sky[1]}"></stop></linearGradient>'
           f'<g clip-path="url(#pl-c)"><rect x="20" y="20" width="{w - 40}" height="{w - 40}" fill="url(#pl-s)"></rect><path d="M20 210 C100 196 200 220 {w - 20} 204 V{w - 20} H20 Z" fill="{ground}"></path>'
           f'<g transform="translate(20 20)">{place(scene)}</g></g>')
    body = (f'<path d="M0 0 H{w} V{h - 34} Q{w - 8} {h - 6} {w - 40} {h} H0 Z" fill="#fbfaf6"></path><path d="M{w} {h - 34} Q{w - 8} {h - 6} {w - 40} {h} Q{w - 22} {h - 22} {w} {h - 34} Z" fill="#d9d4c6"></path>{pic}<rect class="dev" x="20" y="20" width="{w - 40}" height="{w - 40}" fill="#2b2a2e"></rect>'
            + text(w / 2, h - 30, caption, 'script', 30, '#2b2b30', w - 30, weight=700))
    css = '.dev{transition:opacity .3s}.open .dev{opacity:0;transition:opacity 1.6s ease-in .5s}'
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, cls='pol', at=(150, 120), rot=-5), motion=[('', '.dev'), shake('.pol', 5)])


def bookmark(sid, title, *, batch, palette, name, verbs=('Mark the page', 'Close the book'), head, rows=(), strip_col='#2f6d57', ink='#f6efe0',
             tassel='#c9a14a', book_col=('#7a1a16', '#f6efe0'), art=(), key=None, visit=1):
    """A bookmark sitting in a closed book: touching it opens the book and the tassel swings."""
    bk = (f'<g class="bk"><path d="M-170 -96 Q-90 -124 0 -108 V150 Q-90 136 -170 160 Z" fill="{book_col[1]}"></path><path d="M0 -108 Q90 -124 170 -96 V160 Q90 136 0 150 Z" fill="{book_col[1]}"></path>'
          + ''.join(f'<path d="M{s * 20} {-80 + i * 22} H{s * 150}" stroke="#c9c2b0" stroke-width="2"></path>' for s in (-1, 1) for i in range(10)) + '</g>'
          + f'<g class="cov"><path d="M-6 -114 Q90 -130 176 -102 V166 Q90 142 -6 156 Z" fill="{book_col[0]}"></path></g>')
    mark = (f'<g class="bm"><path d="M-40 -170 H40 V120 L0 96 L-40 120 Z" fill="{strip_col}"></path>'
            + f'<g transform="translate(-12 -40) rotate(-90)">{text(0, 8, head, "serif", 22, ink, 240)}</g>'
            + ''.join(f'<g transform="translate({10 + i * 18} -40) rotate(-90)">{text(0, 6, r, "hand", 13, ink, 240)}</g>' for i, r in enumerate(rows[:2]))
            + place(art) + f'<path d="M0 -170 C10 -200 -10 -220 0 -240" fill="none" stroke="{tassel}" stroke-width="3"></path><path class="ts" d="M-8 -240 H8 L4 -270 H-4 Z" fill="{tassel}"></path></g>')
    css = ('.cov{transform-box:view-box;transform-origin:300px 0;transition:transform .5s}.open .cov{transform:scaleX(-.0);transition:transform .6s}'
           '.ts{transform-box:fill-box;transform-origin:50% 100%}.open .ts{animation:tsw .6s ease-in-out .5s 3 alternate}@keyframes tsw{to{transform:rotate(20deg)}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(bk + mark, at=(300, 330), rot=-3), motion=[('', '.cov,.ts'), slide('.bm', 0, -30, -4, .3, .6)])


def handstamp(sid, title, *, batch, palette, name, verbs=('Lights down', 'Lights up'), stamp_text, sub='', skin='#c98a64', ink='#7dffb2', art=(), key=None, visit=1):
    """The stamp on the back of a hand at the door: touching it drops the lights and the UV ink glows."""
    hand = ('M-90 160 C-100 80 -110 20 -100 -40 C-96 -70 -70 -70 -66 -40 L-60 0 L-56 -110 C-54 -140 -24 -140 -22 -110 L-18 -10 L-14 -130 C-12 -160 18 -160 20 -130 L24 -10 '
            'L30 -110 C32 -140 62 -140 64 -110 L66 10 L78 -40 C86 -66 116 -60 112 -30 L96 60 C86 120 60 150 40 170 Z')
    body = (f'<path d="{hand}" fill="{skin}"></path><path d="{hand}" fill="#000" class="dark" opacity="0"></path>'
            + f'<g class="uv" transform="translate(0 70) rotate(-10)" filter="url(#hs-g)">' + text(0, 0, stamp_text, 'marker', 30, ink, 150) + text(0, 22, sub, 'mono', 10, ink, 150) + '</g>'
            + f'<g transform="translate(0 70) rotate(-10)" opacity=".35">' + text(0, 0, stamp_text, 'marker', 30, '#6b4a3a', 150) + '</g>' + place(art))
    css = ('.uv{opacity:0;transition:opacity .2s}.open .uv{opacity:1;transition:opacity .5s .5s}.dark{transition:opacity .3s}.open .dark{opacity:.55;transition:opacity .4s}'
           '.open .uv{animation:uvp 1.2s ease-in-out .9s 3 alternate}@keyframes uvp{to{opacity:.6}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          defs='<filter id="hs-g" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="2" result="b"></feGaussianBlur><feMerge><feMergeNode in="b"></feMergeNode><feMergeNode in="SourceGraphic"></feMergeNode></feMerge></filter>',
          svg=g(body, at=(300, 300), rot=-6), motion=[('', '.uv,.dark')])


def keytag(sid, title, *, batch, palette, name, verbs=('Jingle the keys', 'Hang them up'), head, number='', tag_col='#c0392b', ink='#fff', art=(), key=None, visit=1):
    """A key on a plastic fob (a locker, a room, a rental): touching it jingles on its ring."""
    keyd = 'M0 0 a26 26 0 1 1 0.1 0 Z M-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0 M20 10 H110 V24 H100 V34 H90 V24 H78 V34 H68 V24 H20 Z'
    body = (f'<circle r="40" fill="none" stroke="#9aa0a6" stroke-width="7"></circle>'
            + f'<g transform="translate(30 30) rotate(30)"><path d="{keyd}" fill="#c9a14a" fill-rule="evenodd" transform="translate(30 0)"></path></g>'
            + f'<g transform="translate(-20 30) rotate(70)"><path d="M0 0 C-30 0 -60 20 -70 60 L-40 230 Q-36 250 -16 250 H50 Q70 250 74 230 L104 60 C94 20 64 0 34 0 Z" fill="{tag_col}" transform="translate(-17 0)"></path>'
            + f'<circle cx="0" cy="30" r="10" fill="#000" opacity=".35"></circle>' + f'<g transform="translate(0 140) rotate(-90)">' + text(0, 0, head, 'display', 26, ink, 170, weight=800) + text(0, 26, number, 'mono', 14, ink, 170, weight=700) + '</g>' + place(art) + '</g>')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=f'<path d="M300 60 V130" stroke="#555" stroke-width="3"></path>' + g(body, cls='kt', at=(300, 170)), motion=[swing('.kt', '50% 0%', 14, 0, 1), shake('.kt', 6, .1)])


def seedpacket(sid, title, *, batch, palette, name, verbs=('Shake out the seeds', 'Fold it shut'), head, rows=(), packet='#f2e3b3', ink='#3a2a1a',
               art=(('plant', 0, -40, 1.6),), seed_col='#6b4a2a', key=None, visit=1):
    """A seed or spice packet: touching it tips the packet and seeds pour out."""
    body = (f'<path d="M-120 -170 H120 V150 ' + ''.join(f'L{120 - i * 12} {170 if i % 2 else 150} ' for i in range(1, 21)) + f'Z" fill="{packet}"></path><path d="M-120 -170 H120 V-140 H-120 Z" fill="#00000018"></path>'
            + ''.join(f'<path d="M{-120 + i * 12} -170 l6 -8 l6 8" fill="{packet}"></path>' for i in range(20))
            + f'<rect x="-100" y="-120" width="200" height="150" rx="6" fill="#ffffff66"></rect>' + place(art)
            + text(0, 70, head, 'serif', 26, ink, 220) + ''.join(text(0, 98 + i * 22, r, 'hand', 16, ink, 220) for i, r in enumerate(rows[:3])))
    seeds = ''.join(f'<ellipse class="sd" style="--d:{i * .08:.2f}s" cx="{-100 + (i * 37) % 60}" cy="-180" rx="4" ry="6" fill="{seed_col}"></ellipse>' for i in range(14))
    css = ('.pk{transform-box:fill-box;transform-origin:30% 100%;transition:transform .4s}.open .pk{transform:rotate(-35deg);transition:transform .5s}'
           '.sd{opacity:0}.open .sd{animation:sdp 1s ease-in calc(.5s + var(--d)) both}@keyframes sdp{0%{opacity:1;transform:translate(0,0)}100%{opacity:1;transform:translate(-80px,260px)}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(f'<g class="pk">{body}</g>{seeds}', at=(320, 320)), motion=[('', '.pk,.sd')])


def clapper(sid, title, *, batch, palette, name, verbs=('Action!', 'Cut'), head, rows=(), slate='#1b1b1e', ink='#f6f3ec', key=None, visit=1):
    """A clapperboard: touching it snaps the sticks shut."""
    stripes = ''.join(f'<path d="M{x} 0 l30 0 l-20 40 l-30 0 Z" fill="{ink}"></path>' for x in range(0, 360, 60))
    body = (f'<rect y="40" width="340" height="230" rx="16" fill="{slate}"></rect>' + ''.join(f'<path d="M14 {120 + i * 46} H326" stroke="{ink}" stroke-opacity=".4"></path>' for i in range(3))
            + text(20, 100, head, 'marker', 34, ink, 300, 'start') + ''.join(text(20, 150 + i * 46, r, 'hand', 22, ink, 300, 'start') for i, r in enumerate(rows[:3]))
            + f'<g><rect y="20" width="340" height="24" fill="{slate}"></rect><g transform="translate(0 20)" clip-path="url(#cl-c)">{stripes}</g></g>'
            + f'<g class="stick"><rect y="-24" width="340" height="30" fill="{slate}"></rect><g transform="translate(0 -24)" clip-path="url(#cl-c2)">{stripes}</g></g>')
    css = ('.stick{transform-box:fill-box;transform-origin:0% 100%;transform:rotate(-24deg);transition:transform .3s}'
           '.open .stick{animation:clap .5s cubic-bezier(.5,0,.8,.4) both}@keyframes clap{0%{transform:rotate(-24deg)}70%{transform:rotate(0)}85%{transform:rotate(-4deg)}100%{transform:rotate(0)}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          defs='<clipPath id="cl-c"><rect width="340" height="24"></rect></clipPath><clipPath id="cl-c2"><rect width="340" height="30"></rect></clipPath>',
          svg=g(body, at=(120, 170), rot=-4), motion=[('', '.stick')])


def specimen(sid, title, *, batch, palette, name, verbs=('Lift the tissue', 'Cover it'), head, rows=(), item=('leaf',), card='#f3ecdc', ink='#3a3328', key=None, visit=1):
    """A pressed specimen card under tracing tissue: touching it lifts the tissue to show the flower or leaf, taped down."""
    w, h = 320, 400
    edge = torn([(0, 0), (w, 0), (w, h), (0, h)], seed=len(head), amp=3, step=7)
    body = (f'<path d="{edge}" fill="{card}"></path><path d="M10 10 H{w - 10} V{h - 10} H10 Z" fill="none" stroke="{ink}" stroke-opacity=".3"></path>'
            + f'<g transform="translate({w / 2} 160) scale(2.4) rotate(-12)">{MOTIFS[item[0]](*item[1:])}</g>'
            + ''.join(f'<rect x="{x}" y="{y}" width="50" height="16" fill="#fff8d8" opacity=".8" transform="rotate({r} {x + 25} {y + 8})"></rect>' for x, y, r in [(80, 70, -20), (190, 230, 15)])
            + f'<rect x="20" y="{h - 110}" width="{w - 40}" height="90" fill="#fffdf5" stroke="{ink}" stroke-opacity=".3"></rect>'
            + text(30, h - 78, head, 'serif', 20, ink, w - 60, 'start') + ''.join(text(30, h - 52 + i * 20, r, 'hand', 15, ink, w - 60, 'start') for i, r in enumerate(rows[:2]))
            + f'<g class="tis"><path d="M0 0 H{w} V{h - 120} H0 Z" fill="#ffffff" opacity=".72"></path></g>')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, at=(130, 110), rot=-4), motion=[slide('.tis', -24, -150, -10, 0, .7)])


def sticker(sid, title, *, batch, palette, name, verbs=('Peel it', 'Stick it back'), head, sub='', shape='circle', col='#e0245e', ink='#fff', art=(), key=None, visit=1):
    """A die-cut sticker on its backing sheet: touching it peels it up at the corner."""
    shp = {'circle': '<circle r="130" fill="{c}"></circle>', 'star': f'<path d="{" ".join(("M" if i == 0 else "L") + f"{(150 if i % 2 == 0 else 80) * math.cos(math.radians(-90 + i * 36)):.0f} {(150 if i % 2 == 0 else 80) * math.sin(math.radians(-90 + i * 36)):.0f}" for i in range(10))} Z" fill="{{c}}"></path>',
           'blob': '<path d="M-120 -40 C-130 -110 -30 -140 30 -120 C120 -100 150 -20 120 50 C90 130 -20 140 -80 110 C-130 80 -110 20 -120 -40 Z" fill="{c}"></path>'}[shape]
    body = (f'<rect x="-170" y="-170" width="340" height="340" rx="12" fill="#f3efe6"></rect><path d="M-150 -150 H150 V150 H-150 Z" fill="none" stroke="#d6cfc0" stroke-dasharray="4 5"></path>'
            + f'<g class="stk">{shp.replace("{c}", "#ffffff")}<g transform="scale(.94)">{shp.replace("{c}", col)}</g>' + text(0, 0, head, 'display', 36, ink, 200, weight=800) + text(0, 32, sub, 'mono', 11, ink, 190) + place(art) + '</g>')
    css = ('.stk{transform-box:fill-box;transform-origin:0% 0%;transition:transform .4s}.open .stk{transform:translate(30px,-30px) rotate(-8deg) skewX(-6deg);transition:transform .6s cubic-bezier(.3,1.3,.5,1)}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(body, at=(300, 300), rot=-6), motion=[('', '.stk')])
