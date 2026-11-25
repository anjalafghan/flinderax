"""Batch 3: slips 22-72."""
from objects import *  # noqa: F401,F403
from cutouts import cutout  # noqa: F401
from things import *  # noqa: F401,F403

B = 'b3'

coin('imagicaa', 'Amusement park day', batch=B, palette='plum', name='A gold Imagicaa ride token with a rollercoaster loop on it',
     top='Imagicaa, Khopoli', big='1 RIDE', bottom='keep arms inside', back_big='again!', back_small='one more go',
     art=[('ghat', 0, -78, .5, '#5a3a10', '#e9c46a')], verbs=('Toss the token', 'Pocket it'))
cutout('ranibaug', 'Zoo visit', batch=B, palette='moss', kind='charm', name='A penguin charm on a key ring from Rani Baug, with a little flamingo beside it',
       parts=[('penguin', -20, -30, 3.6), ('flamingo', 70, -10, 2.2)], head='Rani Baug', sub='closed Wednesdays', ribbon='#2f6d57', banner=(0, 90, -3))
cutout('aquarium', 'Aquarium date', batch=B, palette='sea', kind='sticker', name='A sticker of a clownfish and a slow turtle-green bubble from Taraporewala Aquarium',
       parts=[('fish', -20, -40, 3.4, '#e46f2a'), ('fish', 70, 20, 1.6, '#2f6db5'), ('drop', -90, 20, 1.2, '#bfe3f2'), ('drop', -70, -100, .8, '#bfe3f2')], head='Taraporewala', sub='the one that stared back', ribbon='#2f6db5', banner=(0, 80, -2))
cutout('planetarium', 'Planetarium show', batch=B, palette='navy', kind='pin', name='An enamel pin of the planetarium dome under a crescent moon and Orion',
       parts=[('dome', 0, 0, 3.4, '#eef0f8', '#1f2b4d'), ('moon', -80, -110, 1.4, '#e9c46a'), ('star', 70, -120, .9, '#f1c84b'), ('star', 110, -80, .6, '#f1c84b')], head='Nehru Planetarium', sub='we found Orion', ribbon='#1f2b4d', banner=(0, 110, -2))
cutout('aer', 'Mumbai skyline view', batch=B, palette='night', kind='magnet', name='A skyline magnet of Worli at sunset, a cocktail on the ledge, 34 floors up',
       parts=[('sun', 0, -50, 2.8, '#f6a04d'), ('skyline', 0, 0, 3.6, '#2b3a63'), ('cocktail', 120, -10, 1.4, '#e0245e')], head='Aer, 34 floors up', sub='Four Seasons, Worli', ribbon='#22354c', banner=(0, 100, -2))
cutout('worlibike', 'Cycling at Worli Sea Face', batch=B, palette='sea', kind='charm', name='A bicycle charm on a key ring from Anytime Cycling at Worli Sea Face',
       parts=[('bicycle', 0, -10, 3.6, '#2f6db5', '#1d2b30')], head='Worli Sea Face', sub='cycle no. 14', ribbon='#2f6db5', banner=(0, 80, -2))
tag('sgnp', 'Cycling in Sanjay Gandhi National Park', batch=B, palette='moss', name='A forest trail permit tag from Sanjay Gandhi National Park',
    head='SGNP trail permit', rows=['Borivali gate', 'two cycles, two hours', 'watch for the langurs'], stamp_word='ENTRY', stamp_sub='before 9', paper_='#e3ecd0',
    ink='#24401f', ink2='#2f6d57', art=[('leaf', 230, 40, .5)])
cutout('midnightride', 'Midnight cycling ride', batch=B, palette='navy', kind='patch', name='An embroidered patch of a bicycle under the moon for the 25 km midnight ride from Colaba',
       parts=[('moon', 60, -90, 2.2, '#f5edc8'), ('bicycle', 0, 0, 3.4, '#f2c230', '#eef0f8'), ('stars', -70, -90, 1.6)], head='25 km, midnight', sub='Colaba, 10 pm to 5 am', ribbon='#1f2b4d', back=(150, 120, -20, '#2b3a63'), banner=(0, 100, -2))
postcard('marinedrive', 'Sunset date at Marine Drive', batch=B, palette='sea', name="A Queen's Necklace postcard of Marine Drive",
         scene=[('sun', 180, 130, 1, '#f6a04d'), ('necklace', 180, 172, 3.2, '#ffe08a', '#2b3a63'), ('waves', 90, 200, .8, '#ffffff')],
         sky=('#f9c784', '#e7766b'), ground='#36567a', caption="the Queen's Necklace", message=['On the sea wall', 'near Chowpatty, a gola', 'each, kala khatta.', 'Then the lights came on.'], to='to us')
