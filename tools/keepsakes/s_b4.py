"""Batch 4: slips 73-122 (Bandra cafés, heritage walks, the days apart)."""
from objects import *  # noqa: F401,F403
from cutouts import cutout  # noqa: F401
from things import *  # noqa: F401,F403

B = 'b4'

cutout('salsa', 'Learn a salsa step from a video', batch=B, palette='red', kind='sticker', name='A sticker of salsa footprints counting one-two-three, the turn finally done',
       parts=[('footprints', 0, -20, 3.6, '#16161a'), ('note', 110, -100, 1.4, '#d8483a')], head='1-2-3, 5-6-7', sub='the turn: finally', ribbon='#92261e', banner=(0, 100, -2))
cutout('scrapbook', 'Make a scrapbook of us', batch=B, palette='paper', kind='sticker', name='A sticker of a glue stick, scissors-cut hearts and washi tape from our scrapbook',
       parts=[('heart', -60, -40, 2.4, '#e0245e'), ('heart', 40, -70, 1.6, '#f2c230'), ('camera', 50, 10, 2.2, '#3a2a5a', '#f3efe2')], head='a scrapbook of us', sub='washi tape everywhere', ribbon='#3a2a5a', banner=(0, 100, -2))
cutout('bhaudaji', 'Dr. Bhau Daji Lad Museum', batch=B, palette='sand', kind='magnet', name='A magnet of the Bhau Daji Lad Museum facade with its gilded clock tower',
       parts=[('dome', 0, 0, 3.6, '#f1e7d0', '#8f7b52'), ('clock', 0, -90, 1.4, '#f2c230', '#5b3a1f')], head='Bhau Daji Lad', sub='Byculla, gilded ceilings', ribbon='#5b3a1f', banner=(0, 110, -2))
cutout('hajiali', 'Walk out to Haji Ali at sunset', batch=B, palette='sea', kind='magnet', name='A magnet of Haji Ali at low tide, the causeway out across the water',
       parts=[('dome', 60, -40, 2.6, '#f6f3ec', '#f6f3ec'), ('sun', -80, -80, 1.6, '#f6a04d'), ('waves', 0, 40, 3.6, '#2f6db5')], head='Haji Ali', sub='low tide only', ribbon='#326174', banner=(0, 110, -2))
cutout('powailake', 'Evening by Powai Lake', batch=B, palette='sea', kind='patch', name='A lakeside patch of the Powai promenade at dusk, the hills across the water',
       parts=[('hills', 0, -40, 3.6, '#5d7d4f', '#3f5d36'), ('waves', 0, 20, 3.6, '#5aa4d6'), ('moon', 110, -110, 1.2)], head='Powai Lake', sub='the walk, then dinner', ribbon='#326174', back=(160, 100, -10, '#cfe3ea'), banner=(0, 100, -2))
cutout('bandrafort', 'Sunset at Bandra Fort', batch=B, palette='sea', kind='pin', name='An enamel pin of the Sea Link cables at sunset from the Bandra Fort ramparts',
       parts=[('sun', 40, -40, 3.0, '#f6a04d'), ('anchor', -90, -20, 1.8, '#2b3a63'), ('fort', 0, 30, 2.6, '#a08060', '#7a5c40')], head='Bandra Fort', sub='the Sea Link at sunset', ribbon='#326174', banner=(0, 110, -2))
cutout('britannia', 'Berry pulao at Britannia & Co.', batch=B, palette='brick', kind='sticker', name='A sticker of the berry pulao handi and caramel custard from Britannia & Co.',
       parts=[('handi', -40, -20, 3.2, '#7a3a1a', '#d8483a'), ('cake', 90, 0, 1.8, '#f2c14e', '#7a3a1a')], head='Britannia & Co.', sub='berry pulao, since 1923', ribbon='#843b1f', banner=(0, 90, -2))
cutout('madhferry', 'Ferry from Versova to Madh Island', batch=B, palette='sea', kind='charm', name='A little ferry charm from the five-minute boat from Versova to Madh',
       parts=[('ferry', 0, -20, 3.6, '#f4ecd8', '#2f6db5'), ('fish', 90, 40, 1.2, '#5a8fa0')], head='Versova to Madh', sub='five minutes, ₹ in coins', ribbon='#326174', banner=(0, 80, -2))
