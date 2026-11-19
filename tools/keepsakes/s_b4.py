"""Batch 4: slips 73-122 (Bandra cafés, heritage walks, the days apart)."""
from objects import *  # noqa: F401,F403

B = 'b4'

scorecard('salsa', 'Learn a salsa step from a video', batch=B, palette='brick', name='A salsa count sheet with footprints, the turn finally ticked', head='Salsa, one turn',
          sub='sofa pushed back, beginner video', rows=['1, 2, 3 (pause)', '5, 6, 7 (pause)', 'the basic, together', 'one turn, no laughing'], cols=('me', 'you'),
          marks=[('✓', '✓'), ('✓', '✓'), ('✓', '✓'), ('', '✓')], circle=3, shape='card')
strip('scrapbook', 'Make a scrapbook of us', batch=B, palette='kraft', name='A strip of printed photos from our scrapbook with washi tape', film=True,
      frames=[('#f6c27a', '#3f5d7d', [('sun', 75, 50, .7)]), ('#9fc7e0', '#5d7d4f', [('heart', 75, 55, .6)]), ('#5d3564', '#2a1a2a', [('stars', 75, 50, 1)])])
coaster('bhaudaji', 'Dr. Bhau Daji Lad Museum', batch=B, palette='ochre', name='A tile coaster patterned like the gold-and-tile ceilings at Bhau Daji Lad Museum',
        head='Bhau Daji Lad', sub="Mumbai's oldest museum", shape='square', board_='#1e6a66', ink='#f2d38c', drink='cup', drink_col=('#f6f3ec', '#6b3a1f'),
        art=[('flower', -80, 70, .5, '#f2d38c', '#c8312b'), ('flower', 80, 70, .5, '#f2d38c', '#c8312b'), ('dome', 0, 80, .6, '#f2d38c', '#c8312b')])
postcard('hajiali', 'Walk out to Haji Ali at sunset', batch=B, palette='sea', name='A postcard of the causeway to Haji Ali at low tide, with a tide card on the back',
         scene=[('dome', 260, 130, 1.2, '#f6f1e4', '#f1e7d0'), ('sun', 90, 90, .8, '#f6a04d'), ('waves', 120, 180, 1.2, '#ffffff')],
         sky=('#f7b267', '#f4845f'), ground='#4a7a90', caption='Haji Ali, low tide', message=['Checked the tide', 'first. Walked the', 'causeway over the', 'sea at sunset.'], to='to us')
mapcard('powailake', 'Evening by Powai Lake', batch=B, palette='night', name='A map of the Powai Lake promenade with our walk and the dinner after',
        head='Powai Lake, the promenade', water='M0 140 C80 120 200 130 280 170 C340 200 420 190 430 200 V380 H0 Z',
        route='M40 120 C120 100 220 110 300 150 S400 170 410 120', pins=[(300, 150, 'lights on'), (410, 120, 'dinner')], labels=[(200, 300, 'Powai Lake', 0)], art=[('moon', 380, 60, .8)])
postcard('bandrafort', 'Sunset at Bandra Fort', batch=B, palette='sea', name='A postcard from the Bandra Fort ramparts with the Sea Link at sunset',
         scene=[('fort', 80, 150, 1.6, '#a08060', '#7a5c40'), ('sun', 230, 110, .9, '#f6a04d'), ('lights', 280, 150, 1.2, '#ffe08a', '#2b3a63')],
         sky=('#f9c784', '#e7766b'), ground='#36567a', caption='Castella de Aguada', message=['On the old ramparts,', 'the Sea Link lighting', 'up one cable at', 'a time.'], to='to us, Bandra')
receipt('britannia', 'Berry pulao at Britannia & Co.', batch=B, palette='red', name='A Britannia & Co. bill for berry pulao and caramel custard', head='Britannia &amp; Co.',
        sub='Ballard Estate, lunch only', rows=[('berry pulao, chicken', '×1'), ('salli boti', '×1'), ('caramel custard', '×2'), ('raspberry soda', '×2')], total=('paid', 'cash'),
        note='closed Sundays!', art=[('handi', 470, 430, 1.2, '#c0392b', '#e9c46a'), ('steam', 470, 380, .7)])