cutout('lonavala', 'Long drive to Lonavala', batch=B, palette='ghat', kind='magnet', name="A magnet of the Lonavala ghat road winding up into the mist, with a slab of chikki",
       parts=[('ghat', 0, -30, 3.6, '#3c3c3c', '#4f7a45'), ('mist', -40, -70, 1.6), ('chocolate', 120, 40, 1.0, '#c98a3a', '#f2c230')], head="Tiger's Point", sub='chikki at Maganlal', ribbon='#3f553b', banner=(-20, 90, -2))
cutout('ellora', 'Ajanta & Ellora caves weekend', batch=B, palette='clay', kind='pin', name='An enamel pin of the Kailasa temple carved down into the rock at Ellora',
       parts=[('temple', 0, -30, 3.8, '#e8c48a', '#7a4a2a'), ('hills', -100, 20, 1.6, '#b08a5a', '#7a5a3a'), ('hills', 100, 20, 1.6, '#b08a5a', '#7a5a3a')], head='Ellora + Ajanta', sub='cave 16, Kailasa', ribbon='#7a4a2a', banner=(0, 90, -2))
cutout('maisonpvr', 'Movie in a recliner theatre', batch=B, palette='ink', kind='sticker', name='A sticker of a popcorn bucket and a film reel from the recliners at Maison PVR',
       parts=[('cup', -40, -20, 3.4, '#f6efe3', '#f2c230'), ('record', 80, -50, 1.8, '#2a2238', '#e9c46a')], head='Maison PVR', sub='recliners G7 + G8', ribbon='#2a2238', banner=(0, 80, -2))
cutout('pokemon', 'Play Pokémon Emerald together', batch=B, palette='pine', kind='charm', name='A green game-controller charm for our shared Pokémon Emerald save',
       parts=[('gamepad', 0, -20, 3.6, '#1f8a5a', '#f2c230')], head='EMERALD', sub='badges: 3 so far', ribbon='#0b2216', banner=(0, 70, -2))
media('hindirap', 'Hindi rap listening session', key='Hindi rap listening session (watch {b} react)', batch=B, palette='slate',
      name='A cassette with a ten-track Hindi rap tape and its J-card', kind='cassette', head='10 TRACKS', rows=['side A: watch the reaction'],
      shell='#1b1b1e', paper_='#f2c230', ink='#16161a')
cutout('cooktogether', 'Cook together', batch=B, palette='brick', kind='magnet', name='A fridge magnet of a masala dabba and a chilli from the night we cooked a cuisine neither of us knew',
       parts=[('spicetin', -20, -20, 3.2), ('chili', 100, -70, 1.6)], head='first time cooking', sub='splash marks: several', ribbon='#843b1f', banner=(0, 100, -2))
cutout('jigsaw', 'Do a physical jigsaw puzzle together', batch=B, palette='paper', kind='charm', name='The last jigsaw piece, found under the sofa, hung as a charm',
       parts=[('puzzle', 0, -30, 4.2, '#2f6db5')], head='piece 1000', sub='found under the sofa', ribbon='#3a2a5a', banner=(0, 90, -2))
cutout('sketch', 'Draw / sketch together', batch=B, palette='paper', kind='sticker', name='A sticker of a pencil and a brush crossed over a heart: ten-minute portraits of each other',
       parts=[('pencil', -50, -30, 3.0), ('brush', 50, -30, 2.8, '#c47a3a', '#3a2a5a'), ('heart', 0, 20, 1.2, '#e0245e')], head='10 minutes each', sub='no erasing', ribbon='#3a2a5a', banner=(0, 100, -2))
cutout('clayset', 'Clay set — make things together', batch=B, palette='clay', kind='patch', name='An embroidered patch of the two lumpy clay pots we made, air-dried overnight',
       parts=[('pot', -50, -10, 2.8, '#c47a4a', '#a65a30'), ('pot', 60, 0, 2.2, '#b5683a', '#7a3a1a')], head='made of clay', sub='two creatures, unnamed', ribbon='#7e6c53', banner=(0, 90, -2))
