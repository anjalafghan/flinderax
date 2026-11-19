from frame import write, grain

# ---- 10 Nov: the first visit, a cup for two ----
cup_svg = f'''<defs>
{grain('t-grain', .25, .16, .06, .2)}
<linearGradient id="t-cup" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#e9e6df"></stop><stop offset=".25" stop-color="#ffffff"></stop><stop offset=".7" stop-color="#f7f5f0"></stop><stop offset="1" stop-color="#d9d5cc"></stop></linearGradient>
<linearGradient id="t-kraft" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#b3844f"></stop><stop offset=".3" stop-color="#c99d69"></stop><stop offset="1" stop-color="#a87a45"></stop></linearGradient>
<linearGradient id="t-lid" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#d8d3c6"></stop><stop offset=".3" stop-color="#f6f3ec"></stop><stop offset="1" stop-color="#cfc9bb"></stop></linearGradient>
<clipPath id="t-cupclip"><path d="M205 210 L395 210 L372 535 L228 535 Z"></path></clipPath>
</defs>
<ellipse cx="300" cy="540" rx="96" ry="12" fill="#1c2a44" opacity=".3"></ellipse>
<path d="M205 210 L395 210 L372 535 L228 535 Z" fill="url(#t-cup)"></path>
<g clip-path="url(#t-cupclip)">
<!-- written by the barista -->
<g font-family="'Permanent Marker', cursive" fill="#26262a" transform="rotate(-6 300 270)">
<text x="300" y="276" font-size="30" text-anchor="middle">Diya + 1</text>
<text x="352" y="306" font-size="15" text-anchor="middle">10 Nov</text>
</g>
<!-- our note, under the sleeve -->
<g font-family="Kalam, cursive" font-size="18" fill="#2a4a8a" text-anchor="middle" transform="rotate(-2 300 360)">
<text x="300" y="350">first time here.</text>
<text x="300" y="374">back in two days.</text>
</g>
<g class="sleeve">
<path d="M212 318 L388 318 L378 446 L222 446 Z" fill="url(#t-kraft)"></path>
<rect x="200" y="310" width="200" height="140" filter="url(#t-grain)"></rect>
<path d="M212 318 L388 318 M222 446 L378 446" stroke="#8d6434" stroke-width="2"></path>
<g fill="#1d3d6b" text-anchor="middle">
<text x="300" y="378" font-family="Fraunces, serif" font-weight="700" font-size="27">Blue Tokai</text>
<text x="300" y="402" font-family="Figtree, sans-serif" font-weight="600" font-size="11" letter-spacing="1.2">Goregaon East</text>
</g>
<path d="M262 420 l14 -10 l12 8 l14 -14 l12 10 l14 -8 l14 10" fill="none" stroke="#1d3d6b" stroke-width="2" stroke-linejoin="round"></path>
</g>
</g>
<path d="M214 214 L232 530" stroke="#fff" stroke-width="6" stroke-linecap="round" opacity=".5"></path>
<ellipse cx="300" cy="212" rx="95" ry="11" fill="#5a321a"></ellipse>
<ellipse cx="300" cy="212" rx="78" ry="8" fill="#a8693c"></ellipse>
<path d="M266 211 q20 -5 40 0 q14 3 26 -1" fill="none" stroke="#e8c49a" stroke-width="2" stroke-linecap="round" opacity=".8"></path>
<g class="steam" fill="none" stroke="#fff" stroke-width="5" stroke-linecap="round" opacity="0">
<path pathLength="1" d="M270 200 c-14 -18 14 -30 0 -50 c-12 -18 10 -28 2 -46"></path>
<path pathLength="1" d="M304 196 c-14 -18 14 -30 0 -50 c-12 -18 10 -28 2 -46"></path>
<path pathLength="1" d="M336 200 c-14 -18 14 -30 0 -50 c-12 -18 10 -28 2 -46"></path>
</g>
<g class="lid">
<path d="M196 216 L404 216 L400 196 L200 196 Z" fill="url(#t-lid)"></path>
<path d="M206 196 C214 168 386 168 394 196 Z" fill="url(#t-lid)"></path>
<path d="M196 216 L404 216" stroke="#b9b3a4" stroke-width="2"></path>
<path d="M206 196 L394 196" stroke="#c4bdae" stroke-width="1.5"></path>
<ellipse cx="338" cy="182" rx="16" ry="4" fill="#3a3530"></ellipse>
</g>'''