coin('madhferry', 'Ferry from Versova to Madh Island', batch=B, palette='sea', name='A ferry token for the five-minute boat from Versova to Madh', top='Versova to Madh', big='₹5',
     bottom='across the creek', back_big='Madh', back_small='the quiet side', face='#c9ced3', rim='#6b7280', ink='#1d2b30', art=[('boat', 0, -78, .5)])
mapcard('fortwalk', 'Fort heritage walk at night', batch=B, palette='night', name='A heritage map of Fort at night: Flora Fountain, the High Court and the Asiatic steps',
        head='Fort, after dark', route='M50 320 C90 260 120 220 180 210 S260 150 300 120 S360 80 380 60', pins=[(180, 210, 'Flora Fountain'), (300, 120, 'High Court'), (380, 60, 'Asiatic steps')],
        land='#e9e1cc', art=[('dome', 100, 120, .8, '#f1e7d0', '#7a5c40'), ('lantern', 60, 230, .6)])
receipt('chorbazaar', 'Chor Bazaar treasure hunt', batch=B, palette='clay', name='A haggled Chor Bazaar receipt for two strange treasures', head='Chor Bazaar',
        sub='Mutton Street, ₹500 each', rows=[('one brass key (why)', '₹180'), ('a 1970s film poster', '₹300'), ('a doorknob', '₹20'), ('haggled down from', '₹900')], total=('total', '₹500'),
        note='strangest thing wins', art=[('anchor', 470, 420, .9, '#7a5a30')])
stub('mysterygig', 'Go to a gig neither of you knows', batch=B, palette='ink', name='A stub from a gig by a band neither of us had heard of', head='?? live',
     sub='A BAND WE HAD NEVER HEARD OF', rows=['picked from a listings app', 'small room, loud', 'we know one song now'], band='#2a2238', paper_='#f2c230', ink='#16161a',
     art=[('note', 250, 150, .9, '#16161a')], stub_text='MYSTERY', shape='scallop')
scorecard('newcuisine', 'Eat a cuisine neither of you has tried', batch=B, palette='brick', name='A passport page stamped for a cuisine new to us both', head='New cuisine, stamped',
          sub='ordered what we could not pronounce', rows=['the dish we pointed at', 'the sauce', 'the one we will order again'], cols=('me', 'you'),
          marks=[('✓', '✓'), ('✓', ''), ('✓', '✓')], circle=2, shape='card', art=[('globe', 280, 40, .6)])
postcard('vipassana', 'Global Vipassana Pagoda and Gorai beach', batch=B, palette='ochre', name='A postcard of the golden Global Vipassana Pagoda, then Gorai beach',
         scene=[('pagoda', 180, 140, 2.1, '#e8b923', '#b88a1a'), ('waves', 70, 190, .8, '#ffffff'), ('bird', 290, 60, .7)],
         sky=('#fce5b0', '#f2c27a'), ground='#4a7a90', caption='sat inside the dome', message=['Ten minutes of', 'silence inside the', 'golden dome. Then', 'the beach at Gorai.'], to='to us')
postcard('kondana', 'Kondana Caves trek', batch=B, palette='ghat', name='A monsoon card of the Kondana caves with the waterfall in front',
         scene=[('cave', 180, 130, 2.4, '#6a6a58', '#2a221a'), ('waterfall', 290, 140, 1.4, '#bfe3f2', '#4f7a45'), ('mist', 80, 80, 1)],
         sky=('#b8c9b4', '#dfe6d4'), ground='#3f553b', caption='Kondana, monsoon', message=['Old Buddhist caves', 'and a waterfall in', 'front of them.', 'Soaked, happy.'], to='to us, Karjat')
label('sula', 'Vineyard weekend in Nashik', batch=B, palette='plum', name='A wine label from the vineyards in Nashik', kind='bottle', head='Nashik, our vintage',
      rows=['tasting, then lunch', 'sunset over the vines', 'one bottle home'], glass_='#3a1a2e', paper_='#f6efe0', fx='sparkle')
label('fireflies', 'Fireflies at Bhandardara', batch=B, palette='navy', name='A jar of fireflies from the trees around Arthur Lake', kind='jar', head='Bhandardara',
      rows=['Arthur Lake, late May', 'the trees lit up', 'lights off, no torches'], glass_='#1f2b4d', paper_='#f6efe0', fx='sparkle', art=[('firefly', 0, -110, 1.6)])
