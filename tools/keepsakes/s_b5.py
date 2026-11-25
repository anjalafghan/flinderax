"""Batch 5: slips 123-168 (the days apart, then at home)."""
from objects import *  # noqa: F401,F403
from cutouts import cutout  # noqa: F401
from things import *  # noqa: F401,F403

B = 'b5'

cutout('coffeeacross', 'Coffee date across cities', batch=B, palette='kraft', kind='sticker', name='A sticker of two cutting-chai glasses clinking across two cities',
       parts=[('chai', -40, -20, 3.2), ('chai', 50, -10, 3.0, '#b5702a'), ('star', 4, -110, .9, '#f2c230')], head='same chai, 6 pm', sub='two cities, one sip', ribbon='#243a28', banner=(0, 100, -2))
cutout('nextvisit', 'Plan our next visit, hour by hour', batch=B, palette='paper', kind='magnet', name='A calendar magnet with the next visit circled, hour by hour',
       parts=[('calendar', 0, -20, 3.6, '#f3efe2', '#c0392b'), ('heart', 40, -10, .8, '#c0392b')], head='next visit', sub='hour by hour', ribbon='#3a2a5a', banner=(0, 110, -2))
cutout('dreamhome', 'Design our dream home', batch=B, palette='paper', kind='pin', name='An enamel pin of our dream house with a big kitchen window',
       parts=[('house', 0, -20, 4.0, '#f3efe2', '#2c1f46'), ('plant', 100, 30, 1.2)], head='our dream home', sub='the kitchen: argued over', ribbon='#2c1f46', banner=(0, 100, -2))
cutout('newlanguage', 'Start a language together', batch=B, palette='lilac', kind='sticker', name='A sticker of two speech bubbles saying hello in a new language',
       parts=[('heart', -50, -50, 2.8, '#f7e2a2'), ('globe', 60, -10, 2.4)], head='¡hola! / ciao!', sub='day 12 of the streak', ribbon='#553d66', banner=(0, 90, -2))
cutout('onlineclass', 'Take an online class together', batch=B, palette='navy', kind='sticker', name='A sticker of a laptop with a light bulb over it from the class we took side by side',
       parts=[('laptop', -20, 0, 3.0, '#9aa0a6', '#f6efe0'), ('bulb', 80, -80, 1.6)], head='class of two', sub='certificate: both', ribbon='#1f2b4d', banner=(-10, 80, 3), rot=3)
cutout('paintscreenshot', 'Paint each other from a screenshot', batch=B, palette='lilac', kind='sticker', name='A sticker of a paint palette and brush from painting each other off a call screenshot',
       parts=[('spicetin', -30, -10, 3.0, '#f6efe0', '#e0245e'), ('brush', 70, -60, 2.2, '#c47a3a', '#2f6db5')], head='portrait, from a call', sub='your nose: generous', ribbon='#553d66', banner=(0, 100, -2))
cutout('workoutcall', 'Workout together on a call', batch=B, palette='red', kind='patch', name='A gym patch of a dumbbell from the workout we did together on a call',
       parts=[('dumbbell', 0, -20, 4.0, '#16161a', '#fdeeea')], head='3 sets, on a call', sub='no one skipped', ribbon='#92261e', back=(150, 70, -20, '#fdeeea'), banner=(0, 70, -2))
cutout('yogacall', 'Yoga side by side', batch=B, palette='moss', kind='pin', name='An enamel lotus pin from yoga side by side on a call',
       parts=[('lotus', 0, -20, 4.0, '#efe1c0', '#6a7f5e')], head='side by side', sub='ten breaths, two mats', ribbon='#3e5036', banner=(0, 80, -2))
cutout('quizus', 'Write a quiz about yourselves', batch=B, palette='ochre', kind='magnet', name='A magnet of a big question mark and a pencil from the quiz about each other',
       parts=[('pencil', 60, -10, 2.8), ('note', -50, -30, 2.6, '#1d1206')], head='quiz: about us', sub='you scored 9 of 10', ribbon='#a86a2c', banner=(0, 100, -2))
