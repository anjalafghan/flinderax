from frame import write, grain

NOTE = 'M0 0 H300 V246 L246 300 H0 Z'
HORSE = [
    'M70 170 C80 140 170 134 200 158 C216 174 206 196 180 198 C140 204 92 206 72 190 C66 184 66 176 70 170',
    'M90 196 L80 250', 'M112 201 L116 252', 'M164 201 L156 250', 'M188 197 L200 246',
    'M196 160 L226 110',
    'M226 110 C236 90 262 94 266 112 C268 124 250 128 234 122',
    'M236 99 L238 84 L248 97',
    'M222 118 l-9 5 l7 6 l-9 5 l7 6 l-9 5',
    'M71 172 C50 166 44 186 32 198 C46 188 52 198 44 214',
]
horse = ''.join(f'<path class="ink" pathLength="1" style="--i:{i}" d="{d}"></path>' for i, d in enumerate(HORSE))

svg = f'''<defs>
{grain('d-grain', .3, .25, .02, .12)}
<linearGradient id="d-note" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f1d34c"></stop><stop offset=".14" stop-color="#f8e066"></stop><stop offset="1" stop-color="#f6dc5e"></stop></linearGradient>
<linearGradient id="d-curl" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f9e98f"></stop><stop offset="1" stop-color="#d6b836"></stop></linearGradient>
<linearGradient id="d-pink" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f294ad"></stop><stop offset=".16" stop-color="#f7a9be"></stop><stop offset="1" stop-color="#f5a3b9"></stop></linearGradient>
<linearGradient id="d-gold" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffd95a"></stop><stop offset="1" stop-color="#e6ad1f"></stop></linearGradient>
</defs>

<!-- a pink note, tucked underneath -->
<g class="pink"><g transform="translate(384 318) rotate(7)">
<path d="M0 0 H190 V160 H0 Z" fill="url(#d-pink)"></path>
<rect width="190" height="160" filter="url(#d-grain)"></rect>
<g font-family="'Gochi Hand', cursive" fill="#3a1d2a" text-anchor="middle">
<text x="95" y="76" font-size="34">Diya wins</text>
<text x="95" y="114" font-size="26">(as always)</text>
</g>
</g></g>

<!-- the sticky note, stuck to the screen -->
<g transform="translate(96 120) rotate(-4)">
<path d="{NOTE}" fill="url(#d-note)"></path>
<path d="{NOTE}" filter="url(#d-grain)"></path>
<path d="M300 246 L246 300 C262 276 274 262 300 246 Z" fill="url(#d-curl)"></path>
<path d="M246 300 C262 276 274 262 300 246" fill="none" stroke="#b9981f" stroke-width="1"></path>
<g fill="none" stroke="#1f43b5" stroke-width="4.5" stroke-linecap="round" stroke-linejoin="round">{horse}</g>
<circle class="ink-dot" cx="250" cy="108" r="3" fill="#1f43b5"></circle>
<g font-family="'Gochi Hand', cursive" fill="#c0392b">
<g class="guess-1"><text x="40" y="284" font-size="26">dog?</text><path d="M36 276 H96" stroke="#c0392b" stroke-width="2.5" class="strike" pathLength="1"></path></g>
<g class="guess-2"><text x="118" y="284" font-size="26">cow??</text><path d="M114 276 H184" stroke="#c0392b" stroke-width="2.5" class="strike" pathLength="1"></path></g>
</g>
<g class="guess-3" font-family="'Gochi Hand', cursive"><text x="188" y="40" font-size="28" fill="#2c7a3f">horse!</text>
<path d="M186 48 q34 6 70 -2" fill="none" stroke="#2c7a3f" stroke-width="2.5" stroke-linecap="round"></path></g>
<text x="18" y="34" font-family="'Gochi Hand', cursive" font-size="17" fill="#6b5a12">16 Nov, round 3</text>
</g>

<!-- the timer, running out -->
<g transform="translate(486 128)">
<circle r="40" fill="#fbfaf4"></circle>
<circle r="40" fill="none" stroke="#e8e2cf" stroke-width="7"></circle>
<circle class="timer" r="40" fill="none" stroke="#4fb06d" stroke-width="7" pathLength="1" transform="rotate(-90)"></circle>
<path d="M-12 -52 h24 M0 -52 v8" stroke="#fbfaf4" stroke-width="6" stroke-linecap="round"></path>
<text class="t80" y="10" font-family="'Gochi Hand', cursive" font-size="30" fill="#2a2440" text-anchor="middle">80</text><text class="t3" y="10" font-family="'Gochi Hand', cursive" font-size="30" fill="#c0392b" text-anchor="middle">3</text>
</g>

<!-- the blue marker -->
<g transform="translate(300 556) rotate(-14)">
<path d="M0 -13 H120 a13 13 0 0 1 0 26 H0 Z" fill="#f2f2ef"></path>
<path d="M0 -13 H120 a13 13 0 0 1 0 26 H0 Z" fill="none" stroke="#c9c8c0" stroke-width="1"></path>
<path d="M-56 -14 H0 V14 H-56 a14 14 0 0 1 0 -28 Z" fill="#1f43b5"></path>
<path d="M-50 -9 H-6" stroke="#6f8be0" stroke-width="3" stroke-linecap="round"></path>
<text x="58" y="5" font-family="Figtree, sans-serif" font-weight="700" font-size="11" fill="#1f43b5" text-anchor="middle">permanent</text>
</g>

<!-- the crown, for the winner -->
<g class="crown"><g transform="translate(62 54) rotate(-14) scale(1.15)">
<path d="M0 74 L0 16 L30 44 L60 0 L90 44 L120 16 L120 74 Z" fill="url(#d-gold)" stroke="#b9860f" stroke-width="2" stroke-linejoin="round"></path>
<path d="M0 58 H120" stroke="#b9860f" stroke-width="1.5"></path>
<circle cx="0" cy="16" r="6" fill="#d6453d"></circle><circle cx="60" cy="0" r="7" fill="#3f7fd6"></circle><circle cx="120" cy="16" r="6" fill="#d6453d"></circle>
<text x="60" y="72" font-family="'Gochi Hand', cursive" font-size="19" fill="#6a3d05" text-anchor="middle">Diya</text>
</g></g>'''

