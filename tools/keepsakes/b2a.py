"""Batch 2, boards 1-5: karaoke, Saltt, Luna Et Sol, Bombay Canteen, Chaitanya."""
from frame import write, grain, torn

RM = '@media (prefers-reduced-motion:reduce){{{sel}{{transition-duration:.01s!important;transition-delay:0s!important;animation:none!important}}}}'

# ── 1. Karaoke night at Road House Bluez: the pink song-request slip, and the mic ───────────────
SLIP = torn([(0, 0), (290, 0), (290, 370), (0, 370)], seed=3, amp=2.5, step=11)
svg = f'''<defs>{grain('q-g', .3, .1, .2, .14)}
<linearGradient id="q-mic" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f4f4f4"></stop><stop offset=".5" stop-color="#a9adb4"></stop><stop offset="1" stop-color="#5d626b"></stop></linearGradient>
<pattern id="q-mesh" width="6" height="6" patternUnits="userSpaceOnUse"><circle cx="3" cy="3" r="1.3" fill="#4b4f57"></circle></pattern>
<radialGradient id="q-spot" cx=".5" cy=".5" r=".5"><stop offset="0" stop-color="#ffe6f0" stop-opacity=".42"></stop><stop offset=".6" stop-color="#ffe6f0" stop-opacity=".14"></stop><stop offset="1" stop-color="#ffe6f0" stop-opacity="0"></stop></radialGradient></defs>
<ellipse class="spot" cx="330" cy="330" rx="300" ry="270" fill="url(#q-spot)"></ellipse>
<g transform="translate(110 96) rotate(-5)">
<path d="{SLIP}" fill="#f7b9cf"></path><path d="{SLIP}" filter="url(#q-g)"></path>
<g fill="#c4437a"><circle cx="20" cy="10" r="3"></circle><circle cx="270" cy="10" r="3"></circle></g>
<text x="145" y="52" font-family="'Young Serif', serif" font-size="27" fill="#5a1838" text-anchor="middle">Road House Bluez</text>
<text x="145" y="72" font-family="'Space Mono', monospace" font-size="10" fill="#8a2a55" text-anchor="middle">karaoke · song request · Andheri W</text>
<g font-family="'Space Mono', monospace" font-size="11" fill="#8a2a55">
<text x="22" y="118">Song</text><text x="22" y="174">Singer(s)</text><text x="22" y="230">Key</text><text x="160" y="230">Table</text></g>
<g stroke="#c4437a" stroke-width="1"><path d="M22 136 H268 M22 192 H268 M22 248 H140 M160 248 H268"></path></g>
<g font-family="Kalam, cursive" font-size="22" fill="#2846a8">
<text x="70" y="132" fill="#9aa9e0" transform="translate(1.5 1.5)">our duet, obviously</text><text x="70" y="132">our duet, obviously</text>
<text x="100" y="188">both of us</text><text x="58" y="244">whatever</text><text x="214" y="244" font-size="26">7</text></g>
<path d="M30 300 h230" stroke="#c4437a" stroke-dasharray="4 4"></path>
<text x="145" y="330" font-family="'Space Mono', monospace" font-size="12" fill="#5a1838" text-anchor="middle">next up: table 7</text>
<g class="up" transform="translate(150 196) rotate(-12)"><rect x="-92" y="-30" width="184" height="58" rx="8" fill="none" stroke="#c0182e" stroke-width="4"></rect>
<text y="12" font-family="'Permanent Marker', cursive" font-size="32" fill="#c0182e" text-anchor="middle">you’re up!</text></g>
</g>
<g class="notes" font-family="'Young Serif', serif" fill="#ffd5e6"><text class="n1" x="420" y="250" font-size="34">♪</text><text class="n2" x="470" y="200" font-size="26">♫</text><text class="n3" x="440" y="160" font-size="30">♪</text></g>
<g class="waves" fill="none" stroke="#ffd5e6" stroke-width="3" stroke-linecap="round"><path class="w1" d="M476 330 a40 40 0 0 1 0 60"></path><path class="w2" d="M496 314 a62 62 0 0 1 0 92"></path></g>
<path d="M420 520 C470 560 540 520 520 470 C505 440 470 470 440 450" fill="none" stroke="#1b1b20" stroke-width="5" stroke-linecap="round"></path>
<g class="mic" transform="translate(430 440)"><g class="mic-in">
<rect x="-11" y="-6" width="22" height="104" rx="9" fill="#1f2026"></rect>
<rect x="-14" y="-12" width="28" height="12" rx="3" fill="#7f848c"></rect>
<circle cx="0" cy="-42" r="34" fill="url(#q-mic)"></circle><circle cx="0" cy="-42" r="34" fill="url(#q-mesh)" opacity=".55"></circle>
<path d="M-24 -62 a32 32 0 0 1 22 -12" stroke="#fff" stroke-width="3" fill="none" stroke-linecap="round" opacity=".8"></path>
<rect x="-4" y="30" width="8" height="14" rx="2" fill="#d7263d"></rect></g></g>'''
css = '''
.mic,.mic-in,.up{transform-box:fill-box}
.mic-in{transform-origin:50% 100%;transform:rotate(68deg);transition:transform .5s cubic-bezier(.5,0,.75,0)}
.up{transform-origin:50% 50%;opacity:0;scale:1.6;transition:opacity .1s,scale .2s}
.spot,.notes text,.waves path{opacity:0;transition:opacity .3s}
.open .mic-in{transform:rotate(0);transition:transform .7s cubic-bezier(.3,1.4,.5,1)}
.open .spot{opacity:1;transition:opacity .6s .2s}
.open .up{opacity:1;scale:1;transition:opacity .05s 1s,scale .25s cubic-bezier(.3,1.6,.5,1) 1s}
.open .waves path{animation:ring 1.2s ease-out .7s 3 both}.open .w2{animation-delay:.9s}
.open .notes text{animation:float 2.2s ease-out both}.open .n1{animation-delay:.8s}.open .n2{animation-delay:1.1s}.open .n3{animation-delay:1.4s}
@keyframes ring{0%{opacity:0}30%{opacity:1}100%{opacity:0;transform:translateX(14px)}}
@keyframes float{0%{opacity:0;transform:translateY(20px)}25%{opacity:1}100%{opacity:0;transform:translate(10px,-90px) rotate(14deg)}}
''' + RM.format(sel='.mic-in,.up,.spot,.notes text,.waves path')
write('Karaoke.dc.html', title='Karaoke night keepsake', fonts='family=Young+Serif&amp;family=Kalam:wght@400;700&amp;family=Space+Mono&amp;family=Permanent+Marker',
      css=css, svg=svg, focus='#ffd5e6', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #7a4a80 0%, #5d3564 58%, #43244a 100%)',
      text='#f6e8f1', head='#fff7fb', ctabg='#ffd5e6', ctafg='#3a1430', ctahover='#f7b9cf',
      label='A pink song-request slip from Road House Bluez and a microphone, from Karaoke night.',
      h1='Karaoke night at Road House Bluez', p='The carbon copy of our request, filled in with the confidence of two people who have not warmed up. Table seven is next.',
      opened='Pick up the mic', closed='Put it down')