cutout('fortwalk', 'Fort heritage walk at night', batch=B, palette='night', kind='magnet', name='A magnet of the Flora Fountain and the gothic High Court lit up at night',
       parts=[('church', 50, -40, 3.0, '#e9f0f7', '#22354c'), ('waterfall', -80, 0, 1.8, '#bfe3f2', '#2f4865'), ('moon', -80, -110, 1.2)], head='Fort, at night', sub='Flora Fountain to the Asiatic', ribbon='#22354c', banner=(0, 100, -2))
cutout('chorbazaar', 'Chor Bazaar treasure hunt', batch=B, palette='ochre', kind='charm', name='A brass bell charm, one of two strange treasures haggled for in Chor Bazaar',
       parts=[('bell', 0, -20, 3.6, '#c9a14a'), ('clock', 80, 40, 1.2, '#f4ecd8', '#5a3a10')], head='Chor Bazaar', sub='₹450, down from 900', ribbon='#a86a2c', banner=(0, 80, -2))
cutout('mysterygig', 'Go to a gig neither of you knows', batch=B, palette='ink', kind='pin', name='An enamel guitar pin from a gig by a band neither of us had heard of',
       parts=[('guitar', 0, -20, 3.6, '#e0245e', '#241c30'), ('note', 80, -80, 1.4, '#f2c230')], head='who were they?', sub='we bought the tee', ribbon='#362a45', banner=(0, 100, -6))
cutout('newcuisine', 'Eat a cuisine neither of you has tried', batch=B, palette='olive', kind='sticker', name='A passport-style sticker of a globe and a bowl of something new to us both',
       parts=[('globe', -40, -40, 2.8), ('dumpling', 70, 0, 2.0, '#f3ece0', '#d8cdb9')], head='first time, both', sub='stamped and eaten', ribbon='#364033', banner=(0, 90, -2))
cutout('vipassana', 'Global Vipassana Pagoda and Gorai beach', batch=B, palette='ochre', kind='magnet', name='A gold magnet of the Global Vipassana Pagoda, Gorai beach below',
       parts=[('pagoda', 0, -30, 3.8, '#e8b923', '#b88a1a'), ('waves', 0, 60, 2.6, '#2f6db5')], head='Global Pagoda', sub='then Gorai beach', ribbon='#a86a2c', banner=(0, 120, -2))
cutout('kondana', 'Kondana Caves trek', batch=B, palette='ghat', kind='patch', name='A trek patch of the Kondana caves behind a monsoon waterfall',
       parts=[('cave', -30, -20, 3.0, '#7a6a58', '#2a221a'), ('waterfall', 80, 0, 2.0, '#bfe3f2', '#4f7a45')], head='Kondana Caves', sub='monsoon, mud, worth it', ribbon='#3f553b', banner=(0, 90, -2))
label('sula', 'Vineyard weekend in Nashik', batch=B, palette='plum', name='A wine label from the vineyards in Nashik', kind='bottle', head='Nashik, our vintage',
      rows=['tasting, then lunch', 'sunset over the vines', 'one bottle home'], glass_='#3a1a2e', paper_='#f6efe0', fx='sparkle')
cutout('fireflies', 'Fireflies at Bhandardara', batch=B, palette='night', kind='charm', name='A glass-jar charm full of fireflies from the trees by Arthur Lake',
       parts=[('jar', 0, -20, 3.4, '#2f4865', '#c9a14a'), ('firefly', 0, -10, 2.0)], head='Bhandardara', sub='the trees blinked', ribbon='#22354c', banner=(0, 100, -2))
cutout('mokai', 'Matcha at Mokai', batch=B, palette='moss', kind='sticker', name='A sticker of a matcha bowl and its bamboo whisk from Mokai',
       parts=[('handi', 0, -10, 3.2, '#9ccc4a', '#6f8a3a'), ('brush', 90, -70, 1.6, '#d9b27a', '#d9b27a')], head='Mokai', sub='the floating matcha bar', ribbon='#3e5036', banner=(0, 90, -2))
cutout('houseofcroissant', 'Croissants at House of Croissant', batch=B, palette='sand', kind='sticker', name='A sticker of a fat croissant from House of Croissant',
       parts=[('croissant', 0, -20, 4.2)], head='House of Croissant', sub='one plain, one almond', ribbon='#5b3a1f', banner=(0, 70, -2))
cuppa('subko', 'Coffee and bakes at Subko', batch=B, palette='clay', name='A Subko takeaway cup from the Ranwar cottage brew bar', head='Subko', sub='Ranwar, Bandra',
      cup_col='#f6f3ec', band='#2b2b30', ink='#f6f3ec', marker='the new one', drink='#5a3216')
