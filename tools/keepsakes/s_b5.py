"""Batch 5: slips 123-168 (the days apart, then at home)."""
from objects import *  # noqa: F401,F403

B = 'b5'

cuppa('coffeeacross', 'Coffee date across cities', batch=B, palette='kraft', name='A cutting-chai glass for the same chai made at the same time in two cities', kind='cutting',
      head='same chai, two cities', sub='on a call, slowly', drink='#b9773a', verbs=('Pour it', 'Set it down'))
menu('nextvisit', 'Plan our next visit, hour by hour', batch=B, palette='ochre', name='A folded itinerary for our next visit, hour by hour', head='Next visit',
     sub='hour by hour', panels=(['9 am, chai', '11, the market', '1 pm, lunch'], ['4, the sea', '6, sunset', '8, dinner'], ['day two', 'no plans', 'at all']),
     ticks=[(0, 0), (1, 1), (2, 2)], accent='#2f6db5')
scorecard('dreamhome', 'Design our dream home', batch=B, palette='paper', name='A dream-home blueprint card, one room each, the kitchen argued over', head='Dream home',
          sub='one room each, then swap', rows=['your reading nook', 'my balcony garden', 'the kitchen (a fight)', 'a room for the cat we will get'], cols=('me', 'you'),
          marks=[('✓', '✓'), ('✓', '✓'), ('', ''), ('✓', '✓')], circle=2, shape='card', paper_='#e6eef6', ink='#1f3a5a', art=[('house', 280, 40, .7)])
sticker('newlanguage', 'Start a language together', batch=B, palette='sea', name='A flashcard sticker for the language we started together', head='hola, día 1',
        sub='the streak starts today', shape='blob', col='#2f6db5')
badge('onlineclass', 'Take an online class together', batch=B, palette='lilac', name='A workshop certificate rosette from the online class we took together', head='Workshop',
      big='passed', sub='watercolour, two screens', ribbon=('#6f5282', '#e8b923'))
polaroid('paintscreenshot', 'Paint each other from a screenshot', batch=B, palette='paper', name='An instant print of a portrait painted from a call screenshot',
         scene=[('screens', 130, 120, 2.2, '#1f2026', '#f6c27a')], sky=('#f6efe0', '#e9d6b3'), ground='#e9d6b3', caption='you, mid-sentence')
scorecard('workoutcall', 'Workout together on a call', batch=B, palette='red', name='A rep counter for the workout we did on a call', head='Same workout',
          sub='phones propped, no quitting first', rows=['squats', 'push-ups', 'the plank', 'who quit first'], cols=('me', 'you'),
          marks=[('30', '30'), ('12', '20'), ('1:10', '0:50'), ('', 'you')], circle=3, shape='card')
specimen('yogacall', 'Yoga side by side', batch=B, palette='moss', name='A pressed leaf from after our yoga side by side on a call', head='Yoga, side by side',
         rows=['thirty slow minutes', 'then lay there and talked'], item=('lotus',))
scorecard('quizus', 'Write a quiz about yourselves', batch=B, palette='ochre', name='A quiz answer sheet about each other, marked', head='Quiz: about me',
          sub='ten questions, loser plans next', rows=['first pet', 'worst haircut year', 'secret talent', 'the song I hide'], cols=('right?',),
          marks=[('✓',), ('✓',), ('',), ('✓',)], circle=2, shape='clip')
stub('twotruths', 'Two truths and a lie, deep edition', batch=B, palette='ink', name='A lie-detector strip from two truths and a lie, deep edition', head='2 truths, 1 lie',
     sub='DEEP EDITION', rows=['childhood, fears, talents', 'no easy ones', 'explain every one'], band='#2a2238', paper_='#f6efe3', art=[('star', 260, 150, .6, '#e0245e')], stub_text='LIE', shape='torn')
strip('cameraroll', 'Camera roll night', batch=B, palette='plum', name='A contact sheet of photos from a year ago, scrolled on a call', film=True,
      frames=[('#9fc7e0', '#5d7d4f', [('sun', 75, 40, .6)]), ('#f6c27a', '#3f5d7d', [('waves', 75, 80, .8, '#fff')]), ('#5d3564', '#2a1a2a', [('candle', 75, 70, .9)])])
