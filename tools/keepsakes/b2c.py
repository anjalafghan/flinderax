"""Batch 2, boards 11, 12, 14, 15, 16: glass painting, bookstore dares, arcade, gaming café, VR."""
import math
from frame import write, grain, torn

RM = '@media (prefers-reduced-motion:reduce){{{sel}{{transition-duration:.01s!important;transition-delay:0s!important;animation:none!important}}}}'

# ── 11. Glass painting workshop: a leaded suncatcher, our initials in one pane ───────────────
cols = ['#c8312b', '#2f6db5', '#e8b923', '#3b9a5a', '#8a3fa0', '#e46f2a', '#2aa3a8', '#d94f8a']
panes = ''
glows = ''
for i in range(8):
    a0, a1 = math.radians(i * 45 - 90), math.radians((i + 1) * 45 - 90)
    p = f'M0 0 L{110*math.cos(a0):.1f} {110*math.sin(a0):.1f} A110 110 0 0 1 {110*math.cos(a1):.1f} {110*math.sin(a1):.1f} Z'
    panes += f'<path d="{p}" fill="{cols[i]}" fill-opacity=".78"></path>'
    am = (a0 + a1) / 2
    glows += f'<ellipse cx="{300 + 190*math.cos(am):.0f}" cy="{470 + 50*math.sin(am):.0f}" rx="34" ry="14" fill="{cols[i]}" opacity=".55"></ellipse>'
svg = f'''<defs><radialGradient id="g-sheen" cx=".3" cy=".25" r=".8"><stop offset="0" stop-color="#fff" stop-opacity=".55"></stop><stop offset=".4" stop-color="#fff" stop-opacity="0"></stop></radialGradient>
<filter id="g-blur"><feGaussianBlur stdDeviation="8"></feGaussianBlur></filter></defs>
<g class="patches" filter="url(#g-blur)">{glows}</g>
<path d="M300 0 V110" stroke="#e9e1cf" stroke-width="2"></path>
<g class="swing"><g transform="translate(300 250)">
<circle cy="-128" r="14" fill="none" stroke="#9a9ea4" stroke-width="5"></circle>
{panes}
<circle r="46" fill="#f3efe2" fill-opacity=".92"></circle>
<text y="12" font-family="'Young Serif', serif" font-size="34" fill="#3a2a5a" text-anchor="middle">A&amp;D</text>
<g fill="none" stroke="#3b3d42" stroke-width="5" stroke-linejoin="round"><circle r="110"></circle><circle r="46"></circle>
{''.join(f'<path d="M{46*math.cos(math.radians(i*45-90)):.1f} {46*math.sin(math.radians(i*45-90)):.1f} L{110*math.cos(math.radians(i*45-90)):.1f} {110*math.sin(math.radians(i*45-90)):.1f}"></path>' for i in range(8))}</g>
<circle r="110" fill="url(#g-sheen)"></circle>
<path class="glint" d="M-70 -60 L-40 -84" stroke="#fff" stroke-width="5" stroke-linecap="round" opacity="0"></path>
</g></g>
<g transform="translate(380 470) rotate(6)"><path d="M0 0 H180 L190 30 L180 60 H0 Z" fill="#fbf6ea"></path><text x="90" y="26" font-family="Kalam, cursive" font-size="17" fill="#3a2a5a" text-anchor="middle">Glass Painting</text><text x="90" y="46" font-family="'Space Mono', monospace" font-size="9" fill="#6a5a7a" text-anchor="middle">workshop · Vashi</text></g>'''
css = '''
.swing{transform-box:view-box;transform-origin:300px 110px}
.patches{opacity:.15;transition:opacity .4s}
.open .swing{animation:sway 2.6s ease-in-out both}
.open .patches{opacity:1;transition:opacity 1s .2s}
.open .patches ellipse{animation:drift 2.6s ease-in-out both}
.open .glint{animation:glint .8s ease-out .5s both}
@keyframes sway{0%,100%{transform:rotate(0)}20%{transform:rotate(9deg)}45%{transform:rotate(-7deg)}70%{transform:rotate(4deg)}88%{transform:rotate(-2deg)}}
@keyframes drift{20%{transform:translateX(-26px)}45%{transform:translateX(20px)}70%{transform:translateX(-10px)}}
@keyframes glint{0%{opacity:0}40%{opacity:1}100%{opacity:0;transform:translate(60px,30px)}}
''' + RM.format(sel='.swing,.patches,.patches ellipse,.glint')
write('GlassPainting.dc.html', title='Glass painting workshop keepsake', fonts='family=Young+Serif&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f3efe2', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #e8dcc4 0%, #d4c6aa 58%, #b9a98a 100%)',
      text='#2a2238', head='#1c1628', ctabg='#3a2a5a', ctafg='#f3efe2', ctahover='#2c1f46',
      label='A round leaded-glass suncatcher in eight colours with our initials painted in the middle, from Glass painting workshop.',
      h1='Glass painting workshop', p='Eight panes, a steady hand for the lead lines, our initials in the middle. It throws colour on the wall when the sun hits it.',
      opened='Catch the light', closed='Let it settle')

