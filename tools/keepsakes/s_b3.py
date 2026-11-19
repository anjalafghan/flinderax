"""Batch 3: slips 22-72."""
from objects import *  # noqa: F401,F403

B = 'b3'

coin('imagicaa', 'Amusement park day', batch=B, palette='plum', name='A gold Imagicaa ride token with a rollercoaster loop on it',
     top='Imagicaa, Khopoli', big='1 RIDE', bottom='keep arms inside', back_big='again!', back_small='one more go',
     art=[('ghat', 0, -78, .5, '#5a3a10', '#e9c46a')], verbs=('Toss the token', 'Pocket it'))
mapcard('ranibaug', 'Zoo visit', batch=B, palette='moss', name='A zoo map of Rani Baug with a penguin and our route round it',
        head='Rani Baug, Byculla', route='M40 320 C80 280 100 200 160 190 S250 150 300 110 S380 60 390 40',
        pins=[(160, 190, 'elephants'), (300, 110, 'penguins!')], labels=[(90, 60, 'closed Wednesdays', 0)],
        art=[('penguin', 340, 190, 1.6), ('flamingo', 60, 300, 1.1), ('leaf', 200, 300, .9, '#6a9a4a'), ('bird', 180, 60, .9)])
scorecard('aquarium', 'Aquarium date', batch=B, palette='sea', name='A fish-ID card from Taraporewala Aquarium with the fish we spotted ticked',
          head='Spotted at Taraporewala', sub='Marine Drive, fish ID card', rows=['clownfish', 'a very slow turtle', 'the moray in the rocks', 'the one that stared back'],
          cols=('me', 'you'), marks=[('✓', '✓'), ('✓', ''), ('', '✓'), ('✓', '✓')], circle=3, shape='card', art=[('fish', 270, 40, .6, '#e46f2a')])
stub('planetarium', 'Planetarium show', batch=B, palette='navy', name='A star-chart ticket from Nehru Planetarium', head='Nehru Planetarium',
     sub='SKY THEATRE, WORLI', rows=['show at 4:30, dome seats', 'lean back, look up', 'closed Mondays'], band='#1f2b4d', paper_='#eef0f8',
     art=[('stars', 240, 150, .9, '#1f2b4d'), ('moon', 270, 120, .5, '#e9c46a')], note='we found Orion', verbs=('Dim the dome', 'Lights up'))
postcard('aer', 'Mumbai skyline view', batch=B, palette='night', name='A postcard of the Worli skyline at sunset from 34 floors up',
         scene=[('skyline', 90, 150, 1.6, '#2b3a63'), ('skyline', 260, 152, 1.4, '#36467a'), ('sun', 180, 120, .9, '#f6a04d'), ('cocktail', 320, 110, .5, '#e0245e')],
         sky=('#f7b267', '#f4845f'), ground='#1f2b4d', caption='34 floors up', message=['Aer, Four Seasons.', 'We got there an hour', 'before sunset and', 'watched the sea go pink.'], to='to us, Worli')
tag('worlibike', 'Cycling at Worli Sea Face', batch=B, palette='sea', name='A bicycle rental tag from Worli Sea Face', head='Anytime Cycling',
    rows=['Worli Sea Face', 'cycle no. 14, two hours', 'sea link on the left'], stamp_word='RETURNED', stamp_sub='early morning', paper_='#f2e3b3', art=[('bicycle', 200, 140, .6)])
tag('sgnp', 'Cycling in Sanjay Gandhi National Park', batch=B, palette='moss', name='A forest trail permit tag from Sanjay Gandhi National Park',
    head='SGNP trail permit', rows=['Borivali gate', 'two cycles, two hours', 'watch for the langurs'], stamp_word='ENTRY', stamp_sub='before 9', paper_='#e3ecd0',
    ink='#24401f', ink2='#2f6d57', art=[('leaf', 230, 40, .5)])
badge('midnightride', 'Midnight cycling ride', batch=B, palette='navy', name='A midnight brevet badge for the 25 km ride from Colaba',
      head='Midnight ride', big='25 km', sub='Colaba, 10 pm to 5 am', ribbon=('#1f2b4d', '#f2c230'), face='#eef0f8', art=[('moon', 70, -70, .45), ('bicycle', 0, 86, .55)])
postcard('marinedrive', 'Sunset date at Marine Drive', batch=B, palette='sea', name="A Queen's Necklace postcard of Marine Drive",
         scene=[('sun', 180, 130, 1, '#f6a04d'), ('necklace', 180, 172, 3.2, '#ffe08a', '#2b3a63'), ('waves', 90, 200, .8, '#ffffff')],
         sky=('#f9c784', '#e7766b'), ground='#36567a', caption="the Queen's Necklace", message=['On the sea wall', 'near Chowpatty, a gola', 'each, kala khatta.', 'Then the lights came on.'], to='to us')
