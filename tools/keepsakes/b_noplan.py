from frame import write, grain, torn

MAP = torn([(64, 84), (540, 76), (534, 520), (70, 528)], seed=40, amp=4, step=13)
ROADS = [
    ('M168 60 C176 200 160 330 172 560', 17, 13),   # S.V. Road
    ('M268 60 L262 560', 17, 13),                   # Linking Road
    ('M120 196 L560 180', 15, 11),
    ('M120 306 L560 316', 15, 11),
    ('M120 432 L560 424', 15, 11),
    ('M268 246 L430 250', 10, 7),
]
roads = ''.join(f'<path d="{d}" stroke="#d2c4a2" stroke-width="{o}" fill="none"></path>' for d, o, i in ROADS)
roads += ''.join(f'<path d="{d}" stroke="#fbf6e8" stroke-width="{i}" fill="none"></path>' for d, o, i in ROADS)
hw = '<path d="M436 60 L420 560" stroke="#c9ad6e" stroke-width="24" fill="none"></path><path d="M436 60 L420 560" stroke="#f1d796" stroke-width="19" fill="none"></path><path d="M436 60 L420 560" stroke="#fff6dc" stroke-width="1.2" stroke-dasharray="8 7" fill="none"></path>'

turns = [(263, 428, 'L', 1.3), (265, 308, 'R', 1.7), (428, 314, 'L', 2.1), (433, 186, 'R', 2.5)]
turn_svg = ''.join(
    f'<g class="turn" style="--d:{d}s"><circle cx="{x-16}" cy="{y-16}" r="10" fill="#fbf6e8" stroke="#c23a2b" stroke-width="1.6"></circle>'
    f'<text x="{x-16}" y="{y-11.5}" font-family="Kalam, cursive" font-weight="700" font-size="14" fill="#c23a2b" text-anchor="middle">{c}</text></g>'
    for x, y, c, d in turns)