# ── 12. Bookstore Roadies task: a red book under ₹300, and the due-date card of dares ─────────
dares = ['a red cover under ₹300', 'read page 42 aloud', 'a book older than us', 'one to swap, wrapped']
rows = ''
for i, d in enumerate(dares):
    y = 108 + i * 44
    rows += f'<text x="20" y="{y}" font-family="Kalam, cursive" font-size="16" fill="#2b2b30">{d}</text><path d="M14 {y+12} H250" stroke="#b9c3d6"></path>'
    rows += f'<g class="stamp" style="--d:{.7 + i * .35}s" transform="translate(212 {y-6}) rotate({-10 + i*5})"><rect x="-34" y="-13" width="68" height="24" rx="3" fill="none" stroke="#6a2c8c" stroke-width="2"></rect><text y="5" font-family="\'Space Mono\', monospace" font-weight="700" font-size="12" fill="#6a2c8c" text-anchor="middle">DONE</text></g>'
svg = f'''<defs>{grain('bk-g', .2, .05, .05, .14)}
<linearGradient id="bk-cover" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8f1d1d"></stop><stop offset=".08" stop-color="#b52a26"></stop><stop offset="1" stop-color="#a32321"></stop></linearGradient></defs>
<g transform="translate(110 120) rotate(-6)">
<rect width="300" height="400" rx="6" fill="url(#bk-cover)"></rect><rect width="300" height="400" rx="6" filter="url(#bk-g)"></rect>
<path d="M18 0 V400" stroke="#6e1414" stroke-width="3"></path>
<text x="160" y="80" font-family="Fraunces, serif" font-style="italic" font-size="34" fill="#f4e3c4" text-anchor="middle">a red one</text>
<text x="160" y="110" font-family="'Space Mono', monospace" font-size="11" fill="#f4e3c4" text-anchor="middle">under ₹300, as dared</text>
<g transform="translate(236 30) rotate(12)"><circle r="26" fill="#f2c230"></circle><text y="6" font-family="'Space Mono', monospace" font-weight="700" font-size="14" fill="#6e1414" text-anchor="middle">₹299</text></g>
<!-- the library pocket -->
<path d="M40 250 H280 V390 H40 Z" fill="#efe1c0"></path><path d="M40 250 C110 270 210 270 280 250" fill="none" stroke="#c9b48a" stroke-width="2"></path>
</g>
<g class="card"><g transform="translate(130 300) rotate(-4)">
<path d="M0 0 H270 V300 H0 Z" fill="#fbfaf4"></path>
<text x="20" y="36" font-family="'Young Serif', serif" font-size="20" fill="#2b2b30">Crossword, Vashi</text>
<text x="20" y="58" font-family="'Space Mono', monospace" font-size="9.5" fill="#6b6b75">dare · found it · stamped</text>
<path d="M14 70 H256" stroke="#c0392b" stroke-width="1.5"></path>{rows}
</g></g>
<g transform="translate(110 120) rotate(-6)"><path d="M40 300 H280 V400 H40 Z" fill="#efe1c0"></path>
<text x="160" y="352" font-family="'Space Mono', monospace" font-size="11" fill="#7a6440" text-anchor="middle">date due</text></g>'''
css = '''
.card{transform-box:fill-box;transition:transform .4s ease-in}
.stamp{opacity:0;transform-box:fill-box;transform-origin:50% 50%}
.open .card{transform:translate(70px,-250px) rotate(6deg);transition:transform .7s cubic-bezier(.3,1.2,.5,1)}
.open .stamp{animation:thunk .25s ease-out var(--d) both}
@keyframes thunk{0%{opacity:0;scale:1.8}100%{opacity:.9;scale:1}}
''' + RM.format(sel='.card,.stamp')
write('BookstoreRoadies.dc.html', title='Bookstore Roadies task keepsake', fonts='family=Young+Serif&amp;family=Fraunces:ital,opsz,wght@1,9..144,500&amp;family=Kalam&amp;family=Space+Mono:wght@400;700',
      css=css, svg=svg, focus='#efe1c0', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #6a7f5e 0%, #536849 58%, #3e5036 100%)',
      text='#eef3e8', head='#ffffff', ctabg='#efe1c0', ctafg='#2b2b20', ctahover='#e2cfa4',
      label='A red book bought for ₹299 with a due-date card of bookstore dares in its pocket, from Bookstore Roadies task.',
      h1='Bookstore Roadies task', p='Dares set across the shelves at Crossword: a red cover under three hundred, page forty-two read out loud. The card in the pocket keeps score.',
      opened='Pull the card', closed='Tuck it in')