postcard('lonavala', 'Long drive to Lonavala', batch=B, palette='ghat', name="A postcard of the Lonavala ghats: mist, the winding road, Tiger's Point and a chikki stop",
         scene=[('ghat', 120, 140, 2.2, '#3c3c3c', '#4f7a45'), ('mist', 250, 90, 1.2), ('mist', 70, 70, .9), ('hills', 300, 150, 1.4, '#6f8a64', '#4f6a45'), ('sun', 300, 50, .5, '#f6d08a')],
         sky=('#c9d6c4', '#e9ddc0'), ground='#3f553b', caption="Tiger's Point", message=['Old highway, all', 'the hairpins. Chikki', 'at Maganlal, mist at', "Tiger's Point, chai."], to='to us, Lonavala', stamp_art=('ghat',))
stub('ellora', 'Ajanta & Ellora caves weekend', batch=B, palette='clay', name='A ticket to Ellora and Ajanta with a fragment of a cave painting',
     head='Ellora + Ajanta', sub='CAVE 16, KAILASA', rows=['Ellora first, Ajanta next day', 'closed Mon (Ajanta), Tue (Ellora)', 'base: Sambhajinagar'],
     band='#7a4a2a', paper_='#f1e3c4', ink='#3a2414', art=[('cave', 250, 150, .8, '#b08a5a', '#3a2414')], stub_text='2 ADULTS')
stub('maisonpvr', 'Movie in a recliner theatre', batch=B, palette='ink', name='A recliner-seat ticket from Maison PVR', head='Maison PVR',
     sub='JIO WORLD DRIVE, BKC', rows=['recliners G7 + G8', 'blanket, popcorn, feet up', 'evening show'], band='#2a2238', paper_='#f6efe3',
     art=[('star', 260, 150, .6, '#e9c46a')], stub_text='RECLINER', note='fell asleep for 4 minutes')
media('pokemon', 'Play Pokémon Emerald together', batch=B, palette='pine', name='A game cartridge with our team and starters written on its label',
      kind='cartridge', head='EMERALD', rows=['our save file', 'starters: Treecko, Mudkip', 'nuzlocke: on', 'badges: 3 so far', 'do not overwrite'],
      shell='#1f8a5a', paper_='#f6efe0', verbs=('Load the save', 'Turn it off'))
media('hindirap', 'Hindi rap listening session', key='Hindi rap listening session (watch {b} react)', batch=B, palette='slate',
      name='A cassette with a ten-track Hindi rap tape and its J-card', kind='cassette', head='10 TRACKS', rows=['side A: watch the reaction'],
      shell='#1b1b1e', paper_='#f2c230', ink='#16161a')
label('cooktogether', 'Cook together', batch=B, palette='brick', name='A recipe card tucked in a spice tin for a cuisine neither of us had cooked',
      kind='box', head='first time cooking', rows=['a cuisine neither of us', 'had tried before', 'splash marks: several'], glass_='#c0392b', paper_='#fbf6ea', fx='steam')
badge('jigsaw', 'Do a physical jigsaw puzzle together', batch=B, palette='paper', name='The last jigsaw piece, kept as a badge', head='piece 1000',
      big='DONE', sub='found it under the sofa', ribbon=('#2f6db5', '#e8b923'), art=[('puzzle', 0, 86, .9, '#2f6db5')], verbs=('Press it in', 'Lift it out'))
strip('sketch', 'Draw / sketch together', batch=B, palette='paper', name='Two ten-minute portraits of each other, side by side on a sketch strip',
      frames=[('#f6f1e4', '#f6f1e4', [('heart', 75, 60, .5, '#2b2b30')]), ('#f6f1e4', '#f6f1e4', [('star', 75, 60, .5, '#2b2b30')])], caption='10 minutes each', verbs=('Draw it in', 'Rub it out'))
tag('clayset', 'Clay set — make things together', batch=B, palette='clay', name='A tag tied to the little clay creatures we made', head='made of clay',
    rows=['two creatures, unnamed', 'air-dried overnight', 'paint them next time'], stamp_word='DRY', stamp_sub='by morning', paper_='#e7c9a6', art=[('dumpling', 230, 40, .5, '#c47a4a', '#a65a30')])