cutout('twotruths', 'Two truths and a lie, deep edition', batch=B, palette='ink', kind='charm', name='A playing-card charm for two truths and a lie, deep edition',
       parts=[('card', -24, -20, 2.8, '#ece6f2', '#241c30'), ('card', 30, -10, 2.8, '#fbf8f1', '#c0392b')], head='spot the lie', sub='deep edition', ribbon='#362a45', banner=(0, 100, -2))
strip('cameraroll', 'Camera roll night', batch=B, palette='plum', name='A contact sheet of photos from a year ago, scrolled on a call', film=True,
      frames=[('#9fc7e0', '#5d7d4f', [('sun', 75, 40, .6)]), ('#f6c27a', '#3f5d7d', [('waves', 75, 80, .8, '#fff')]), ('#5d3564', '#2a1a2a', [('candle', 75, 70, .9)])])
cutout('firstmessages', 'Read our first messages', batch=B, palette='plum', kind='sticker', name='A sticker of a phone with our first chat bubbles, printed out',
       parts=[('phone', -20, -20, 3.4, '#43244a', '#fbf4f8'), ('heart', 70, -90, 1.4, '#ffd5e6')], head='“hey :)”', sub='the very first one', ribbon='#5d3564', banner=(0, 100, -2))
cutout('sillysong', 'Write a silly song together', batch=B, palette='ochre', kind='sticker', name='A sticker of a ukulele and three notes from the silly song we wrote',
       parts=[('guitar', -20, -20, 3.2, '#f2c230', '#1d1206'), ('note', 70, -60, 1.6, '#1d1206'), ('note', 90, 10, 1.2, '#1d1206')], head='the silly song', sub='verse 2 is the worst', ribbon='#a86a2c', banner=(0, 100, -2))
cutout('duetonline', 'Duet karaoke online', batch=B, palette='plum', kind='pin', name='An enamel pin of two microphones crossed for a duet sung online',
       parts=[('mic', -36, -20, 3.0, '#f7b9cf', '#3a1430'), ('mic', 36, -20, 3.0, '#ffd5e6', '#3a1430')], head='the duet', sub='you took the high part', ribbon='#5d3564', banner=(0, 110, -2))
cutout('concertlive', 'Watch a concert together', batch=B, palette='ink', kind='magnet', name='A magnet of a stage of lights and a TV screen: a concert watched together',
       parts=[('tv', 0, -10, 3.6, '#241c30', '#f2c230'), ('lights', 0, -110, 2.6, '#f2c230', '#ece6f2')], head='live, together', sub='front row, on the sofa', ribbon='#362a45', banner=(0, 100, -2))
seedpacket('sameseeds', 'Plant the same seeds', batch=B, palette='moss', name='A seed packet for the race between your pot and mine', head='Seed race',
           rows=['same seeds, two cities', 'photo every week', 'mine is winning'], packet='#e3ecd0')
cutout('bucketlist', 'Write our bucket list', batch=B, palette='sea', kind='sticker', name='A sticker of a little bucket spilling fifty folded wishes and stars',
       parts=[('pot', 0, 10, 3.0, '#5d97ac', '#326174'), ('star', -60, -70, 1.4, '#fbe7a8'), ('heart', 50, -90, 1.2, '#fbe7a8'), ('star', 10, -130, .9, '#fbe7a8')], head='bucket list: 50', sub='same city, someday', ribbon='#326174', banner=(0, 110, -2))
cutout('streetview', 'Street View holiday', batch=B, palette='sea', kind='magnet', name='A magnet of a map pin dropped on a city skyline we walked in Street View',
       parts=[('skyline', 0, 10, 3.6, '#326174'), ('mappin', 40, -90, 2.0, '#f2c230', '#1d2b30')], head='Street View holiday', sub='we walked Lisbon', ribbon='#1d2b30', banner=(0, 100, -2))