cutout('bagelshop', 'Brunch at The Bagel Shop', batch=B, palette='moss', kind='magnet', name='A magnet of a bagel and a potted fern from the greenhouse at The Bagel Shop',
       parts=[('cookie', -40, 0, 3.0, '#d9963a', '#f6efe0'), ('plant', 80, -40, 2.0)], head='The Bagel Shop', sub='Pali Hill greenhouse', ribbon='#3e5036', banner=(0, 100, -2))
cutout('pardonourfrench', 'Pardon Our French', batch=B, palette='plum', kind='pin', name='An enamel pin of a slice of brown butter cake from Pardon Our French',
       parts=[('slice', 0, -20, 3.6, '#f2e3b3', '#8a5a2e')], head='Pardon Our French', sub='brown butter cake', ribbon='#43244a', banner=(0, 70, -2))
cutout('kgcafe', 'Kala Ghoda Cafe', batch=B, palette='paper', kind='charm', name='A little coffee-cup charm from Kala Ghoda Cafe under its skylight',
       parts=[('cup', 0, -20, 3.8, '#f6f3ec', '#5a3216'), ('sun', 0, -110, 1.0, '#f2c230')], head='Kala Ghoda Cafe', sub='under the skylight', ribbon='#3a2a5a', banner=(0, 80, -2))
menu('mondegar', 'Cafe Mondegar and the Mario Miranda walls', batch=B, palette='brick', name='A folded Cafe Mondegar menu with Mario Miranda-style doodles',
     head='Mondegar', sub='Colaba, the cartoon walls', panels=(['jukebox', 'chilled beer', 'chicken burger'], ['find yourself', 'on the walls', '(we did)'], ['since 1932', 'Colaba Causeway', 'open 8 to 11:30']),
     ticks=[(0, 1), (0, 2), (1, 0)], accent='#c8312b', art=[('note', 225, 210, .8, '#fff')])
cakebox('concu', 'Conçu in Colaba', batch=B, palette='paper', name='A patisserie box from Conçu tied with a ribbon tag', head='Conçu', sub='Pasta Lane, Colaba',
        box='#fbf6ea', ribbon='#2f4a33', ink='#2f4a33', treat=('cake', '#f2d3a0', '#c0392b'))
cutout('prithvicafe', 'Irish coffee at Prithvi Cafe', batch=B, palette='brick', kind='sticker', name='A sticker of an Irish coffee in a stemmed glass under the Prithvi trees',
       parts=[('wine', 0, -20, 3.6, '#4a2a14', '#f6efe0'), ('leaf', -90, -80, 1.4, '#4f8a3a'), ('leaf', 90, -70, 1.2, '#6a9a4a')], head='Prithvi Cafe', sub='Irish coffee, obviously', ribbon='#843b1f', banner=(0, 90, -2))
cutout('silverbeach', 'Breakfast at Silver Beach Cafe', batch=B, palette='sea', kind='magnet', name='A magnet of eggs in a pan and a beach umbrella from Silver Beach Cafe',
       parts=[('umbrella', -60, -40, 2.6, '#2f6db5', '#f6f3ec'), ('plate', 40, 10, 2.6, '#f6f3ec', '#f2c230')], head='Silver Beach Cafe', sub='Juhu, breakfast', ribbon='#326174', banner=(0, 90, -2))
cutout('leapingwindows', 'Comics and coffee at Leaping Windows', batch=B, palette='lilac', kind='sticker', name='A comic-burst sticker with a speech bubble from Leaping Windows',
       parts=[('star', 0, -20, 5.0, '#f2c230'), ('book', 0, -10, 1.8, '#e0245e', '#f6efe0')], head='Leaping Windows', sub='POW! more coffee', ribbon='#553d66', banner=(0, 90, -2))
cutout('elnpink', 'Pink tea at EL&N', batch=B, palette='plum', kind='charm', name='A pink teacup charm from EL&amp;N under the flower wall',
       parts=[('cup', 0, 0, 3.4, '#f7c9d4', '#e0245e'), ('flower', -60, -80, 1.4, '#f7c9d4', '#c2185b'), ('flower', 60, -90, 1.2, '#f7b9cf', '#c2185b')], head='EL&amp;N', sub='pink everything', ribbon='#c2185b', banner=(0, 90, -2))
