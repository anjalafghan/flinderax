from frame import write, grain

PAD = 'M-150 40 C-150 -40 -60 -72 0 -72 C60 -72 150 -40 150 40 C150 128 80 150 0 134 C-80 150 -150 128 -150 40 Z'
TOES = [(-150, -102, -24), (-56, -174, -8), (56, -174, 8), (150, -102, 24)]
toe_shapes = ''.join(f'<ellipse cx="0" cy="0" rx="44" ry="56" transform="translate({x} {y}) rotate({r})"></ellipse>' for x, y, r in TOES)

svg = f'''<defs>
{grain('c-grain', .25, .16, .08, .14)}
<linearGradient id="c-card" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f6ead2"></stop><stop offset="1" stop-color="#e9d6b3"></stop></linearGradient>
<linearGradient id="c-fur" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#e59a4c"></stop><stop offset="1" stop-color="#c9772f"></stop></linearGradient>
<clipPath id="c-shape"><path d="{PAD}"></path>{toe_shapes}</clipPath>
</defs>

<!-- a seekh kebab, for the miya -->
<g transform="translate(64 556) rotate(-9) scale(.9)">
<path d="M-20 0 H230" stroke="#9aa1a4" stroke-width="4" stroke-linecap="round"></path>
<circle cx="236" cy="0" r="7" fill="none" stroke="#9aa1a4" stroke-width="3"></circle>
<path d="M14 0 H186" stroke="#6e3417" stroke-width="22" stroke-linecap="round"></path>
<path d="M14 -3 H186" stroke="#8c4a22" stroke-width="12" stroke-linecap="round"></path>
<g stroke="#3d1a0a" stroke-width="3" stroke-linecap="round" opacity=".7"><path d="M40 -8 l-6 14 M76 -9 l-6 15 M112 -9 l-6 15 M148 -8 l-6 14"></path></g>
<g fill="#5f8f3a"><circle cx="60" cy="-2" r="1.6"></circle><circle cx="96" cy="3" r="1.5"></circle><circle cx="130" cy="-4" r="1.6"></circle><circle cx="164" cy="2" r="1.4"></circle></g>
<path d="M-6 26 a20 20 0 0 1 40 0 Z" fill="#e9e36a" stroke="#c4b63b" stroke-width="2" transform="rotate(14 14 26)"></path>
<ellipse cx="200" cy="30" rx="18" ry="8" fill="none" stroke="#b5527a" stroke-width="3" opacity=".85"></ellipse>
</g>

<!-- a cockle from Versova beach -->
<g transform="translate(520 96) rotate(18)">
<path d="M-26 10 C-26 -16 26 -16 26 10 L8 18 L-8 18 Z" fill="#eadcc4" stroke="#b79c78" stroke-width="1.5"></path>
<g stroke="#b79c78" stroke-width="1.3" fill="none"><path d="M0 -10 V16 M-12 -6 L-5 17 M12 -6 L5 17 M-21 1 L-7 17 M21 1 L7 17"></path></g>
</g>

<!-- the card, cut in the shape of a paw -->
<g transform="translate(300 300) rotate(-7)">
<g fill="url(#c-card)"><path d="{PAD}"></path>{toe_shapes}</g>
<g clip-path="url(#c-shape)"><rect x="-220" y="-260" width="440" height="420" filter="url(#c-grain)"></rect></g>
<g fill="none" stroke="#b9996a" stroke-width="1.2" stroke-opacity=".7"><path d="{PAD}"></path>{toe_shapes}</g>

<!-- toe pads: one visit stamped, three to come -->
<g transform="translate(-150 -102) rotate(-24)">
<circle r="28" fill="none" stroke="#b5452c" stroke-width="2.5"></circle>
<circle r="23" fill="none" stroke="#b5452c" stroke-width=".9"></circle>
<text y="2" font-family="'Young Serif', serif" font-size="20" fill="#b5452c" text-anchor="middle">9</text>
<text y="15" font-family="Figtree, sans-serif" font-weight="700" font-size="8" letter-spacing=".6" fill="#b5452c" text-anchor="middle">Nov</text>
</g>
<g fill="none" stroke="#a68a62" stroke-width="1.4" stroke-dasharray="3 4">
<circle r="26" transform="translate(-56 -174)"></circle><circle r="26" transform="translate(56 -174)"></circle><circle r="26" transform="translate(150 -102)"></circle>
</g>

<!-- printed on the big pad -->
<g font-family="'Young Serif', serif" fill="#5b3524" text-anchor="middle">
<text y="-26" font-size="26">Cat Café Studio</text>
<text y="-6" font-family="Figtree, sans-serif" font-weight="600" font-size="11" letter-spacing="1">Versova, by the beach</text>
<text y="118" font-family="Newsreader, serif" font-style="italic" font-size="12" fill="#7a5640">please don’t feed the cats</text>
</g>

<!-- our note, under where she sleeps -->
<g class="note" font-family="Kalam, cursive" font-size="19" fill="#34407a" text-anchor="middle" transform="rotate(-3)">
<text y="34">Beach, cats,</text>
<text y="58">inappropriate room</text>
<text y="82">and tasty food</text>
</g>

<!-- the regular, asleep on the card -->
<g class="cat"><g transform="translate(6 60)">
<ellipse cx="2" cy="44" rx="98" ry="12" fill="#3a2410" opacity=".18"></ellipse>
<path d="M-96 18 C-100 -30 -40 -46 10 -44 C64 -42 104 -20 100 18 C96 46 40 50 0 48 C-46 48 -92 46 -96 18 Z" fill="url(#c-fur)"></path>
<g fill="none" stroke="#b0602a" stroke-width="5" stroke-linecap="round" opacity=".8"><path d="M-10 -40 c-6 12 -6 22 0 30"></path><path d="M16 -40 c-6 14 -6 24 0 32"></path><path d="M42 -34 c-6 14 -6 22 0 30"></path><path d="M68 -24 c-5 12 -5 20 0 26"></path></g>
<g class="tail"><path d="M92 20 C96 50 40 58 -10 54 C-40 52 -70 46 -76 34" fill="none" stroke="#d07d34" stroke-width="18" stroke-linecap="round"></path>
</g>
<g class="head">
<path d="M-108 -26 L-102 -66 L-80 -46 Z" fill="#d98a3d"></path><path d="M-104 -32 L-100 -58 L-86 -45 Z" fill="#e8a49a"></path>
<path d="M-48 -30 L-46 -70 L-70 -50 Z" fill="#d98a3d"></path><path d="M-52 -34 L-50 -62 L-66 -49 Z" fill="#e8a49a"></path>
<ellipse cx="-76" cy="-16" rx="42" ry="34" fill="#e19448"></ellipse>
<g fill="none" stroke="#b0602a" stroke-width="3.5" stroke-linecap="round" opacity=".8"><path d="M-86 -46 v10 M-76 -48 v12 M-66 -46 v10"></path></g>
<ellipse cx="-76" cy="2" rx="18" ry="12" fill="#f4e2c6"></ellipse>
<path d="M-80 -4 h8 l-4 5 z" fill="#c96e6e"></path>
<g class="eyes-shut" fill="none" stroke="#4a2a14" stroke-width="2.6" stroke-linecap="round"><path d="M-98 -16 q7 6 14 0"></path><path d="M-68 -16 q7 6 14 0"></path></g>
<g class="eyes-open"><ellipse cx="-91" cy="-17" rx="7" ry="8" fill="#9fb14a"></ellipse><ellipse cx="-61" cy="-17" rx="7" ry="8" fill="#9fb14a"></ellipse><ellipse cx="-91" cy="-17" rx="2.2" ry="6.5" fill="#1f1408"></ellipse><ellipse cx="-61" cy="-17" rx="2.2" ry="6.5" fill="#1f1408"></ellipse><circle cx="-89" cy="-20" r="1.6" fill="#fff"></circle><circle cx="-59" cy="-20" r="1.6" fill="#fff"></circle></g>
<g stroke="#f7ecd8" stroke-width="1.3" stroke-linecap="round" opacity=".9"><path d="M-94 2 l-30 -4 M-94 6 l-30 4 M-58 2 l30 -4 M-58 6 l30 4"></path></g>
</g>
<g class="zz" font-family="Kalam, cursive" font-weight="700" fill="#5b3524" opacity=".7"><text x="-128" y="-60" font-size="16">z</text><text x="-142" y="-76" font-size="12">z</text></g>
</g></g>
</g>'''