# ── 14. Arcade games at Timezone: the holographic card, the reader, the tickets ─────────────
tickets = ''.join(f'<g transform="translate(0 {i*34})"><rect x="0" y="0" width="70" height="32" fill="#f2c230"></rect><path d="M0 16 a5 5 0 0 0 0 0" ></path><circle cx="0" cy="16" r="5" fill="#2a1f45"></circle><circle cx="70" cy="16" r="5" fill="#2a1f45"></circle><text x="35" y="21" font-family="\'Space Mono\', monospace" font-weight="700" font-size="10" fill="#7a3a00" text-anchor="middle">TICKET</text></g>' for i in range(7))
HEART = ''.join(f'<rect x="{x*6}" y="{y*6}" width="6" height="6"></rect>' for y, row in enumerate(['01100110', '11111111', '11111111', '01111110', '00111100', '00011000']) for x, c in enumerate(row) if c == '1')
svg = f'''<defs>
<linearGradient id="t-holo" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#7ef0ff"></stop><stop offset=".3" stop-color="#c7a4ff"></stop><stop offset=".6" stop-color="#ffb3e6"></stop><stop offset="1" stop-color="#9dffc4"></stop></linearGradient>
<linearGradient id="t-shine" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff" stop-opacity="0"></stop><stop offset=".5" stop-color="#fff" stop-opacity=".7"></stop><stop offset="1" stop-color="#fff" stop-opacity="0"></stop></linearGradient>
<clipPath id="t-cardclip"><path d="M16 0 H250 L300 50 V174 Q300 190 284 190 H16 Q0 190 0 174 V16 Q0 0 16 0 Z M262 150 a10 10 0 1 0 0.1 0 Z" clip-rule="evenodd"></path></clipPath>
<clipPath id="t-out"><rect x="380" y="200" width="100" height="400"></rect></clipPath></defs>
<!-- the reader on the machine -->
<g transform="translate(360 120)"><rect width="140" height="80" rx="12" fill="#2a2a34"></rect><rect x="20" y="30" width="100" height="8" rx="4" fill="#0e0e12"></rect><circle class="led" cx="120" cy="16" r="5" fill="#e33"></circle>
<text x="16" y="66" font-family="'Space Mono', monospace" font-size="10" fill="#9a9aa8">swipe</text></g>
<g clip-path="url(#t-out)"><g class="tickets" transform="translate(395 200)">{tickets}</g></g>
<g class="card"><g transform="translate(60 260) rotate(-8)">
<g clip-path="url(#t-cardclip)"><rect width="300" height="190" fill="url(#t-holo)"></rect>
<rect class="shine" x="-120" y="-40" width="80" height="270" fill="url(#t-shine)" transform="rotate(20)"></rect>
<rect y="132" width="300" height="26" fill="#1b1b22"></rect></g>
<text x="22" y="46" font-family="Fraunces, serif" font-weight="700" font-size="30" fill="#2a1f45">Timezone</text>
<text x="22" y="68" font-family="'Space Mono', monospace" font-size="10" fill="#2a1f45">Inorbit Mall, Vashi · game card</text>
<text x="22" y="114" font-family="'Space Mono', monospace" font-weight="700" font-size="14" fill="#2a1f45">credits: 0</text>
<g transform="translate(220 70)" fill="#e0245e">{HEART}</g>
<text x="232" y="122" font-family="'Space Mono', monospace" font-size="9" fill="#2a1f45">hi-score</text>
</g></g>'''
css = '''
.card,.tickets{transform-box:fill-box}
.tickets{transform:translate(395px,-50px);transition:transform .3s}
.open .card{animation:swipe 1s cubic-bezier(.5,0,.3,1) both}
.open .shine{animation:shine 1.2s ease-out .2s}
.open .led{fill:#3f3;transition:fill 0s .7s}
.open .tickets{transform:translate(395px,200px);transition:transform 1.6s cubic-bezier(.3,.9,.4,1) .8s}
@keyframes swipe{0%{transform:none}40%{transform:translate(330px,-150px) rotate(8deg)}60%{transform:translate(330px,-110px) rotate(8deg)}100%{transform:none}}
@keyframes shine{from{transform:rotate(20deg) translateX(0)}to{transform:rotate(20deg) translateX(460px)}}
''' + RM.format(sel='.card,.tickets,.shine,.led')
write('Arcade.dc.html', title='Arcade games keepsake', fonts='family=Fraunces:opsz,wght@9..144,700&amp;family=Space+Mono:wght@400;700',
      css=css, svg=svg, focus='#9dffc4', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #3a2d6e 0%, #2a2054 58%, #1c153c 100%)',
      text='#ece8fb', head='#ffffff', ctabg='#9dffc4', ctafg='#1c153c', ctahover='#7ff0ad',
      label='A holographic Timezone game card with zero credits left and a pixel heart high score, from Arcade games.',
      h1='Arcade games at Timezone', p='The card we topped up at Inorbit and spent down to nothing. The high score is a heart; the tickets are still coming.',
      opened='Swipe the card', closed='Pocket it')