cutout('benne', 'Butter dosas at Benne', batch=B, palette='ochre', kind='sticker', name='A sticker of a folded benne dosa and a steel tumbler of filter coffee',
       parts=[('dosa', -30, 0, 3.4, '#e0a64a', '#f6f3ec'), ('tumbler', 110, -30, 1.6)], head='Benne', sub='butter, and more butter', ribbon='#a86a2c', banner=(0, 80, -2))
cutout('doughjoe', 'Pizza and coffee at Dough & Joe', batch=B, palette='brick', kind='pin', name='An enamel pin of a pizza slice crossed with a coffee cup from Dough &amp; Joe',
       parts=[('pizza', -40, -20, 3.2), ('cup', 70, 10, 1.8, '#f6f3ec', '#5a3216')], head='Dough &amp; Joe', sub='pizza, then coffee', ribbon='#843b1f', banner=(0, 110, -2))
cutout('boojee', 'Brunch at Boojee Cafe', batch=B, palette='olive', kind='magnet', name='A magnet of a stack of pancakes and a queue number from Boojee Cafe',
       parts=[('cake', 0, -20, 3.6, '#e0a64a', '#c0392b'), ('tag', 100, 30, 1.4, '#f6efe0')], head='Boojee Cafe', sub='queue no. 14', ribbon='#364033', banner=(0, 90, -2))
cutout('bokka', 'Coffee on tap at Bokka', batch=B, palette='kraft', kind='charm', name='A beer-mug-shaped coffee-on-tap charm from Bokka, 16th Road',
       parts=[('beer', 0, -20, 3.4, '#4a2a14', '#e9d6b3')], head='Bokka', sub='coffee, on tap', ribbon='#243a28', banner=(0, 90, -2))
cutout('greysoul', 'Grey Soul Coffee in Kala Ghoda', batch=B, palette='slate', kind='pin', name='An enamel pin of three coffee beans for the light roast at Grey Soul',
       parts=[('coffeebean', -50, -10, 2.4), ('coffeebean', 0, -40, 2.6), ('coffeebean', 50, -10, 2.4)], head='Grey Soul', sub='light roast, Kala Ghoda', ribbon='#26292e', banner=(0, 60, -2))
cutout('notfar', 'Cookies and cupcakes at NotFar', batch=B, palette='lilac', kind='sticker', name='A sticker of a cupcake and a chocolate-chip cookie from NotFar',
       parts=[('cupcake', -40, -30, 3.0), ('cookie', 70, 10, 2.0)], head='NotFar', sub='to share (we didn’t)', ribbon='#553d66', banner=(0, 90, -2))
cutout('magari', 'Hot chocolate at Magari', batch=B, palette='brick', kind='magnet', name='A magnet of a tall mug of thick hot chocolate with steam, from Magari among the books',
       parts=[('cup', 0, -10, 4.0, '#843b1f', '#3a1a0c'), ('steam', -10, -110, 1.6, '#fbeee3')], head='Magari', sub='thick enough to stand a spoon', ribbon='#843b1f', banner=(0, 90, -2))
cutout('croffle', 'Croffles at The Croffle Guys', batch=B, palette='ochre', kind='sticker', name='A sticker of a croffle, sweet against savoury at The Croffle Guys',
       parts=[('waffle', 0, -10, 4.2), ('strawberry', 90, -70, 1.2)], head='The Croffle Guys', sub='sweet won, 2 to 1', ribbon='#a86a2c', banner=(0, 70, -2))
clapper('pressplay', 'Press play together on the same movie', batch=B, palette='ink', name='A clapperboard for pressing play at the same time in two cities',
        head='3, 2, 1, play', rows=['two cities, one film', 'pause rule: either of us', 'scene: the good part'])
cutout('samerecipe', 'Cook the same recipe on a video call', batch=B, palette='kraft', kind='magnet', name='A magnet of two pans on a phone screen: the same recipe cooked on a call',
       parts=[('screens', 0, -30, 3.4), ('steam', -40, -130, 1.4), ('steam', 40, -130, 1.4)], head='same recipe', sub='yours burnt first', ribbon='#243a28', banner=(0, 100, -2))
cutout('ordereachother', 'Order each other dinner', batch=B, palette='brick', kind='charm', name='A scooter charm for the surprise dinners we each sent to the other’s door',
       parts=[('scooter', 0, -20, 3.6, '#d8483a'), ('handi', 0, -100, 1.0, '#7a3a1a', '#c9a14a')], head='delivery for you', sub='a surprise, each', ribbon='#843b1f', banner=(0, 80, -2))
