"""Batch 2, boards 17-21: go-karting, horse riding, shooting range, rage room, Snow World."""
import math
from frame import write, grain, torn

RM = '@media (prefers-reduced-motion:reduce){{{sel}{{transition-duration:.01s!important;transition-delay:0s!important;animation:none!important}}}}'

# ── 17. Go-karting at Republic of Karting: the lap-time printout, chequered flag on top ───────
CHK = ''.join(f'<rect x="{x*14}" y="{y*14}" width="14" height="14" fill="#16161a"></rect>' for y in range(2) for x in range(16) if (x + y) % 2 == 0)
laps = [('1', '58.41', '61.02'), ('2', '55.87', '57.30'), ('3', '54.92', '55.11'), ('4', '53.66', '54.80'), ('5', '54.10', '53.95')]
rows = ''.join(f'<g class="lap" style="--d:{.35 + i*.18}s"><text x="20" y="{150 + i*30}">lap {n}</text><text x="118" y="{150 + i*30}">{a}</text><text x="170" y="{150 + i*30}">{b}</text></g>' for i, (n, a, b) in enumerate(laps))
REC = torn([(0, 28), (224, 28), (224, 380), (0, 380)], seed=21, amp=2.4, step=9)
svg = f'''<defs>{grain('k-g', .2, .2, .2, .1)}<clipPath id="k-feed"><rect x="0" y="0" width="260" height="420"></rect></clipPath></defs>
<!-- the printer it came out of -->
<g transform="translate(150 70)"><rect x="-20" y="-20" width="264" height="70" rx="12" fill="#2b2d33"></rect><rect x="0" y="22" width="224" height="8" rx="4" fill="#0e0f12"></rect>
<text x="10" y="6" font-family="'Space Mono', monospace" font-size="10" fill="#8a8d95">timing</text><circle class="busy" cx="222" cy="0" r="5" fill="#3f3"></circle></g>
<g transform="translate(150 96)" clip-path="url(#k-feed)"><g class="paper">
<path d="{REC}" fill="#fbfaf6"></path><path d="{REC}" filter="url(#k-g)"></path>
<g class="flag" transform="translate(0 28)">{CHK}</g>
<g font-family="'Space Mono', monospace" fill="#2b2b30">
<text x="112" y="90" font-family="'Young Serif', serif" font-size="19" text-anchor="middle">Republic of Karting</text>
<text x="112" y="108" font-size="9.5" text-anchor="middle">Navghar · night session</text>
<text x="118" y="128" font-size="10" font-weight="700">me</text><text x="170" y="128" font-size="10" font-weight="700">you</text>
<g font-size="12">{rows}</g>
<path d="M14 300 H210" stroke="#999" stroke-dasharray="3 3"></path>
<text x="20" y="324" font-size="12" font-weight="700">best lap</text><text x="206" y="324" font-size="12" font-weight="700" text-anchor="end">53.66</text></g>
<g class="star" transform="translate(96 318) rotate(-12)"><path d="M0 -20 L6 -6 L21 -6 L9 3 L13 18 L0 9 L-13 18 L-9 3 L-21 -6 L-6 -6 Z" fill="none" stroke="#c0392b" stroke-width="2.4"></path></g>
</g></g>
<g transform="translate(470 470) rotate(-20)"><rect x="-6" y="-120" width="6" height="150" fill="#9aa0a6"></rect><g class="wave"><path d="M0 -120 C30 -132 60 -108 90 -120 V-70 C60 -58 30 -82 0 -70 Z" fill="#fff"></path>
{''.join(f'<rect x="{x*15}" y="{-120 + y*16 + (4 if x % 2 else 0)}" width="15" height="16" fill="#16161a" opacity="{1 if (x+y)%2==0 else 0}"></rect>' for y in range(3) for x in range(6))}</g></g>'''
css = '''
.paper{transform:translateY(-150px);transition:transform .3s}
.lap,.star{opacity:0}
.wave{transform-box:fill-box;transform-origin:0 50%}
.open .paper{transform:none;transition:transform 1.8s steps(14)}
.open .lap{opacity:1;transition:opacity 0s var(--d)}
.open .star{opacity:1;transition:opacity .1s 2s}
.open .busy{animation:bl .2s steps(2) 9}
.open .wave{animation:flap .6s ease-in-out 1.8s 3}
@keyframes bl{50%{opacity:0}}
@keyframes flap{50%{transform:skewY(8deg) scaleX(.9)}}
''' + RM.format(sel='.paper,.lap,.star,.busy,.wave')
write('GoKarting.dc.html', title='Go-karting keepsake', fonts='family=Young+Serif&amp;family=Space+Mono:wght@400;700',
      css=css, svg=svg, focus='#ffd23c', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #d8483a 0%, #b9352a 58%, #92261e 100%)',
      text='#fdeeea', head='#ffffff', ctabg='#16161a', ctafg='#fdeeea', ctahover='#000',
      label='A thermal printout of our lap times from Republic of Karting, the best lap starred, with a chequered flag, from Go-karting.',
      h1='Go-karting at Republic of Karting', p='Night karting on the highway at Navghar. The timing printer settles every argument about who was faster.',
      opened='Print the laps', closed='Tear it off')

