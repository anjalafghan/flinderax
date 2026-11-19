"""Shared board frame, in the KOKO board's structure."""
import random, math, os
from string import Template

OUT = os.path.join(os.path.dirname(__file__), 'boards')  # board html for the canvas (not committed)
FIG = "family=Newsreader:ital,opsz,wght@0,6..72,400..800;1,6..72,400..600&amp;family=Figtree:wght@500..700"

FRAME = Template('''<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>$title</title>
<script src="./support.js"></script>
</head>
<body>
<x-dc>
<helmet>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?$fonts&amp;$fig&amp;display=swap">
<style>
body{margin:0}
.scene{display:block;padding:0;border:0;background:none;cursor:pointer;filter:$shadow}
.scene:focus-visible,.cta:focus-visible{outline:2px solid $focus;outline-offset:6px;border-radius:8px}
.scene svg{display:block;overflow:visible}
$css
.cta{justify-self:start;min-height:44px;padding:0 20px;border:1.5px solid $ctabg;border-radius:999px;background:$ctabg;color:$ctafg;font:600 15px/1 'Figtree',sans-serif;cursor:pointer}
.cta:hover{background:$ctahover}
</style>
</helmet>
<div style="width: 960px; height: 700px; box-sizing: border-box; display: grid; grid-template-columns: 600px minmax(0, 1fr); align-items: center; gap: 32px; padding: 40px 56px 40px 40px; background: $bg; font-family: 'Figtree', system-ui, sans-serif; color: $text;">

<button type="button" class="{{sceneCls}}" onClick="{{toggle}}" aria-expanded="{{open}}" aria-label="$label {{ctaLabel}}.">
<svg viewBox="0 0 600 600" width="600" height="600" aria-hidden="true">
$svg
</svg>
</button>

<div style="display: grid; gap: 16px; max-width: 30ch;">
<h1 style="margin: 0; font: 700 36px/1.05 'Newsreader', Georgia, serif; color: $head; text-wrap: balance;">$h1</h1>
<p style="margin: 0; font: italic 400 19px/1.45 'Newsreader', Georgia, serif; color: $text;">$p</p>
<button type="button" class="cta" onClick="{{toggle}}" aria-expanded="{{open}}">{{ctaLabel}}</button>
</div>
</div>
</x-dc>
<script type="text/x-dc" data-dc-script data-props='{"$$preview":{"width":960,"height":700}}'>
class Component extends DCLogic {
  constructor(props) { super(props); this.state = { open: false }; }
  renderVals() {
    const open = this.state.open;
    return {
      open: open ? 'true' : 'false',
      sceneCls: 'scene' + (open ? ' open' : ''),
      ctaLabel: open ? '$closed' : '$opened',
      toggle: () => this.setState({ open: !open }),
    };
  }
}
</script>
</body>
</html>
''')


def grain(fid, r=.2, g=.12, b=.05, a=.16, freq=.85):
    return (f'<filter id="{fid}" x="0" y="0" width="100%" height="100%"><feTurbulence type="fractalNoise" baseFrequency="{freq}" numOctaves="2" stitchTiles="stitch"></feTurbulence>'
            f'<feColorMatrix values="0 0 0 0 {r}  0 0 0 0 {g}  0 0 0 0 {b}  0 0 0 {a} 0"></feColorMatrix><feComposite in2="SourceGraphic" operator="in"></feComposite></filter>')


def torn(corners, seed=1, amp=3.5, step=12):
    """A polygon with a hand-torn edge."""
    rnd = random.Random(seed)
    pts = []
    n = len(corners)
    for i in range(n):
        (x0, y0), (x1, y1) = corners[i], corners[(i + 1) % n]
        L = math.hypot(x1 - x0, y1 - y0); k = max(1, int(L / step))
        nx, ny = -(y1 - y0) / L, (x1 - x0) / L
        for j in range(k):
            t = j / k; d = 0 if j == 0 else rnd.uniform(-amp, amp)
            pts.append((x0 + (x1 - x0) * t + nx * d, y0 + (y1 - y0) * t + ny * d))
    return 'M' + ' L'.join(f'{x:.1f} {y:.1f}' for x, y in pts) + ' Z'


def write(name, **kw):
    kw.setdefault('fig', FIG)
    kw.setdefault('shadow', 'drop-shadow(0 2px 2px rgba(20,14,8,.3)) drop-shadow(0 16px 22px rgba(20,14,8,.26))')
    os.makedirs(OUT, exist_ok=True)
    html = FRAME.substitute(**kw)
    open(os.path.join(OUT, name), 'w').write(html)
    return html