cup_css = '''
.sleeve,.lid,.steam path{transform-box:fill-box}
.lid{transform-origin:0% 100%;transition:transform .4s cubic-bezier(.5,0,.75,0) .3s}
.sleeve{transition:transform .5s cubic-bezier(.5,0,.75,0)}
.steam path{stroke-dasharray:.3 1;stroke-dashoffset:.3}
.open .lid{transform:translate(-70px,-40px) rotate(-18deg);transition:transform .6s cubic-bezier(.2,.8,.25,1)}
.open .sleeve{transform:translateY(80px);transition:transform .7s cubic-bezier(.2,.8,.25,1) .45s}
.open .steam{opacity:.75;transition:opacity .4s .6s}
.open .steam path{animation:rise 2.4s linear .6s 2}
.open .steam path:nth-child(2){animation-delay:.9s}
.open .steam path:nth-child(3){animation-delay:1.2s}
@keyframes rise{from{stroke-dashoffset:.3}to{stroke-dashoffset:-1}}
@media (prefers-reduced-motion:reduce){.lid,.sleeve{transition-duration:.01s!important;transition-delay:0s!important}.open .steam path{animation:none}.open .steam{opacity:0}}
'''

write('BlueTokaiFirst.dc.html', title='Blue Tokai, first visit keepsake',
      fonts='family=Permanent+Marker&amp;family=Kalam:wght@400;700&amp;family=Fraunces:opsz,wght@9..144,700',
      css=cup_css, svg=cup_svg, focus='#13284a',
      bg='radial-gradient(ellipse 55% 65% at 34% 48%, #a3b8d6 0%, #8aa1c4 58%, #6c84aa 100%)',
      text='#13213b', head='#0c1830', ctabg='#1d3d6b', ctafg='#f4efe4', ctahover='#152f55',
      label='A paper coffee cup from Blue Tokai in Goregaon East, 10 November 2026, with Diya plus one written on it by the barista.',
      h1='Blue Tokai',
      p='The cup from our first time at Blue Tokai, Diya’s name in the barista’s marker. Lift the lid and what happened next is under the sleeve.',
      opened='Lift the lid', closed='Put it back')

# ---- 12 Nov: the second visit, a bean punch card ----
BEAN = 'M0 -205 C110 -205 160 -90 150 10 C140 130 70 205 -10 205 C-110 205 -160 100 -150 -10 C-140 -120 -90 -205 0 -205 Z'
CREASE = 'M-6 -186 C40 -120 -50 -40 0 20 C50 80 -30 140 6 188'
holes = [(-90, 70), (-40, 112), (20, 140), (78, 112), (112, 56), (120, -10)]

def hole(x, y, cls):
    return f'<g transform="translate({x} {y}) rotate(-20)" class="{cls}"><ellipse rx="13" ry="17" class="hole-fill"></ellipse><path d="M-1 -15 C5 -6 -5 4 1 15" fill="none" stroke-width="2" class="hole-crease"></path></g>'

holes_svg = hole(*holes[0], 'punched') + hole(*holes[1], 'punch-me') + ''.join(
    f'<g transform="translate({x} {y}) rotate(-20)"><ellipse rx="13" ry="17" fill="none" stroke="#9c8a6a" stroke-width="1.4" stroke-dasharray="3 3.5"></ellipse></g>' for x, y in holes[2:])