# ── 15. Gaming café: the session token, a controller, player two joins ───────────────────────
svg = f'''<defs><radialGradient id="gc-tok" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#6fe3a1"></stop><stop offset="1" stop-color="#17804a"></stop></radialGradient></defs>
<!-- the token -->
<ellipse cx="200" cy="420" rx="90" ry="14" fill="#06120c" opacity=".35"></ellipse>
<g class="toss"><g transform="translate(200 300)"><g class="flip">
<circle r="96" fill="url(#gc-tok)" stroke="#0d5a33" stroke-width="4"></circle>
<circle r="80" fill="none" stroke="#c9f5da" stroke-width="2" stroke-dasharray="4 6"></circle>
<g class="fa"><text y="-22" font-family="'Space Mono', monospace" font-size="12" fill="#063a20" text-anchor="middle">Global Gaming Cafe</text>
<text y="26" font-family="Fraunces, serif" font-weight="700" font-size="46" fill="#063a20" text-anchor="middle">PC 07</text>
<text y="52" font-family="'Space Mono', monospace" font-size="11" fill="#063a20" text-anchor="middle">Vashi</text></g>
<g class="fb" transform="scale(1 -1)"><text y="8" font-family="Fraunces, serif" font-weight="700" font-size="40" fill="#063a20" text-anchor="middle">2 hrs</text><text y="36" font-family="'Space Mono', monospace" font-size="11" fill="#063a20" text-anchor="middle">session time</text></g>
</g></g></g>
<!-- the controller -->
<g class="pad"><g transform="translate(410 380) rotate(-12)">
<path d="M-90 -20 C-90 -50 -60 -56 -30 -50 H30 C60 -56 90 -50 90 -20 C96 30 100 70 70 72 C46 74 40 40 20 34 H-20 C-40 40 -46 74 -70 72 C-100 70 -96 30 -90 -20 Z" fill="#23232b"></path>
<path d="M-62 -24 h20 M-52 -34 v20" stroke="#8a8a98" stroke-width="7" stroke-linecap="round"></path>
<circle cx="50" cy="-34" r="7" fill="#e0245e"></circle><circle cx="64" cy="-20" r="7" fill="#2f6db5"></circle><circle cx="36" cy="-20" r="7" fill="#3b9a5a"></circle><circle cx="50" cy="-6" r="7" fill="#e8b923"></circle>
<circle cx="-24" cy="10" r="12" fill="#3a3a44"></circle><circle cx="24" cy="10" r="12" fill="#3a3a44"></circle></g></g>
<g class="p2"><g transform="translate(380 150) rotate(4)"><rect x="-6" y="-6" width="172" height="72" rx="10" fill="#0b0b10"></rect><rect width="160" height="60" rx="6" fill="#0f3a26"></rect>
<text x="80" y="27" font-family="'Space Mono', monospace" font-weight="700" font-size="14" fill="#7dffb2" text-anchor="middle">PLAYER 2</text><text x="80" y="46" font-family="'Space Mono', monospace" font-size="12" fill="#7dffb2" text-anchor="middle">has joined</text></g></g>'''
css = '''
.toss,.flip,.pad,.p2{transform-box:fill-box;transform-origin:50% 50%}
.fb{opacity:0}
.p2{opacity:0;scale:.6;transition:opacity .2s,scale .2s}
.open .toss{animation:tossup 1.1s cubic-bezier(.3,0,.7,1)}
.open .flip{animation:fl .275s linear 4}
.open .fa{animation:fa .55s step-end 2 both}
.open .fb{animation:fb .55s step-end 2 both}
.open .pad{animation:rumble .08s linear 1.2s 10}
.open .p2{opacity:1;scale:1;transition:opacity .2s 1.3s,scale .4s cubic-bezier(.3,1.6,.5,1) 1.3s}
@keyframes tossup{50%{transform:translateY(-160px)}}
@keyframes fl{0%{transform:scaleY(1)}25%{transform:scaleY(0)}50%{transform:scaleY(-1)}75%{transform:scaleY(0)}100%{transform:scaleY(1)}}
@keyframes fa{0%{opacity:1}25%{opacity:0}75%{opacity:1}}
@keyframes fb{0%{opacity:0}25%{opacity:1}75%{opacity:0}}
@keyframes rumble{0%{transform:translate(0,0)}25%{transform:translate(2px,-1px)}50%{transform:translate(-2px,1px)}75%{transform:translate(1px,2px)}}
''' + RM.format(sel='.toss,.flip,.fa,.fb,.pad,.p2')
write('GamingCafe.dc.html', title='Gaming café keepsake', fonts='family=Fraunces:opsz,wght@9..144,700&amp;family=Space+Mono:wght@400;700',
      css=css, svg=svg, focus='#7dffb2', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #2d4a3c 0%, #20372c 58%, #15261e 100%)',
      text='#e6f5ec', head='#ffffff', ctabg='#7dffb2', ctafg='#0b2216', ctahover='#5cf09a',
      label='A green session token for PC 7 at Global Gaming Cafe and a controller, from Gaming café.',
      h1='Gaming café', p='Machine seven at Global Gaming Cafe in Vashi, two hours on the clock, and a second controller for whoever’s losing.',
      opened='Join the game', closed='Log off')

