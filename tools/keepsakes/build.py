"""One command for keepsakes: boards for the Design canvas, the checks, the contact sheets and the app code.

    python3 tools/keepsakes/build.py            # everything
    python3 tools/keepsakes/build.py b3         # one batch: its boards and contact sheets (app code is always all)

Steps: import every board module (each calls frame.write; we collect instead of writing) → write thin
canvas boards to out/boards → write out/sheet.html → run check.ts headless (text inside its object,
nothing covering text, closed state shows the object, no plain rectangle as the object, the moment
changes something; crops each drawing) → write web/src/joy/keepsakes/scenes.gen.ts and
web/src/joy/styles/scenes.css from the crops → out/publish.json, the files map for the canvas publish.

Adding slips: write a module (see kit.py), add it to MODULES and its boards to REGISTRY.
"""
import importlib, json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
OUT = os.path.join(HERE, 'out')
WEB = os.path.join(ROOT, 'web', 'src', 'joy')
sys.path.insert(0, HERE)
import frame  # noqa: E402

# board file -> (scene id, slip key = the slip's title lower-cased, visit, batch, what it is for a screen reader)
REGISTRY = {
    'NoPlan.dc.html': ('noplan', 'no-plan date', 1, 'b1', 'A torn street map and the bronze coin that chose every turn'),
    'CatCafe.dc.html': ('catcafe', 'cat café', 1, 'b1', 'A paw-shaped card from Cat Café Studio, with a ginger cat asleep on it'),
    'BlueTokaiFirst.dc.html': ('tokai1', 'blue tokai', 1, 'b1', 'A Blue Tokai takeaway cup marked Diya + 1'),
    'BlueTokaiRegulars.dc.html': ('tokai2', 'blue tokai', 2, 'b1', 'A coffee-bean punch card from Blue Tokai and half a sandwich tied in red and green'),
    'DrawingGame.dc.html': ('drawing', 'drawing guessing game', 1, 'b1', 'A yellow sticky note with a terrible drawing of a horse'),
    'KOKO.dc.html': ('koko', 'dress up for koko', 1, 'b1', 'The KOKO chopstick sleeve and a whisky with one big cube'),
    'Karaoke.dc.html': ('karaoke', 'karaoke night at road house bluez', 1, 'b2', 'A pink song-request slip from Road House Bluez and a microphone'),
    'Saltt.dc.html': ('saltt', 'dinner at saltt', 1, 'b2', 'A receipt from Saltt in Karjat with the ghat road pencilled beside it'),
    'LunaEtSol.dc.html': ('lunaetsol', 'dinner at luna et sol', 1, 'b2', 'A fondue pot on a red check cloth with a fork tagged Luna Et Sol, fork no. 2'),
    'BombayCanteen.dc.html': ('bombaycanteen', 'dinner at the bombay canteen', 1, 'b2', 'A torn teal menu strip from The Bombay Canteen and a toy kaali-peeli taxi'),
    'Chaitanya.dc.html': ('chaitanya', 'malvani seafood at chaitanya', 1, 'b2', 'A banana-leaf Malvani thali with a tumbler of sol kadhi'),
    'PCO.dc.html': ('pco', 'speakeasy cocktails at pco', 1, 'b2', 'An arched black PCO card with an old STD ISD PCO booth and a scratch panel'),
    'ComedyNight.dc.html': ('comedy', 'stand-up comedy night', 1, 'b2', 'A looped neon wristband from Canvas Laugh Club and a mic on a stand'),
    'CarterRoad.dc.html': ('carterroad', 'food critique date — carter road hop', 1, 'b2', 'A clipboard scorecard for the Carter Road stalls, scored in two inks'),
    'Perfume.dc.html': ('perfume', 'perfume making workshop', 1, 'b2', 'A vial labelled Our blend no. 1 and a blotter strip'),
    'Pottery.dc.html': ('pottery', 'pottery class together', 1, 'b2', 'A terracotta tag with two thumbprints, and a bowl on the wheel'),
    'GlassPainting.dc.html': ('glass', 'glass painting workshop', 1, 'b2', 'A leaded-glass suncatcher with our initials in the middle'),
    'BookstoreRoadies.dc.html': ('bookstore', 'bookstore roadies task', 1, 'b2', 'A red book bought for ₹299 with a card of bookstore dares in its pocket'),
    'Arcade.dc.html': ('arcade', 'arcade games', 1, 'b2', 'A holographic Timezone game card with zero credits left'),
    'GamingCafe.dc.html': ('gamingcafe', 'gaming café', 1, 'b2', 'A session token for PC 07 at Global Gaming Cafe and a controller'),
    'VirtualAdda.dc.html': ('vr', 'laser tag & vr gaming', 1, 'b2', 'A headset-shaped pass for sim pod B at Virtual Adda'),
    'GoKarting.dc.html': ('karting', 'go-karting', 1, 'b2', 'A lap-time printout from Republic of Karting with a chequered flag'),
    'HorseRiding.dc.html': ('horse', 'horse riding', 1, 'b2', 'A stitched leather horseshoe tag from a trial ride at D Sports Star'),
    'ShootingRange.dc.html': ('shooting', 'shooting range', 1, 'b2', 'A paper target from Champions Shooting Range, holes near the bull'),
    'RageRoom.dc.html': ('rage', 'rage room', 1, 'b2', 'A hazard-striped Mad Rage Room waiver and a plate'),
    'SnowWorld.dc.html': ('snow', 'snow world', 1, 'b2', 'A frosty Snow World token and one red mitten'),
}
# warnings looked at and accepted: a chopstick sleeve and a book really are rectangles; the HA's float in the spotlight; the dare card is pulled out over the book; the kart printout starts inside the printer
ACCEPTED = {'koko': ['plain rectangle'], 'comedy': ['“ha”', '“HA!”'], 'carterroad': ['“7”'], 'bookstore': ['plain rectangle', 'is covered (open)'], 'karting': ['runs off its object (closed)']}

