"""Batch 6: slips 169-214 (at home, games apart)."""
from objects import *  # noqa: F401,F403
from cutouts import cutout  # noqa: F401
from things import *  # noqa: F401,F403

B = 'b6'

clapper('dancevideo', 'Learn a dance from a video', batch=B, palette='plum', name='A clapperboard for the final take of the dance we learnt from a video',
        head='Final take', rows=['the routine, all of it', 'take 14 of 14', 'just for us'], slate='#3a1a3a')
cuppa('hotchocbar', 'Hot chocolate bar', batch=B, palette='kraft', name='A mug of hot chocolate built for each other, marshmallows and chilli', kind='mug', head='built for you',
      sub='marshmallow, cinnamon, chilli', cup_col='#c0392b', ink='#fff', drink='#4a2414', art=[('chili', 60, 60, .6, '#f2c230')])
meal('indoorpicnic', 'Indoor picnic on the floor', batch=B, palette='brick', name='An indoor picnic on a gingham cloth on the floor', base='tray', base_col=('#e46f6f', '#f6efe0'),
     items=[('plate', 230, 300, 2, '#f6f3ec', '#9ccc4a'), ('strawberry', 380, 280, 1.2), ('glass', 420, 380, 1.4, '#f2b33d'), ('croissant', 250, 400, 1.4)], tag_head='Indoor picnic', tag_sub='no sofa allowed')
stub('cardgame', 'Teach each other a card game', batch=B, palette='red', name='A playing card from the childhood card games we taught each other', head='Three in a row',
     sub="YOUR GAME, THEN MINE", rows=['taught from childhood', 'house rules apply', 'three wins in a row'], band='#7a1a16', paper_='#fbf6ea', art=[('heart', 260, 150, .7, '#c0392b')], stub_text='DEAL', shape='scallop')
tag('neighbourcookies', 'Bake cookies for the neighbours', batch=B, palette='ochre', name='A gift tag for the cookies we baked for the neighbours', head='for next door',
    rows=['one batch for us', 'one batch for you', 'we rang together'], stamp_word='BAKED', stamp_sub='this morning', paper_='#f6efe0', ink2='#a8321f', art=[('cake', 236, 40, .5, '#d9963a', '#7a3a1a')])
strip('dayinlife', 'Film a day in our life', batch=B, palette='slate', name='Film frames from the day in our life we filmed and edited together', film=True,
      frames=[('#f6c27a', '#c9a87a', [('cup', 75, 70, .9)]), ('#9fc7e0', '#5d7d4f', [('bicycle', 75, 70, .8)]), ('#2b3a63', '#1f2b4d', [('moon', 75, 50, .7)])])
label('cocktaillab', 'Mocktail or cocktail lab', batch=B, palette='plum', name='A lab bottle labelled with the drink each of us invented and named after the other', kind='bottle',
      head='"The Diya"', rows=['three ingredients each', 'one drink each', 'named after you'], glass_='#e0245e', paper_='#fbf6ea', fx='bubbles')
seedpacket('herbbox', 'Start a herb box together', batch=B, palette='moss', name='A herb seed packet for the windowsill box: basil, mint, coriander', head='Windowsill herbs',
           rows=['basil, mint, coriander', 'water: whoever is up first', 'cook with it in a month'], packet='#d6e8c4', seed_col='#3a2a1a', art=(('leaf', 0, -40, 1.6),))
sticker('tiedye', 'Tie-dye T-shirts', batch=B, palette='lilac', name='A tie-dye swatch sticker from the T-shirts we made for each other', head='tie-dyed',
        sub='made by you, worn by me', shape='blob', col='#8a3fa0', art=[('flower', 0, -70, .9, '#f2c230', '#e0245e')])
paper_crane('cranes', 'Fold a hundred paper cranes', batch=B, palette='sea', name='One folded paper crane on its thread, tagged 1 of 100')
scorecard('seriescalls', 'Start a series, only on calls', batch=B, palette='night', name='An episode tracker for the show we only watch together on calls', head='Our show, calls only',
          sub='no watching ahead', rows=['episode 1', 'episode 2', 'episode 3', 'episode 4 (you dozed)'], cols=('watched',), marks=[('✓',), ('✓',), ('✓',), ('½',)], shape='card')
