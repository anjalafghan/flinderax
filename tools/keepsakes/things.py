"""One-off keepsakes drawn for a single slip, where no kit object says it right.

Each function is one slip's object; the slip module calls it with the usual sid/title/batch/palette.
"""
import math
from objects import text, place  # noqa: F401
from kit import board, g, swing, drawon, appear, slide, glow, shake, pop  # noqa: F401


def blanket_fort(sid, title, *, batch, palette, name, verbs=('Lights down', 'Lights up'), key=None, visit=1):
    """The corner of the blanket fort: a quilt pegged to a string, fairy lights along its edge, which come on."""
    quilt = 'M-190 -120 C-120 -112 -40 -124 40 -114 C110 -106 160 -118 200 -110 C190 -60 170 -20 150 10 C120 40 90 30 60 70 C30 110 -10 120 -50 160 C-90 200 -130 190 -150 150 C-180 60 -186 -40 -190 -120 Z'
    checks = ''.join(f'<path d="M{-200 + i * 44} -130 L{-60 + i * 44} 170" stroke="#f6efe0" stroke-opacity=".35" stroke-width="10"></path>' for i in range(10))
    checks += ''.join(f'<path d="M-200 {-100 + i * 40} C-60 {-108 + i * 40} 60 {-92 + i * 40} 200 {-104 + i * 40}" stroke="#3a2a5a" stroke-opacity=".25" stroke-width="8"></path>' for i in range(7))
    peg = lambda x: (f'<g transform="translate({x} -124) rotate(4)"><rect x="-9" y="-34" width="18" height="62" rx="4" fill="#d9b27a"></rect>'
                     f'<path d="M0 -34 V28" stroke="#a8803f" stroke-width="2"></path><rect x="-11" y="-12" width="22" height="7" fill="#9aa0a6"></rect></g>')
    pts = [(-182, -40), (-176, 20), (-166, 80), (-150, 130), (-110, 170), (-60, 146), (-20, 112), (30, 84), (80, 50), (130, 20), (170, -30)]
    wire = 'M' + ' L'.join(f'{x} {y + 10}' for x, y in pts)
    bulbs = ''.join(f'<g transform="translate({x} {y + 10}) rotate({(i * 37) % 50 - 25})"><rect x="-3" y="-4" width="6" height="7" fill="#3a3a3a"></rect>'
                    f'<ellipse cy="9" rx="6" ry="9" fill="#e8dcae"></ellipse><ellipse class="lit" style="--d:{i * .09:.2f}s" cy="9" rx="6" ry="9" fill="#ffd34d"></ellipse>'
                    f'<circle class="lit" style="--d:{i * .09:.2f}s" cy="9" r="20" fill="#ffd34d" opacity=".3"></circle></g>' for i, (x, y) in enumerate(pts))
    tagp = 'M-70 -26 H60 L76 0 L60 26 H-70 Z'
    tag_ = (f'<g transform="translate(10 40) rotate(-5)"><path d="{tagp}" fill="#fbf6ea"></path><circle cx="62" cy="0" r="5" fill="#6f5282"></circle>'
            + text(-4, -2, 'FORT', 'marker', 26, '#3a2a5a', 120) + text(-4, 16, 'phones stay outside', 'mono', 9, '#3a2a5a', 124) + '</g>')
    body = (f'<path d="M-240 -128 Q0 -112 240 -132" fill="none" stroke="#efe4cf" stroke-width="3"></path>'
            f'<clipPath id="bf-q"><path d="{quilt}"></path></clipPath><path d="{quilt}" fill="#8a6a9c"></path><g clip-path="url(#bf-q)">{checks}</g>'
            + peg(-150) + peg(150) + tag_ + f'<path d="{wire}" fill="none" stroke="#3a3a3a" stroke-width="2"></path>' + bulbs)
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, cls='bf', at=(300, 290), rot=-3), motion=[glow('.lit', 1), swing('.bf', '50% 0%', 2)], css='.lit{opacity:0}')