MODULES = ['b_noplan', 'b_cat', 'b_tokai', 'b_draw', 'b2a', 'b2b', 'b2c', 'b2d']

FONTS = [
    ('Caveat', 400, 'normal', 'caveat-latin-400-normal'), ('Caveat', 600, 'normal', 'caveat-latin-600-normal'), ('Caveat', 700, 'normal', 'caveat-latin-700-normal'),
    ('Kalam', 400, 'normal', 'kalam-latin-400-normal'), ('Kalam', 700, 'normal', 'kalam-latin-700-normal'),
    ('Gochi Hand', 400, 'normal', 'gochi-hand-latin-400-normal'), ('Permanent Marker', 400, 'normal', 'permanent-marker-latin-400-normal'),
    ('Young Serif', 400, 'normal', 'young-serif-latin-400-normal'),
    ('Shippori Mincho', 600, 'normal', 'shippori-mincho-latin-600-normal'), ('Shippori Mincho', 800, 'normal', 'shippori-mincho-latin-800-normal'),
    ('Space Mono', 400, 'normal', 'space-mono-latin-400-normal'), ('Space Mono', 700, 'normal', 'space-mono-latin-700-normal'),
    ('Fraunces', '100 900', 'normal', 'fraunces-latin-full-normal'), ('Fraunces', '100 900', 'italic', 'fraunces-latin-full-italic'),
]


def collect():
    """Every board's write() arguments, by file name."""
    got = {}
    frame.write = lambda name, **kw: got.__setitem__(name, kw)
    for m in MODULES:
        mod = importlib.import_module(m)
        if getattr(mod, 'write', None) is not frame.write:  # imported before the patch: run it again
            mod.write = frame.write
            importlib.reload(mod)
    # kit-based slip modules (s_<batch>_*.py) register themselves
    import kit
    for f in sorted(os.listdir(HERE)):
        if f.startswith('s_') and f.endswith('.py'):
            importlib.import_module(f[:-3])
    REGISTRY.update(kit.REGISTRY)
    koko =open(os.path.join(HERE, 'KOKO.dc.html')).read()
    got['KOKO.dc.html'] = dict(
        svg=re.search(r'<svg viewBox="0 0 600 600"[^>]*>(.*?)</svg>\n</button>', koko, re.S).group(1),
        css='\n'.join(l for l in re.search(r'<style>(.*?)</style>', koko, re.S).group(1).splitlines() if not l.startswith(('body{', '.scene', '.cta'))),
        fonts='family=Shippori+Mincho:wght@600;800', original=koko, opened='Open the sleeve', closed='Fold it back')
    missing = set(REGISTRY) - set(got)
    if missing:
        sys.exit(f'registered but not built: {sorted(missing)}')
    return got