scorecard('scavengercall', 'Video-call scavenger hunt', batch=B, palette='brick', name='A scavenger hunt list from a video call, first to show it wins', head='Show me...',
          sub='first to the camera wins', rows=['something blue', 'the oldest thing you own', 'a spoon, any spoon', 'a photo of us'], cols=('me', 'you'),
          marks=[('✓', ''), ('', '✓'), ('✓', ''), ('✓', '✓')], circle=3, shape='clip')
coin('ludoonline', 'Online board game night', batch=B, palette='red', name='A Ludo token from our online board game night', top='online, Ludo', big='home!',
     bottom='loser picks the movie', back_big='six!', back_small='roll again', face='#c0392b', rim='#7a1a16', ink='#fff')
media('voicenotes', 'A voice-notes-only day', batch=B, palette='olive', name='A cassette of a whole day of voice notes, like little radio messages', kind='cassette',
      head='RADIO US', rows=['no texts, all day'], shell='#4a5646', paper_='#efe8d6', ink='#1d211b')
sticker('matchingoutfits', 'Matching outfits on a call', batch=B, palette='sea', name='Matching swatches sticker for the outfits we revealed at the same moment', head='matching',
        sub='theme: blue, reveal on 3', shape='star', col='#2f6db5')
scorecard('snacktest', 'Snack taste test, two cities', batch=B, palette='ochre', name='A snack tasting chart from two cities, rated blind on a call', head='Snacks, two cities',
          sub='same list, rated blind', rows=['chakli', 'banana chips', 'the spicy mix', 'the mystery one'], cols=('me', 'you'), marks=[('8', '9'), ('6', '7'), ('9', '9'), ('3', '2')], circle=2, shape='card')
mapcard('neighbourhood', "Tour each other's neighbourhood", batch=B, palette='sea', name='A hand-drawn tour map of the streets we walked for each other on a call',
        head='My street, your street', route='M40 320 C90 280 120 250 160 220 S240 180 280 160 S360 120 390 80', pins=[(160, 220, 'my chai stall'), (390, 80, 'the shortcut')], art=[('cup', 300, 290, .8)])
bookmark('journalcall', 'Journal side by side', batch=B, palette='kraft', name='A bookmark in the journal we wrote side by side on a call', head='twenty quiet minutes',
         rows=['then one line each', 'read aloud'], strip_col='#7a4a2a', book_col=('#3a2414', '#fbf3dc'))
polaroid('roomfrommemory', "Draw each other's room from memory", batch=B, palette='paper', name="An instant photo of the drawing of each other's room from memory",
         scene=[('house', 130, 140, 2, '#f6efe0', '#2b2b30'), ('lantern', 60, 60, .8)], sky=('#fbfaf6', '#f3efe6'), ground='#f3efe6', caption='the bed is not there')
media('samesong', 'Learn the same song, apart', batch=B, palette='lilac', name='A record of the one song we each learnt apart and played for the other', kind='record',
      head='One song', rows=['learnt apart', 'voice and guitar', 'performed on a call', 'applause: two'], paper_='#f7eefb', accent='#6f5282')
menu('reversedinner', 'Reverse dinner: dessert first, starter last', batch=B, palette='brick', name='An upside-down menu: dessert first, starter last', head='Reverse dinner',
     sub='dessert first', panels=(['1. dessert', 'your pick', 'for me'], ['2. main', 'the usual', ''], ['3. starter', 'last, finally', 'cold by now']), ticks=[(0, 0), (1, 0), (2, 0)], accent='#7a1a16')
scorecard('paperplanes', 'Paper plane distance contest', batch=B, palette='sea', name="A paper plane flight log from the hallway contest", head='Flight log',
          sub='three planes each, same spot', rows=['plane 1', 'plane 2', 'plane 3', 'loser makes chai'], cols=('me', 'you'), marks=[('3m', '4m'), ('5m', '2m'), ('6m', '6m'), ('', '✓')], circle=3, shape='card')