def henna_cone(sid, title, *, batch, palette, name, verbs=('Draw a vine', 'Wipe it off'), key=None, visit=1):
    """A mehendi cone; touching it squeezes out the pattern we drew on each other: a moon, a vine, us."""
    cone = 'M-34 -150 C-20 -160 20 -160 34 -150 L6 120 Q0 132 -6 120 Z'
    wrap = ''.join(f'<path d="M{-30 + i * 2} {-110 + i * 40} L{30 - i * 2} {-130 + i * 40}" stroke="#7a5a3a" stroke-opacity=".35" stroke-width="2"></path>' for i in range(6))
    top = '<path d="M-34 -150 C-30 -180 -8 -170 0 -196 C8 -170 30 -180 34 -150 Z" fill="#e9dcc2"></path><path d="M-20 -176 L18 -168" stroke="#c0392b" stroke-width="5" stroke-linecap="round"></path>'
    band = f'<g transform="translate(0 -60) rotate(-8)"><rect x="-58" y="-18" width="116" height="36" rx="4" fill="#c0392b"></rect>' + text(0, 6, 'moon, vine, us', 'hand', 16, '#fff8ea', 104) + '</g>'
    paste = '<path d="M0 126 C-2 140 4 150 0 160" stroke="#5a2e12" stroke-width="5" fill="none" stroke-linecap="round"></path>'
    vine = ('<g fill="none" stroke="#6b3412" stroke-width="5" stroke-linecap="round">'
            '<path class="hv" pathLength="1" d="M0 160 C40 176 80 150 110 170 C140 190 170 160 200 176"></path>'
            '<path class="hv" style="--d:.5s" pathLength="1" d="M60 164 c-4 -16 10 -24 16 -12 M120 176 c4 16 -10 24 -16 12 M170 168 c-4 -16 10 -24 16 -12"></path>'
            '<path class="hv" style="--d:.9s" pathLength="1" d="M230 150 a22 22 0 1 0 6 32 a16 16 0 1 1 -6 -32 Z"></path>'
            '<path class="hv" style="--d:1.2s" pathLength="1" d="M-40 196 C-20 176 0 196 -20 214 C-40 196 -60 176 -40 196 Z"></path></g>')
    body = f'<path d="{cone}" fill="#b87a3e"></path>{wrap}<path d="M-34 -150 L-12 120" stroke="#fff" stroke-opacity=".25" stroke-width="6"></path>{top}{band}{paste}{vine}'
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(body, cls='hc', at=(240, 270), rot=-28, scale=1.25), motion=[drawon('.hv', .2, .7), shake('.hc', 4)])