cuppa('mokai', 'Matcha at Mokai', batch=B, palette='moss', name='A glass of matcha from the floating matcha bar at Mokai', kind='teacup', head='Mokai', sub='Pali Hill, upstairs',
      cup_col='#f6f3ec', band='#7aa35a', drink='#8fbf5a', verbs=('Whisk it', 'Set it down'))
cakebox('houseofcroissant', 'Croissants at House of Croissant', batch=B, palette='kraft', name='A pastry bag from House of Croissant with a croissant peeking out',
        head='House of Croissant', sub='near Mount Mary, from 8:30', box='#efe1c0', ribbon='#7a4a2a', ink='#3a2414', treat=('croissant',))
cuppa('subko', 'Coffee and bakes at Subko', batch=B, palette='clay', name='A Subko takeaway cup from the Ranwar cottage brew bar', head='Subko', sub='Ranwar, Bandra',
      cup_col='#f6f3ec', band='#2b2b30', ink='#f6f3ec', marker='the new one', drink='#5a3216')
tag('bagelshop', 'Brunch at The Bagel Shop', batch=B, palette='moss', name='A greenhouse tag from The Bagel Shop on Pali Hill', head='The Bagel Shop',
    rows=['Pali Hill, the greenhouse', 'bagel, smoothie, a dog', 'near Carter Road'], stamp_word='BRUNCH', stamp_sub='Pali Hill', paper_='#e3ecd0', ink='#24401f', ink2='#2f6d57', art=[('plant', 236, 40, .5)])
cakebox('pardonourfrench', 'Pardon Our French', batch=B, palette='lilac', name='A Parisian cake box from Pardon Our French, brown butter cake inside', head='Pardon Our French',
        sub='Ballard Estate, 22 seats', box='#f3e6f0', ribbon='#3a2a5a', ink='#3a2a5a', treat=('cake', '#e9c9a0', '#7a4a2a'))
coaster('kgcafe', 'Kala Ghoda Cafe', batch=B, palette='paper', name='A coaster from Kala Ghoda Cafe under its skylight', head='Kala Ghoda Café', sub='Ropewalk Lane, skylit',
        shape='round', board_='#f6efe0', ink='#3a2a20', drink='cup', drink_col=('#f6f3ec', '#6b3a1f'), art=[('sun', 0, 70, .6, '#e8b923')])
menu('mondegar', 'Cafe Mondegar and the Mario Miranda walls', batch=B, palette='brick', name='A folded Cafe Mondegar menu with Mario Miranda-style doodles',
     head='Mondegar', sub='Colaba, the cartoon walls', panels=(['jukebox', 'chilled beer', 'chicken burger'], ['find yourself', 'on the walls', '(we did)'], ['since 1932', 'Colaba Causeway', 'open 8 to 11:30']),
     ticks=[(0, 1), (0, 2), (1, 0)], accent='#c8312b', art=[('note', 225, 210, .8, '#fff')])
cakebox('concu', 'Conçu in Colaba', batch=B, palette='paper', name='A patisserie box from Conçu tied with a ribbon tag', head='Conçu', sub='Pasta Lane, Colaba',
        box='#fbf6ea', ribbon='#2f4a33', ink='#2f4a33', treat=('cake', '#f2d3a0', '#c0392b'))
cuppa('prithvicafe', 'Irish coffee at Prithvi Cafe', batch=B, palette='kraft', name='An Irish coffee in a glass at Prithvi Cafe under the trees', kind='cutting',
      head='Prithvi Café', sub='Irish coffee, bun maska', drink='#4a2a14', verbs=('Stir it', 'Set it down'))
menu('silverbeach', 'Breakfast at Silver Beach Cafe', batch=B, palette='sea', name='A Silver Beach Cafe breakfast menu, our order ticked', head='Silver Beach',
     sub='Juhu, by the sand', panels=(['eggs, any style', 'pancakes', 'avocado toast'], ['cold coffee', 'fresh juice', 'chai'], ['then a walk', 'on the beach', '']), ticks=[(0, 0), (0, 1), (1, 0), (2, 0)],
     accent='#2f6db5', art=[('sun', 225, 220, .6, '#f2c230')])