cutout('prithvi', 'A play at Prithvi Theatre', batch=B, palette='brick', kind='pin', name='An enamel pin of a lamp over the thrust stage at Prithvi, and an Irish coffee',
       parts=[('lantern', -40, -40, 3.0, '#f2c230', '#3a1a0c'), ('cup', 70, 10, 2.0, '#f6f3ec', '#6b3a1f')], head='Prithvi Theatre', sub='row C, house full', ribbon='#7a1a16', banner=(0, 100, -2))
cutout('elephanta', 'Ferry to Elephanta Caves', batch=B, palette='sea', kind='magnet', name='A fridge magnet of the Elephanta ferry crossing to the Shiva cave',
       parts=[('cave', 0, -50, 3.0, '#9a8a78', '#2a221a'), ('waves', 0, 30, 3.2, '#2f6db5'), ('ferry', -10, 40, 2.4)], head='Elephanta', sub='first boat around 9', ribbon='#2f6db5')
cutout('flamingo', 'Flamingo boat ride', batch=B, palette='sea', kind='charm', name='A pink flamingo charm from the boat ride on Thane Creek',
       parts=[('flamingo', 0, -40, 4.2, '#f48aa6', '#3a1a28'), ('waves', 0, 60, 2.6, '#2f6db5')], head='Thane Creek', sub='high tide, all pink', ribbon='#c2507a', banner=(0, 120, -2))
cutout('kanheri', 'Hike to Kanheri Caves', batch=B, palette='moss', kind='patch', name='A trail patch of the Kanheri caves in the forest with leaves around it',
       parts=[('cave', 0, -10, 3.4, '#8a7a68', '#2a221a'), ('leaf', -110, -60, 1.6), ('leaf', 110, -40, 1.4, '#6a9a4a')], head='Kanheri', sub='7 km forest walk', ribbon='#3e5036', banner=(0, 90, -2))
cutout('mohammedali', 'Late-night food walk on Mohammed Ali Road', batch=B, palette='brick', kind='sticker', name='A sticker of a hanging lantern, a crescent moon and a kebab from the night walk on Mohammed Ali Road',
       parts=[('lantern', -70, -40, 2.6, '#f2c230'), ('moon', 60, -90, 1.6), ('kebab', 30, 20, 2.6)], head='Mohammed Ali Road', sub='malpua, kebabs, after 9', ribbon='#843b1f', banner=(0, 100, -2))
cutout('matheran', 'Matheran day trip', batch=B, palette='ghat', kind='charm', name='A toy-train charm from Neral to Matheran',
       parts=[('train', 0, -20, 3.8, '#c8362a', '#f2c230'), ('hills', 0, -80, 2.6, '#6f8a64', '#4f6a45')], head='Neral to Matheran', sub='no cars up top', ribbon='#c8362a', banner=(0, 70, -2))
cutout('childhood', "Cook a dish from each other's childhood", batch=B, palette='kraft', name="A jar charm holding the secret ingredient from each other's childhood dish", kind='charm',
       parts=[('jar', 0, -20, 3.4, '#e9c9a0', '#8a5a2e'), ('heart', 0, -10, 1.2, '#c0392b')], head="mum's secret", sub='called home to check', ribbon='#2f4a33', banner=(0, 100, -2))
envelope_('oneyear', 'Write letters to open in a year', batch=B, palette='kraft', name='A sealed envelope marked do not open for a year', front='open in one year, not before',
          letter=['Dear us,', 'if you are reading this', 'it has been a year.', 'Did we keep the promise?', 'There is a photo inside.'], art=[('heart', 180, 136, .4, '#ffffff')])
polaroid('firstdate', 'Recreate your first date at home', batch=B, palette='plum', name='An instant photo from recreating our first date at home', scene=[('candle', 90, 150, 1.2), ('candle', 170, 150, 1.2), ('heart', 130, 70, 1), ('plate', 130, 200, 1.4)], sky=('#5d3564', '#a86a8a'), ground='#3a2030', caption='first date, take two')
cutout('livingkaraoke', 'Bollywood karaoke night in the living room', batch=B, palette='plum', kind='pin', name='An enamel pin of a microphone with music notes from living-room karaoke',
       parts=[('mic', -10, -20, 3.6, '#e9c9e0', '#3a1430'), ('note', 80, -70, 1.8, '#f2c230'), ('note', -90, -90, 1.2, '#f2c230')], head='living-room karaoke', sub='the 90s duet: 9/10', ribbon='#5d3564', banner=(0, 110, -2))
