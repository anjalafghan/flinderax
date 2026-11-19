"""Batch 2, boards 6-10: PCO, Canvas Laugh Club, Carter Road, perfume, pottery."""
from frame import write, grain, torn

RM = '@media (prefers-reduced-motion:reduce){{{sel}{{transition-duration:.01s!important;transition-delay:0s!important;animation:none!important}}}}'

# ── 6. Speakeasy cocktails at PCO: a black card with an old STD·ISD·PCO booth, a scratch panel ──
svg = f'''<defs>{grain('p-g', .3, .3, .3, .1)}
<linearGradient id="p-silver" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#e9ebee"></stop><stop offset=".5" stop-color="#a8adb5"></stop><stop offset="1" stop-color="#d9dce0"></stop></linearGradient>
<radialGradient id="p-glow" cx=".5" cy=".5" r=".5"><stop offset="0" stop-color="#ffcf6a" stop-opacity=".9"></stop><stop offset="1" stop-color="#ffcf6a" stop-opacity="0"></stop></radialGradient>
<radialGradient id="p-coin" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#f6e2a0"></stop><stop offset="1" stop-color="#9c7a2a"></stop></radialGradient>
<clipPath id="p-panel"><rect x="60" y="250" width="200" height="54" rx="6"></rect></clipPath></defs>
<g transform="translate(130 90) rotate(-4)">
<path d="M0 130 C0 50 70 0 160 0 C250 0 320 50 320 130 V400 Q320 420 300 420 H20 Q0 420 0 400 Z" fill="#121214"></path><path d="M0 130 C0 50 70 0 160 0 C250 0 320 50 320 130 V400 Q320 420 300 420 H20 Q0 420 0 400 Z" filter="url(#p-g)"></path><path d="M0 130 C0 50 70 0 160 0 C250 0 320 50 320 130 V400 Q320 420 300 420 H20 Q0 420 0 400 Z" fill="none" stroke="#f2c230" stroke-opacity=".35" stroke-width="1.5" transform="translate(8 8) scale(.95)"></path>
<!-- the booth -->
<g transform="translate(110 54)">
<ellipse class="glow" cx="50" cy="90" rx="120" ry="110" fill="url(#p-glow)" opacity=".15"></ellipse>
<rect x="-6" y="0" width="112" height="30" fill="#f2c230"></rect>
<text x="50" y="21" font-family="'Space Mono', monospace" font-weight="700" font-size="14" fill="#7a1010" text-anchor="middle">STD·ISD·PCO</text>
<rect x="0" y="30" width="100" height="150" fill="#2a2a30" stroke="#f2c230" stroke-width="3"></rect>
<g class="door"><rect x="4" y="34" width="92" height="142" fill="#1b1b20"></rect><rect x="14" y="44" width="72" height="80" fill="#32353d"></rect><circle cx="82" cy="110" r="4" fill="#f2c230"></circle></g>
<g transform="translate(50 120)"><rect x="-22" y="-6" width="44" height="30" rx="5" fill="#f2c230"></rect><circle cy="9" r="9" fill="#121214"></circle><path d="M-26 -10 C-26 -24 26 -24 26 -10" stroke="#f2c230" stroke-width="8" fill="none" stroke-linecap="round"></path></g>
</g>
<text x="160" y="250" font-family="'Young Serif', serif" font-size="26" fill="#f2ecdf" text-anchor="middle" dy="-16">PCO</text>
<text x="160" y="320" font-family="'Space Mono', monospace" font-size="10" fill="#9a9aa0" text-anchor="middle" dy="28">password only · Kamala Mills</text>
<g transform="translate(0 0)">
<text x="160" y="286" font-family="'Permanent Marker', cursive" font-size="26" fill="#f2c230" text-anchor="middle">trunk call</text>
<g clip-path="url(#p-panel)"><g class="foil"><rect x="60" y="250" width="200" height="54" fill="url(#p-silver)"></rect>
<path d="M60 262 h200 M60 276 h200 M60 290 h200" stroke="#fff" stroke-opacity=".35"></path>
<text x="160" y="282" font-family="'Space Mono', monospace" font-size="10" fill="#5d626b" text-anchor="middle">scratch for tonight’s word</text></g></g>
<rect x="60" y="250" width="200" height="54" rx="6" fill="none" stroke="#5d626b" stroke-dasharray="3 3"></rect></g>
</g>
<g class="coin"><g transform="translate(470 470)"><circle r="30" fill="url(#p-coin)" stroke="#7a5a1a" stroke-width="2"></circle><text y="9" font-family="'Young Serif', serif" font-size="24" fill="#6b4c12" text-anchor="middle">₹1</text></g></g>
<g class="flakes" fill="#c9cdd3"><rect x="200" y="380" width="5" height="3"></rect><rect x="240" y="392" width="4" height="4"></rect><rect x="290" y="384" width="6" height="3"></rect><rect x="330" y="396" width="4" height="3"></rect></g>'''
css = '''
.coin,.foil,.door,.flakes rect{transform-box:fill-box}
.foil{transform-origin:100% 50%;transition:transform .3s}
.door{transform-origin:0% 50%;transition:transform .5s}
.flakes rect{opacity:0}
.open .coin{animation:scrape 1.3s ease-in-out both}
.open .foil{transform:scaleX(0);transition:transform 1.1s cubic-bezier(.5,0,.5,1) .15s}
.open .flakes rect{animation:flake 1s ease-out .3s both}.open .flakes rect:nth-child(2n){animation-delay:.6s}
.open .glow{opacity:1;transition:opacity .6s 1.3s}
.open .door{transform:scaleX(.18);transition:transform .7s cubic-bezier(.3,1,.4,1) 1.3s}
@keyframes scrape{0%{transform:translate(0,0)}15%{transform:translate(-190px,-160px)}35%{transform:translate(-300px,-160px) rotate(-20deg)}55%{transform:translate(-210px,-150px) rotate(10deg)}75%{transform:translate(-330px,-154px) rotate(-10deg)}100%{transform:translate(0,0)}}
@keyframes flake{0%{opacity:1;transform:translate(0,0)}100%{opacity:0;transform:translate(6px,60px) rotate(90deg)}}
''' + RM.format(sel='.coin,.foil,.door,.flakes rect,.glow')
write('PCO.dc.html', title='Speakeasy cocktails at PCO keepsake', fonts='family=Young+Serif&amp;family=Space+Mono:wght@400;700&amp;family=Permanent+Marker',
      css=css, svg=svg, focus='#f2c230', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #4a3a5c 0%, #362a45 58%, #241c30 100%)',
      text='#ece6f2', head='#ffffff', ctabg='#f2c230', ctafg='#1c1408', ctahover='#e0ae1c',
      label='A black PCO card with an old STD ISD PCO phone booth and a scratch panel hiding the password, from Speakeasy cocktails at PCO.',
      h1='Speakeasy cocktails at PCO', p='A phone booth door at Kamala Mills, and a password that changes every few days. Scratch it off with a rupee.',
      opened='Scratch the panel', closed='Hang up')