strip('leapingwindows', 'Comics and coffee at Leaping Windows', batch=B, palette='ochre', name='Comic panels we picked for each other at Leaping Windows', film=False,
      frames=[('#f2c230', '#f2c230', [('cup', 75, 60, .9)]), ('#9fc7e0', '#9fc7e0', [('book', 75, 60, .9)]), ('#f48aa6', '#f48aa6', [('heart', 75, 60, .7)])], caption='pow!')
cuppa('elnpink', 'Pink tea at EL&N', batch=B, palette='lilac', name='A pink teacup from EL&amp;N under the flower wall', kind='teacup', head='EL&amp;N', sub='Jio World Plaza, BKC',
      cup_col='#f7c9d4', band='#c2185b', drink='#f29ab8', art=[('flower', -60, 0, .5, '#f29ab8', '#c2185b'), ('flower', 60, 10, .4, '#f29ab8', '#c2185b')])
meal('benne', 'Butter dosas at Benne', batch=B, palette='ochre', name='A benne dosa on butter paper with filter coffee, from Benne', base='paper', base_col=('#fbf3dc', '#d9c49a'),
     items=[('dosa', 280, 300, 3.2), ('tumbler', 450, 330, 1.8)], tag_head='Benne, 16th Road', tag_sub='Bangalore-style, lots of butter')
meal('doughjoe', 'Pizza and coffee at Dough & Joe', batch=B, palette='brick', name='A pizza on a peel and two coffees from Dough &amp; Joe', base='tray', base_col=('#c99c6c', '#a07a4a'),
     items=[('pizza', 270, 320, 3.2), ('cup', 440, 260, 1.6), ('cup', 440, 380, 1.6)], tag_head='Dough &amp; Joe', tag_sub='Hill Road, slow-fermented')
coin('boojee', 'Brunch at Boojee Cafe', batch=B, palette='kraft', name='A queue number token from Boojee Cafe', top='Boojee Café, Bandra', big='No. 23', bottom='worth the queue',
     back_big='swap!', back_small='halfway through', face='#f2c230', rim='#9c7a2a', ink='#3a2408')
cuppa('bokka', 'Coffee on tap at Bokka', batch=B, palette='slate', name='A glass of coffee on tap from Bokka, 16th Road', kind='cutting', head='Bokka', sub='coffee on tap, 16th Road', drink='#2a160a')
specimen('greysoul', 'Grey Soul Coffee in Kala Ghoda', batch=B, palette='paper', name='A light-roast tasting card from Grey Soul Coffee with a coffee bean taped on', head='Grey Soul, light roast',
         rows=['the one neither of us would pick', 'notes: citrus? we argued'], item=('coffeebean',), card='#f2ece0')
cakebox('notfar', 'Cookies and cupcakes at NotFar', batch=B, palette='brick', name='A cookie box from NotFar to share', head='NotFar', sub='Bandra, a box to share',
        box='#f2e3b3', ribbon='#c0392b', ink='#5a1d0c', treat=('cake', '#f7c9d4', '#7a3a1a'))
cuppa('magari', 'Hot chocolate at Magari', batch=B, palette='kraft', name='A mug of thick hot chocolate at Magari among the books', kind='mug', head='Magari', sub='Four Bungalows',
      cup_col='#2f4a33', ink='#f3ead8', drink='#4a2414')
scorecard('croffle', 'Croffles at The Croffle Guys', batch=B, palette='ochre', name='A waffle-grid card scoring a sweet croffle against a savoury one', head='Croffle-off',
          sub='Lokhandwala, split both', rows=['sweet croffle', 'savoury croffle', 'which one again'], cols=('me', 'you'), marks=[('9', '8'), ('7', '9'), ('sweet', 'savoury')], shape='clip')
clapper('pressplay', 'Press play together on the same movie', batch=B, palette='ink', name='A clapperboard for pressing play at the same time in two cities',
        head='3, 2, 1, play', rows=['two cities, one film', 'pause rule: either of us', 'scene: the good part'])
menu('samerecipe', 'Cook the same recipe on a video call', batch=B, palette='ochre', name='A split recipe card for cooking the same dish on a video call', head='Same recipe',
     sub='two kitchens, one call', panels=(['your kitchen', 'same pan', 'same mess'], ['the recipe', 'onions first', 'then everything'], ['my kitchen', 'phone propped', 'on the jar']), ticks=[(1, 1), (1, 2)], accent='#7a4a2a')