def paper_crane(sid, title, *, batch, palette, name, verbs=('Make it fly', 'Let it rest'), key=None, visit=1):
    """One folded crane on its thread, a bead under it and a tag: number 1 of a hundred. It flaps."""
    c1, c2, c3 = '#e46f2a', '#c4561c', '#f29a5a'
    crane = (f'<path d="M-150 10 L-40 30 L0 60 Z" fill="{c2}"></path>'          # tail side
             f'<path d="M-40 30 L40 30 L0 80 Z" fill="{c3}"></path>'              # belly
             f'<path d="M40 30 L130 -40 L150 -50 L140 -30 L0 60 Z" fill="{c2}"></path>'  # neck and head
             f'<path d="M150 -50 L170 -40 L146 -36 Z" fill="{c1}"></path>'
             f'<path d="M-150 10 L-110 0 L-40 30 Z" fill="{c1}"></path>')
    wings = (f'<g class="wl"><path d="M-40 30 L40 30 L-30 -120 Z" fill="{c1}"></path><path d="M0 30 L-30 -120" stroke="{c2}" stroke-width="2"></path></g>'
             f'<g class="wr"><path d="M-40 30 L40 30 L60 -110 Z" fill="{c3}"></path></g>')
    thread = '<path d="M-4 -240 V30" stroke="#efe4cf" stroke-width="2"></path><circle cx="-4" cy="110" r="10" fill="#c0392b"></circle><path d="M-4 80 V100 M-4 120 V150" stroke="#efe4cf" stroke-width="2"></path>'
    tag_ = (f'<g transform="translate(-4 150) rotate(6)"><path d="M-40 0 H40 V60 L0 74 L-40 60 Z" fill="#fbf6ea"></path>'
            + text(0, 30, '1 of 100', 'hand', 18, '#2b2b30', 70) + text(0, 50, 'folded at midnight', 'mono', 8, '#2b2b30', 72) + '</g>')
    css = ('.wl,.wr{transform-box:fill-box}.wl{transform-origin:50% 100%}.wr{transform-origin:50% 100%}'
           '.open .wl{animation:fl .5s ease-in-out 4 alternate}.open .wr{animation:fl .5s ease-in-out 4 alternate}@keyframes fl{to{transform:scaleY(.35)}}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(thread + wings + crane + tag_, cls='cr', at=(300, 270), rot=4, scale=.9), motion=[('', '.wl,.wr'), swing('.cr', '50% 0%', 5)])


def rangoli_plate(sid, title, *, batch, palette, name, verbs=('Draw the border', 'Sweep it up'), key=None, visit=1):
    """A steel plate of rangoli colours in little heaps, and the lotus border we raced to finish traced in white."""
    heaps = [(-120, -6, '#d8483a'), (-60, -26, '#f2b33d'), (0, -34, '#2f9a6a'), (60, -26, '#2f6db5'), (120, -6, '#c2185b'), (-30, 16, '#f6efe0'), (40, 14, '#e46f2a')]
    plate = ('<ellipse cy="10" rx="210" ry="80" fill="#9aa0a6"></ellipse><ellipse cy="4" rx="196" ry="70" fill="#c9ced3"></ellipse>'
             '<ellipse cy="10" rx="160" ry="52" fill="#b9bec4"></ellipse>')
    hp = ''.join(f'<path d="M{x - 40} {y + 20} C{x - 30} {y - 20} {x + 30} {y - 20} {x + 40} {y + 20} Q{x} {y + 30} {x - 40} {y + 20} Z" fill="{c}"></path>' for x, y, c in heaps)
    tile = ('<g transform="translate(0 100)"><path d="M-230 -40 L230 -40 L260 60 L-260 60 Z" fill="#a8693c"></path>'
            '<g fill="none" stroke="#fff8ea" stroke-width="5" stroke-linecap="round">'
            + ''.join(f'<path class="rl" style="--d:{i * .12:.2f}s" pathLength="1" d="M{x} 40 C{x - 22} 20 {x - 12} -6 {x} -20 C{x + 12} -6 {x + 22} 20 {x} 40 Z"></path>' for i, x in enumerate(range(-180, 200, 60)))
            + '</g>' + ''.join(f'<circle class="rd" style="--d:{.8 + i * .05:.2f}s" cx="{x}" cy="52" r="4" fill="#f2b33d"></circle>' for i, x in enumerate(range(-210, 230, 30))) + '</g>')
    lbl = f'<g transform="translate(150 -86) rotate(8)"><rect x="-74" y="-20" width="148" height="40" rx="6" fill="#fbf6ea"></rect>' + text(0, 6, 'rangoli battle', 'hand', 18, '#7a1a2e', 136) + '</g>'
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(tile + plate + hp + lbl, cls='rg', at=(300, 270), rot=-3, scale=.95), motion=[drawon('.rl', .1, .5), appear('.rd'), shake('.rg', 3)])


def question_deck(sid, title, *, batch, palette, name, verbs=('Turn the next one', 'Stack them'), key=None, visit=1):
    """The 36 questions as a fanned deck of cards; touching it turns up the last one."""
    cards = ''
    for i, a in enumerate((-24, -14, -4, 6)):
        cards += (f'<g transform="rotate({a} 0 160)"><rect x="-90" y="-130" width="180" height="250" rx="14" fill="{("#5d3564", "#7a4a80", "#8a5a90", "#fbf4f8")[i]}" stroke="#fbf4f8" stroke-width="5"></rect>'
                  + '</g>')
    top = (f'<g class="tc"><g transform="rotate(6 0 160)"><rect x="-90" y="-130" width="180" height="250" rx="14" fill="#fbf4f8" stroke="#5d3564" stroke-width="3"></rect>'
           + text(0, -80, 'No. 36', 'display', 34, '#5d3564', 150, weight=800)
           + ''.join(text(0, -30 + i * 26, r, 'hand', 17, '#3a1430', 150) for i, r in enumerate(['Share a problem', 'and ask how they', 'would handle it.']))
           + text(0, 92, 'then: four minutes, eyes only', 'mono', 9, '#5d3564', 160) + '</g></g>')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit,
          svg=g(cards + top, at=(300, 280), rot=-2), motion=[slide('.tc', 150, -30, 18, .1, .6)])