# ── 7. Stand-up comedy night at Canvas Laugh Club: the neon wristband and the mic on its stand ──
svg = f'''<defs>
<linearGradient id="w-band" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#d8f23a"></stop><stop offset=".5" stop-color="#f2ff6e"></stop><stop offset="1" stop-color="#c3dd25"></stop></linearGradient>
<radialGradient id="w-spot" cx=".5" cy=".5" r=".5"><stop offset="0" stop-color="#fff6d2" stop-opacity=".5"></stop><stop offset="1" stop-color="#fff6d2" stop-opacity="0"></stop></radialGradient></defs>
<ellipse class="spot" cx="420" cy="300" rx="190" ry="260" fill="url(#w-spot)" opacity="0"></ellipse>
<!-- the band, still looped the way it came off a wrist -->
<g transform="translate(250 330) rotate(-12)">
<ellipse rx="170" ry="66" fill="none" stroke="#a9bf1c" stroke-width="44"></ellipse>
<path d="M-170 0 A170 66 0 0 0 170 0" fill="none" stroke="url(#w-band)" stroke-width="44"></path>
<path d="M-170 0 A170 66 0 0 0 170 0" fill="none" stroke="#fffbd0" stroke-width="2" stroke-opacity=".6" transform="translate(0 -14)"></path>
<g font-family="'Space Mono', monospace" font-weight="700" fill="#1b1b12">
<text x="-118" y="54" font-size="15" transform="rotate(8 -118 54)">Canvas Laugh Club</text>
<text x="56" y="52" font-size="14" transform="rotate(-8 56 52)">18+ · one night</text></g>
<g transform="translate(-150 -36)"><circle r="12" fill="#1b1b12"></circle><path d="M-6 -2 a6 6 0 0 0 12 0 Z" fill="#d8f23a"></path></g>
<path d="M168 -14 l12 6 l-12 6" fill="none" stroke="#1b1b12" stroke-width="2"></path>
</g>
<!-- the stand -->
<g transform="translate(440 120)"><path d="M0 40 V420 M-50 430 H50" stroke="#2a2a2e" stroke-width="7" stroke-linecap="round"></path>
<g class="mic"><rect x="-9" y="10" width="18" height="60" rx="7" fill="#1c1c20"></rect><circle cy="0" r="22" fill="#9aa0a8"></circle><circle cy="0" r="22" fill="none" stroke="#5c6168" stroke-width="2" stroke-dasharray="2 3"></circle></g></g>
<g class="ha" font-family="'Permanent Marker', cursive" fill="#f2ff6e">
<text class="h1" x="350" y="120" font-size="44">HA</text><text class="h2" x="470" y="90" font-size="34">ha</text><text class="h3" x="380" y="60" font-size="30">HA!</text><text class="h4" x="500" y="170" font-size="26">heh</text></g>'''
css = '''
.ha text,.mic{transform-box:fill-box;transform-origin:50% 100%}
.ha text{opacity:0}
.open .spot{opacity:1;transition:opacity .5s}
.open .ha text{animation:pop .9s cubic-bezier(.3,1.6,.5,1) both}
.open .h1{animation-delay:.3s}.open .h2{animation-delay:.6s}.open .h3{animation-delay:.9s}.open .h4{animation-delay:1.25s}
.open .mic{animation:wob .5s ease-in-out .2s 3}
@keyframes pop{0%{opacity:0;transform:scale(.2) rotate(-20deg)}60%{opacity:1;transform:scale(1.15) rotate(6deg)}100%{opacity:1;transform:scale(1) rotate(-4deg)}}
@keyframes wob{50%{transform:rotate(6deg)}}
''' + RM.format(sel='.ha text,.mic,.spot')
write('ComedyNight.dc.html', title='Stand-up comedy night keepsake', fonts='family=Space+Mono:wght@700&amp;family=Permanent+Marker',
      css=css, svg=svg, focus='#f2ff6e', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #3b3550 0%, #2a253a 58%, #1c1828 100%)',
      text='#ebe8f4', head='#ffffff', ctabg='#f2ff6e', ctafg='#1b1b12', ctahover='#dcef3e',
      label='A looped neon-yellow wristband from Canvas Laugh Club and a microphone on a stand, from Stand-up comedy night.',
      h1='Stand-up comedy night', p='The wristband from Canvas Laugh Club in Lower Parel, still in its loop. The mic is waiting for a punchline.',
      opened='Cue the laugh', closed='Lights down')

