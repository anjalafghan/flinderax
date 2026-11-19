"""Batch 9: slips 299-333 (the restaurants, bars and cafés near the office and home)."""
from objects import *  # noqa: F401,F403

B = 'b9'

tag('patchofsky', 'Veg rooftop at Patch of Sky', batch=B, palette='sea', name='A poolside booth tag from Patch of Sky at the Aureole Hotel', head='Patch of Sky',
    rows=['Aureole Hotel, Andheri E', 'booth for two, poolside', 'all veg'], stamp_word='BOOTH 3', stamp_sub='rooftop', paper_='#e6f2f6', ink='#1d2b30', ink2='#2f6db5')
lanyard('gardenalfresco', 'Open-air dinner at Garden Alfresco', batch=B, palette='moss', name='A fairy-light lantern from the open-air dinner at Garden Alfresco', head='Garden Alfresco',
        sub='The Fern, among the green', big='outdoors', card='#2f6d57', ink='#ffe08a', art=[('lantern', 0, 270, 1.2)])
coin('trento', 'Garden dinner at Trento', batch=B, palette='moss', name='A guitar pick from the acoustic set at Trento', top='Trento, garden', big='acoustic',
     bottom='weekends, live', back_big='encore', back_small='one more song', face='#c0392b', rim='#7a1a16', ink='#fff6ea', art=[('guitar', 0, -78, .5)])
sticker('bombay63', 'Lively rooftop at Bombay63', batch=B, palette='ink', name='A neon sign sticker from the Bombay63 rooftop in Gokuldham', head='Bombay63', sub='Gokuldham, rooftop',
        shape='blob', col='#e0245e')
matchbook('bombar', "High-floor bar at Bomb'ar", batch=B, palette='slate', name="A matchbook from Bomb'ar on the 14th floor at 1st Avenue", head="Bomb'ar",
          sub='1st Avenue, 14th floor', cover='#c0392b', ink='#fff6ea', inside='the view, the drinks, us')
meal('asiakitchen', 'Dim sum and sushi at Asia Kitchen', batch=B, palette='red', name='A dim sum steamer from Asia Kitchen at Oberoi Mall', base='plate', base_col=('#c99c6c', '#a07a4a'),
     items=[('dumpling', 230, 300, 2), ('dumpling', 360, 300, 2), ('dumpling', 300, 380, 2), ('chili', 450, 250, .9)], tag_head='Asia Kitchen', tag_sub='dim sum, sushi, Oberoi Mall')
tag('copperchimney', 'No-stress dinner at Copper Chimney', batch=B, palette='brick', name='A copper handi tag from Copper Chimney at Oberoi Mall', head='Copper Chimney',
    rows=['Oberoi Mall', 'dal, kebabs, naan', 'no stress, as promised'], stamp_word='SERVED', stamp_sub='table 12', paper_='#e9b98c', ink='#3a1a0a', ink2='#7a1a16', art=[('handi', 236, 36, .5, '#b5683a', '#e9c46a')])
stub('pizzaexpress', 'Pizza and wine at PizzaExpress', batch=B, palette='brick', name='A ticket for pizza and a glass of wine at PizzaExpress', head='PizzaExpress',
     sub='OBEROI MALL, LOW EFFORT', rows=['one pizza, two halves', 'a glass of red each', 'cosy corner'], band='#2b2b30', paper_='#f6efe3', art=[('wine', 260, 150, .6, '#7a1a2e', '#2b2b30')], stub_text='TABLE 6')
label('farzi', 'Theatrical plates at Farzi Cafe', batch=B, palette='ink', name='A dry-ice card from the theatrical plates at Farzi Cafe', kind='box', head='Farzi Café',
      rows=['Oberoi Mall', 'the smoke, the show', 'the food was good too'], glass_='#2b2d33', paper_='#f6efe0', fx='steam')
menu('olivebistro', 'Bistro night at Olive Bistro & Bar', batch=B, palette='moss', name='An olive-branch bistro menu from Olive Bistro &amp; Bar', head='Olive Bistro',
     sub='Oberoi Mall, 3rd floor', panels=(['nachos', 'quesadilla', ''], ['risotto', 'grilled fish', ''], ['churros', 'a coffee', '']), ticks=[(0, 0), (1, 1), (2, 0)], accent='#5a6a3a')