cutout('randomdish', "Cook a random country's dish", batch=B, palette='olive', kind='magnet', name='A magnet of a spun globe and a frying pan for the random country we cooked',
       parts=[('wheel', -50, -30, 2.6), ('handi', 70, 10, 2.0, '#2b2b30', '#e0a64a')], head='spin, then cook', sub='it landed on Peru', ribbon='#364033', banner=(0, 100, -2))
cutout('escapeonline', 'Online escape room for two', batch=B, palette='ink', kind='pin', name='An enamel padlock-and-key pin from the online escape room for two',
       parts=[('bell', 0, -10, 3.4, '#c9a14a'), ('anchor', 90, -60, 1.0, '#c9a14a')], head='escaped: online', sub='code: 4-1-7', ribbon='#362a45', banner=(0, 90, -2))
cutout('minecraft', 'Build a Minecraft world together', batch=B, palette='moss', kind='sticker', name='A blocky sticker of the pixel house we built together',
       parts=[('brick', -40, 20, 2.6, '#6a7f5e'), ('brick', 40, 20, 2.6, '#8a5a2e'), ('brick', 0, -40, 2.6, '#c0392b')], head='our world', sub='seed: 0214', ribbon='#3e5036', banner=(0, 100, -2))
cutout('sunrisecall', 'Sunrise call', batch=B, palette='ochre', kind='patch', name='A sunrise patch: one sun rising over two windows',
       parts=[('sun', 0, -30, 4.0, '#f2b33d'), ('hills', 0, 30, 3.6, '#a86a2c', '#843b1f')], head='5:58 am', sub='the same sunrise', ribbon='#843b1f', banner=(0, 100, -2))
cutout('dressupvideo', 'Dress-up dinner on video', batch=B, palette='plum', kind='charm', name='A wine-glass charm from the dress-up dinner on video',
       parts=[('wine', 0, -20, 3.8, '#7a1a2e', '#fbf4f8')], head='table for two', sub='dress code: fancy', ribbon='#43244a', banner=(0, 90, -2))
cutout('teachone', 'Teach each other one thing', batch=B, palette='brick', kind='pin', name='An enamel pin of a light bulb: one thing each taught the other',
       parts=[('bulb', 0, -20, 3.6), ('star', 70, -80, .8, '#f6e2b3')], head='skill swap', sub='knots for chords', ribbon='#843b1f', banner=(0, 90, -2))
cutout('sharedplaylist', 'Listen to a shared playlist together', batch=B, palette='lilac', kind='sticker', name='A sticker of a record with a split earphone cable: one playlist at the same moment',
       parts=[('record', -30, -10, 3.0, '#553d66', '#f7e2a2'), ('headphones', 80, -60, 1.6, '#2b1d33', '#f7e2a2')], head='press play: now', sub='the blended list', ribbon='#553d66', banner=(0, 100, -2))
cutout('handwritten', 'A handwritten letter, read aloud', batch=B, palette='brick', kind='magnet', name='A magnet of a fountain pen over a stamped envelope, read aloud before posting',
       parts=[('envelope', 0, 0, 3.6, '#fbeee3', '#a8321f'), ('pencil', 70, -70, 2.0, '#2a3f7a', '#2b2b30')], head='read aloud, then posted', sub='postmark: Tuesday', ribbon='#843b1f', banner=(0, 100, -2))
cutout('sametakeaway', 'Same takeaway, two cities', batch=B, palette='brick', kind='sticker', name='A sticker of two takeaway boxes with chopsticks, one in each city',
       parts=[('cup', -50, -20, 3.0, '#fbeee3', '#c0392b'), ('cup', 60, -10, 2.6, '#f6e2b3', '#843b1f')], head='same order, 2 cities', sub='yours was better', ribbon='#843b1f', banner=(0, 100, -2))