cutout('museumonline', 'Wander a museum online', batch=B, palette='paper', kind='pin', name='An enamel pin of headphones and a picture frame from a museum walked online',
       parts=[('headphones', 0, -40, 3.0, '#3a2a5a', '#e9c46a'), ('laptop', 0, 30, 2.0, '#9aa0a6', '#f6efe0')], head='museum, online', sub='room 4 was our favourite', ribbon='#3a2a5a', banner=(0, 110, -2))
cutout('stargazecall', 'Stargaze on a call', batch=B, palette='navy', kind='patch', name='A night-sky patch: the same moon and stars from two windows',
       parts=[('moon', -30, -30, 3.6, '#f5edc8'), ('stars', 60, 0, 2.4), ('star', 90, -90, 1.0)], head='the same moon', sub='two windows, one call', ribbon='#1f2b4d', back=(140, 110, -20, '#2b3a63'), banner=(0, 110, -2))
cutout('coopgame', 'Co-op game night', batch=B, palette='arcade', kind='sticker', name='A sticker of two controllers joined by one cable for co-op game night',
       parts=[('gamepad', -50, -40, 2.4, '#3a2d6e', '#9dffc4'), ('gamepad', 50, 10, 2.4, '#e0245e', '#f2c230')], head='player 2 joined', sub='one farm, two players', ribbon='#1c153c', banner=(0, 100, -2))
cutout('chessstakes', 'Online chess with stakes', batch=B, palette='slate', kind='charm', name='A chess knight charm, with a forfeit poem owed on its back',
       parts=[('knight', 0, -20, 4.0, '#16161a')], head='check, and mate', sub='loser writes a poem', ribbon='#26292e', banner=(0, 100, -2))
cutout('readaloud', 'Read a book aloud to each other', batch=B, palette='moss', kind='pin', name='An enamel pin of an open book with a voice note rising from it',
       parts=[('book', 0, -10, 3.6, '#2f6d57', '#f6efe0'), ('note', 0, -90, 1.6, '#f2c230')], head='a chapter a night', sub='voices for everyone', ribbon='#3e5036', banner=(0, 80, -2))
cutout('bedtimestory', 'Make up a bedtime story together', batch=B, palette='navy', kind='sticker', name='A sticker of a sleepy moon on a pillow-cloud for the bedtime story we made up',
       parts=[('moon', 30, -60, 2.8, '#f5edc8'), ('mist', -20, 20, 3.0), ('stars', -80, -80, 1.4)], head='once upon a…', sub='one sentence each', ribbon='#1f2b4d', back=(140, 60, 10, '#e9f0f7'), banner=(0, 100, -2))
cutout('fallasleep', 'Fall asleep on a call', batch=B, palette='night', kind='magnet', name='A magnet of a phone glowing on a pillow, the call timer at 3:14 am',
       parts=[('phone', 0, -30, 3.0, '#1f2026', '#9fd3f2'), ('moon', 90, -100, 1.2), ('mist', 0, 60, 2.6)], head='3:14 am', sub='still connected', ribbon='#22354c', banner=(0, 110, -2))
cutout('openwhen', 'Write "open when" letters', batch=B, palette='kraft', kind='charm', name='A bundle of tiny open-when envelopes on a ring, the first one opened',
       parts=[('envelope', -30, -40, 2.4, '#f6efe0', '#a8321f'), ('envelope', 30, 0, 2.4, '#efe4cc', '#2a3f7a'), ('envelope', -20, 40, 2.4, '#f6efe0', '#c0392b')], head='open when…', sub='you miss me: opened', ribbon='#2f4a33', banner=(0, 120, -2), key='write "open when" letters')
cutout('carepackage', 'Care package swap', batch=B, palette='kraft', kind='charm', name='A little parcel charm, string-tied, from the care packages we swapped',
       parts=[('parcel', 0, -20, 3.6)], head='care package', sub='small things, posted', ribbon='#243a28', banner=(0, 100, -2))
cutout('walkcities', 'Walk your own cities together', batch=B, palette='moss', kind='sticker', name='A sticker of two map pins joined by a dotted walk, one in each city',
       parts=[('mappin', -70, -40, 2.6, '#d8483a'), ('mappin', 70, -20, 2.6, '#2f6db5'), ('waves', 0, 30, 2.0, '#3e5036')], head='two cities, one walk', sub='same playlist', ribbon='#3e5036', banner=(0, 110, -2))