meal('dimsumwu', 'Cozy dumplings at Dimsum Wū', batch=B, palette='kraft', name='Dumplings on a dark plate from Dimsum Wū', base='plate', base_col=('#2b2d33', '#4a4d55'),
     items=[('dumpling', 250, 310, 2.4), ('dumpling', 360, 320, 2.4), ('cup', 450, 240, 1.2, '#f6f3ec', '#6b3a1f')], tag_head='Dimsum Wū', tag_sub='IB Patel Road, near the office')
coin('canto', 'Late talk at Canto Cafe', batch=B, palette='night', name='An after-midnight clock token from late talk at Canto Cafe', top='Canto Café', big='12:47',
     bottom='still talking', back_big='1 am', back_small='they turned the lights down', face='#2b3a63', rim='#141c35', ink='#f6efe0', art=[('clock', 0, -78, .5)])
coin('saltinorbit', 'Grill night at Salt, Inorbit', batch=B, palette='slate', name='A salt-shaker token from grill night at Salt in Inorbit', top='Salt, Inorbit', big='grill',
     bottom='bar and grill', back_big='pinch', back_small='of salt, obviously', face='#f6f3ec', rim='#9aa0a6', ink='#2b2b30')
lanyard('bbqnation', 'Very hungry? Barbeque Nation, Inorbit', batch=B, palette='red', name='A skewer flag from unlimited grill at Barbeque Nation, Inorbit', head='Barbeque Nation',
        sub='Inorbit, unlimited', big='flag down', card='#c0392b', ink='#fff', art=[('kebab', 0, 270, 1.4)])
handstamp('goregaonsocial', 'Loud and fun at Goregaon Social', batch=B, palette='ink', name='A party stamp on a hand from Goregaon Social', stamp_text='SOCIAL', sub='loud, cheap, fun', ink='#f2c230')
coaster('fisherman', 'Seafood and live music at The Fisherman', batch=B, palette='sea', name="A fisherman's net-float coaster from The Fisherman Kitchen + Bar", head='The Fisherman',
        sub='New Link Road, live music', shape='round', board_='#2f6db5', ink='#fff', drink='glass', drink_col=('#f2b33d', '#ffffff'), art=[('fish', 0, 70, .7, '#f2c230')])
scorecard('sthamba', 'Brewery night at Sthamba', batch=B, palette='ochre', name='A brewery tasting-flight card from Sthamba', head='Sthamba flight',
          sub='Lokhandwala, four small glasses', rows=['the wheat one', 'the lager', 'the IPA (too bitter)', 'the stout'], cols=('me', 'you'),
          marks=[('8', '9'), ('7', '7'), ('4', '8'), ('9', '6')], circle=0, shape='card')
stub('foo', 'Trendy Asian at Foo, NESCO', batch=B, palette='red', name='A fortune slip from the bar at Foo in NESCO', head='Foo, gate 3',
     sub='YOUR FORTUNE', rows=['small plates are big', 'share the bao', 'you will order again'], band='#7a1a16', paper_='#fbf6ea', stub_text='LUCKY', shape='torn')
receipt('jayprakash', 'Dim and pocket-friendly at Jay Prakash', batch=B, palette='slate', name='A family bar bill on a spike from Hotel Jay Prakash', head='Jay Prakash',
        sub='family restaurant and bar', rows=[('chicken lollipop', '₹220'), ('dal fry', '₹140'), ('2 roti', '₹40'), ('a cold one each', '₹300')], total=('total', '₹700'), note='better for team outings')
cakebox('cococafe', 'Chocolate at Coco Café', batch=B, palette='kraft', name='A chocolate box from Coco Café by Cococart', head='Coco Café', sub='by Cococart, Oberoi Mall',
        box='#4a2414', ribbon='#c9a14a', ink='#f3e3c0', treat=('cake', '#7a3a1a', '#c9a14a'))
seedpacket('kitchengarden', 'Light bite at Kitchen Garden by Suzette', batch=B, palette='moss', name='A herb-garden seed tag from Kitchen Garden by Suzette', head='Kitchen Garden',
           rows=['by Suzette, Oberoi Mall', 'something green, light', 'closes 10:30'], packet='#e3ecd0', seed_col='#3a2a1a', art=(('plant', 0, -40, 1.6),))
cakebox('entisi', 'Dessert walk to Entisi', batch=B, palette='lilac', name='A gelato spoon card from the dessert walk to Entisi', head='Entisi',
        sub='chocolates and gelato', box='#f7eefb', ribbon='#6f5282', ink='#3a2a5a', treat=('scoop', '#c9a0dc', '#d9a85c'))