# ── 2. Dinner at Saltt, Karjat: the receipt from the stop on a long drive ─────────────────────
REC = torn([(0, 0), (210, 0), (210, 400), (0, 400)], seed=8, amp=3, step=9)
ROAD = 'M372 560 C330 520 470 500 440 450 C410 400 330 420 350 360 C370 300 470 320 450 250 C430 190 360 210 380 140'
svg = f'''<defs>{grain('s-g', .2, .18, .12, .12)}
<linearGradient id="s-sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1f2b46"></stop><stop offset="1" stop-color="#3d4c6a"></stop></linearGradient><filter id="s-blur" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="28"></feGaussianBlur></filter></defs>
<g transform="translate(70 90) rotate(-3)">
<path d="{REC}" fill="#fbfaf5"></path><path d="{REC}" filter="url(#s-g)"></path>
<g font-family="'Space Mono', monospace" fill="#3b3b40">
<text x="105" y="44" font-family="'Young Serif', serif" font-size="30" fill="#25262b" text-anchor="middle">Saltt</text>
<text x="105" y="64" font-size="9.5" text-anchor="middle">restaurant &amp; bar · Karjat</text>
<text x="105" y="80" font-size="9.5" text-anchor="middle">old Mumbai–Pune road</text>
<path d="M14 96 H196" stroke="#8a8a8a" stroke-dasharray="3 3"></path>
<text x="14" y="120" font-size="11">table for 2</text><text x="196" y="120" font-size="11" text-anchor="end">window</text>
<text x="14" y="146" font-size="11">dinner, both</text><text x="196" y="146" font-size="11" text-anchor="end">✓</text>
<text x="14" y="168" font-size="11">fresh lime soda</text><text x="196" y="168" font-size="11" text-anchor="end">×2</text>
<text x="14" y="190" font-size="11">cutting chai</text><text x="196" y="190" font-size="11" text-anchor="end">×2</text>
<path d="M14 206 H196" stroke="#8a8a8a" stroke-dasharray="3 3"></path>
<text x="14" y="232" font-size="13" font-weight="700">km on the clock</text><text x="196" y="232" font-size="13" font-weight="700" text-anchor="end">62</text>
<text x="105" y="270" font-size="9.5" text-anchor="middle">thank you · drive safe</text>
<text x="105" y="286" font-size="9.5" text-anchor="middle">the ghats are wet tonight</text></g>
<g class="circ"><path d="M30 318 C40 296 180 296 186 322 C192 348 40 352 28 330" fill="none" stroke="#2a54a6" stroke-width="2" pathLength="1"></path></g>
<text x="105" y="330" font-family="Kalam, cursive" font-size="20" fill="#2a54a6" text-anchor="middle" transform="rotate(-3 105 330)">dinner on the way</text>
</g>
<!-- the ghat, pencilled up the side -->
<g transform="translate(0 0)">
<ellipse class="sky" cx="440" cy="330" rx="170" ry="260" fill="url(#s-sky)" opacity="0" filter="url(#s-blur)"></ellipse>
<circle class="moon" cx="510" cy="130" r="20" fill="#f5edc8" opacity="0"></circle>
<path d="M300 300 C340 250 380 270 410 220 C440 180 470 210 500 170 C520 150 540 160 560 150" fill="none" stroke="#5c6d4f" stroke-width="2"></path>
<path d="M300 380 C360 330 400 360 440 300 C470 270 520 290 560 250" fill="none" stroke="#5c6d4f" stroke-width="2"></path>
<path d="{ROAD}" fill="none" stroke="#3c3c3c" stroke-width="16" stroke-linecap="round"></path>
<path d="{ROAD}" fill="none" stroke="#f5f1e3" stroke-width="1.6" stroke-dasharray="7 7"></path>
<g fill="#7d9c57"><circle cx="330" cy="470" r="12"></circle><circle cx="480" cy="390" r="10"></circle><circle cx="320" cy="250" r="9"></circle><circle cx="490" cy="200" r="11"></circle></g>
</g>
<g class="car"><g transform="translate(-14 -9)">
<path class="beam" d="M28 4 L80 -14 L80 26 Z" fill="#fff1a8" opacity="0"></path>
<rect width="28" height="18" rx="5" fill="#c8362a"></rect><rect x="6" y="3" width="13" height="7" rx="2" fill="#bfe1ee"></rect>
<circle cx="6" cy="18" r="3.4" fill="#222"></circle><circle cx="22" cy="18" r="3.4" fill="#222"></circle></g></g>'''
css = '''
.car{transform:translate(372px,560px) rotate(-120deg)}
.open .car{animation:drive 3.2s cubic-bezier(.45,0,.4,1) .2s both}
.open .beam{opacity:.75;transition:opacity .2s .2s}
.open .sky{opacity:.55;transition:opacity 1.5s .4s}.open .moon{opacity:1;transition:opacity 1s 1.6s}
.circ path{stroke-dasharray:1;stroke-dashoffset:1;transition:stroke-dashoffset .3s}
.open .circ path{stroke-dashoffset:0;transition:stroke-dashoffset .7s ease-out 3.2s}
@keyframes drive{0%{transform:translate(372px,560px) rotate(-120deg)}14%{transform:translate(420px,500px) rotate(-30deg)}26%{transform:translate(440px,452px) rotate(-100deg)}40%{transform:translate(360px,410px) rotate(-170deg)}52%{transform:translate(352px,356px) rotate(-80deg)}66%{transform:translate(440px,300px) rotate(-20deg)}78%{transform:translate(446px,240px) rotate(-110deg)}90%{transform:translate(390px,190px) rotate(-120deg)}100%{transform:translate(380px,142px) rotate(-90deg)}}
''' + RM.format(sel='.car,.beam,.sky,.moon,.circ path')
write('Saltt.dc.html', title='Dinner at Saltt keepsake', fonts='family=Young+Serif&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f2e3b3', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #6f8a64 0%, #57704f 58%, #3f553b 100%)',
      text='#eef2e6', head='#fbfff4', ctabg='#f2e3b3', ctafg='#26301f', ctahover='#e6d394',
      label='A receipt from Saltt in Karjat with a ghat road pencilled up the side, from Dinner at Saltt.',
      h1='Dinner at Saltt', p='The stop on the long drive out of the city. Someone drew the road up the side of the bill while we waited.',
      opened='Take the ghat', closed='Head back')