def blocks(css):
    out, i = [], 0
    while (j := css.find('{', i)) >= 0:
        depth, k = 1, j + 1
        while depth:
            depth += {'{': 1, '}': -1}.get(css[k], 0)
            k += 1
        out.append((css[i:j].strip(), css[j + 1:k - 1]))
        i = k
    return out


def scope(css, cls, sfx, root='.joy '):
    """Every rule under `.joy .<cls>` (`.open` on the scene itself); keyframes renamed so scenes never clash."""
    names = re.findall(r'@keyframes\s+([\w-]+)', css)
    ren = lambda b: re.sub(r'(?<![\w-])(' + '|'.join(map(re.escape, names)) + r')(?![\w-])', r'\1' + sfx, b) if names else b
    def rules(css):
        res = []
        for pre, body in blocks(css):
            if pre.startswith('@keyframes'):
                res.append(f'{ren(pre)}{{{body}}}')
            elif pre.startswith('@media'):
                res.append(f'{pre}{{{rules(body)}}}')
            else:
                sel = ','.join(f'{root}.{cls}.open{s.strip()[5:]}' if s.strip().startswith('.open') else f'{root}.{cls} {s.strip()}' for s in pre.split(','))
                res.append(sel + '{' + ren(body) + '}')
        return '\n'.join(res)
    return rules(css)


def scope_ids(svg, p):
    for i in sorted(set(re.findall(r'\bid="([^"]+)"', svg)), key=len, reverse=True):
        svg = svg.replace(f'id="{i}"', f'id="{p}{i}"')
        svg = re.sub(rf'#{re.escape(i)}(?=[")\s])', f'#{p}{i}', svg)
    return re.sub(r'\n\s*\n', '\n', re.sub(r'<!--.*?-->', '', svg, flags=re.S)).strip()


def font_faces(base):
    return '\n'.join(f'@font-face {{ font-family: "{f}"; font-style: {st}; font-weight: {w}; font-display: swap; src: url("{base}/{n}.woff2") format("woff2"); }}' for f, w, st, n in FONTS)