# ── 8. Food critique on Carter Road: a clipboard scorecard, two inks ─────────────────────────
dishes = [('momos', '7', '8'), ('bhutta, lime + chilli', '9', '8'), ('pav bhaji', '8', '9'), ('gola, kala khatta', '10', '9'), ('the brownie stall', '6', '7')]
rows = ''
for i, (d, a, b) in enumerate(dishes):
    y = 196 + i * 46
    rows += f'<text x="24" y="{y}" font-family="Kalam, cursive" font-size="18" fill="#2b2b30">{d}</text>'
    rows += f'<text class="sc" style="--d:{.2 + i * .25}s" x="236" y="{y}" font-family="Kalam, cursive" font-weight="700" font-size="22" fill="#1f4bb5" text-anchor="middle">{a}</text>'
    rows += f'<text class="sc" style="--d:{.32 + i * .25}s" x="292" y="{y}" font-family="Kalam, cursive" font-weight="700" font-size="22" fill="#c0392b" text-anchor="middle">{b}</text>'
    rows += f'<path d="M20 {y+14} H310" stroke="#c9c2b0" stroke-width="1"></path>'
svg = f'''<defs>{grain('cr-g', .2, .15, .1, .12)}
<linearGradient id="cr-board" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#c8955a"></stop><stop offset="1" stop-color="#9c6c38"></stop></linearGradient>
<linearGradient id="cr-clip" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#e9ecef"></stop><stop offset="1" stop-color="#8e949b"></stop></linearGradient></defs>
<g transform="translate(120 60) rotate(-3)">
<rect width="360" height="480" rx="16" fill="url(#cr-board)"></rect><rect width="360" height="480" rx="16" filter="url(#cr-g)"></rect>
<g transform="translate(22 40)"><path d="M0 0 H316 V420 H0 Z" fill="#fbf8ef"></path>
<!-- the promenade, in pen at the top -->
<path d="M20 62 C80 52 140 70 200 58 C250 50 290 60 310 56" fill="none" stroke="#6a9bb4" stroke-width="2"></path>
<path d="M20 74 C80 64 140 82 200 70 C250 62 290 72 310 68" fill="none" stroke="#6a9bb4" stroke-width="1.2"></path>
<text x="24" y="44" font-family="'Young Serif', serif" font-size="22" fill="#2b2b30">Carter Road, scored</text>
<text x="236" y="120" font-family="'Space Mono', monospace" font-size="10" fill="#1f4bb5" text-anchor="middle">me</text>
<text x="292" y="120" font-family="'Space Mono', monospace" font-size="10" fill="#c0392b" text-anchor="middle">you</text>
<text x="24" y="120" font-family="'Space Mono', monospace" font-size="10" fill="#6b6658">stall / dish        out of 10</text>
<path d="M20 132 H310" stroke="#2b2b30" stroke-width="1.2"></path>
<g transform="translate(0 -40)">{rows}</g>
<path class="best" pathLength="1" d="M14 286 C60 268 250 262 318 280 C330 300 120 310 18 300 C8 294 10 288 20 284" fill="none" stroke="#2c7a3f" stroke-width="2.4"></path>
<text class="best-t" x="24" y="384" font-family="Kalam, cursive" font-size="19" fill="#2c7a3f" transform="rotate(-3 24 384)">best bite: the gola</text>
<ellipse cx="270" cy="390" rx="22" ry="16" fill="#d9a85c" opacity=".22" transform="rotate(-20 270 390)"></ellipse>
</g>
<g transform="translate(130 18)"><rect width="100" height="44" rx="8" fill="url(#cr-clip)"></rect><rect x="30" y="-10" width="40" height="20" rx="10" fill="none" stroke="#8e949b" stroke-width="5"></rect></g>
</g>
<g transform="translate(470 480) rotate(30)"><path d="M0 0 C-8 -30 -6 -90 0 -110 C6 -90 8 -30 0 0 Z" fill="#e8c34a"></path><g fill="#d1a22d"><circle cx="-2" cy="-30" r="2"></circle><circle cx="2" cy="-50" r="2"></circle><circle cx="-2" cy="-70" r="2"></circle></g><path d="M0 0 V30" stroke="#8a6a3a" stroke-width="5"></path></g>'''
css = '''
.sc{opacity:0}.best{stroke-dasharray:1;stroke-dashoffset:1}.best-t{opacity:0}
.open .sc{opacity:1;transition:opacity .15s var(--d)}
.open .best{stroke-dashoffset:0;transition:stroke-dashoffset .7s ease-out 1.6s}
.open .best-t{opacity:1;transition:opacity .3s 2.2s}
''' + RM.format(sel='.sc,.best,.best-t')
write('CarterRoad.dc.html', title='Food critique on Carter Road keepsake', fonts='family=Young+Serif&amp;family=Kalam:wght@400;700&amp;family=Space+Mono',
      css=css, svg=svg, focus='#fbe7a8', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #5d97ac 0%, #467d92 58%, #326174 100%)',
      text='#eaf4f7', head='#ffffff', ctabg='#fbe7a8', ctafg='#1d2b30', ctahover='#f1d880',
      label='A clipboard scorecard for the food stalls along Carter Road, scored out of ten in two inks, from Food critique date.',
      h1='Food critique on Carter Road', p='A stall-by-stall scorecard along the seafront at sunset, one column each, in our own pens. Arguments settled by the gola.',
      opened='Score the stalls', closed='Wipe the board')