bean_svg = f'''<defs>
{grain('b-grain', .25, .18, .08, .14)}
<linearGradient id="b-card" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f6ecd8"></stop><stop offset="1" stop-color="#e6d6b6"></stop></linearGradient>
<linearGradient id="b-bread" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f5e2b4"></stop><stop offset="1" stop-color="#e9cd92"></stop></linearGradient>
<clipPath id="b-clip"><path d="{BEAN}"></path></clipPath>
</defs>

<!-- half a sandwich, wrapped, tied in red and green like us that day -->
<g transform="translate(408 418) rotate(-8)">
<path d="M0 0 L172 0 L0 132 Z" fill="url(#b-bread)" stroke="#b9773a" stroke-width="9" stroke-linejoin="round"></path>
<path d="M14 118 L156 8" stroke="#6aa84f" stroke-width="7" stroke-linecap="round"></path>
<path d="M20 124 L162 14" stroke="#d0453b" stroke-width="4" stroke-linecap="round"></path>
<path d="M4 72 L82 72 L170 44 L176 150 L-4 150 Z" fill="#f8f5ee" opacity=".93"></path>
<path d="M4 72 L82 72 L170 44" fill="none" stroke="#d9d3c4" stroke-width="1.5"></path>
<path d="M40 74 L46 150 M120 58 L118 150" stroke="#e2dccd" stroke-width="1.5"></path>
<path d="M-6 112 C60 104 120 100 180 92" stroke="#b8322c" stroke-width="3.5" fill="none"></path>
<path d="M-6 112 C60 104 120 100 180 92" stroke="#2f6b45" stroke-width="3.5" stroke-dasharray="7 7" fill="none"></path>
<path d="M96 102 c-14 -16 -26 -6 -10 4 M96 102 c14 -18 28 -6 8 4" stroke="#b8322c" stroke-width="3" fill="none" stroke-linecap="round"></path>
</g>

<!-- the card, cut in the shape of a bean -->
<g transform="translate(262 286) rotate(-24)">
<g class="card">
<path d="{BEAN}" fill="url(#b-card)"></path>
<g clip-path="url(#b-clip)"><rect x="-170" y="-220" width="340" height="440" filter="url(#b-grain)"></rect></g>
<path d="{BEAN}" fill="none" stroke="#b9a37c" stroke-width="1.2"></path>
<g class="front">
<path d="{CREASE}" fill="none" stroke="#b39466" stroke-width="10" stroke-linecap="round"></path>
<path d="{CREASE}" fill="none" stroke="#8a6d43" stroke-width="3" stroke-linecap="round"></path>
<path d="{CREASE}" fill="none" stroke="#fffaf0" stroke-width="2" stroke-linecap="round" transform="translate(-7 0)" opacity=".8"></path>
<g fill="#1d3d6b" transform="rotate(24)">
<text x="-6" y="-92" font-family="Fraunces, serif" font-weight="700" font-size="36" text-anchor="middle">Blue Tokai</text>
<text x="-6" y="-66" font-family="Fraunces, serif" font-style="italic" font-size="19" text-anchor="middle">regulars now</text>
<text x="-6" y="-42" font-family="Figtree, sans-serif" font-weight="600" font-size="10.5" letter-spacing="1.2" text-anchor="middle">Goregaon East</text>
</g>
{holes_svg}
<g font-family="Kalam, cursive" font-size="14" fill="#5b4a2e" text-anchor="middle">
<text x="-90" y="104" transform="rotate(-6 -90 104)">10</text>
<text class="d12" x="-40" y="146" transform="rotate(-6 -40 146)">12</text>
</g>
</g>
<g class="back">
<g transform="rotate(24)" text-anchor="middle">
<text x="0" y="-70" font-family="Figtree, sans-serif" font-weight="600" font-size="11" letter-spacing="1.2" fill="#1d3d6b">12 November, visit two</text>
<text x="0" y="0" font-family="Kalam, cursive" font-weight="700" font-size="34" fill="#2a3f80">Kya pela</text>
<text x="0" y="40" font-family="Kalam, cursive" font-weight="700" font-size="34" fill="#2a3f80">hai bhai</text>
<path d="M-54 58 q54 12 108 0" fill="none" stroke="#2a3f80" stroke-width="2.4" stroke-linecap="round"></path>
</g>
</g>
</g>
</g>
<!-- the bit the punch took out -->
<g class="chad"><g transform="translate(212 412) rotate(-44)"><ellipse rx="13" ry="17" fill="#5a3a20"></ellipse><path d="M-1 -15 C5 -6 -5 4 1 15" fill="none" stroke="#2e1a0c" stroke-width="2"></path></g></g>'''

bean_css = '''
.hole-fill{fill:#b77d5f}
.hole-crease{stroke:#8f5a3f}
.punch-me .hole-fill{fill:none;stroke:#9c8a6a;stroke-width:1.4;stroke-dasharray:3 3.5;transition:fill .05s}
.punch-me .hole-crease{stroke:none}
.d12{opacity:0}
.card,.chad{transform-box:fill-box;transform-origin:50% 50%}
.back{opacity:0}
.chad{opacity:0}
.open .punch-me .hole-fill{fill:#b77d5f;stroke:none;transition-delay:.25s}
.open .punch-me .hole-crease{stroke:#8f5a3f;transition:stroke 0s .25s}
.open .d12{opacity:1;transition:opacity .3s .4s}
.open .chad{animation:fall .9s cubic-bezier(.4,0,.8,.6) .25s both}
.open .card{animation:flip .9s ease-in-out 1.3s both}
.open .front{animation:hideA .9s step-end 1.3s both}
.open .back{animation:showB .9s step-end 1.3s both}
@keyframes fall{0%{opacity:1;transform:translate(0,0) rotate(0)}100%{opacity:1;transform:translate(26px,150px) rotate(220deg)}}
@keyframes flip{0%{transform:scaleX(1)}50%{transform:scaleX(0)}100%{transform:scaleX(1)}}
@keyframes hideA{0%{opacity:1}50%,100%{opacity:0}}
@keyframes showB{0%{opacity:0}50%,100%{opacity:1}}
@media (prefers-reduced-motion:reduce){.open .card,.open .chad{animation:none}.open .front{animation:none;opacity:0}.open .back{animation:none;opacity:1}}
'''

write('BlueTokaiRegulars.dc.html', title='Blue Tokai, regulars keepsake',
      fonts='family=Kalam:wght@400;700&amp;family=Fraunces:ital,opsz,wght@0,9..144,700;1,9..144,400',
      css=bean_css, svg=bean_svg, focus='#2a1a10',
      bg='radial-gradient(ellipse 55% 65% at 34% 48%, #d29a79 0%, #bd8263 58%, #9d6649 100%)',
      text='#2a1a10', head='#1e1209', ctabg='#1d3d6b', ctafg='#f4efe4', ctahover='#152f55',
      label='A bean-shaped punch card from Blue Tokai in Goregaon East, 12 November 2026, five stars, with half a sandwich tied in red and green.',
      h1='Blue Tokai, again',
      p='Two days later, a second bean punched, and the sandwich tied in our Christmas colours. Our verdict is on the back of the card.',
      opened='Punch the card', closed='Turn it back')