svg = f'''<defs>
{grain('n-grain')}
<linearGradient id="n-paper" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f3ecdb"></stop><stop offset="1" stop-color="#e4d8bd"></stop></linearGradient>
<radialGradient id="n-coin" cx=".38" cy=".34" r=".75"><stop offset="0" stop-color="#f3d08a"></stop><stop offset=".55" stop-color="#cf9f52"></stop><stop offset="1" stop-color="#94621f"></stop></radialGradient>
<clipPath id="n-clip"><path d="{MAP}"></path></clipPath>
<path id="n-rose" d="M0 -11 L2.4 -2.4 L11 0 L2.4 2.4 L0 11 L-2.4 2.4 L-11 0 L-2.4 -2.4 Z"></path>
</defs>

<!-- the map, torn out of an old city guide -->
<g transform="rotate(-3 300 300)">
<path d="{MAP}" fill="url(#n-paper)"></path>
<g clip-path="url(#n-clip)">
<path d="M40 60 L150 60 C130 140 160 200 128 260 C104 310 140 380 118 440 C106 480 130 520 120 560 L40 560 Z" fill="#bcd2cb"></path>
<g stroke="#9fbdb4" stroke-width="1.4" fill="none"><path d="M78 150 q8 -5 16 0 t16 0"></path><path d="M84 230 q8 -5 16 0 t16 0"></path><path d="M74 400 q8 -5 16 0 t16 0"></path><path d="M86 480 q8 -5 16 0 t16 0"></path></g>
<text transform="translate(100 330) rotate(-90)" font-family="Newsreader, serif" font-style="italic" font-size="13" fill="#5f8a80" text-anchor="middle">Arabian Sea</text>
{roads}{hw}
<path d="M352 60 L346 560" stroke="#6f6a5f" stroke-width="6" fill="none"></path><path d="M352 60 L346 560" stroke="#f6efdc" stroke-width="3" stroke-dasharray="7 7" fill="none"></path>
<g transform="translate(349 296) rotate(-1)"><rect x="-15" y="-12" width="30" height="24" rx="2" fill="#e7d9b8" stroke="#8f8268" stroke-width="1.2"></rect><text y="4" font-family="'Space Mono', monospace" font-size="8.5" fill="#6f6450" text-anchor="middle">stn</text></g>
<text transform="translate(362 120) rotate(-89)" font-family="'Space Mono', monospace" font-size="8.5" fill="#8f8268" text-anchor="end">Western line</text>
<g font-family="'Space Mono', monospace" font-size="9" fill="#8f8268">
<text transform="translate(181 250) rotate(88)">S.V. Road</text>
<text transform="translate(276 520) rotate(-89)">Linking Road</text>
<text transform="translate(447 290) rotate(-88)">W.E. Highway</text>
<text x="196" y="300">Station Road</text>
</g>
<path d="M300 70 V540" stroke="#fff" stroke-opacity=".55" stroke-width="1.4"></path><path d="M302 70 V540" stroke="#5a4a2a" stroke-opacity=".1" stroke-width="1.4"></path>
<path d="M50 302 H560" stroke="#fff" stroke-opacity=".5" stroke-width="1.4"></path><path d="M50 304 H560" stroke="#5a4a2a" stroke-opacity=".09" stroke-width="1.4"></path>
<!-- meet halfway -->
<path d="M160 422 l18 18 M178 422 l-18 18" stroke="#4a4a48" stroke-width="2.4" stroke-linecap="round"></path>
<text x="150" y="476" font-family="Kalam, cursive" font-size="16" fill="#4a4a48">meet halfway</text>
<text class="ask" x="214" y="410" font-family="Kalam, cursive" font-weight="700" font-size="22" fill="#4a4a48">?</text>
<text x="300" y="506" font-family="Kalam, cursive" font-size="15" fill="#4a4a48" text-anchor="middle">Sun 18 Oct</text>
<!-- where the coin took us -->
<path class="route" pathLength="1" d="M169 431 L263 428 L265 308 L428 314 L433 186 L486 183" fill="none" stroke="#c23a2b" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"></path>
{turn_svg}
<g class="end">
<g fill="none" stroke="#c23a2b" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
<path d="M480 144 C476 166 490 178 506 178 C522 178 536 166 532 144"></path>
<ellipse cx="506" cy="143" rx="28" ry="6"></ellipse>
<path d="M482 140 C486 118 526 118 530 140"></path>
<path d="M500 121 l6 -6 l6 6"></path>
<path d="M496 106 c-4 -6 4 -10 0 -16 M514 106 c-4 -6 4 -10 0 -16"></path>
</g>
<g font-family="Kalam, cursive" font-size="20" fill="#c23a2b" text-anchor="middle" transform="rotate(-4 330 130)">
<text x="330" y="124">Don’t have</text>
<text x="330" y="150">biriyani for lunch</text>
</g>
<path d="M418 140 q24 -4 50 4" fill="none" stroke="#c23a2b" stroke-width="1.6" stroke-linecap="round"></path>
<path d="M462 138 l7 6 l-9 3" fill="none" stroke="#c23a2b" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"></path>
</g>
</g>
<path d="{MAP}" filter="url(#n-grain)"></path>
<path d="{MAP}" fill="none" stroke="#fff" stroke-opacity=".5" stroke-width="1"></path>
</g>

<!-- the coin that chose every turn -->
<ellipse class="coin-shadow" cx="452" cy="506" rx="40" ry="8" fill="#1b2a26" opacity=".32"></ellipse>
<g class="toss"><g transform="translate(452 462)"><g class="spin">
<circle r="44" fill="url(#n-coin)" stroke="#7d521d" stroke-width="2.5"></circle>
<circle r="38" fill="none" stroke="#7d521d" stroke-width="1.2" stroke-dasharray="1.5 2.6"></circle>
<circle r="33" fill="none" stroke="#7d521d" stroke-opacity=".5" stroke-width=".8"></circle>
<g class="face-a">
<use href="#n-rose" transform="translate(0 -16)" fill="#7d521d"></use>
<text y="15" font-family="'Fraunces', serif" font-style="italic" font-weight="700" font-size="20" fill="#f6dda2" text-anchor="middle" transform="translate(.8 .8)">left</text>
<text y="15" font-family="'Fraunces', serif" font-style="italic" font-weight="700" font-size="20" fill="#6b4416" text-anchor="middle">left</text>
</g>
<g class="face-b" transform="scale(1 -1)">
<use href="#n-rose" transform="translate(0 -16) rotate(45)" fill="#7d521d"></use>
<text y="15" font-family="'Fraunces', serif" font-style="italic" font-weight="700" font-size="20" fill="#6b4416" text-anchor="middle">right</text>
</g>
<path d="M-26 -22 a34 34 0 0 1 22 -12" stroke="#fff4d4" stroke-width="3" stroke-linecap="round" fill="none" opacity=".7"></path>
</g></g></g>'''