css = '''
.ink{stroke-dasharray:1;stroke-dashoffset:0}
.guess-2,.guess-3,.strike,.t3{opacity:0}
.open .t80{opacity:0;transition:opacity 0s 2.6s}
.open .t3{opacity:1;transition:opacity 0s 2.6s}
.strike{stroke-dasharray:1;stroke-dashoffset:1}
.crown,.pink{transform-box:fill-box;transform-origin:50% 50%}
.crown{opacity:0;transform:translateY(-120px) rotate(-30deg);transition:opacity .15s linear,transform .3s ease-in}
.pink{transform:translate(-170px,-70px) rotate(-6deg);transition:transform .35s ease-in}
.timer{stroke-dasharray:1;stroke-dashoffset:0;transition:stroke-dashoffset .3s linear}
.open .ink{animation:draw .5s ease-in-out both;animation-delay:calc(var(--i) * .16s)}
.open .ink-dot{animation:pop .2s ease-out 1.5s both}
.open .timer{stroke-dashoffset:.86;transition:stroke-dashoffset 3s linear}
.open .guess-1 .strike{opacity:1;stroke-dashoffset:0;transition:stroke-dashoffset .25s ease-out 1.7s}
.open .guess-2{opacity:1;transition:opacity .1s 1.95s}
.open .guess-2 .strike{opacity:1;stroke-dashoffset:0;transition:stroke-dashoffset .25s ease-out 2.25s}
.open .guess-3{opacity:1;transition:opacity .2s 2.55s}
.open .crown{opacity:1;transform:none;transition:opacity .1s 2.9s,transform .7s cubic-bezier(.3,1.5,.5,1) 2.9s}
.open .pink{transform:none;transition:transform .6s cubic-bezier(.2,.8,.25,1) 3.3s}
@keyframes draw{from{stroke-dashoffset:1}to{stroke-dashoffset:0}}
@keyframes pop{from{transform:scale(0)}to{transform:scale(1)}}
@media (prefers-reduced-motion:reduce){.crown,.pink,.timer,.strike,.guess-2,.guess-3{transition-duration:.01s!important;transition-delay:0s!important}.open .ink,.open .ink-dot{animation:none}}
'''

write('DrawingGame.dc.html', title='Drawing guessing game keepsake',
      fonts='family=Gochi+Hand',
      css=css, svg=svg, focus='#fbe58a',
      bg='radial-gradient(ellipse 55% 65% at 34% 48%, #8a79b4 0%, #6f5f9a 58%, #54467c 100%)',
      text='#f3eefb', head='#fffdf6', ctabg='#fbe27a', ctafg='#2a2440', ctahover='#f1d24f',
      label='A yellow sticky note with a terrible drawing of a horse, from Drawing guessing game, 16 November 2026, five stars.',
      h1='Drawing guessing game',
      p='The worst horse ever drawn, stuck to the screen on a night apart, with the first guess still on it. Play the round again to see who won.',
      opened='Play the round', closed='Start over')