stub('prithvi', 'A play at Prithvi Theatre', batch=B, palette='brick', name='A Prithvi Theatre playbill-ticket with an Irish coffee ring', head='Prithvi Theatre',
     sub='JUHU, THRUST STAGE', rows=['row C, as close as it gets', 'Irish coffee at the café first', 'house full'], band='#7a1a16', paper_='#f6eddc',
     art=[('cup', 260, 160, .6, '#f6f3ec', '#6b3a1f')], note='the café after, obviously')
stub('elephanta', 'Ferry to Elephanta Caves', batch=B, palette='sea', name='A ferry boarding token to Elephanta and the Shiva cave', head='Gateway to Elephanta',
     sub='FERRY, ONE HOUR EACH WAY', rows=['first boat around 9', 'the toy train up the jetty', 'closed Mondays'], band='#2f6db5', paper_='#eef4f6',
     art=[('ferry', 250, 160, .7), ('cave', 210, 150, .45, '#9a8a78', '#2a221a')], stub_text='RETURN')
lanyard('flamingo', 'Flamingo boat ride', batch=B, palette='sea', name='A flamingo boat pass for Thane Creek', head='Flamingo boat', sub='Thane Creek, high tide',
        big='pink!', card='#f48aa6', ink='#3a1a28', art=[('flamingo', -60, 290, .8, '#ffffff', '#3a1a28'), ('waves', 50, 300, .6, '#ffffff')])
mapcard('kanheri', 'Hike to Kanheri Caves', batch=B, palette='moss', name='A rubbing of Kanheri carvings with the forest route up from the gate',
        head='Kanheri, inside SGNP', route='M30 330 C60 300 90 260 140 250 S220 200 250 160 S320 100 360 70', pins=[(140, 250, 'shuttle'), (360, 70, 'cave 3')],
        labels=[(200, 340, '7 km forest walk', 0)], art=[('cave', 320, 140, 1.9, '#8a7a68', '#2a221a'), ('leaf', 70, 190, 1.1), ('leaf', 230, 290, .9, '#6a9a4a'), ('bird', 140, 50, .8)])
meal('mohammedali', 'Late-night food walk on Mohammed Ali Road', batch=B, palette='brick', name='Malpua on grease paper with kebabs, from the night walk on Mohammed Ali Road',
     base='paper', base_col=('#f3e3c0', '#c9a87a'), items=[('kebab', 270, 250, 2.6), ('kebab', 290, 300, 2.5, '#7a3e1a'), ('dosa', 290, 390, 2.6, '#d9963a', '#f3e3c0'), ('lantern', 450, 220, 1.4, '#f2c230'), ('moon', 160, 230, .9)],
     tag_head='Mohammed Ali Road', tag_sub='malpua, kebabs, phirni, after 9')
stub('matheran', 'Matheran day trip', batch=B, palette='ghat', name='A toy train ticket from Neral to Matheran', head='Neral to Matheran', sub='TOY TRAIN, NO CARS UP TOP',
     rows=['narrow gauge, two coaches', 'then red mud paths', 'horse or walk, we walked'], band='#c8362a', paper_='#f6eddc', art=[('train', 250, 160, .6)], stub_text='MATHERAN')
label('childhood', "Cook a dish from each other's childhood", batch=B, palette='kraft', name="A jar labelled with the secret ingredient from each other's childhood recipe",
      kind='jar', head="mum's secret", rows=['one dish each', 'from when we were small', 'called home to check'], glass_='#c99c6c', paper_='#fbf6ea', fx='steam')
envelope_('oneyear', 'Write letters to open in a year', batch=B, palette='kraft', name='A sealed envelope marked do not open for a year', front='open in one year, not before',
          letter=['Dear us,', 'if you are reading this', 'it has been a year.', 'Did we keep the promise?', 'There is a photo inside.'], art=[('heart', 180, 136, .4, '#ffffff')])
stub('firstdate', 'Recreate your first date at home', batch=B, palette='plum', name='A rewind ticket for our first date, recreated at home', head='First date, again',
     sub='REWIND, AT HOME', rows=['same food, same songs', 'same outfit (almost)', 'what you remember vs me'], band='#5d3564', paper_='#fbf0f4', art=[('heart', 260, 150, .6)], stub_text='TAKE 2')
scorecard('livingkaraoke', 'Bollywood karaoke night in the living room', batch=B, palette='plum', name='A living-room karaoke score card, each song out of ten',
          head='Living-room karaoke', sub='pick a decade, score each other', rows=['the 90s duet', 'the sad one', 'the dance number', 'the one we both forgot'],
          cols=('me', 'you'), marks=[('9', '8'), ('7', '10'), ('10', '9'), ('5', '6')], circle=2, art=[('mic', 290, 40, .6)])