# ── 9. Perfume making workshop: the vial, its label, a blotter ───────────────────────────────
svg = f'''<defs>
<linearGradient id="v-glass" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#ffffff" stop-opacity=".55"></stop><stop offset=".3" stop-color="#ffffff" stop-opacity=".1"></stop><stop offset=".85" stop-color="#ffffff" stop-opacity=".08"></stop><stop offset="1" stop-color="#ffffff" stop-opacity=".45"></stop></linearGradient>
<linearGradient id="v-juice" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f3c977"></stop><stop offset="1" stop-color="#d9932f"></stop></linearGradient>
<linearGradient id="v-gold" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f7e2a2"></stop><stop offset="1" stop-color="#b08a33"></stop></linearGradient>
<clipPath id="v-in"><rect x="-58" y="-150" width="116" height="250" rx="14"></rect></clipPath></defs>
<g transform="translate(250 330) rotate(-6)">
<ellipse cy="112" rx="70" ry="12" fill="#1d1420" opacity=".25"></ellipse>
<g clip-path="url(#v-in)"><rect x="-58" y="-60" width="116" height="160" fill="url(#v-juice)"></rect><ellipse cy="-60" rx="58" ry="6" fill="#f7dc9a"></ellipse></g>
<rect x="-58" y="-150" width="116" height="250" rx="14" fill="url(#v-glass)" stroke="#fff" stroke-opacity=".7" stroke-width="2"></rect>
<rect x="-22" y="-196" width="44" height="48" rx="4" fill="url(#v-gold)"></rect>
<g class="bulb"><path d="M-18 -196 C-30 -260 30 -260 18 -196 Z" fill="#2b1d33"></path></g>
<!-- the label -->
<g transform="translate(0 0)"><path d="M-48 -40 H48 V60 H-48 Z" fill="#fbf6ea"></path><path d="M-42 -34 H42 V54 H-42 Z" fill="none" stroke="#b08a33" stroke-width="1"></path>
<text y="-14" font-family="'Young Serif', serif" font-size="13" fill="#3a2a40" text-anchor="middle">Our blend</text>
<text y="2" font-family="'Young Serif', serif" font-size="13" fill="#3a2a40" text-anchor="middle">no. 1</text>
<g font-family="Kalam, cursive" font-size="10.5" fill="#6a4c7a" text-anchor="middle"><text y="20">top: citrus</text><text y="33">heart: jasmine</text><text y="46">base: sandalwood</text></g>
<path d="M0 66 C-8 76 -8 84 0 88 C8 84 8 76 0 66 Z" fill="url(#v-gold)"></path></g>
</g>
<!-- the blotter strip -->
<g class="strip"><g transform="translate(420 160) rotate(14)"><path d="M0 0 H34 V260 L17 280 L0 260 Z" fill="#fbf8f2"></path><text transform="translate(22 230) rotate(-90)" font-family="'Space Mono', monospace" font-size="9" fill="#8a7a90">Small World Perfumery</text>
<ellipse class="wet" cx="17" cy="250" rx="12" ry="18" fill="#e9b65a" opacity="0"></ellipse></g></g>
<g class="drop"><path d="M270 120 C262 132 262 140 270 144 C278 140 278 132 270 120 Z" fill="#e9b65a" opacity="0"></path></g>
<g class="notes" font-family="Newsreader, serif" font-style="italic" fill="#fbeaf5">
<text class="n1" x="330" y="500" font-size="24">bergamot</text><text class="n2" x="440" y="540" font-size="22">jasmine</text><text class="n3" x="340" y="572" font-size="24">sandalwood</text></g>
<g class="mist" fill="#fbeaf5"><circle cx="300" cy="96" r="3"></circle><circle cx="320" cy="84" r="2"></circle><circle cx="340" cy="100" r="2.5"></circle><circle cx="312" cy="70" r="1.8"></circle></g>'''
css = '''
.bulb,.strip,.notes text,.mist circle{transform-box:fill-box}
.bulb{transform-origin:50% 100%}
.notes text,.mist circle{opacity:0}
.strip{transform-origin:50% 0%;transition:transform .5s}
.open .bulb{animation:squeeze .5s ease-in-out 2}
.open .mist circle{animation:mist 1.2s ease-out .2s both}.open .mist circle:nth-child(2n){animation-delay:.5s}
.open .strip{animation:wave 1.4s ease-in-out .6s 2}
.open .wet{opacity:.8;transition:opacity .4s .6s}
.open .notes text{animation:waft 3s ease-out both}.open .n1{animation-delay:1s}.open .n2{animation-delay:1.5s}.open .n3{animation-delay:2s}
@keyframes squeeze{50%{transform:scale(.8,.86)}}
@keyframes mist{0%{opacity:0;transform:translate(0,0)}30%{opacity:1}100%{opacity:0;transform:translate(70px,-30px) scale(3)}}
@keyframes wave{0%,100%{transform:rotate(0)}30%{transform:rotate(8deg)}70%{transform:rotate(-6deg)}}
@keyframes waft{0%{opacity:0;transform:translate(0,20px)}30%{opacity:1}100%{opacity:.85;transform:translate(-30px,-40px)}}
''' + RM.format(sel='.bulb,.strip,.notes text,.mist circle,.wet')
write('Perfume.dc.html', title='Perfume making workshop keepsake', fonts='family=Young+Serif&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f7e2a2', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #8a6a9c 0%, #6f5282 58%, #553d66 100%)',
      text='#f7eefb', head='#ffffff', ctabg='#f7e2a2', ctafg='#2b1d33', ctahover='#ebd082',
      label='A small vial labelled Our blend number one, with its top, heart and base notes written in, and a blotter strip, from Perfume making workshop.',
      h1='Perfume making workshop', p='Ten millilitres of something only we smell like, mixed drop by drop at the workshop in Vashi.',
      opened='Spray it', closed='Cap it')