cutout('dailypuzzles', 'Daily puzzles together', batch=B, palette='paper', kind='magnet', name='A magnet of a pencil across a crossword square from the daily puzzles we race through',
       parts=[('dice', -30, 0, 3.0, '#f3efe2', '#2a2238'), ('pencil', 70, -40, 2.6)], head='daily puzzles', sub='streak: 41 days', ribbon='#3a2a5a', banner=(0, 100, 4), rot=6)
cutout('redecorate', 'Redecorate a corner together', batch=B, palette='moss', kind='magnet', name='A magnet of a lamp and a plant in the corner we redecorated',
       parts=[('plant', -40, 0, 3.0), ('lantern', 70, -40, 2.0, '#efe1c0', '#2b2b20')], head='our new corner', sub='before / after', ribbon='#3e5036', banner=(0, 100, -2))
cutout('meditate', 'Meditate together', batch=B, palette='sea', kind='pin', name='An enamel singing-bowl pin from ten quiet minutes together',
       parts=[('handi', 0, -10, 3.4, '#c9a14a', '#9a6e2a'), ('mist', 0, -80, 1.6)], head='ten quiet minutes', sub='one bell, two breaths', ribbon='#326174', banner=(0, 70, -2))
cutout('podcast', 'Record a podcast about us', batch=B, palette='slate', kind='sticker', name='A sticker of a studio microphone on its stand: our podcast, episode one',
       parts=[('mic', 0, -30, 4.0, '#9aa0a6', '#16161a'), ('note', 90, -90, 1.2, '#f2c230')], head='episode 1: us', sub='three questions', ribbon='#26292e', banner=(0, 120, -2))
cutout('shop500', 'Shop for each other with ₹500', batch=B, palette='olive', kind='charm', name='A shopping-bag charm with a ₹500 tag for the surprise we bought each other',
       parts=[('parcel', 0, -20, 3.2, '#efe8d6', '#c0392b'), ('tag', 70, 30, 1.4)], head='₹500 each', sub='a surprise, under budget', ribbon='#364033', banner=(0, 100, -2))
cutout('docargue', 'Watch a documentary and argue', batch=B, palette='slate', kind='pin', name='An enamel lion pin from the wildlife documentary we paused halfway to argue about',
       parts=[('lion', 0, -20, 3.6)], head='paused at 47:10', sub='we both won', ribbon='#26292e', banner=(0, 90, 3), rot=2)
cutout('legobuild', 'Build the same LEGO set', batch=B, palette='red', kind='sticker', name='A sticker of two stacked toy bricks from the set we built on the same call',
       parts=[('brick', 0, 10, 3.2, '#f2c230'), ('brick', 0, -50, 3.2, '#16161a')], head='same set, same call', sub='step 41: one piece short', ribbon='#92261e', banner=(0, 100, -2))
cutout('wouldyourather', '"Would you rather" marathon', batch=B, palette='lilac', kind='charm', name='A spinner-wheel charm from the would-you-rather marathon',
       parts=[('wheel', 0, -20, 3.4)], head='would you rather…', sub='fly, obviously', ribbon='#553d66', banner=(0, 90, -2), key='"would you rather" marathon')
cutout('videocapsule', 'Make a video for us in a year', batch=B, palette='kraft', kind='pin', name='An enamel pin of a camera and a calendar page: a video to open next year',
       parts=[('camera', -30, -20, 2.8, '#2f4a33', '#f3ead8'), ('calendar', 80, -60, 1.4, '#f3ead8', '#c0392b')], head='open: next year', sub='video capsule', ribbon='#243a28', banner=(0, 90, -2))
cutout('slowdance', 'Slow dance in the kitchen', batch=B, palette='plum', kind='magnet', name='A fridge magnet of a record spinning by a hanging bulb, kitchen lights low',
       parts=[('record', -20, -10, 3.2, '#3a1430', '#f7b9cf'), ('bulb', 90, -80, 1.4, '#ffd5e6')], head='slow dance', sub='kitchen, lights low', ribbon='#43244a', banner=(0, 110, -2))