# ── 18. Horse riding at D Sports Star: a horseshoe tag in stitched leather ────────────────────
SHOE = 'M-120 -40 C-120 -150 120 -150 120 -40 C120 30 96 90 74 120 L36 108 C58 72 74 20 74 -40 C74 -96 -74 -96 -74 -40 C-74 20 -58 72 -36 108 L-74 120 C-96 90 -120 30 -120 -40 Z'
holes = ''.join(f'<rect x="{x-4}" y="{y-7}" width="8" height="14" rx="2" fill="#3a2414" transform="rotate({r} {x} {y})"></rect>' for x, y, r in [(-96, -10, 8), (-100, -54, 0), (-88, -96, -24), (96, -10, -8), (100, -54, 0), (88, -96, 24)])
svg = f'''<defs>{grain('h-g', .25, .12, .05, .2)}
<linearGradient id="h-leather" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#a3622e"></stop><stop offset="1" stop-color="#6e3b18"></stop></linearGradient></defs>
<path d="M300 40 V120" stroke="#3a3a3a" stroke-width="2"></path><circle cx="300" cy="40" r="6" fill="#777"></circle>
<g class="shoe"><g transform="translate(300 280)">
<path d="{SHOE}" fill="url(#h-leather)"></path><path d="{SHOE}" filter="url(#h-g)"></path>
<path d="{SHOE}" fill="none" stroke="#f0d8a8" stroke-width="2" stroke-dasharray="6 5" transform="scale(.9) translate(0 -4)"></path>
{holes}
<path d="M0 -150 V-120" stroke="#3a3a3a" stroke-width="2"></path><circle cy="-118" r="7" fill="none" stroke="#c9a14a" stroke-width="3"></circle>
<path id="h-arc" d="M-97 -30 C-97 -122 97 -122 97 -30" fill="none"></path>
<text font-family="'Young Serif', serif" font-size="15" fill="#fff1d2" dy="5"><textPath href="#h-arc" startOffset="50%" text-anchor="middle">trial ride at D Sports Star, Nerul</textPath></text>
<path d="M96 -10 C104 30 110 60 116 86" stroke="#c9a14a" stroke-width="1.6" fill="none"></path>
<g transform="translate(116 86) rotate(-8)"><path d="M-6 0 H112 V46 H-6 Z" fill="#fbf4e2"></path><circle cx="2" cy="8" r="3" fill="#c9a14a"></circle>
<text x="54" y="20" font-family="Kalam, cursive" font-size="14" fill="#5b3a1f" text-anchor="middle">her name was</text><path d="M10 36 H100" stroke="#5b3a1f" stroke-dasharray="3 3"></path></g>
</g></g>
<g class="hoof" fill="#5b3a1f">{''.join(f'<path class="hp" style="--d:{.5 + i*.22}s" d="M{80 + i*70} {520 - (i%2)*24} c-10 0 -14 -16 -6 -22 c6 -4 18 -4 24 0 c8 6 4 22 -6 22 c-2 -6 -10 -6 -12 0 Z"></path>' for i in range(7))}</g>'''
css = '''
.shoe{transform-box:view-box;transform-origin:300px 40px}
.hp{opacity:0}
.open .shoe{animation:swing 2.2s ease-in-out both}
.open .hp{animation:step .3s ease-out var(--d) both}
@keyframes swing{0%,100%{transform:rotate(0)}20%{transform:rotate(10deg)}45%{transform:rotate(-8deg)}70%{transform:rotate(4deg)}88%{transform:rotate(-2deg)}}
@keyframes step{0%{opacity:0;transform:scale(.4)}100%{opacity:.75;transform:none}}
''' + RM.format(sel='.shoe,.hp')
write('HorseRiding.dc.html', title='Horse riding keepsake', fonts='family=Young+Serif&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f6e4c0', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #c9b48a 0%, #b09a6e 58%, #8f7b52 100%)',
      text='#2b1d0e', head='#1c1206', ctabg='#5b3a1f', ctafg='#f6e4c0', ctahover='#472c16',
      label='A horseshoe-shaped stitched leather tag from a trial ride at D Sports Star in Nerul, from Horse riding.',
      h1='Horse riding', p='A morning trial ride near Nerul station: a stitched leather horseshoe, and a card on a thread for the horse’s name.',
      opened='Give it a swing', closed='Hang it up')