# ── 3. Dinner at Luna Et Sol: the fondue pot and the fork's little tag ─────────────────────────
svg = f'''<defs>{grain('l-g', .2, .1, .05, .12)}
<pattern id="l-check" width="24" height="24" patternUnits="userSpaceOnUse"><rect width="24" height="24" fill="#f6efe3"></rect><rect width="12" height="12" fill="#c8312b"></rect><rect x="12" y="12" width="12" height="12" fill="#c8312b"></rect></pattern>
<linearGradient id="l-pot" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#7d1a16"></stop><stop offset=".4" stop-color="#c8312b"></stop><stop offset="1" stop-color="#6a1411"></stop></linearGradient>
<radialGradient id="l-cheese" cx=".5" cy=".4" r=".7"><stop offset="0" stop-color="#ffe7a0"></stop><stop offset="1" stop-color="#eab94a"></stop></radialGradient>
<linearGradient id="l-brass" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f6d98a"></stop><stop offset="1" stop-color="#a77a26"></stop></linearGradient></defs>
<path d="M60 436 C200 420 380 420 540 404 C548 456 552 500 560 560 C400 580 220 570 50 584 C56 530 52 480 60 436 Z" fill="url(#l-check)"></path>
<path d="M50 584 C220 570 400 580 560 560" fill="none" stroke="#f6efe3" stroke-width="5" stroke-dasharray="2 5"></path>
<g class="flame"><path d="M300 470 C288 452 296 440 300 428 C306 442 314 452 300 470 Z" fill="#ffb347"></path><path d="M300 466 C294 456 298 450 300 442 C303 450 306 456 300 466 Z" fill="#fff1b8"></path></g>
<path d="M232 476 H368 L356 492 H244 Z" fill="#3a3a3a"></path>
<path d="M190 330 C190 420 220 456 300 456 C380 456 410 420 410 330 Z" fill="url(#l-pot)"></path>
<path d="M410 346 C470 340 500 350 512 356" stroke="#5c1210" stroke-width="12" stroke-linecap="round" fill="none"></path>
<ellipse cx="300" cy="330" rx="110" ry="26" fill="#5c1210"></ellipse>
<ellipse class="cheese" cx="300" cy="334" rx="98" ry="20" fill="url(#l-cheese)"></ellipse>
<g class="snow" fill="#ffffff"><circle cx="120" cy="80" r="3"></circle><circle cx="470" cy="60" r="2.5"></circle><circle cx="200" cy="40" r="2"></circle><circle cx="380" cy="110" r="3.4"></circle><circle cx="530" cy="180" r="2.2"></circle><circle cx="90" cy="200" r="2.6"></circle></g>
<g class="fork"><g transform="translate(300 330) rotate(18)">
<path class="string" d="M0 0 C-4 20 4 30 0 50" stroke="#f3cf68" stroke-width="5" fill="none" stroke-linecap="round"></path>
<rect x="-3" y="-250" width="6" height="246" fill="#c9ccd1"></rect><rect x="-6" y="-260" width="12" height="70" rx="5" fill="#2c4a7a"></rect>
<path d="M-8 -4 L-8 18 M0 -4 L0 22 M8 -4 L8 18" stroke="#c9ccd1" stroke-width="2.4"></path>
<rect class="bread" x="-15" y="14" width="30" height="26" rx="5" fill="#d9a85c"></rect>
<path class="drip" d="M-10 38 C-10 52 -2 50 -2 62 C-2 70 -12 70 -10 62" fill="#f3cf68"></path>
<!-- the brass-ringed tag -->
<g transform="translate(0 -150)"><circle r="13" fill="none" stroke="url(#l-brass)" stroke-width="5"></circle>
<g transform="translate(16 6) rotate(-18)"><rect width="126" height="78" rx="8" fill="url(#l-check)"></rect><rect x="7" y="7" width="112" height="64" rx="5" fill="#fbf6ec"></rect>
<text x="63" y="32" font-family="'Young Serif', serif" font-size="17" fill="#7d1a16" text-anchor="middle">Luna Et Sol</text>
<text x="63" y="50" font-family="'Space Mono', monospace" font-size="9" fill="#4a3a30" text-anchor="middle">fork no. 2</text>
<path d="M63 56 l0 10 M58 61 h10 M59 57 l8 8 M67 57 l-8 8" stroke="#2c4a7a" stroke-width="1.4"></path></g></g>
</g></g>
<g transform="translate(400 470) rotate(12)"><rect width="110" height="56" rx="6" fill="#fbf6ec"></rect><text x="55" y="26" font-family="Kalam, cursive" font-size="15" fill="#7d1a16" text-anchor="middle">save room for</text><text x="55" y="44" font-family="Kalam, cursive" font-size="15" fill="#7d1a16" text-anchor="middle">the apple strudel</text></g>'''
css = '''
.fork,.flame,.snow circle,.drip,.string{transform-box:fill-box}
.fork{transition:transform .45s ease-in}
.string,.drip{opacity:0}
.flame{transform-origin:50% 100%;animation:flick 1.6s ease-in-out infinite alternate}
.open .fork{transform:translate(40px,-130px) rotate(-8deg);transition:transform .9s cubic-bezier(.3,1.2,.5,1)}
.open .string{opacity:1;transition:opacity .2s .1s}
.open .drip{animation:drip 1.4s ease-in .9s 2}
.open .snow circle{animation:fall 3s linear both}
.open .snow circle:nth-child(2n){animation-delay:.5s}.open .snow circle:nth-child(3n){animation-delay:1s}
@keyframes flick{from{transform:scale(1,1)}to{transform:scale(.9,1.12)}}
@keyframes drip{0%{opacity:1;transform:translateY(0)}70%{opacity:1;transform:translateY(40px)}100%{opacity:0;transform:translateY(60px)}}
@keyframes fall{from{transform:translateY(-40px);opacity:0}20%{opacity:1}to{transform:translateY(240px);opacity:0}}
''' + RM.format(sel='.fork,.flame,.snow circle,.drip,.string')
write('LunaEtSol.dc.html', title='Dinner at Luna Et Sol keepsake', fonts='family=Young+Serif&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f6efe3', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #3f5d7d 0%, #2f4865 58%, #22354c 100%)',
      text='#e9f0f7', head='#ffffff', ctabg='#f6efe3', ctafg='#22354c', ctahover='#e5dccb',
      label='A fondue pot on a red check cloth with a fork tagged Luna Et Sol, fork number two, from Dinner at Luna Et Sol.',
      h1='Dinner at Luna Et Sol', p='An alpine night in Lower Parel: fondue at the table, a brass-ringed tag on each fork so nobody steals yours.',
      opened='Dip the bread', closed='Back in the pot')