cuppa('bluetokaicalm', 'Calm coffee at Blue Tokai', batch=B, palette='sea', name='A Blue Tokai takeaway cup from a calm coffee near the office', head='Blue Tokai',
      sub='Goregaon East, calm', cup_col='#f6f3ec', band='#2f6db5', ink='#fff', marker='the calm one', drink='#5a3216')
cakebox('bostoncafe', 'Dessert at The Boston Cafe', batch=B, palette='paper', name='A patisserie box with a cake fork from The Boston Cafe', head='The Boston Café',
        sub='Lokhandwala, quiet, dessert', box='#fbf6ea', ribbon='#2f6db5', ink='#2f3a6a', treat=('cake', '#f2d3a0', '#7a3a1a'))
cuppa('lovelatte', 'Late latte at Love & Latte', batch=B, palette='brick', name='A latte with a heart in the foam from Love &amp; Latte', kind='teacup', head='Love &amp; Latte',
      sub='Lokhandwala, till midnight', cup_col='#f6f3ec', band='#c0392b', drink='#c99c6c', art=[('heart', 0, -40, .5, '#f6f3ec')])
tag('whitelama', 'Italian café at White Lama Villa', batch=B, palette='paper', name='A villa gate card from White Lama Villa in Lokhandwala', head='White Lama Villa',
    rows=['Lokhandwala', 'pasta, then dessert', 'no alcohol, no need'], stamp_word='BUONO', stamp_sub='Lokhandwala', paper_='#fbfaf6', ink2='#2f6d57')
specimen('silkroad', 'Garden coffee at Silk Road', batch=B, palette='moss', name='A caravan card with a coffee leaf from Silk Road Coffee Company in the garden', head='Silk Road Coffee',
         rows=['Aram Nagar, the garden', 'quiet, under the trees'], item=('leaf', '#3f7d2c'))
cuppa('thirdwave', 'Casual coffee at Third Wave', batch=B, palette='slate', name='A Third Wave cup with its sleeve, from Oberoi Mall', head='Third Wave', sub='Oberoi Mall',
      cup_col='#f6f3ec', band='#2b2b30', ink='#f6f3ec', marker='2 cold brews', drink='#3a2010')
meal('persiandarbar', 'Biryani at Persian Darbar', batch=B, palette='ochre', name='A biryani handi sealed with dough from Persian Darbar', base='plate', base_col=('#c9ced3', '#9aa0a6'),
     items=[('handi', 290, 300, 3, '#7a3a1a', '#e9c46a'), ('kebab', 300, 420, 1.6)], tag_head='Persian Darbar', tag_sub='biryani and kebabs, Lokhandwala')
coin('tewari', 'Pure veg at Tewari Brothers', batch=B, palette='ochre', name='A pure-veg thali token from Tewari Brothers', top='Tewari Brothers', big='veg',
     bottom='Goregaon East', back_big='thali', back_small='refills welcome', face='#3b9a5a', rim='#1f5a33', ink='#f6f3ec')
meal('jpslunch', "Malvani thali at JP's Lunch Home", batch=B, palette='brick', name="A steel Malvani thali with fish from JP's Lunch Home", base='thali',
     items=[('fish', 300, 300, 2.2, '#c98a3a'), ('plate', 220, 400, 1.2, '#f6f3ec', '#d8641e'), ('plate', 380, 400, 1.2, '#f6f3ec', '#b0461f'), ('cup', 420, 240, 1, '#e7a7bd', '#c2507a')],
     tag_head="JP's Lunch Home", tag_sub='Malvani thali, no frills')
cakebox('j3pizza', 'Pizza next door at J3', batch=B, palette='red', name='A pizza delivery box flap from J3 Pizza next door', head='J3 Pizza', sub='IB Patel Road, next door',
        box='#f6efe0', ribbon='#c0392b', ink='#7a1a16', treat=('pizza',))
receipt('maroosh', 'Quick Lebanese at Maroosh', batch=B, palette='sea', name='A quick Maroosh bill for a falafel wrap', head='Maroosh', sub='Oberoi Mall, quick bite',
        rows=[('falafel wrap', '×1'), ('shawarma', '×1'), ('garlic dip, extra', '×2')], total=('total', 'cheap'), note='eaten walking')
coin('ratna', 'South Indian at Ratna, till midnight', batch=B, palette='night', name='A midnight idli token from Ratna on SV Road', top='Ratna, SV Road', big='11:58',
     bottom='idli, just in time', back_big='idli', back_small='with extra chutney', face='#f6f3ec', rim='#9aa0a6', ink='#1f2b4d', art=[('moon', 0, -78, .5)])