# ── 19. Shooting range: the paper target, holes near the bull ───────────────────────────────
rings = ''.join(f'<circle r="{r}" fill="{"#16161a" if i >= 3 else "#f4efe2"}" stroke="{"#f4efe2" if i >= 3 else "#16161a"}" stroke-width="1.6"></circle>' for i, r in enumerate([150, 125, 100, 75, 50, 25]))
nums = ''.join(f'<text x="{r - 12}" y="5" font-family="\'Space Mono\', monospace" font-size="10" fill="{"#f4efe2" if i >= 3 else "#16161a"}" text-anchor="middle">{i+5}</text>' for i, r in enumerate([150, 125, 100, 75, 50]))
shots = [(-8, 6), (14, -10), (4, 18), (-20, -16), (26, 22)]
holes = ''.join(f'<g transform="translate({x} {y})"><g class="hole" style="--d:{.3 + i*.3}s"><circle r="8.5" fill="#f4efe2" opacity=".9"></circle><circle r="6" fill="#3a2a14"></circle><circle r="6" fill="none" stroke="#a89878" stroke-width="1.2"></circle></g></g>' for i, (x, y) in enumerate(shots))
svg = f'''<defs>{grain('sr-g', .2, .2, .15, .12)}</defs>
<path d="M60 70 H540" stroke="#555" stroke-width="3"></path>
<g class="tgt"><g transform="translate(300 300)">
<path d="M-170 -210 H170 V200 L150 220 H-170 Z" fill="#efe8d6"></path><path d="M-170 -210 H170 V200 L150 220 H-170 Z" filter="url(#sr-g)"></path>
<path d="M150 220 L170 200 L150 200 Z" fill="#d6ccb3"></path>
{rings}{nums}
<path d="M-160 0 H160 M0 -160 V160" stroke="#7a7466" stroke-width=".8" stroke-dasharray="4 4"></path>
{holes}
<text x="-160" y="-186" font-family="'Young Serif', serif" font-size="17" fill="#16161a">Champions Shooting Range</text>
<text x="-160" y="-168" font-family="'Space Mono', monospace" font-size="9" fill="#5d564a">Chandivali, Powai · 10 m air</text>
<text class="score" x="120" y="194" font-family="Kalam, cursive" font-size="26" fill="#c0392b" text-anchor="end" transform="rotate(-6 120 194)">48 / 50</text>
<g transform="translate(-120 -230)"><rect x="-18" y="0" width="36" height="22" rx="3" fill="#9aa0a6"></rect></g>
<g transform="translate(120 -230)"><rect x="-18" y="0" width="36" height="22" rx="3" fill="#9aa0a6"></rect></g>
</g></g>'''
css = '''
.tgt{transform-box:view-box;transform-origin:300px 70px}
.hole{opacity:0;transform-box:fill-box;transform-origin:50% 50%}
.score{opacity:0}
.open .hole{animation:hit .12s ease-out var(--d) both}
.open .tgt{animation:jolt .25s ease-out .3s 5}
.open .score{opacity:1;transition:opacity .3s 2s}
@keyframes hit{from{opacity:0;transform:scale(2.4)}to{opacity:1;transform:scale(1)}}
@keyframes jolt{40%{transform:rotate(1.4deg)}80%{transform:rotate(-.6deg)}}
''' + RM.format(sel='.tgt,.hole,.score')
write('ShootingRange.dc.html', title='Shooting range keepsake', fonts='family=Young+Serif&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#efe8d6', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #5d6a58 0%, #4a5646 58%, #364033 100%)',
      text='#eef1ea', head='#ffffff', ctabg='#efe8d6', ctafg='#1d211b', ctahover='#ddd3bb',
      label='A paper target from Champions Shooting Range with five holes close round the bull and a pencilled score, from Shooting range.',
      h1='Shooting range', p='Ten metres at Chandivali, a coach who said breathe out first, and a sheet we were allowed to keep.',
      opened='Take the shots', closed='New sheet')