# ── 16. Laser tag & VR at Virtual Adda: a lanyard pass with a visor cut-out, a speedometer ────
svg = f'''<defs>{grain('vr-g', .1, .1, .2, .12)}
<linearGradient id="vr-pass" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ff5a3c"></stop><stop offset="1" stop-color="#c22a4f"></stop></linearGradient>
<linearGradient id="vr-lens" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#0d1430"></stop><stop offset=".5" stop-color="#3a4ea8"></stop><stop offset="1" stop-color="#0d1430"></stop></linearGradient></defs>
<path d="M300 0 C260 60 240 100 252 150 M300 0 C340 60 360 100 348 150" fill="none" stroke="#111" stroke-width="10"></path>
<g class="pass"><g transform="translate(300 150)">
<rect x="-14" y="-6" width="28" height="22" rx="4" fill="#9aa0a6"></rect>
<path d="M-130 40 Q-130 10 -100 10 H100 Q130 10 130 40 V58 H150 Q162 58 162 70 V112 Q162 124 150 124 H130 V320 Q130 350 100 350 H30 L0 374 L-30 350 H-100 Q-130 350 -130 320 V124 H-150 Q-162 124 -162 112 V70 Q-162 58 -150 58 H-130 Z" fill="url(#vr-pass)"></path>
<path d="M-130 40 Q-130 10 -100 10 H100 Q130 10 130 40 V58 H150 Q162 58 162 70 V112 Q162 124 150 124 H130 V320 Q130 350 100 350 H30 L0 374 L-30 350 H-100 Q-130 350 -130 320 V124 H-150 Q-162 124 -162 112 V70 Q-162 58 -150 58 H-130 Z" filter="url(#vr-g)"></path>
<!-- the visor cut-out -->
<path d="M-96 50 H96 Q110 50 110 64 V112 Q110 128 94 128 H30 Q14 128 8 116 L0 100 L-8 116 Q-14 128 -30 128 H-94 Q-110 128 -110 112 V64 Q-110 50 -96 50 Z" fill="url(#vr-lens)"></path>
<path class="sweep" d="M-90 70 L-60 70" stroke="#9ff3ff" stroke-width="3" stroke-linecap="round" opacity=".7"></path>
<text y="168" font-family="Fraunces, serif" font-weight="700" font-size="30" fill="#fff4ea" text-anchor="middle">Virtual Adda</text>
<text y="190" font-family="'Space Mono', monospace" font-size="11" fill="#ffe0d6" text-anchor="middle">sim racing &amp; VR · Thane</text>
<text y="222" font-family="'Space Mono', monospace" font-weight="700" font-size="16" fill="#fff4ea" text-anchor="middle">SIM POD B</text>
<!-- the speedo -->
<g transform="translate(0 300)"><path d="M-70 0 A70 70 0 0 1 70 0" fill="none" stroke="#fff4ea" stroke-width="4"></path>
<path d="M40 -57 A70 70 0 0 1 70 0" fill="none" stroke="#ffd23c" stroke-width="8"></path>
<g stroke="#fff4ea" stroke-width="2">{''.join(f'<path d="M{62*math.cos(math.radians(180+i*30)):.1f} {62*math.sin(math.radians(180+i*30)):.1f} L{70*math.cos(math.radians(180+i*30)):.1f} {70*math.sin(math.radians(180+i*30)):.1f}"></path>' for i in range(7))}</g>
<g class="needle"><path d="M0 0 L-56 -6" stroke="#ffd23c" stroke-width="4" stroke-linecap="round"></path></g><circle r="7" fill="#fff4ea"></circle></g>
</g></g>
<g class="lines" stroke="#fff4ea" stroke-width="3" stroke-linecap="round" opacity="0"><path d="M100 300 H40"></path><path d="M110 360 H20"></path><path d="M500 320 H570"></path><path d="M490 400 H580"></path></g>'''
css = '''
.pass,.needle{transform-box:fill-box}
.pass{transform-origin:50% 0%}
.needle{transform-origin:100% 50%;transition:transform .4s}
.open .needle{transform:rotate(154deg);transition:transform 1.4s cubic-bezier(.5,0,.3,1.3) .2s}
.open .pass{animation:sway 1.6s ease-in-out .6s}
.open .lines{opacity:1;transition:opacity .2s .9s}
.open .lines path{animation:dash .5s linear .9s 4}
.open .sweep{animation:sw 1.2s ease-in-out 2}
@keyframes sway{30%{transform:rotate(5deg)}65%{transform:rotate(-4deg)}}
@keyframes dash{from{transform:translateX(0)}to{transform:translateX(-30px);opacity:0}}
@keyframes sw{50%{transform:translateX(150px)}}
''' + RM.format(sel='.pass,.needle,.lines,.lines path,.sweep')
write('VirtualAdda.dc.html', title='Laser tag and VR gaming keepsake', fonts='family=Fraunces:opsz,wght@9..144,700&amp;family=Space+Mono:wght@400;700',
      css=css, svg=svg, focus='#ffd23c', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #2b3a63 0%, #1f2b4d 58%, #141c35 100%)',
      text='#e8edfa', head='#ffffff', ctabg='#ffd23c', ctafg='#1b1405', ctahover='#f0c11d',
      label='A red lanyard pass for sim pod B at Virtual Adda with a visor-shaped window and a speedometer, from Laser tag and VR gaming.',
      h1='Laser tag & VR at Virtual Adda', p='The pass for sim pod B in Thane, halfway between us. The needle says what it says.',
      opened='Floor it', closed='Pit stop')