css = '''
.route{stroke-dasharray:1;stroke-dashoffset:1;transition:stroke-dashoffset .3s linear}
.turn,.end{opacity:0;transition:opacity .15s linear}
.ask{transition:opacity .3s linear .2s}
.toss,.spin,.coin-shadow{transform-box:fill-box;transform-origin:50% 50%}
.face-b{opacity:0}
.open .route{stroke-dashoffset:0;transition:stroke-dashoffset 1.6s cubic-bezier(.4,0,.6,1) 1.15s}
.open .turn{opacity:1;transition:opacity .25s ease-out var(--d)}
.open .end{opacity:1;transition:opacity .5s ease-out 2.8s}
.open .ask{opacity:0;transition-delay:0s}
.open .toss{animation:toss 1.1s cubic-bezier(.3,0,.7,1)}
.open .spin{animation:spin .275s linear 4}
.open .face-a{animation:fa .275s step-end 4}
.open .face-b{animation:fb .275s step-end 4}
.open .coin-shadow{animation:shadow 1.1s cubic-bezier(.3,0,.7,1)}
@keyframes toss{0%{transform:translateY(0)}50%{transform:translateY(-190px)}88%{transform:translateY(0)}94%{transform:translateY(-8px)}100%{transform:translateY(0)}}
@keyframes spin{0%{transform:scaleY(1)}25%{transform:scaleY(0)}50%{transform:scaleY(-1)}75%{transform:scaleY(0)}100%{transform:scaleY(1)}}
@keyframes fa{0%{opacity:1}25%{opacity:0}75%{opacity:1}}
@keyframes fb{0%{opacity:0}25%{opacity:1}75%{opacity:0}}
@keyframes shadow{0%,100%{transform:scale(1);opacity:.32}50%{transform:scale(.45);opacity:.12}}
@media (prefers-reduced-motion:reduce){.route,.turn,.end,.ask{transition-duration:.01s!important;transition-delay:0s!important}.open .toss,.open .spin,.open .face-a,.open .face-b,.open .coin-shadow{animation:none}}
'''

write('NoPlan.dc.html', title='No-plan date keepsake',
      fonts='family=Kalam:wght@400;700&amp;family=Space+Mono&amp;family=Fraunces:ital,opsz,wght@1,9..144,700',
      css=css, svg=svg, focus='#f4d38a',
      bg='radial-gradient(ellipse 55% 65% at 34% 48%, #7f9e94 0%, #66857c 58%, #4d6a62 100%)',
      text='#f2ede1', head='#fffaf0', ctabg='#f2d38c', ctafg='#2a1c0c', ctahover='#e6c26f',
      label='A torn street map and a bronze coin from No-plan date, 18 October 2026, five stars.',
      h1='No-plan date',
      p='A torn page of the city and the coin that picked every turn. Where it led us is written at the end of the road.',
      opened='Flip the coin', closed='Fold the map')