cutout('tastetest', 'Blind taste test', batch=B, palette='ochre', kind='magnet', name='A magnet of five spoons in a row for five hidden samples, one blindfold',
       parts=[('spoon', -80, -30, 2.0), ('spoon', -40, -40, 2.0), ('spoon', 0, -44, 2.0), ('spoon', 40, -40, 2.0), ('spoon', 80, -30, 2.0)], head='Blind taste test', sub='loser plans the next one', ribbon='#a86a2c', banner=(0, 80, -2))
cutout('playlists', 'Make each other a playlist', batch=B, palette='lilac', kind='charm', name='A little mixtape charm: ten songs we picked for each other',
       parts=[('cassette', 0, -20, 3.4, '#6f5282', '#f7eefb')], head='FOR YOU', sub='ten songs, one reason each', ribbon='#553d66', banner=(0, 80, -2))
bookmark('bookclub', 'Start a two-person book club', batch=B, palette='moss', name='A bookmark with a tassel in the book our two-person club is reading', head='Book club of two', rows=['a chapter a day', 'Sundays, with chai'], strip_col='#2f6d57')
cutout('chapelroad', 'Street-art walk around Chapel Road', batch=B, palette='brick', kind='sticker', name='A sticker of a spray can mid-spray and a painted eye from the Chapel Road murals',
       parts=[('spray', -40, -10, 3.4, '#2aa3a8'), ('heart', 70, -80, 1.6, '#f2c230'), ('star', 90, -10, 1.2, '#e0245e')], head='Chapel Road', sub='Bandra mural lanes', ribbon='#2aa3a8', banner=(0, 110, -2))
cutout('banganga', 'Banganga Tank and the Walkeshwar lanes', batch=B, palette='sea', kind='magnet', name='A magnet of the temple spire over the stepped Banganga tank',
       parts=[('temple', 0, -40, 3.4, '#e8b923', '#b5452c'), ('waves', 0, 50, 2.8, '#5a8fa0'), ('bird', 90, -110, 1.2)], head='Banganga', sub='steps down to the water', ribbon='#326174', banner=(0, 110, -2))
specimen('hanginggardens', 'Morning at the Hanging Gardens', batch=B, palette='moss', name='A pressed hibiscus from the Hanging Gardens, taped on a specimen card', head='Hanging Gardens', rows=['hibiscus, pressed flat', 'Malabar Hill, early morning'], item=('flower', '#d6334a', '#f2c230'))
receipt('kyani', 'Breakfast at Kyani & Co.', batch=B, palette='clay', name='An old Irani café bill from Kyani & Co.', head='Kyani & Co.', sub='Irani café, Dhobi Talao',
        rows=[('bun maska', '×2'), ('akuri', '×1'), ('Irani chai', '×2'), ('mawa cake', '×1')], total=('paid', 'cash'), note='since 1904', art=[('cup', 470, 420, .9, '#f6f3ec', '#8a5a2e'), ('steam', 470, 380, .6)])
cutout('jehangir', 'Jehangir Art Gallery and a Kala Ghoda wander', batch=B, palette='paper', kind='pin', name='An enamel pin of a paint palette from the Jehangir Art Gallery',
       parts=[('flower', -30, -40, 2.2, '#3a2a5a', '#e8b923'), ('heart', 50, -20, 1.6, '#c0392b')], head='Jehangir', sub='Kala Ghoda', ribbon='#3a2a5a', back=(120, 90, -30, '#f6efe0'), banner=(0, 70, -2))
cutout('sassoon', 'Sunrise at Sassoon Docks', batch=B, palette='sea', kind='patch', name='An embroidered patch of a bombil fish and the dawn sun over Sassoon Docks',
       parts=[('sun', 40, -60, 2.2, '#f2b33d'), ('fish', -10, 10, 3.6, '#5a8fa0', '#fff'), ('bird', -80, -70, 1.4)], head='Sassoon Docks', sub='5 am, the gulls first', ribbon='#1d2b30')
cutout('juhu', 'Juhu Beach street-food evening', batch=B, palette='sea', kind='sticker', name='A sticker of a kala khatta gola and a palm against the Juhu sunset',
       parts=[('sun', 40, -40, 2.6, '#f6a04d'), ('palm', -70, -30, 2.8), ('scoop', 40, 10, 2.6, '#7a1a2e', '#f6f3ec')], head='Juhu Beach', sub='pav bhaji, gola, dusk', ribbon='#326174', banner=(0, 110, -2))