# ── 10. Pottery class: two thumbprints on a terracotta tag, and our wobbly bowl on the wheel ──
TAG = 'M0 20 Q0 0 20 0 H170 L210 50 L170 100 H20 Q0 100 0 80 Z'
svg = f'''<defs>{grain('pt-g', .3, .15, .05, .18)}
<pattern id="pt-print" width="10" height="10" patternUnits="userSpaceOnUse"><path d="M0 5 Q5 0 10 5" fill="none" stroke="#7e3c1c" stroke-width="1"></path></pattern>
<radialGradient id="pt-wheel" cx=".5" cy=".4" r=".7"><stop offset="0" stop-color="#9aa0a6"></stop><stop offset="1" stop-color="#5a6066"></stop></radialGradient>
<linearGradient id="pt-clay" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8f4a26"></stop><stop offset=".45" stop-color="#c47a4a"></stop><stop offset="1" stop-color="#7d3e1f"></stop></linearGradient></defs>
<!-- the wheel -->
<ellipse cx="300" cy="420" rx="210" ry="66" fill="#3b3f44"></ellipse>
<g class="spin"><ellipse cx="300" cy="400" rx="190" ry="58" fill="url(#pt-wheel)"></ellipse>
<g stroke="#4a4f55" stroke-width="2"><path d="M140 400 H460"></path><path d="M300 344 V456"></path></g>
<ellipse cx="300" cy="400" rx="120" ry="36" fill="none" stroke="#c47a4a" stroke-opacity=".5" stroke-width="3"></ellipse></g>
<g class="bowl"><path d="M220 330 C214 380 240 404 300 404 C362 404 388 378 378 326 C350 340 250 342 220 330 Z" fill="url(#pt-clay)"></path>
<ellipse cx="299" cy="328" rx="80" ry="16" fill="#6b3218"></ellipse><path d="M222 330 C260 316 340 314 378 326" fill="none" stroke="#d9925e" stroke-width="3"></path>
<path d="M300 344 C296 360 304 372 300 390" stroke="#a65a30" stroke-width="2" fill="none"></path></g>
<g class="splat" fill="#b5683a"><circle cx="150" cy="330" r="5"></circle><circle cx="460" cy="320" r="4"></circle><circle cx="430" cy="280" r="3"></circle><circle cx="170" cy="290" r="3"></circle></g>
<!-- the tag -->
<g class="tag"><g transform="translate(110 110) rotate(-8)">
<path d="{TAG}" fill="#c4683c"></path><path d="{TAG}" filter="url(#pt-g)"></path>
<circle cx="186" cy="50" r="7" fill="#3a2a20"></circle>
<path d="M186 50 C220 40 250 70 280 50" fill="none" stroke="#efe4cf" stroke-width="2"></path>
<g transform="translate(46 52)"><ellipse rx="20" ry="26" fill="#a9522a" transform="rotate(-10)"></ellipse><g transform="rotate(-10)" fill="none" stroke="#6e2e12" stroke-width="1.6" opacity="0.9"><ellipse rx="18.0" ry="24.0"></ellipse><ellipse rx="14.0" ry="18.7"></ellipse><ellipse rx="10.1" ry="13.4"></ellipse><ellipse rx="6.1" ry="8.2"></ellipse></g></g>
<g transform="translate(72 50)"><ellipse rx="19" ry="25" fill="#a9522a" opacity=".8" transform="rotate(14)"></ellipse><g transform="rotate(14)" fill="none" stroke="#6e2e12" stroke-width="1.6" opacity="0.75"><ellipse rx="17.0" ry="23.0"></ellipse><ellipse rx="13.3" ry="17.9"></ellipse><ellipse rx="9.5" ry="12.9"></ellipse><ellipse rx="5.8" ry="7.8"></ellipse></g></g>
<text x="138" y="40" font-family="'Young Serif', serif" font-size="15" fill="#3a1a0c" text-anchor="middle">UV’s Pottery</text>
<g transform="translate(138 66) rotate(-6)"><rect x="-30" y="-12" width="60" height="22" rx="3" fill="none" stroke="#3a1a0c" stroke-width="1.6"></rect><text y="4" font-family="'Space Mono', monospace" font-size="11" fill="#3a1a0c" text-anchor="middle">fired</text></g>
</g></g>'''
css = '''
.spin,.bowl,.tag,.splat circle{transform-box:fill-box}
.spin{transform-origin:50% 50%}
.bowl{transform-origin:50% 100%}
.tag{transform-origin:85% 50%}
.splat circle{opacity:0}
.open .spin{animation:spin .35s linear 6}
.open .bowl{animation:wobble 2.1s ease-in-out both}
.open .splat circle{animation:splat .6s ease-out .9s both}
.open .tag{animation:swing 1.8s ease-in-out .3s}
@keyframes spin{to{transform:scaleX(-1)}}
@keyframes wobble{0%{transform:none}20%{transform:skewX(6deg) scaleY(1.06)}40%{transform:skewX(-7deg) scaleY(1.1)}60%{transform:skewX(5deg) scaleY(1.12)}80%{transform:skewX(-3deg) scaleY(1.1)}100%{transform:skewX(2deg) scaleY(1.08)}}
@keyframes splat{0%{opacity:0;transform:scale(0)}50%{opacity:1;transform:scale(1.3)}100%{opacity:1;transform:scale(1)}}
@keyframes swing{0%,100%{transform:rotate(0)}30%{transform:rotate(6deg)}65%{transform:rotate(-4deg)}}
''' + RM.format(sel='.spin,.bowl,.tag,.splat circle')
write('Pottery.dc.html', title='Pottery class keepsake', fonts='family=Young+Serif&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f3e2c6', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #b9a58a 0%, #9d896d 58%, #7e6c53 100%)',
      text='#251a10', head='#1c120a', ctabg='#5a2d16', ctafg='#f7eadc', ctahover='#47220f',
      label='A terracotta tag from UV’s Pottery with two overlapping thumbprints, and a wobbly bowl on the wheel, from Pottery class together.',
      h1='Pottery class together', p='Two thumbprints pressed into the tag before it went in the kiln, and the bowl that came out the shape it wanted to be.',
      opened='Spin the wheel', closed='Let it dry')