# ── 4. Dinner at The Bombay Canteen: the day's menu strip, and a kaali-peeli taxi ─────────────
MENU = torn([(0, 0), (250, 0), (250, 390), (0, 390)], seed=14, amp=3.5, step=12)
items = [('eggs kejriwal', True), ('ghee roast chicken seekh', True), ('kanda bhaji', False), ('kokum something', False), ('the dessert of the day', False)]
rows = ''
for i, (t, tick) in enumerate(items):
    y = 160 + i * 38
    rows += f'<text x="44" y="{y}" font-family="Fraunces, serif" font-size="15" fill="#f4ecd8">{t}</text>'
    if tick:
        rows += f'<path class="tick" style="--d:{.3 + i * .4}s" pathLength="1" d="M20 {y-8} l6 7 l12 -16" stroke="#f1c84b" stroke-width="2.6" fill="none" stroke-linecap="round"></path>'
svg = f'''<defs>{grain('b-g', .1, .2, .2, .14)}</defs>
<g transform="translate(92 92) rotate(-4)">
<path d="{MENU}" fill="#1e6a66"></path><path d="{MENU}" filter="url(#b-g)"></path>
<g class="chim" transform="translate(196 36)"><path d="M0 70 V8 H14 V70 Z M-24 70 H40" stroke="#f4ecd8" stroke-width="1.6" fill="none"></path>
<path d="M2 18 h10 M2 30 h10 M2 42 h10" stroke="#f4ecd8" stroke-width="1"></path>
<g class="smoke" fill="none" stroke="#f4ecd8" stroke-width="1.6" stroke-linecap="round"><path d="M7 2 c-6 -8 6 -12 0 -20"></path><path d="M10 -24 c-6 -8 6 -12 0 -20"></path></g></g>
<text x="24" y="58" font-family="Fraunces, serif" font-style="italic" font-size="26" fill="#f4ecd8">The Bombay</text>
<text x="24" y="88" font-family="Fraunces, serif" font-style="italic" font-size="26" fill="#f4ecd8">Canteen</text>
<text x="24" y="118" font-family="'Space Mono', monospace" font-size="9.5" fill="#bfe0d8">today · Kamala Mills, Lower Parel</text>
{rows}
<text x="125" y="370" font-family="Kalam, cursive" font-size="17" fill="#f1c84b" text-anchor="middle">reserve for the weekend</text>
</g>
<!-- a kaali-peeli, the kind on the walls there -->
<g class="taxi"><g transform="translate(330 430)">
<rect x="0" y="34" width="200" height="54" rx="14" fill="#161616"></rect>
<path d="M30 36 C40 4 60 0 100 0 C140 0 156 6 168 36 Z" fill="#161616"></path>
<path d="M44 34 C52 12 64 8 96 8 L96 34 Z M104 8 C134 8 146 12 156 34 L104 34 Z" fill="#9fc7d6"></path>
<rect x="0" y="62" width="200" height="26" rx="10" fill="#f2c230"></rect>
<rect x="76" y="-14" width="48" height="14" rx="3" fill="#f2c230"></rect><text x="100" y="-3" font-family="'Space Mono', monospace" font-size="9" fill="#161616" text-anchor="middle">TAXI</text>
<circle cx="44" cy="90" r="17" fill="#222"></circle><circle cx="44" cy="90" r="7" fill="#999"></circle>
<circle cx="158" cy="90" r="17" fill="#222"></circle><circle cx="158" cy="90" r="7" fill="#999"></circle>
<circle class="lamp" cx="196" cy="54" r="5" fill="#fff4b0"></circle></g></g>'''
css = '''
.tick{stroke-dasharray:1;stroke-dashoffset:1}
.smoke path{opacity:0}
.taxi{transform-box:fill-box;transition:transform .4s ease-in}
.open .tick{stroke-dashoffset:0;transition:stroke-dashoffset .35s ease-out var(--d)}
.open .smoke path{animation:puff 2.2s ease-out .4s 2}.open .smoke path:nth-child(2){animation-delay:.9s}
.open .taxi{transform:translateX(-40px);transition:transform 1.2s cubic-bezier(.3,1.1,.4,1) .6s}
.open .lamp{animation:blink .4s steps(2) .6s 4}
@keyframes puff{0%{opacity:0;transform:translateY(0)}30%{opacity:.9}100%{opacity:0;transform:translate(6px,-30px)}}
@keyframes blink{50%{opacity:.2}}
''' + RM.format(sel='.tick,.smoke path,.taxi,.lamp')
write('BombayCanteen.dc.html', title='Dinner at The Bombay Canteen keepsake', fonts='family=Fraunces:ital,opsz,wght@0,9..144,500;1,9..144,500&amp;family=Kalam&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f1c84b', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #e3a35a 0%, #cc8840 58%, #a86a2c 100%)',
      text='#2a1a0a', head='#1d1206', ctabg='#1e6a66', ctafg='#f4ecd8', ctahover='#17524f',
      label='A torn teal menu strip from The Bombay Canteen and a toy kaali-peeli taxi, from Dinner at The Bombay Canteen.',
      h1='Dinner at The Bombay Canteen', p='The day’s menu, torn off at Kamala Mills, with the two dishes we came for ticked in pencil.',
      opened='Order the usual', closed='Rub it out')