envelope_('firstmessages', 'Read our first messages', batch=B, palette='plum', name='Our first messages, printed and folded into an envelope', front='where we began',
          letter=['hi :)', 'hey! is this the', 'right number?', 'yes. hi.', '(cringe allowed)'], seal='#5d3564', art=[('heart', 180, 136, .4, '#ffffff')])
media('sillysong', 'Write a silly song together', batch=B, palette='ochre', name='A cassette labelled with the silly song we wrote and recorded as a voice note', kind='cassette',
      head='OUR SONG (BAD)', rows=['lyrics: you, tune: me'], shell='#c0392b', paper_='#f2c230', ink='#16161a')
media('duetonline', 'Duet karaoke online', batch=B, palette='plum', name='A record sleeve for the ballad we sang as a duet online', kind='record', head='Ballads only',
      rows=['duet, online', 'track 1: the sad one', 'track 2: the sadder one', 'both of us off-key', 'encore by request'], paper_='#f7c9d4', accent='#5d3564')
lanyard('concertlive', 'Watch a concert together', batch=B, palette='navy', name='A livestream lanyard for the concert we watched together', head='Live, together',
        sub='two screens, one moment', big='ENCORE', card='#e0245e', ink='#fff')
seedpacket('sameseeds', 'Plant the same seeds', batch=B, palette='moss', name='A seed packet for the race between your pot and mine', head='Seed race',
           rows=['same seeds, two cities', 'photo every week', 'mine is winning'], packet='#e3ecd0')
envelope_('bucketlist', 'Write our bucket list', batch=B, palette='kraft', name='A bucket list of fifty things to do in the same city, folded in an envelope', front='50 things, together',
          letter=['1. the same sunrise', '2. a long train', '3. learn to swim', '4. a dog, eventually', '... 46 more'], seal='#7a4a2a')
postcard('streetview', 'Street View holiday', batch=B, palette='sea', name='A postcard from a city we walked together in Street View', scene=[('skyline', 120, 150, 1.6, '#c0392b'), ('dome', 270, 140, 1, '#f1e7d0', '#2f6db5'), ('sun', 60, 60, .7)],
         sky=('#9fc7e0', '#d6e8f0'), ground='#8a8a8a', caption='virtually there', message=['Walked the old town', 'in Street View.', 'Picked our café.', 'One day, for real.'], to='to us, someday')
coin('randomdish', "Cook a random country's dish", batch=B, palette='ochre', name='A globe token for the country the spin landed on', top='spin the globe', big='?', bottom='cook what it lands on',
     back_big='landed!', back_small='rate each other', face='#5aa4d6', rim='#2f6db5', ink='#0f2a4a', art=[('globe', 0, -78, .5)])
keytag('escapeonline', 'Online escape room for two', batch=B, palette='navy', name='A digital escape code on a key fob', head='CODE 4417', number='one reads, one solves', tag_col='#2f6db5')
sticker('minecraft', 'Build a Minecraft world together', batch=B, palette='pine', name='A pixel-block house sticker from the world we built together', head='our house',
        sub='block by block', shape='circle', col='#3b9a5a', art=[('house', 0, -70, .9, '#c47a3a', '#5a3216')])
polaroid('sunrisecall', 'Sunrise call', batch=B, palette='ochre', name='An instant print of the same sunrise from two windows', scene=[('sun', 130, 140, 1.4, '#f6a04d'), ('skyline', 70, 200, 1.2, '#3a2a40'), ('skyline', 200, 202, 1.1, '#4a3a50')],
         sky=('#f7b267', '#fbe3c4'), ground='#3a2a40', caption='then back to sleep')
stub('dressupvideo', 'Dress-up dinner on video', batch=B, palette='night', name='A table reservation card for our dress-up dinner on video', head='Table for 2 screens',
     sub='DRESS CODE: NO PYJAMAS', rows=['candle lit, same meal', 'best clothes, top half', 'and bottom half, as agreed'], band='#22354c', paper_='#f6efe3', art=[('candle', 260, 150, .7)], stub_text='RESERVED', shape='scallop')