# ── 20. Rage room: the hazard-striped waiver and a plate that doesn't make it ─────────────────
stripes = ''.join(f'<path d="M{x} 0 l20 0 l-28 28 l-20 0 Z" fill="#16161a"></path>' for x in range(0, 360, 40))
shards = [('M0 0 L-60 -20 L-40 30 Z', -90, -70, -40), ('M0 0 L-40 30 L10 50 Z', -60, 80, 30), ('M0 0 L10 50 L56 24 Z', 70, 90, 60), ('M0 0 L56 24 L60 -30 Z', 100, -40, -50), ('M0 0 L60 -30 L-10 -56 Z', 40, -110, 20), ('M0 0 L-10 -56 L-60 -20 Z', -40, -120, -30)]
shard_svg = ''.join(f'<g class="shard" style="--x:{x}px;--y:{y}px;--r:{r}deg"><path d="{d}" fill="#f6f3ec" stroke="#2f5fa8" stroke-width="2"></path></g>' for d, x, y, r in shards)
svg = f'''<defs>{grain('rr-g', .2, .2, .1, .12)}<clipPath id="rr-band"><rect width="360" height="28"></rect></clipPath></defs>
<g transform="translate(110 130) rotate(-3)">
<path d="M0 0 H360 V380 L330 400 H0 Z" fill="#f2c230"></path><path d="M0 0 H360 V380 L330 400 H0 Z" filter="url(#rr-g)"></path>
<g clip-path="url(#rr-band)">{stripes}</g><g transform="translate(0 352)" clip-path="url(#rr-band)">{stripes}</g>
<text x="24" y="70" font-family="Fraunces, serif" font-weight="700" font-size="30" fill="#16161a">Mad Rage Room</text>
<text x="24" y="92" font-family="'Space Mono', monospace" font-size="10" fill="#16161a">Powai · waiver · 16+ · gear on</text>
<g font-family="'Space Mono', monospace" font-size="11" fill="#16161a"><text x="24" y="132">I will wear the helmet.</text><text x="24" y="152">I will not throw toward the other person.</text><text x="24" y="172">I will feel much better after.</text>
<text x="24" y="212">plates broken:</text></g>
<text x="150" y="214" font-family="'Permanent Marker', cursive" font-size="24" fill="#c0392b">lots</text>
<path d="M24 290 H160 M190 290 H330" stroke="#16161a"></path>
<path d="M30 284 c20 -20 30 10 44 -6 s20 -10 30 4 s16 -6 40 -4" fill="none" stroke="#1f43b5" stroke-width="2"></path>
<path d="M196 284 c14 -14 24 6 40 -10 c10 10 20 -2 30 8 c10 -8 30 -2 50 -4" fill="none" stroke="#1f43b5" stroke-width="2"></path>
<path class="crack" pathLength="1" d="M360 120 L300 150 L320 190 L250 230 L270 270 L200 320 L230 360" fill="none" stroke="#16161a" stroke-width="2.4"></path>
</g>
<g transform="translate(450 470)"><g class="plate"><circle r="70" fill="#f6f3ec"></circle><circle r="54" fill="none" stroke="#2f5fa8" stroke-width="3"></circle><circle r="70" fill="none" stroke="#c9c4b8" stroke-width="2"></circle></g>
<g class="shards">{shard_svg}</g></g>'''
css = '''
.plate{transform-box:fill-box;transform-origin:50% 50%}
.shards{opacity:0}
.crack{stroke-dasharray:1;stroke-dashoffset:1}
.open .plate{animation:smash .5s ease-in both}
.open .shards{opacity:1;transition:opacity 0s .45s}
.open .shard{animation:fly 1s cubic-bezier(.2,.7,.3,1) .45s both}
.open .crack{stroke-dashoffset:0;transition:stroke-dashoffset .5s ease-out .5s}
@keyframes smash{0%{transform:none;opacity:1}80%{transform:translate(-20px,30px) scale(1.1);opacity:1}100%{transform:translate(-20px,30px) scale(1.1);opacity:0}}
@keyframes fly{from{transform:translate(-20px,30px)}to{transform:translate(var(--x),calc(var(--y) - 40px)) rotate(var(--r))}}
''' + RM.format(sel='.plate,.shards,.shard,.crack')
write('RageRoom.dc.html', title='Rage room keepsake', fonts='family=Fraunces:opsz,wght@9..144,700&amp;family=Space+Mono&amp;family=Permanent+Marker',
      css=css, svg=svg, focus='#f2c230', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #4a4f57 0%, #383c43 58%, #26292e 100%)',
      text='#eceef1', head='#ffffff', ctabg='#f2c230', ctafg='#16161a', ctahover='#e0ae1c',
      label='A yellow and black hazard-striped waiver from Mad Rage Room with two shaky signatures and a plate, from Rage room.',
      h1='Rage room', p='One hour, two helmets, a crate of plates in Powai. We signed the waiver; the waiver did not survive either.',
      opened='Smash it', closed='Sweep up')