scorecard('tastetest', 'Blind taste test', batch=B, palette='ochre', name='A blind taste-test grid with five hidden samples', head='Blind taste test',
          sub='labels hidden, loser plans next', rows=['sample A', 'sample B', 'sample C', 'sample D', 'sample E'], cols=('me', 'you'),
          marks=[('3', '4'), ('5', '2'), ('1', '1'), ('4', '5'), ('2', '3')], circle=1, shape='card')
media('playlists', 'Make each other a playlist', batch=B, palette='lilac', name='A mixtape J-card with the ten songs we picked for each other', kind='cassette',
      head='FOR YOU', rows=['ten songs, one reason each'], shell='#6f5282', paper_='#f7eefb', ink='#2b1d33')
tag('bookclub', 'Start a two-person book club', batch=B, palette='moss', name='A bookmark for our two-person book club', head='Book club of two',
    rows=['a chapter a day', 'talk it over on Sunday', 'with chai'], stamp_word='CH. 12', stamp_sub='this week', paper_='#efe1c0', art=[('book', 230, 40, .5)])
label('chapelroad', 'Street-art walk around Chapel Road', batch=B, palette='brick', name='A spray-paint can labelled for the Chapel Road murals', kind='bottle',
      head='Chapel Road', rows=['Bandra mural lanes', 'morning light', 'breakfast after'], glass_='#2aa3a8', paper_='#f6efe0', fx='sparkle')
postcard('banganga', 'Banganga Tank and the Walkeshwar lanes', batch=B, palette='sea', name='A postcard of the Banganga tank steps and temples',
         scene=[('temple', 90, 120, 1, '#e8b923', '#b5452c'), ('temple', 270, 118, .9, '#f1e7d0', '#7a4a2a'), ('waves', 180, 190, 1.4, '#ffffff'), ('bird', 180, 60, .6, '#3a3a3a')],
         sky=('#cfe3ea', '#f3e1c4'), ground='#5a8fa0', caption='Banganga', message=['Steps all the way', 'down to the water.', 'Hidden in Malabar Hill.', 'Went early, so quiet.'], to='to us')
tag('hanginggardens', 'Morning at the Hanging Gardens', batch=B, palette='moss', name='A bookmark with a pressed hibiscus from the Hanging Gardens', head='Hanging Gardens',
    rows=['Malabar Hill, early', 'the old-shoe park after', 'free entry'], stamp_word='PRESSED', stamp_sub='one hibiscus', paper_='#f6efe0', art=[('flower', 236, 40, .5, '#d6334a', '#f2c230')])
receipt('kyani', 'Breakfast at Kyani & Co.', batch=B, palette='clay', name='An old Irani café bill from Kyani & Co.', head='Kyani & Co.', sub='Irani café, Dhobi Talao',
        rows=[('bun maska', '×2'), ('akuri', '×1'), ('Irani chai', '×2'), ('mawa cake', '×1')], total=('paid', 'cash'), note='since 1904', art=[('cup', 470, 420, .9, '#f6f3ec', '#8a5a2e'), ('steam', 470, 380, .6)])
tag('jehangir', 'Jehangir Art Gallery and a Kala Ghoda wander', batch=B, palette='paper', name='A gallery entry tag from Jehangir Art Gallery', head='Jehangir Art Gallery',
    rows=['Kala Ghoda', "free, closed Sundays", 'then the bookshops'], stamp_word='VISITED', stamp_sub='Kala Ghoda', paper_='#fbfaf6', ink2='#3a2a5a')
tag('sassoon', 'Sunrise at Sassoon Docks', batch=B, palette='sea', name='A fish-market crate tag from Sassoon Docks at dawn', head='Sassoon Docks',
    rows=['5 am, closed shoes', 'bombil, pomfret, prawns', 'the gulls get there first'], stamp_word='DAWN', stamp_sub='Colaba', paper_='#dfe9ee', ink='#1d2b30', ink2='#2f6db5', art=[('fish', 236, 40, .5, '#5a8fa0')])
meal('juhu', 'Juhu Beach street-food evening', batch=B, palette='sea', name='A paper plate of pav bhaji and a gola from Juhu Beach at dusk', base='paper',
     base_col=('#f6f3ec', '#c9c4b8'), items=[('plate', 280, 330, 3.6, '#f6f3ec', '#d8641e'), ('scoop', 450, 250, 2.2, '#e0245e', '#f6f3ec'), ('sun', 170, 220, 1.1, '#f6a04d'), ('waves', 300, 450, 1.6, '#5aa4d6')],
     tag_head='Juhu Beach', tag_sub='pav bhaji, gola, sand at dusk')