badge('teachone', 'Teach each other one thing', batch=B, palette='brick', name='A skill-swap certificate rosette', head='Skill swap', big='learnt',
      sub='a card trick for a recipe', ribbon=('#c0392b', '#e8b923'))
media('sharedplaylist', 'Listen to a shared playlist together', batch=B, palette='lilac', name='A blended playlist sleeve for the songs we played at the same moment', kind='record',
      head='Blend', rows=['our playlist', 'my pick: the old one', 'your pick: the new one', 'played at 9, both of us'], paper_='#c9a0dc', accent='#2b1d33')
envelope_('handwritten', 'A handwritten letter, read aloud', batch=B, palette='kraft', name='A handwritten letter with a stamp, read aloud on a call before posting', front='by post, after reading',
          letter=['My love,', 'I am writing this by', 'hand, so excuse the', 'crossings-out. I will', 'read it to you first.'], seal='#a8321f')
scorecard('sametakeaway', 'Same takeaway, two cities', batch=B, palette='red', name='A two-city takeaway scorecard, each city defended', head='Same dish, two cities',
          sub='each city defends its own', rows=['taste', 'spice', 'portion', 'the winner'], cols=('mine', 'yours'), marks=[('8', '9'), ('9', '7'), ('7', '8'), ('', '✓')], circle=3, shape='card')
scorecard('dailypuzzles', 'Daily puzzles together', batch=B, palette='paper', name="A crossword corner with the day's puzzles scored between us", head="Today's puzzles",
          sub='hints shared, score kept', rows=['the word one', 'the crossword', 'the connections one', 'running total'], cols=('me', 'you'),
          marks=[('4', '3'), ('✓', ''), ('2', '4'), ('12', '11')], circle=3, shape='clip')
polaroid('redecorate', 'Redecorate a corner together', batch=B, palette='moss', name='A before-and-after instant photo of the corner we redecorated on a call',
         scene=[('plant', 80, 170, 1.6), ('lantern', 180, 90, 1.2), ('book', 200, 190, 1)], sky=('#efe6d0', '#e3d7bb'), ground='#c9a87a', caption='after, obviously')
coin('meditate', 'Meditate together', batch=B, palette='lilac', name='A meditation bell token from ten quiet minutes together', top='ten minutes', big='ding',
     bottom='same time, two rooms', back_big='grateful', back_small='one thing each', face='#d9a84a', rim='#9c7a2a', ink='#3a2408', art=[('bell', 0, -78, .5, '#7a5a1a')])
media('podcast', 'Record a podcast about us', batch=B, palette='slate', name='A cassette of our podcast episode, three questions, recorded on a call', kind='cassette',
      head='EP. 1: US', rows=['three questions, one file'], shell='#2b2d33', paper_='#e0245e', ink='#fff')
tag('shop500', 'Shop for each other with ₹500', batch=B, palette='plum', name='A ₹500 gift tag from the surprise we bought each other', head='under ₹500',
    rows=['a surprise, for you', 'unboxed on a call', 'no peeking at the box'], stamp_word='₹499', stamp_sub='just under', paper_='#f7c9d4', ink='#3a1430', ink2='#c2185b', art=[('heart', 236, 40, .5)])
stub('docargue', 'Watch a documentary and argue', batch=B, palette='olive', name='A prediction slip from the documentary we paused halfway to argue about', head='Pause. Predict.',
     sub='HALFWAY THROUGH', rows=['me: it was the brother', 'you: it was the dog', 'it was neither'], band='#364033', paper_='#efe8d6', art=[('clock', 260, 150, .6)], stub_text='BOTH WRONG')
menu('legobuild', 'Build the same LEGO set', batch=B, palette='red', name='A folded instruction page from the LEGO set we built on the same call', head='Step 1 of 64',
     sub='same set, two tables', panels=(['bag 1', 'the base', 'click'], ['bag 2', 'the walls', 'click click'], ['bag 3', 'the roof', 'first done waits']), ticks=[(0, 0), (1, 0), (2, 0)], accent='#c0392b')
coin('wouldyourather', '"Would you rather" marathon', key='"would you rather" marathon', batch=B, palette='plum', name='A would-you-rather spinner token', top='would you rather', big='A or B',
     bottom='explain every answer', back_big='B', back_small='obviously', face='#f48aa6', rim='#c2185b', ink='#3a1428')