rangoli_plate('rangoli', 'Rangoli battle on the living-room floor', batch=B, palette='ochre', name='A steel plate of rangoli colours and the lotus border we raced to finish')
henna_cone('mehendi', "Mehendi on each other's hands, symbols only", batch=B, palette='brick', name='A mehendi cone, and the moon and vine we drew on each other, no words')
stub('sockpuppet', 'Sock-puppet argument-off', batch=B, palette='plum', name='A debate card from the sock-puppet argument about whether dosa is a snack or a meal',
     head='Dosa: snack or meal?', sub='SOCK PUPPETS ONLY', rows=['me (left sock): meal', 'you (right sock): snack', 'winner: the dosa'], band='#5d3564', paper_='#fbf0f4', stub_text='DEBATE', shape='torn')
stub('spaceship', 'Cardboard spaceship voyage', batch=B, palette='navy', name='A boarding pass for our cardboard spaceship voyage around the flat', head='Flight to the kitchen',
     sub='CARDBOARD SPACESHIP', rows=['gate: the hall', 'planet: the balcony', 'snacks on board'], band='#1f2b4d', paper_='#eef0f8', art=[('star', 260, 150, .6, '#e9c46a')], stub_text='BOARDING')
house_scroll('houserules', 'Write five house rules and sign them', batch=B, palette='kraft', name='A scroll of our five house rules, signed by both of us and sealed')
mapcard('memorypalace', 'Memory palace walk', batch=B, palette='kraft', name='A floor key of the flat with a memory pinned in each room', head='Our memory palace',
        route='M50 300 C100 250 140 260 180 200 S260 140 320 160 S380 90 390 60', pins=[(180, 200, 'the first fight'), (320, 160, 'the cake'), (390, 60, 'the rain')], land='#f3e6c8')
envelope_('tiffinswap', 'Tiffin swap for tomorrow', batch=B, palette='ochre', name='A note tucked into the tiffin we packed for each other', front='open at lunch',
          letter=['Dal, rice, the good pickle.', 'Eat the fruit first.', 'Drink water.', 'I packed a chocolate.', 'Miss you by 1 pm.'], seal='#7a4a2a')
coin('hideseek', 'Hide-and-seek around the flat', batch=B, palette='night', name='A counting token from hide-and-seek in the dark flat', top='lights off', big='30',
     bottom='count, eyes closed', back_big='found you', back_small='behind the curtain', face='#2b3a63', rim='#141c35', ink='#f6efe0', art=[('moon', 0, -78, .5)])
menu('frontpage', 'Fake front page of your week', batch=B, palette='paper', name='A fake newspaper front page about our week', head='The Weekly Us',
     sub='all the news that fits', panels=(['LOCAL MAN', 'FINDS SOCK', 'p. 2'], ['WEATHER:', 'mostly sunny', 'some sulking'], ['SPORTS:', 'she won', 'again']),
     ticks=[(0, 0), (1, 1), (2, 1)], accent='#2b2b30')
stub('shadowpuppets', 'Torch shadow-puppet theatre', batch=B, palette='ink', name='A programme for our torch shadow-puppet theatre on the bedroom wall', head='Shadow Theatre',
     sub='ONE TORCH, ONE WALL', rows=['act 1: the swan', 'act 2: the dog (dog?)', 'act 3: improvised'], band='#16161a', paper_='#f2c230', ink='#16161a', stub_text='ACT 3')
label('spicejars', 'Spice-jar challenge: three blind picks', batch=B, palette='brick', name='A mystery spice jar from the three blind picks', kind='jar', head='Mystery spice',
      rows=['three picks, no looking', 'cook something edible', 'it was hing'], glass_='#c0392b', paper_='#fbf6ea', fx='petals')
lanyard('sankranti', 'Kite flying on Sankranti', batch=B, palette='sea', name='A kite on its manja from Makar Sankranti on the terrace', head='Sankranti', sub='one flies, one holds the spool',
        big='kaipo!', card='#e0245e', ink='#fff', strap='#f2c230', art=[('kite', 0, 260, 1, '#f2c230', '#fff')])