stub('operahouse', 'A show at the Royal Opera House', batch=B, palette='red', name='A box ticket for the Royal Opera House', head='Royal Opera House',
     sub='GIRGAON, SINCE 1911', rows=['box seats, gilded', 'dressed up a little', 'the chandelier'], band='#7a1a16', paper_='#f6eddc',
     art=[('dome', 250, 155, .7, '#f1e7d0', '#7a1a16')], stub_text='BOX 2')
coin('escaperoom', 'Escape room challenge', batch=B, palette='ink', name='A brass key token from an escape room', top='escape room, 60 min', big='OUT',
     bottom='with 4 min left', back_big='clue 3', back_small='was the clock', face='#d9a84a', rim='#7a5a1a', ink='#3a2408')
scorecard('bowling', 'Bowling night', batch=B, palette='slate', name='A bowling score sheet for two games', head='Bowling night', sub='loser buys the fries',
          rows=['frame 1-3', 'frame 4-6', 'frame 7-9', 'frame 10', 'total'], cols=('me', 'you'), marks=[('X', '9'), ('7', 'X'), ('X', 'X'), ('9', '8'), ('142', '138')], circle=4, shape='card')
scorecard('icecream', 'Ice-cream crawl', batch=B, palette='lilac', name='An ice-cream crawl passport with three stamps', head='Ice-cream crawl',
          sub='three scoops, three places', rows=['scoop one', 'scoop two', 'the flavour we would never order'], cols=('me', 'you'),
          marks=[('✓', '✓'), ('✓', '✓'), ('✓', '✓')], circle=2, shape='card', art=[('scoop', 290, 40, .6, '#f2a3b9')])
scorecard('karnala', 'Karnala Bird Sanctuary hike', batch=B, palette='moss', name='A field-guide page from Karnala with the birds we ticked', head='Karnala birds',
          sub='up to the fort pinnacle', rows=['paradise flycatcher', 'a hornbill (maybe)', 'drongo', 'kingfisher by the stream'], cols=('saw', ''),
          marks=[('✓',), ('✓',), ('✓',), ('',)], shape='card', art=[('bird', 290, 40, .7, '#2f6d57')])
coin('vasaifort', 'Vasai Fort and the beach', batch=B, palette='clay', name='A heavy old key token from Vasai Fort', top='Vasai Fort', big='1560s',
     bottom='Portuguese ruins', back_big='beach', back_small='then the sea', face='#c9a46e', rim='#7a5a30', ink='#3a2414', art=[('fort', 0, -78, .5, '#7a5a30', '#3a2414')])
badge('kolad', 'River rafting at Kolad', batch=B, palette='sea', name='A rafting helmet sticker from the Kundalika at Kolad', head='Kundalika',
      big='RAPIDS', sub='Kolad, monsoon water', ribbon=('#e46f2a', '#2f6db5'), face='#f2f6f8', scallop=False, art=[('waves', 0, 86, .9, '#2f6db5')])
stub('alibaug', 'A slow weekend in Alibaug', batch=B, palette='sea', name='A Gateway to Mandwa ferry ticket for a slow Alibaug weekend', head='Gateway to Mandwa',
     sub='FERRY, THEN THE BEACH', rows=['homestay by the sand', 'no plans past sunset', 'Alibaug weekend'], band='#2f6db5', paper_='#eef4f6', art=[('boat', 255, 150, .7)], stub_text='MANDWA')
lanyard('blanketfort', 'Blanket fort and a movie inside it', batch=B, palette='lilac', name='A hand-drawn entry pass to our blanket fort', head='Blanket fort',
        sub='phones stay outside', big='ENTER', card='#f2c230', ink='#2b1d33', strap='#6f5282', art=[('lights', 0, 280, 1)])
envelope_('36questions', 'The 36 questions', batch=B, palette='plum', name='An envelope of the 36 questions, the last one still sealed', front='questions 1 to 36',
          letter=['set I: 1 to 12', 'set II: 13 to 24', 'set III: 25 to 36', 'then four minutes,', 'eyes only'], seal='#5d3564')
label('spanight', 'Spa night at home', batch=B, palette='lilac', name='A bottle of massage oil with a menu for spa night at home', kind='bottle',
      head='Spa night', rows=['face masks', 'a foot soak', 'one slow massage each'], glass_='#c9a0dc', paper_='#fbf6ff', fx='bubbles')
badge('bakeoff', 'Two-person bake-off', batch=B, palette='ochre', name="A judge's rosette from our two-person bake-off", head='Bake-off',
      big='1st', sub='same recipe, one oven', ribbon=('#2f6db5', '#e8b923'), art=[('cake', 0, 86, .9)])