label('videocapsule', 'Make a video for us in a year', batch=B, palette='navy', name='A video time-capsule jar labelled open this date next year', kind='jar', head='Open next year',
      rows=['a few minutes each', 'about right now', 'same date, next year'], glass_='#2b3a63', paper_='#f6efe0', fx='sparkle')
media('slowdance', 'Slow dance in the kitchen', batch=B, palette='brick', name='A record for slow dancing in the kitchen with the lights low', kind='record', head='Side A',
      rows=['kitchen, lights low', 'one song each', 'no talk of the dishes'], paper_='#f6efe0', accent='#c0392b')
label('massagenight', 'Massage night', batch=B, palette='lilac', name='A bottle of warm massage oil for massage night', kind='bottle', head='Warm oil', rows=['twenty minutes each', 'a calm playlist', 'who goes first: you'],
      glass_='#c98a4a', paper_='#fbf6ea', fx='bubbles')
scorecard('boardgames', 'Board game marathon', batch=B, palette='moss', name='A score pad from our best-of-five board game marathon', head='Best of five',
          sub='a game we had never played', rows=['game 1', 'game 2', 'game 3', 'game 4', 'game 5'], cols=('me', 'you'),
          marks=[('W', ''), ('', 'W'), ('W', ''), ('', 'W'), ('W', '')], circle=4, shape='clip', art=[('dice', 290, 40, .6)])
strip('polaroidshoot', 'Polaroid photoshoot at home', batch=B, palette='paper', name='A strip of photos from our photoshoot around the house',
      frames=[('#f6c27a', '#c9a87a', [('plant', 75, 70, .8)]), ('#9fc7e0', '#e9d6b3', [('cup', 75, 70, .9)]), ('#f7c9d4', '#e9d6b3', [('heart', 75, 60, .7)])], caption='for the fridge')
meal('breakfastbed', 'Breakfast in bed for each other', batch=B, palette='ochre', name='A breakfast tray with a flower and a note, for breakfast in bed', base='tray', base_col=('#f6efe0', '#c9b48a'),
     items=[('plate', 250, 320, 2.6, '#f6f3ec', '#f2c14e'), ('cup', 420, 270, 1.6), ('flower', 420, 390, 1.2, '#f29ab8', '#c2185b')], tag_head='Breakfast in bed', tag_sub='Saturday you, Sunday me')
sticker('onecanvas', 'Paint one canvas together', batch=B, palette='paper', name='A paint-swatch sticker from the canvas we painted stroke by stroke', head='one canvas',
        sub='two brushes, hung anyway', shape='blob', col='#e46f2a')
stub('bedsheetcinema', 'Bedsheet cinema', batch=B, palette='night', name='A ticket for our bedsheet cinema at home', head='Bedsheet Cinema', sub='SCREEN: ONE SHEET',
     rows=['seats: the floor, cushions', 'popcorn, home-made', 'projector borrowed'], band='#22354c', paper_='#f6efe3', art=[('star', 260, 150, .6, '#f2c230')], stub_text='HOUSEFULL')
meal('pizzascratch', 'Pizza from scratch', batch=B, palette='brick', name='A pizza from scratch, half and half, each half a secret', base='plate', base_col=('#c99c6c', '#a07a4a'),
     items=[('pizza', 300, 320, 4.2)], tag_head='Pizza, from scratch', tag_sub='your half, my half')
tag('phonesdrawer', 'Candlelit dinner, phones in a drawer', batch=B, palette='night', name='A place card for the candlelit dinner with our phones in a drawer', head='Phones: drawer',
    rows=['candlelit, table set', 'something simple', 'till dessert, at least'], stamp_word='OFF', stamp_sub='both phones', paper_='#f6efe3', ink='#22354c', ink2='#a8321f', art=[('candle', 236, 30, .5)])
mapcard('rearrange', 'Rearrange the room together', batch=B, palette='paper', name='A floor plan of our room, the sofa moved, everything else around it', head='The room, version 2',
        route='M80 280 C140 260 200 240 240 180', pins=[(80, 280, 'sofa was'), (240, 180, 'sofa is')], labels=[(330, 300, 'try it a week', 0)], land='#f3efe6')