def main():
    batch = sys.argv[1] if len(sys.argv) > 1 else None
    got = collect()
    os.makedirs(os.path.join(OUT, 'boards'), exist_ok=True)
    picked = [b for b in REGISTRY if batch is None or REGISTRY[b][3] == batch]

    # 1. canvas boards: thin (object, title, one button); KOKO stays exactly as approved
    for b in picked:
        kw = got[b]
        if 'original' in kw:
            html = kw['original']
        else:
            html = frame.FRAME.substitute(**{**kw, 'fig': frame.FIG, 'shadow': kw.get('shadow', 'drop-shadow(0 2px 2px rgba(20,14,8,.3)) drop-shadow(0 16px 22px rgba(20,14,8,.26))'), 'p': ''})
            html = re.sub(r'\n<p style="margin: 0; font: italic[^\n]*</p>', '', html)
        open(os.path.join(OUT, 'boards', b), 'w').write(html)

    # 2. the contact sheet: every picked scene, scoped exactly as the app will scope it
    fonts_dir = os.path.relpath(os.path.join(WEB, 'fonts'), OUT)
    tiles, css = [], [font_faces(fonts_dir)]
    for b in picked:
        sid = REGISTRY[b][0]
        css.append(scope(got[b]['css'], f'ksc-{sid}', '_' + sid, root=''))
        tiles.append(f'<figure class="tile" data-id="{sid}" data-board="{b}"><div class="ksc ksc-{sid}"><svg viewBox="0 0 600 600">{scope_ids(got[b]["svg"], sid + "-")}</svg></div><figcaption>{sid}</figcaption></figure>')
    sheet = f'''<!doctype html><meta charset="utf-8"><title>keepsakes {batch or "all"}</title>
<style>body{{margin:0;padding:16px;background:#8f8778;font:13px system-ui;display:grid;grid-template-columns:repeat(5,300px);gap:14px}}
.tile{{margin:0;background:#a39b8b;border-radius:8px;padding:4px}}.ksc svg{{display:block;width:292px;height:292px;overflow:visible}}
figcaption{{padding:4px 6px;color:#fff;font-weight:600}}.tile.bad{{outline:4px solid #e0245e}}.tile.bad figcaption{{background:#e0245e}}
{chr(10).join(css)}</style>
{''.join(tiles)}'''
    open(os.path.join(OUT, 'sheet.html'), 'w').write(sheet)

    # 3. headless checks: crops, problems, the two contact-sheet screenshots
    r = subprocess.run(['bun', os.path.join(HERE, 'check.ts'), os.path.join(OUT, 'sheet.html')], cwd=ROOT)
    if r.returncode != 0:
        sys.exit('check.ts failed to run')
    report = json.load(open(os.path.join(OUT, 'report.json')))
    crops_path = os.path.join(HERE, 'crops.json')
    crops = json.load(open(crops_path)) if os.path.exists(crops_path) else {}
    for sid, rep in report.items():
        crops[sid] = rep['crop']
    json.dump(crops, open(crops_path, 'w'), indent=1, sort_keys=True)

    # 4. app code, from every registered scene
    ts = ['// Generated by tools/keepsakes/build.py from the keepsake boards: do not edit by hand.',
          '// Each scene is one kept date\'s own illustrated object; `svg` is static markup we drew, never user text.',
          'import type { SceneDef } from "./scene-types";', '', 'export const SCENES: SceneDef[] = [']
    app_css = ['/* Generated by tools/keepsakes/build.py: do not edit by hand. Each scene\'s motion is scoped to',
               '   .ksc-<id>; .open plays it. Fonts are self-hosted (OFL, from @fontsource). */', font_faces('../fonts')]
    for b, (sid, key, visit, _batch, name) in REGISTRY.items():
        kw = got[b]
        vb = crops.get(sid, [0, 0, 600, 600])
        app_css.append(f'/* {sid} */\n' + scope(kw['css'], f'ksc-{sid}', '_' + sid))
        ts.append(f'  {{ id: {json.dumps(sid)}, key: {json.dumps(key, ensure_ascii=False)}, visit: {visit}, name: {json.dumps(name, ensure_ascii=False)},')
        ts.append(f'    vb: [{", ".join(str(v) for v in vb)}], open: {json.dumps(kw.get("opened", "Open it"))}, close: {json.dumps(kw.get("closed", "Put it back"))},')
        ts.append(f'    svg: {json.dumps(scope_ids(kw["svg"], sid + "-"), ensure_ascii=False)} }},')
    ts.append('];')
    open(os.path.join(WEB, 'keepsakes', 'scenes.gen.ts'), 'w').write('\n'.join(ts) + '\n')
    open(os.path.join(WEB, 'styles', 'scenes.css'), 'w').write('\n'.join(app_css) + '\n')

    # 5. what to send to the canvas
    json.dump({f'project/{b}': os.path.join(OUT, 'boards', b) for b in picked}, open(os.path.join(OUT, 'publish.json'), 'w'), indent=1)

    for sid, rep in report.items():
        rep['issues'] = [i for i in rep['issues'] if not any(a in i for a in ACCEPTED.get(sid, []))]
    bad = {sid: rep['issues'] for sid, rep in report.items() if rep['issues']}
    print(f'{len(picked)} boards, {len(REGISTRY)} scenes in the app. Sheets: out/sheet-closed.png, out/sheet-open.png')
    for sid, issues in bad.items():
        print(f'  {sid}: ' + '; '.join(issues))
    if not bad:
        print('  no problems found')


if __name__ == '__main__':
    main()