cutout('operahouse', 'A show at the Royal Opera House', batch=B, palette='red', kind='pin', name='An enamel pin of the Royal Opera House chandelier',
       parts=[('bell', 0, -40, 3.0, '#e9c46a'), ('lights', 0, 20, 2.6, '#ffe08a', '#7a1a16')], head='Royal Opera House', sub='box seats, since 1911', ribbon='#7a1a16', banner=(0, 100, -2))
keytag('escaperoom', 'Escape room challenge', batch=B, palette='ink', name='The brass key that got us out of the escape room', head='ROOM 3', number='out, 4 min left', tag_col='#d9a84a', ink='#3a2408')
scorecard('bowling', 'Bowling night', batch=B, palette='slate', name='A bowling score sheet for two games', head='Bowling night', sub='loser buys the fries',
          rows=['frame 1-3', 'frame 4-6', 'frame 7-9', 'frame 10', 'total'], cols=('me', 'you'), marks=[('X', '9'), ('7', 'X'), ('X', 'X'), ('9', '8'), ('142', '138')], circle=4, shape='card')
cutout('icecream', 'Ice-cream crawl', batch=B, palette='lilac', kind='sticker', name='A sticker of three cones, one per stop on our ice-cream crawl',
       parts=[('scoop', -70, -30, 2.6, '#f2a3b9', '#d9a85c'), ('scoop', 0, -50, 2.8, '#fff3b0', '#d9a85c'), ('scoop', 70, -30, 2.6, '#8a5a3a', '#d9a85c')], head='3 scoops', sub='1 strange flavour', ribbon='#5a1838', banner=(0, 70, -3))
cutout('karnala', 'Karnala Bird Sanctuary hike', batch=B, palette='moss', kind='patch', name='A birding patch of a paradise flycatcher on a branch at Karnala',
       parts=[('olive', 0, 20, 3.2, '#4f8a3a', '#3e5036'), ('bird', -30, -60, 2.6, '#2f6d57'), ('peak', 100, -50, 1.4, '#7a8a6a')], head='Karnala birds', sub='a hornbill (maybe)', ribbon='#2f6d57', banner=(0, 100, -2))
cutout('vasaifort', 'Vasai Fort and the beach', batch=B, palette='clay', kind='magnet', name='A magnet of the ruined Portuguese walls of Vasai Fort with a palm growing through them',
       parts=[('fort', -20, -10, 3.0, '#c9a46e', '#3a2414'), ('palm', 90, -60, 2.6), ('shell', -120, 50, 1.0)], head='Vasai Fort', sub='1560s, then the sea', ribbon='#7a5a30', banner=(0, 100, -2))
cutout('kolad', 'River rafting at Kolad', batch=B, palette='sea', kind='patch', name='A rafting patch: a paddle over white water on the Kundalika at Kolad',
       parts=[('waves', 0, 0, 3.6, '#2f6db5'), ('cricket', -10, -40, 2.4, '#e46f2a', '#2f6db5')], head='Kundalika rapids', sub='Kolad, monsoon water', ribbon='#e46f2a', back=(150, 90, -10, '#dfe9ee'), banner=(0, 100, -2))
cutout('alibaug', 'A slow weekend in Alibaug', batch=B, palette='sea', kind='charm', name='A sailboat charm from the Mandwa ferry and a slow Alibaug weekend',
       parts=[('boat', 0, -30, 3.8, '#2f6db5', '#f4ecd8'), ('shell', 80, 40, 1.2)], head='Mandwa', sub='no plans past sunset', ribbon='#2f6db5', banner=(0, 80, -2))
blanket_fort('blanketfort', 'Blanket fort and a movie inside it', batch=B, palette='lilac', name='The corner of our blanket fort, pegged to a string, with the fairy lights that came on for the movie')
question_deck('36questions', 'The 36 questions', batch=B, palette='plum', name='The 36 questions as a fanned deck of cards, the last one turned up')
cutout('spanight', 'Spa night at home', batch=B, palette='lilac', kind='sticker', name='A sticker of a lotus, a candle and oil drops for spa night at home',
       parts=[('lotus', -30, -20, 3.2, '#c9a0dc', '#8a6a9c'), ('candle', 90, -10, 1.8), ('drop', -110, -80, 1.0, '#c9a0dc')], head='Spa night', sub='masks, soak, slow', ribbon='#6f5282', banner=(0, 90, -2))
badge('bakeoff', 'Two-person bake-off', batch=B, palette='ochre', name="A judge's rosette from our two-person bake-off", head='Bake-off',
      big='1st', sub='same recipe, one oven', ribbon=('#2f6db5', '#e8b923'), art=[('cake', 0, 86, .9)])