# ── 5. Malvani seafood at Chaitanya: the banana-leaf thali ───────────────────────────────────
LEAF = 'M40 330 C40 230 140 120 300 104 C420 92 520 150 580 250 C540 250 526 262 560 300 C530 300 520 316 548 340 C510 440 420 506 290 516 C150 526 40 450 40 330 Z'
svg = f'''<defs>{grain('c2-g', .05, .2, .05, .16)}
<linearGradient id="c2-leaf" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#5f9a3e"></stop><stop offset=".6" stop-color="#3f7d2c"></stop><stop offset="1" stop-color="#2d6420"></stop></linearGradient>
<linearGradient id="c2-steel" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#8e949a"></stop><stop offset=".5" stop-color="#eef1f3"></stop><stop offset="1" stop-color="#7d838a"></stop></linearGradient></defs>
<g transform="rotate(-8 300 300)"><path d="{LEAF}" fill="url(#c2-leaf)"></path><path d="{LEAF}" filter="url(#c2-g)"></path>
<path d="M44 330 C200 300 400 290 576 252" stroke="#a8cf7a" stroke-width="4" fill="none"></path>
<g stroke="#8fbd62" stroke-width="1.4" fill="none" opacity=".8"><path d="M130 300 C150 240 180 200 220 170 M200 300 C220 230 260 180 300 150 M280 300 C300 220 340 170 380 140 M360 300 C380 230 420 190 460 170 M130 300 C150 360 180 400 220 430 M200 300 C220 370 260 420 300 450 M280 300 C300 380 340 430 380 460 M360 300 C380 370 420 410 460 430"></path></g></g>
<!-- rice, and the katoris round it -->
<ellipse cx="290" cy="320" rx="70" ry="44" fill="#f7f3e6"></ellipse><g fill="#e8e1cc"><circle cx="270" cy="306" r="3"></circle><circle cx="300" cy="330" r="3"></circle><circle cx="320" cy="310" r="3"></circle></g>
<g><ellipse cx="200" cy="200" rx="56" ry="38" fill="url(#c2-steel)"></ellipse><ellipse cx="200" cy="196" rx="46" ry="28" fill="#d8641e"></ellipse>
<g fill="#f2a46b" stroke="#c2501a" stroke-width="1.5"><path d="M176 192 c8 -14 26 -12 28 2 c-8 -4 -16 -2 -20 6 z"></path><path d="M206 200 c8 -14 26 -12 28 2 c-8 -4 -16 -2 -20 6 z"></path></g></g>
<g><ellipse cx="380" cy="190" rx="52" ry="34" fill="url(#c2-steel)"></ellipse><ellipse cx="380" cy="186" rx="42" ry="25" fill="#b0461f"></ellipse></g>
<g transform="translate(410 330) rotate(-20)"><path d="M-70 0 C-50 -34 50 -34 70 0 C50 30 -50 30 -70 0 Z" fill="#c98a3a"></path><path d="M-70 0 C-50 -34 50 -34 70 0 C50 30 -50 30 -70 0 Z" fill="none" stroke="#a5662a" stroke-width="2" stroke-dasharray="2 4"></path><path d="M70 0 l20 -14 v28 z" fill="#c98a3a"></path></g>
<g fill="#f5d6a8"><ellipse cx="190" cy="410" rx="44" ry="22"></ellipse><ellipse cx="210" cy="424" rx="44" ry="22" fill="#efc890"></ellipse></g>
<!-- sol kadhi, in a steel tumbler -->
<ellipse class="stain" cx="470" cy="476" rx="26" ry="9" fill="#d98aa6" opacity="0"></ellipse>
<g class="glass"><g transform="translate(470 470)">
<path d="M-26 -96 H26 L22 0 H-22 Z" fill="url(#c2-steel)"></path><ellipse cx="0" cy="-96" rx="26" ry="7" fill="#e7a7bd"></ellipse><ellipse cx="0" cy="-96" rx="26" ry="7" fill="none" stroke="#9aa0a6" stroke-width="2"></ellipse></g></g>
<g class="steam" fill="none" stroke="#fff" stroke-width="3" stroke-linecap="round" opacity="0"><path d="M190 168 c-8 -12 8 -18 0 -32"></path><path d="M210 164 c-8 -12 8 -18 0 -32"></path><path d="M380 160 c-8 -12 8 -18 0 -32"></path></g>
<g transform="translate(70 486) rotate(-6)"><path d="M0 0 H196 V58 H0 Z" fill="#fbf6ec"></path><circle cx="14" cy="29" r="5" fill="none" stroke="#7a2a12" stroke-width="1.5"></circle><text x="104" y="26" font-family="'Young Serif', serif" font-size="18" fill="#7a2a12" text-anchor="middle">Chaitanya</text><text x="104" y="44" font-family="'Space Mono', monospace" font-size="9" fill="#4a3a30" text-anchor="middle">Malvani prawns thali, Dadar</text></g>'''
css = '''
.glass,.steam path{transform-box:fill-box}
.glass{transform-origin:50% 100%;transition:transform .4s}
.open .steam{opacity:1}
.open .steam path{animation:rise 2s ease-out infinite}.open .steam path:nth-child(2){animation-delay:.6s}.open .steam path:nth-child(3){animation-delay:1.1s}
.open .glass{animation:tilt 1.6s ease-in-out .3s both}
.open .stain{opacity:.8;transition:opacity .3s 1.2s}
@keyframes rise{0%{opacity:0;transform:translateY(8px)}40%{opacity:.8}100%{opacity:0;transform:translateY(-26px)}}
@keyframes tilt{0%,100%{transform:rotate(0)}40%,60%{transform:rotate(-22deg)}}
''' + RM.format(sel='.glass,.steam path,.stain')
write('Chaitanya.dc.html', title='Malvani seafood at Chaitanya keepsake', fonts='family=Young+Serif&amp;family=Space+Mono',
      css=css, svg=svg, focus='#f6e2b3', bg='radial-gradient(ellipse 55% 65% at 34% 48%, #c7663a 0%, #a94f2a 58%, #843b1f 100%)',
      text='#fbeee3', head='#fff8f0', ctabg='#f6e2b3', ctafg='#4b1c0b', ctahover='#ecd193',
      label='A banana-leaf thali with Malvani prawns, fried fish and a tumbler of sol kadhi, from Malvani seafood at Chaitanya.',
      h1='Malvani seafood at Chaitanya', p='The prawns thali, the way Dadar does it, on a leaf. The pink ring is where the sol kadhi stood.',
      opened='Serve the thali', closed='Clear the leaf')