def house_scroll(sid, title, *, batch, palette, name, verbs=('Unroll it', 'Roll it up'), key=None, visit=1):
    """Five house rules on a scroll, signed by both of us; touching it unrolls to the signatures."""
    rules = ['1. shoes off, always', '2. last chai is shared', '3. no phones at dinner', '4. sorry before sleep', '5. Sunday is ours']
    paper_ = '#f3e6c8'
    sheet = (f'<clipPath id="hr-c"><rect class="hrc" x="-150" y="-150" width="300" height="190"></rect></clipPath>'
             f'<g clip-path="url(#hr-c)"><rect x="-150" y="-150" width="300" height="330" fill="{paper_}"></rect>'
             + text(0, -110, 'House Rules', 'display', 30, '#5a2d16', 260, weight=700)
             + ''.join(text(-120, -70 + i * 30, r, 'hand', 18, '#3a2414', 250, 'start') for i, r in enumerate(rules))
             + '<path d="M-120 140 c20 -16 30 6 46 -6 s20 -10 30 4 M40 140 c10 -20 26 -2 34 -12 c8 -8 16 6 30 -4" fill="none" stroke="#2a3f7a" stroke-width="3"></path>'
             + text(-70, 164, 'me', 'mono', 10, '#5a2d16', 60) + text(70, 164, 'you', 'mono', 10, '#5a2d16', 60) + '</g>')
    roll = lambda y, cls='': (f'<g{f" class={chr(34)}{cls}{chr(34)}" if cls else ""}><rect x="-166" y="{y - 18}" width="332" height="36" rx="18" fill="#e3cfa4"></rect>'
                              f'<rect x="-166" y="{y - 6}" width="332" height="6" fill="#fff" opacity=".3"></rect><circle cx="-170" cy="{y}" r="12" fill="#7a3a1a"></circle><circle cx="170" cy="{y}" r="12" fill="#7a3a1a"></circle></g>')
    seal = '<g class="wax" transform="translate(110 150)"><circle r="22" fill="#a8321f"></circle>' + text(0, 6, 'us', 'serif', 16, '#fbe6d6', 30) + '</g>'
    css = ('.hrc{transition:height .4s}.open .hrc{height:340px;transition:height .8s ease-out .1s}'
           '.bot{transform-box:view-box;transition:transform .4s}.open .bot{transform:translateY(150px);transition:transform .8s ease-out .1s}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(sheet + roll(-150) + roll(48, 'bot') + seal, at=(300, 250), rot=-4), motion=[('', '.hrc,.bot'), appear('.wax', .8)])


def tape_measure(sid, title, *, batch, palette, name, verbs=('Measure the wall', 'Let it snap back'), key=None, visit=1):
    """A tape measure from rearranging the room; touching it pulls the tape out to where the sofa went."""
    case = ('<rect x="-110" y="-110" width="220" height="220" rx="60" fill="#f2c230"></rect><circle r="64" fill="#2b2b30"></circle><circle r="54" fill="#f2c230" opacity=".9"></circle>'
            '<circle r="20" fill="#2b2b30"></circle><rect x="96" y="40" width="34" height="60" rx="8" fill="#2b2b30"></rect>')
    tape = ('<clipPath id="tm-c"><rect class="tmc" x="110" y="66" width="40" height="40"></rect></clipPath><g clip-path="url(#tm-c)"><rect x="110" y="70" width="330" height="34" fill="#f6e08a"></rect>'
            + ''.join(f'<path d="M{120 + i * 10} 70 v{12 if i % 5 == 0 else 6}" stroke="#2b2b30" stroke-width="1.5"></path>' for i in range(32))
            + text(280, 98, 'sofa: 210 cm', 'mono', 13, '#2b2b30', 200, weight=700) + '</g>'
            + '<rect class="hook" x="146" y="64" width="8" height="46" fill="#9aa0a6"></rect>')
    note = (f'<g transform="translate(-10 0) rotate(-6)"><rect x="-80" y="-34" width="160" height="68" rx="4" fill="#fbf6ea"></rect>'
            + text(0, -6, 'rearranged!', 'hand', 22, '#2b2b30', 150) + text(0, 18, 'bed by the window', 'mono', 10, '#2b2b30', 150) + '</g>')
    css = ('.tmc{transition:width .3s}.open .tmc{width:330px;transition:width .9s cubic-bezier(.3,1.2,.5,1) .1s}'
           '.hook{transform-box:view-box;transition:transform .3s}.open .hook{transform:translateX(290px);transition:transform .9s cubic-bezier(.3,1.2,.5,1) .1s}')
    board(sid, title, batch=batch, palette=palette, name=name, verbs=verbs, key=key, visit=visit, css=css,
          svg=g(case + tape + note, at=(200, 290), rot=-5), motion=[('', '.tmc,.hook')])
