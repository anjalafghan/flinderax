"""Collect each keepsake's svg + css without writing boards, plus KOKO from its published file."""
import os, re, sys, importlib
HERE = os.path.dirname(__file__)
sys.path.insert(0, HERE)
import frame

CAPTURED = {}
_real = frame.write
def _cap(name, **kw):
    CAPTURED[name] = kw
frame.write = _cap

for m in ['b_cat', 'b_draw', 'b_tokai', 'b_noplan']:
    mod = importlib.import_module(m)
    mod.write = _cap  # modules did `from frame import write`
    importlib.reload(mod)

koko = open(os.path.join(HERE, 'KOKO.dc.html')).read()
svg = re.search(r'<svg viewBox="0 0 600 600"[^>]*>(.*?)</svg>\n</button>', koko, re.S).group(1)
style = re.search(r'<style>(.*?)</style>', koko, re.S).group(1)
css = '\n'.join(l for l in style.splitlines() if not l.startswith(('body{', '.scene', '.cta')))
CAPTURED['KOKO.dc.html'] = dict(svg=svg, css=css, fonts='family=Shippori+Mincho:wght@600;800',
                               label='KOKO chopstick sleeve from Dress up for KOKO, 17 November 2026, five stars.')


def split_blocks(css):
    """Top-level (prelude, body) pairs."""
    out, i, n = [], 0, len(css)
    while i < n:
        j = css.find('{', i)
        if j < 0: break
        pre = css[i:j].strip(); depth = 1; k = j + 1
        while depth:
            if css[k] == '{': depth += 1
            elif css[k] == '}': depth -= 1
            k += 1
        out.append((pre, css[j + 1:k - 1])); i = k
    return out


def scope(css, cls, sfx):
    names = re.findall(r'@keyframes\s+([\w-]+)', css)
    def rename(body):
        for nm in names:
            body = re.sub(rf'(?<![\w-]){nm}(?![\w-])', nm + sfx, body)
        return body
    def rules(css):
        res = []
        for pre, body in split_blocks(css):
            if pre.startswith('@keyframes'):
                res.append(f'{rename(pre)}{{{body}}}')
            elif pre.startswith('@media'):
                res.append(f'{pre}{{{rules(body)}}}')
            else:
                sels = []
                for s in pre.split(','):
                    s = s.strip()
                    sels.append(f'.{cls}.open' + s[5:] if s.startswith('.open') else f'.{cls} {s}')
                res.append(','.join(sels) + '{' + rename(body) + '}')
        return '\n'.join(res)
    return rules(css)


def scope_ids(svg, p):
    ids = re.findall(r'\bid="([^"]+)"', svg)
    for i in sorted(set(ids), key=len, reverse=True):
        svg = re.sub(rf'id="{re.escape(i)}"', f'id="{p}{i}"', svg)
        svg = re.sub(rf'#{re.escape(i)}(?=[")\s])', f'#{p}{i}', svg)
    return svg