postcard('showwindow', 'Show me your window', batch=B, palette='sea', name="A postcard of the light in each other's window", scene=[('sun', 90, 80, .8, '#f6d08a'), ('skyline', 230, 150, 1.6, '#2b3a63'), ('bird', 160, 50, .6)],
         sky=('#9fc7e0', '#f6e1c4'), ground='#3f5d7d', caption='guess the time', message=['Turned the camera', 'to the window.', 'You guessed 5 pm.', 'It was 5:20.'], to='to you')
media('soundmap', 'Sound map of your street', batch=B, palette='olive', name='A cassette of sixty seconds of street sounds sent to each other', kind='cassette',
      head='60 SEC OF MY STREET', rows=['guess what you hear'], shell='#364033', paper_='#efe8d6', ink='#1d211b')
scorecard('hidefind', 'Hide-and-find on video', batch=B, palette='red', name='A hot-and-cold card from hide-and-find on video', head='Hot or cold',
          sub='one hides, one walks, on a call', rows=['cold', 'warmer', 'warm', 'HOT', 'found it'], cols=('',), marks=[('✓',), ('✓',), ('✓',), ('✓',), ('✓',)], shape='card')
scorecard('birdcount', 'Ten-minute balcony bird count', batch=B, palette='moss', name='A tally of birds counted from two balconies in ten minutes', head='Balcony birds',
          sub='ten minutes, two windows', rows=['crows', 'pigeons', 'a bulbul', 'the parakeets'], cols=('me', 'you'), marks=[('7', '4'), ('12', '9'), ('1', ''), ('', '6')], circle=3, shape='card', art=[('bird', 290, 40, .7)])
cakebox('pomodoro', 'Silent co-work: two Pomodoros on camera', batch=B, palette='red', name='A tomato timer box for two Pomodoros of silent co-working on camera', head='2 × 25 min',
        sub='camera on, mics off', box='#f6efe0', ribbon='#c0392b', ink='#7a1a16', treat=('tomato',))
media('humguess', 'Hum-it, guess-it', batch=B, palette='plum', name='A four-bar staff from hum-it, guess-it', kind='record', head='Hum it',
      rows=['four bars only', 'you got it in two', 'I hummed it wrong', 'still counts'], paper_='#f7c9d4', accent='#3a1430')
badge('jenga', 'Jenga by description', batch=B, palette='kraft', name='A Jenga block rosette from playing Jenga by description', head='Jenga',
      big='fell', sub='your tower, my words', ribbon=('#c99c6c', '#7a4a2a'), scallop=False)
lanyard('commentate', "Commentate each other's evening", batch=B, palette='moss', name="A commentary-box pass for narrating each other's evening live", head='Commentary box',
        sub='live from your kitchen', big='SIX!', card='#2f6d57', ink='#fff', art=[('cricket', 0, 280, .9)])
scorecard('headlines', 'Dueling headlines for one moment', batch=B, palette='paper', name='Two rival headline clippings for the same moment', head='Same moment, two papers',
          sub='headline it', rows=['LOCAL MAN FINDS KEYS', 'KEYS IN HAND ALL ALONG', 'readers divided'], cols=('me', 'you'), marks=[('✓', ''), ('', '✓'), ('✓', '✓')], circle=1, shape='card')
specimen('showtell', 'Show-and-tell with invented histories', batch=B, palette='ochre', name='A museum label for an object given an invented history', head='Object no. 3',
         rows=['a spoon, said to belong', 'to a pirate (it did not)'], item=('spoon',))
postcard('weatherreport', 'One-line weather report for each other', batch=B, palette='sea', name='A weather postcard of how we each feel, one line a morning', scene=[('sun', 100, 90, 1.1, '#f2c230'), ('moon', 260, 70, .8), ('waves', 180, 190, 1.2, '#ffffff')],
         sky=('#9fc7e0', '#f6e1c4'), ground='#5a8fa0', caption='sunny, some clouds', message=['Monday: drizzle.', 'Tuesday: clearing.', 'Wednesday: you called,', 'so: sunny.'], to='to you, daily')
clapper('dialoguetennis', 'Bollywood dialogue tennis', batch=B, palette='brick', name='A clapper strip for Bollywood dialogue tennis', head='Dialogue tennis',
        rows=['you serve a line', 'I return the next', 'deuce, again'], slate='#7a1a16')