css = '''
.cat,.tail,.head{transform-box:fill-box}
.cat{transform-origin:20% 80%;transition:transform .5s cubic-bezier(.5,0,.75,0)}
.tail{transform-origin:100% 0%}
.head{transform-origin:70% 90%;transition:transform .4s ease}
.note{opacity:0;transition:opacity .2s linear}
.eyes-open{opacity:0;transition:opacity .1s linear}
.eyes-shut,.zz{transition:opacity .1s linear}
.open .eyes-open{opacity:1;transition-delay:.15s}
.open .eyes-shut,.open .zz{opacity:0}
.open .head{transform:rotate(-12deg) translateY(-6px)}
.open .cat{transform:translate(150px,148px) rotate(10deg);transition:transform .9s cubic-bezier(.3,.9,.3,1.05) .55s}
.open .note{opacity:1;transition:opacity .5s ease-out 1.1s}
.open .tail{animation:swish 1.4s ease-in-out 1.5s 2}
@keyframes swish{0%,100%{transform:rotate(0)}30%{transform:rotate(-14deg)}65%{transform:rotate(8deg)}}
@media (prefers-reduced-motion:reduce){.cat,.head,.note,.eyes-open,.eyes-shut,.zz{transition-duration:.01s!important;transition-delay:0s!important}.open .tail{animation:none}}
'''

write('CatCafe.dc.html', title='Cat café keepsake',
      fonts='family=Young+Serif&amp;family=Kalam:wght@400;700',
      css=css, svg=svg, focus='#1d3a38',
      bg='radial-gradient(ellipse 55% 65% at 34% 48%, #a6c3bf 0%, #8aaba6 58%, #6d8f8a 100%)',
      text='#16302d', head='#0f2422', ctabg='#1d3a38', ctafg='#f4ead6', ctahover='#142b29',
      label='A paw-shaped card from Cat Café Studio in Versova, 9 November 2026, five stars, with a ginger cat asleep on it.',
      h1='Cat café',
      p='Our card from Cat Café Studio, the first toe stamped, and one of the regulars asleep on it. What we wrote is underneath her.',
      opened='Wake the cat', closed='Let her sleep')