cutout('massagenight', 'Massage night', batch=B, palette='sand', kind='sticker', name='A sticker of a warm oil drop, a candle and a eucalyptus sprig for massage night',
       parts=[('drop', -40, -30, 3.4, '#e0a64a'), ('olive', 70, 0, 2.0, '#8f9a6a', '#5b6a3a'), ('candle', 40, -80, 1.2)], head='massage night', sub='warm oil, slow hands', ribbon='#5b3a1f', banner=(0, 90, -2))
cutout('boardgames', 'Board game marathon', batch=B, palette='olive', kind='patch', name='A patch of a pawn and two dice from our best-of-five board game marathon',
       parts=[('pawn', -40, -20, 3.4, '#364033'), ('dice', 60, 0, 2.0)], head='best of five', sub='3 to 2, rematch owed', ribbon='#364033', back=(140, 80, -10, '#efe8d6'), banner=(0, 100, -2))
cutout('polaroidshoot', 'Polaroid photoshoot at home', batch=B, palette='ochre', kind='charm', name='An instant-camera charm from our photoshoot around the house',
       parts=[('camera', 0, -20, 3.6, '#f4ecd8', '#1e6a66')], head='photoshoot at home', sub='10 shots, 2 good', ribbon='#1e6a66', banner=(0, 80, -2))
cutout('breakfastbed', 'Breakfast in bed for each other', batch=B, palette='sand', kind='magnet', name='A magnet of a breakfast tray: eggs, chai and a flower, for breakfast in bed',
       parts=[('plate', -30, 0, 2.8, '#f6f3ec', '#f2c230'), ('chai', 90, -20, 1.6), ('flower', -90, -60, 1.2, '#d6334a', '#f2c230')], head='breakfast in bed', sub='crumbs everywhere', ribbon='#5b3a1f', back=(160, 50, 10, '#d9b27a'), banner=(0, 90, -2))
cutout('onecanvas', 'Paint one canvas together', batch=B, palette='brick', kind='pin', name='An enamel pin of two brushes crossed over one canvas painted stroke by stroke',
       parts=[('brush', -30, -20, 2.8, '#c47a3a', '#2f6db5'), ('brush', 30, -20, 2.8, '#c47a3a', '#e0245e')], head='one canvas', sub='stroke by stroke', ribbon='#843b1f', banner=(0, 100, -2))
cutout('bedsheetcinema', 'Bedsheet cinema', batch=B, palette='night', kind='sticker', name='A sticker of a torch-lit bedsheet screen and a bowl of popcorn',
       parts=[('shirt', 0, -40, 3.4, '#e9f0f7', '#e9f0f7'), ('cup', 80, 30, 1.6, '#f6f3ec', '#f2c230')], head='bedsheet cinema', sub='tonight: our pick', ribbon='#22354c', banner=(0, 100, -2))
cutout('pizzascratch', 'Pizza from scratch', batch=B, palette='red', kind='charm', name='A pizza charm, half and half, each half a secret',
       parts=[('pizza', 0, -20, 4.0)], head='half and half', sub='each half a secret', ribbon='#92261e', banner=(0, 100, -2))
cutout('phonesdrawer', 'Candlelit dinner, phones in a drawer', batch=B, palette='plum', kind='magnet', name='A magnet of two candles and a drawer shut on our phones',
       parts=[('candle', -40, -40, 2.6), ('candle', 20, -50, 2.8), ('phone', 90, 20, 1.2, '#43244a', '#2b2b30')], head='phones: in a drawer', sub='candles: lit', ribbon='#43244a', back=(130, 40, 30, '#7a4a80'), banner=(0, 100, -2))
tape_measure('rearrange', 'Rearrange the room together', batch=B, palette='kraft', name='A tape measure pulled out to where the sofa went when we rearranged the room')