# ── 21. Snow World: the frosty token, a mitten, socks mandatory ─────────────────────────────
def flake(r):
    arms = ''.join(f'<g transform="rotate({a})"><path d="M0 0 V{-r} M0 {-r*.55} l{-r*.22} {-r*.22} M0 {-r*.55} l{r*.22} {-r*.22}"></path></g>' for a in range(0, 360, 60))
    return arms
snow = ''.join(f'<circle class="sn" style="--d:{(i*0.37)%2:.2f}s;--x:{(i*53)%40-20}px" cx="{(i*97)%560+20}" cy="{-(i*31)%80}" r="{2+(i%3)}"></circle>' for i in range(26))
svg = f'''<defs><radialGradient id="sw-tok" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#e9f7ff"></stop><stop offset=".6" stop-color="#9fd3f2"></stop><stop offset="1" stop-color="#5aa4d6"></stop></radialGradient></defs>
<g transform="translate(260 290)">
<ellipse cy="130" rx="110" ry="16" fill="#123049" opacity=".3"></ellipse>
<circle r="120" fill="url(#sw-tok)"></circle><circle r="104" fill="none" stroke="#fff" stroke-width="2" stroke-dasharray="2 6"></circle>
<g stroke="#fff" stroke-width="5" stroke-linecap="round" fill="none" opacity=".9">{flake(54)}</g>
<g stroke="#3d7fb3" stroke-width="2" stroke-linecap="round" fill="none" opacity=".5" transform="translate(1.5 1.5)">{flake(54)}</g>
<path id="sw-arc" d="M-92 0 A92 92 0 0 1 92 0" fill="none"></path>
<text font-family="'Young Serif', serif" font-size="19" fill="#1e4d75"><textPath href="#sw-arc" startOffset="50%" text-anchor="middle">Snow World</textPath></text>
<text y="78" font-family="'Space Mono', monospace" font-size="10" fill="#1e4d75" text-anchor="middle">Phoenix Marketcity</text><text y="94" font-family="'Space Mono', monospace" font-size="10" fill="#1e4d75" text-anchor="middle">socks mandatory</text>
<circle class="frost" r="120" fill="none" stroke="#fff" stroke-width="10" stroke-dasharray="2 9" opacity="0"></circle>
</g>
<g class="mitt"><g transform="translate(470 420) rotate(18)">
<path d="M-40 60 V-10 C-40 -50 -20 -70 6 -70 C30 -70 44 -50 44 -20 V60 Z" fill="#d6455d"></path>
<path d="M-40 -6 C-62 -14 -72 10 -60 26 C-52 36 -44 30 -40 22 Z" fill="#d6455d"></path>
<rect x="-46" y="52" width="96" height="30" rx="6" fill="#f6f1e6"></rect>
<g stroke="#f6f1e6" stroke-width="3" fill="none"><path d="M-20 -30 l8 8 l8 -8 l8 8 l8 -8"></path><path d="M-20 0 l8 8 l8 -8 l8 8 l8 -8"></path></g></g></g>
<g class="snow" fill="#fff">{snow}</g>'''
css = '''
.sn{opacity:0}
.mitt{transform-box:fill-box;transform-origin:50% 100%}
.open .sn{animation:fall 2.6s linear var(--d) 2}
.open .frost{opacity:.85;transition:opacity 1.2s .6s}
.open .mitt{animation:waveh .5s ease-in-out .3s 4}
@keyframes fall{0%{opacity:0;transform:translate(0,0)}15%{opacity:1}100%{opacity:.2;transform:translate(var(--x),640px)}}
@keyframes waveh{50%{transform:rotate(-16deg)}}
''' + RM.format(sel='.sn,.frost,.mitt')
write('SnowWorld.dc.html', title='Snow World keepsake', fonts='family=Young+Serif&amp;family=Space+Mono',
      css=css, svg=svg, focus='#e9f7ff', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #3c6f9a 0%, #2c5679 58%, #1f3f5a 100%)',
      text='#eaf4fb', head='#ffffff', ctabg='#e9f7ff', ctafg='#123049', ctahover='#cfe9f8',
      label='A frosty blue Snow World token with a snowflake and one red mitten, from Snow World.',
      h1='Snow World', p='Minus something in Kurla on a thirty-degree afternoon. Socks were mandatory; the mitten was a bonus.',
      opened='Let it snow', closed='Warm up')