receipt('ordereachother', 'Order each other dinner', batch=B, palette='red', name="A delivery slip for the surprise dinner we each ordered to the other's door",
        head='Delivery for you', sub='ordered by me, from far away', rows=[('your favourite thing', '×1'), ('something new', '×1'), ('a dessert you will deny', '×1')],
        total=('unbox on call', '✓'), note='eat together, apart')
tag('museumonline', 'Wander a museum online', batch=B, palette='paper', name='An audio-guide tag for a museum we walked together online', head='Gallery, online',
    rows=['shared screen, one museum', 'we each chose one piece', 'and argued about it'], stamp_word='VIEWED', stamp_sub='two cities', paper_='#fbfaf6', ink2='#3a2a5a', art=[('screens', 236, 40, .5)])
postcard('stargazecall', 'Stargaze on a call', batch=B, palette='navy', name='A postcard of the same moon from two windows', scene=[('moon', 120, 70, 1.2), ('stars', 250, 60, 1.6), ('skyline', 90, 150, 1.3, '#141c35'), ('skyline', 270, 152, 1.2, '#1f2b4d')],
         sky=('#141c35', '#2b3a63'), ground='#0f1428', caption='same moon', message=['Lights off, the app', 'open, found the same', 'star from both', 'windows.'], to='to you, far away')
media('coopgame', 'Co-op game night', batch=B, palette='pine', name='A co-op save file card, two players, one farm', kind='cartridge', head='CO-OP SAVE',
      rows=['player 1: you', 'player 2: me', 'day 14, spring', 'one friend pass', 'do not overwrite'], shell='#2f6d57')
coin('chessstakes', 'Online chess with stakes', batch=B, palette='slate', name='A chess token with the forfeit poem on its back', top='best of three', big='mate',
     bottom='online, two cities', back_big='a poem', back_small='four lines, read aloud', face='#f6f3ec', rim='#2b2b30', ink='#2b2b30')
bookmark('readaloud', 'Read a book aloud to each other', batch=B, palette='lilac', name='A ribbon bookmark in the book we read aloud, a chapter a night', head='a chapter a night',
         rows=['taking turns', 'something gentle'], strip_col='#6f5282', book_col=('#3a2a5a', '#f6efe0'))
envelope_('bedtimestory', 'Make up a bedtime story together', batch=B, palette='navy', name='A scroll of the bedtime story we made up one sentence at a time', front='once upon a time, sort of',
          letter=['Once there was a cat', 'who could not sleep, so', 'she called the moon,', 'who was also awake,', 'and then you fell asleep.'], seal='#1f2b4d', art=[('moon', 180, 136, .4)])
coin('fallasleep', 'Fall asleep on a call', batch=B, palette='navy', name='A token stamped with the call timer at 3:14 am', top='call time', big='3:14',
     bottom='am, both asleep', back_big='zzz', back_small='call still on', face='#2b3a63', rim='#141c35', ink='#f6efe0', art=[('moon', 0, -78, .5)])
envelope_('openwhen', 'Write "open when" letters', key='write "open when" letters', batch=B, palette='plum', name='A bundle of open-when envelopes, the first one opened',
          front='open when you miss me', letter=['Open when you miss me:', 'I miss you more.', 'Look up, same sky.', 'Call me, even late.', '(four more to go)'], seal='#c2185b', art=[('heart', 180, 136, .4, '#ffffff')])
label('carepackage', 'Care package swap', batch=B, palette='kraft', name='A care package box with a customs label listing small things', kind='box', head='Care package',
      rows=['small things that', 'remind me of you', 'open on a call'], glass_='#c99c6c', paper_='#fbf6ea', fx='sparkle')
mapcard('walkcities', 'Walk your own cities together', batch=B, palette='sea', name='A map of two walks in two cities, earphones in, same playlist', head='Two walks, one playlist',
        route='M30 300 C80 260 120 240 160 200 M260 330 C300 280 340 240 400 200', pins=[(160, 200, 'my street'), (400, 200, 'yours')], labels=[(110, 90, 'my city', 0), (330, 90, 'your city', 0)])
