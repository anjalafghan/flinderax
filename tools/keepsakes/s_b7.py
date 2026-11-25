"""Batch 7: slips 215-262 (restaurants, bars and the city)."""
from objects import *  # noqa: F401,F403
from cutouts import cutout  # noqa: F401
from things import *  # noqa: F401,F403

B = 'b7'

meal('trishna', 'Butter garlic crab at Trishna', batch=B, palette='brick', name='A butter garlic crab with a cracker on a plate, from Trishna in Kala Ghoda', base='plate',
     items=[('crab', 300, 310, 3.2, '#d8483a'), ('spoon', 470, 320, 1.6)], tag_head='Trishna, Kala Ghoda', tag_sub='butter garlic crab, prawns, rice')
meal('bademiya', 'Late-night kebabs at Bademiya', batch=B, palette='ink', name='Seekh kebabs and rumali roti on paper from the Bademiya grill, late', base='paper', base_col=('#f3e3c0', '#c9a87a'),
     items=[('kebab', 290, 270, 3), ('kebab', 300, 330, 2.8, '#7a3e1a'), ('chili', 440, 400, 1.2), ('moon', 160, 220, .9)], tag_head='Bademiya, Tulloch Road', tag_sub='behind the Taj, after midnight')
coaster('leopold', 'Old Colaba dinner at Leopold Cafe', batch=B, palette='red', name='A beer mat from Leopold Cafe on Colaba Causeway', head='Leopold', sub='Colaba Causeway, since 1871',
        shape='scallop', board_='#f2c230', ink='#7a1a16', drink='glass', drink_col=('#f2b33d', '#ffffff'))
menu('jimmyboy', 'Parsi classics at Jimmy Boy', batch=B, palette='ochre', name="A folded Jimmy Boy menu with dhansak and caramel custard ticked", head='Jimmy Boy',
     sub='Bank Street, Fort', panels=(['dhansak', 'patra ni machhi', 'salli boti'], ['raspberry soda', 'lime soda', ''], ['caramel custard', 'lagan nu custard', '']), ticks=[(0, 0), (0, 1), (2, 0)], accent='#2f6d57')
cuppa('chenab', 'Desi comfort plates at Chenab, Vashi', batch=B, palette='ochre', name='A mocktail with a paper umbrella from Chenab in Vashi', kind='cutting', head='Chenab, Vashi',
      sub='mocktails, open-air seats', drink='#e0245e', verbs=('Stir it', 'Set it down'))
coaster('skylounge', 'Cocktails with city lights at Sky Lounge', batch=B, palette='night', name='A cocktail coaster from Sky Lounge over the Vashi lights', head='Sky Lounge',
        sub='Palm Beach Galleria, Vashi', shape='round', board_='#1f2b4d', ink='#f2d38c', drink='cocktail', drink_col=('#e0245e', '#ffffff'), art=[('lights', 0, 70, 1)])
stub('nazaara', 'Slow rooftop dinner at Nazaara, Turbhe', batch=B, palette='night', name='A rooftop reservation card from Nazaara in Turbhe', head='Nazaara rooftop',
     sub='TURBHE, NAVI MUMBAI', rows=['table for two, rooftop', 'the lights of Navi Mumbai', 'a long, slow dinner'], band='#22354c', paper_='#f6efe3', art=[('lights', 250, 150, .8)], stub_text='ROOFTOP', shape='scallop')
coaster('rudelounge', 'Powai views at Rude Lounge', batch=B, palette='slate', name='A coaster from Rude Lounge on the eighth floor over Powai', head='Rude Lounge',
        sub='8th floor, Powai', shape='square', board_='#2b2d33', ink='#f2c230', drink='glass', drink_col=('#f2b33d', '#ffffff'))
label('fratelli', 'Lake-side Italian at Fratelli Fresh', batch=B, palette='sea', name='A bottle of sparkling wine from the lake-facing table at Fratelli Fresh', kind='bottle',
      head='Fratelli Fresh', rows=['Powai, lake side', 'pasta to share', 'one bottle, fizzy'], glass_='#2f6d57', paper_='#f6efe0', fx='bubbles')
media('pianoman', 'Late jazz set at Piano Man, Khar', batch=B, palette='ink', name='A record for the late jazz set at Piano Man in Khar', kind='record', head='Piano Man',
      rows=['Khar West, late set', 'a table by the stage', 'one more song', 'the bassist, wow'], paper_='#1b1b1e', ink='#f2c230', accent='#c8312b')
media('bluebop', 'Jazz nights at BlueBop Cafe, Khar', batch=B, palette='navy', name='A blue note record for jazz night at BlueBop Cafe', kind='record', head='BlueBop',
      rows=['318 Linking Road', 'house cocktails', 'near the band', 'blue notes all night'], paper_='#2f6db5', ink='#fff', accent='#f2c230')
handstamp('bonobo', 'Wednesday live sets at Bonobo', batch=B, palette='pine', name='The club stamp on a hand from the Wednesday live sets at Bonobo', stamp_text='BONOBO',
          sub='Wednesday, 10:30 pm', ink='#7dffb2')
sticker('localstar', 'Indie gig at Local Star, Versova', batch=B, palette='ink', name="An indie band sticker from a gig at Local Star in Versova", head='Local Star',
        sub='JP Road, Versova, live', shape='star', col='#f2c230', ink='#16161a')
stub('thehouse', 'North Indian plates and live music at The House', batch=B, palette='brick', name='A table tent ticket from The House in Versova, live set', head='The House',
     sub='JP ROAD, ARAM NAGAR', rows=['North Indian plates', 'a round of drinks', 'stayed for the set'], band='#7a1a16', paper_='#f6eddc', art=[('guitar', 260, 150, .6)], stub_text='LIVE')
scorecard('hymus', 'Karaoke duets at Hymus, Lokhandwala', batch=B, palette='plum', name='A duet lyric card from karaoke night at Hymus', head='Karaoke at Hymus',
          sub='one duet, sung badly', rows=['the duet', 'the encore', 'the dessert after'], cols=('me', 'you'), marks=[('6', '7'), ('9', '9'), ('✓', '✓')], circle=1, shape='clip', art=[('mic', 290, 40, .6)])
lanyard('18thcentury', 'Rooftop live music at 18th Century', batch=B, palette='night', name='A string of fairy lights from the rooftop at 18th Century, Lokhandwala', head='18th Century',
        sub='rooftop, Lokhandwala Road', big='live', card='#22354c', ink='#ffe08a', art=[('lights', 0, 280, 1.2)])
badge('90ft', 'Rooftop views at 90 Ft Above, Borivali', batch=B, palette='olive', name='A height-marker plaque from 90 Ft Above in Borivali', head='90 Ft Above',
      big='90 ft', sub='Borivali West, outdoors', ribbon=('#2f6d57', '#c9a14a'), scallop=False, face='#c9a14a', ink='#1d211b')
meal('cafemadras', 'Dosa breakfast at Cafe Madras, Matunga', batch=B, palette='moss', name='A masala dosa and filter coffee in its davara at Cafe Madras', base='leaf', base_col=('#3f7d2c', '#a8cf7a'),
     items=[('dosa', 280, 300, 3.4), ('tumbler', 450, 360, 2)], tag_head='Cafe Madras, Matunga', tag_sub='masala dosa, filter coffee, early')
coin('sardar', 'Pav bhaji at Sardar, Tardeo', batch=B, palette='brick', name='A tava token from Sardar in Tardeo for butter pav bhaji', top='Sardar, Tardeo', big='AMUL',
     bottom='butter pav bhaji', back_big='+ chai', back_small='at the counter', face='#f2c230', rim='#c0392b', ink='#7a1a16', art=[('plate', 0, -78, .5, '#f6f3ec', '#d8641e')])
meal('chowpattybhel', 'Bhel by the Chowpatty carts', batch=B, palette='sea', name='A paper cone of bhel from the Chowpatty carts at sunset', base='paper', base_col=('#f3e3c0', '#c9a87a'),
     items=[('plate', 290, 310, 3, '#f3e3c0', '#d9a85c'), ('sun', 160, 220, 1.1, '#f6a04d'), ('waves', 300, 450, 1.6, '#5aa4d6'), ('chili', 440, 300, 1)], tag_head='Chowpatty, cart no. 7', tag_sub='bhel, at sunset')
coaster('grilltonic', 'Rooftop bar at Grill and Tonic, Thane', batch=B, palette='moss', name='A gin and tonic coaster from Grill and Tonic in Thane', head='Grill &amp; Tonic',
        sub='Thane, towards the hills', shape='round', board_='#e3ecd0', ink='#24401f', drink='glass', drink_col=('#d6f0e0', '#ffffff'), art=[('hills', 0, 70, .8)])
stub('mitron', 'Rooftop dinner at Mitron, Ghodbunder Road', batch=B, palette='night', name='A table number ticket from the Mitron rooftop on Ghodbunder Road', head='Mitron, table 9',
     sub='ROYAL PLAZA ROOFTOP', rows=['open-air, Ghodbunder Rd', 'starters to share', 'then the slow walk'], band='#22354c', paper_='#f6efe3', art=[('star', 260, 150, .6, '#f2c230')], stub_text='TABLE 9')
meal('chulha', 'Chulha Village on Ghodbunder Road', batch=B, palette='clay', name='A clay chulha meal and chai from Chulha Village', base='thali',
     items=[('handi', 230, 300, 1.8, '#a65a30', '#e9c46a'), ('plate', 380, 280, 1.6, '#f6f3ec', '#d9963a'), ('cup', 320, 410, 1.6, '#f6f3ec', '#8a5a2e')], tag_head='Chulha Village', tag_sub='regional plates, then chai')
specimen('idealcorner', 'Weekday lunch at Ideal Corner, Fort', batch=B, palette='paper', name="The day's special chalked on a slate from Ideal Corner", head="Ideal Corner, today",
         rows=["the day's special", 'Gunbow Street, weekdays'], item=('plate', '#f6f3ec', '#d8641e'), card='#2b2d33', ink='#f6f3ec')
stub('ncpaexp', "Watch a play at NCPA's Experimental Theatre", key="watch a play at ncpa's experimental theatre", batch=B, palette='ink', name='A seat stub from the Experimental Theatre at NCPA',
     head='NCPA Experimental', sub='NARIMAN POINT, 7 PM', rows=['seats in the round', 'a play we had not heard of', 'stood for the bow'], band='#2a2238', paper_='#f6efe3', stub_text='ROW B', shape='torn')
sticker('ngma', 'Slow afternoon at the National Gallery of Modern Art', batch=B, palette='paper', name='A modern-art entry sticker from the NGMA in Fort', head='NGMA',
        sub='Jehangir Hall, Fort', shape='circle', col='#e46f2a')
menu('csmvs', 'Wander the Chhatrapati Shivaji Maharaj museum', batch=B, palette='ochre', name='A folded gallery guide from the Chhatrapati Shivaji Maharaj Vastu Sangrahalaya', head='CSMVS',
     sub='Kala Ghoda, the dome', panels=(['Indian art', 'miniatures', 'sculpture'], ['natural history', 'the stuffed tiger', ''], ['the café', 'the garden', '']), ticks=[(0, 1), (1, 1), (2, 1)], accent='#2f6d57')
badge('sciencecentre', 'Hands-on science afternoon in Worli', batch=B, palette='sea', name='A science badge from the Nehru Science Centre in Worli', head='Science Centre',
      big='EUREKA', sub='Worli, hands on', ribbon=('#2f6db5', '#e8b923'), art=[('stars', 0, 86, .7, '#2f6db5')])
specimen('hornbill', 'Birds and beasts at Hornbill House', batch=B, palette='moss', name='A naturalist specimen label from Hornbill House, Bombay Natural History Society', head='BNHS, Hornbill House',
         rows=['opposite Lion Gate, Fort', 'the hornbill, of course'], item=('bird', '#2a2a2a'))
tag('siddhivinayak', 'Darshan at Siddhivinayak Temple', batch=B, palette='ochre', name='A prasad wrapper tag from darshan at Siddhivinayak Temple', head='Siddhivinayak',
    rows=['Prabhadevi, early morning', 'not a Tuesday', 'modaks for home'], stamp_word='PRASAD', stamp_sub='Prabhadevi', paper_='#f6d36b', ink='#5a1d0c', ink2='#c0392b', art=[('marigold', 236, 40, .5)])
label('mountmary', 'Sunday Mass at Mount Mary, Bandra', batch=B, palette='sea', name='A votive candle from Sunday Mass at Mount Mary Basilica', kind='jar', head='Mount Mary',
      rows=['Sunday, 7:30 am', 'up the steps', 'the sea below'], glass_='#f6efe0', paper_='#2f6db5', ink='#ffffff', fx='sparkle', art=[('candle', 0, -110, 1.4)])
tag('dhobighat', 'Dhobi Ghat from the bridge at sunrise', batch=B, palette='sea', name='A laundry tag from watching Dhobi Ghat at sunrise from the bridge', head='Mahalaxmi Dhobi Ghat',
    rows=['from the bridge, sunrise', 'washermen at work', 'photos from afar'], stamp_word='LOT 418', stamp_sub='washed', paper_='#f6efe0', ink2='#2f6db5')
seedpacket('crawford', 'Browse the stalls at Crawford Market', batch=B, palette='brick', name='A spice packet from the stalls at Crawford Market', head='Crawford Market',
           rows=['garam masala, the good one', 'under the Victorian roof', 'haggled, a bit'], packet='#c0392b', ink='#fff6ea', seed_col='#7a3a1a', art=(('chili', 0, -40, 1.6, '#f2c230'),))
tag('dadarflowers', 'Dadar flower market at dawn', batch=B, palette='ochre', name='A marigold garland tag from the Dadar flower market at dawn', head='Dadar Phool Bazaar',
    rows=['by the station, dawn', 'marigolds by the kilo', 'one garland home'], stamp_word='FRESH', stamp_sub='5 am', paper_='#f6efe0', ink2='#e46f2a', art=[('marigold', 236, 40, .6)])
stub('lionsafari', 'Lion and tiger safari in Sanjay Gandhi park', batch=B, palette='moss', name='A safari bus ticket from the lion and tiger safari in SGNP', head='Lion Safari',
     sub='SGNP, BORIVALI GATE', rows=['the 9 am slot', 'the bus with the cage', 'saw one (sleeping)'], band='#2f6d57', paper_='#eef2e6', stub_text='SAFARI', shape='torn')
postcard('tungareshwar', 'Trek to the Tungareshwar waterfall', batch=B, palette='ghat', name='A monsoon postcard of the Tungareshwar waterfall', scene=[('waterfall', 180, 130, 2.4, '#bfe3f2', '#4f7a45'), ('mist', 80, 80, 1), ('bird', 290, 50, .6)],
         sky=('#b8c9b4', '#dfe6d4'), ground='#3f553b', caption='Tungareshwar', message=['Weekday, monsoon.', 'Shared rick from', 'Vasai. The falls', 'at full strength.'], to='to us, Vasai')
specimen('yeoor', 'Forest walk at Yeoor Hills', batch=B, palette='moss', name='A pressed leaf and a bird tally from the Yeoor Hills trails', head='Yeoor Hills', rows=['Thane, early', 'birds heard: many'], item=('leaf', '#3f7d2c'))
badge('parsik', 'Sunrise climb up Parsik Hill', batch=B, palette='ochre', name='A sunrise summit badge from Parsik Hill', head='Parsik Hill',
      big='sunrise', sub='Kalwa side, over the creek', ribbon=('#f6a04d', '#2f6db5'), art=[('sun', 0, 86, .8, '#f6a04d')])
specimen('mahimpark', 'Stroll through Mahim Nature Park', batch=B, palette='moss', name='A ficus leaf pressing from the Mahim Nature Park trail', head='Mahim Nature Park', rows=['the ficus trail', 'towards the mangroves'], item=('leaf', '#5a8a3a'))
tag('trampoline', 'An hour of bouncing in Ghatkopar', batch=B, palette='red', name='A trampoline sock tag from an hour of bouncing in Ghatkopar', head='Let Inner Child Out',
    rows=['Ghatkopar, one hour', 'grip socks: mandatory', 'rest after: mandatory'], stamp_word='BOUNCED', stamp_sub='60 min', paper_='#f2c230', ink='#16161a', ink2='#c0392b')
tag('bouldering', 'Try bouldering in Nerul', batch=B, palette='slate', name='A chalk bag tag from beginner bouldering in Nerul', head='Boulder 21',
    rows=['Girivihar wall, Nerul', 'the yellow route', 'chalk everywhere'], stamp_word='TOPPED', stamp_sub='V0 (yellow)', paper_='#efe8d6', ink2='#2f6db5')
postcard('artdeco', 'Art Deco walk along Marine Drive', batch=B, palette='sea', name='An Art Deco facade postcard from the walk along Marine Drive', scene=[('skyline', 120, 150, 1.6, '#f1e7d0'), ('skyline', 260, 152, 1.5, '#e9d6b3'), ('sun', 300, 60, .6, '#f6a04d')],
         sky=('#9fd3e0', '#f6e1c4'), ground='#5a8fa0', caption='Art Deco, Churchgate', message=['Curved balconies,', 'sunburst gates,', 'the Eros at the', 'corner. Walked slow.'], to='to us')
lanyard('wankhede', 'Museum and stadium tour at the Wankhede', batch=B, palette='sea', name='A stadium tour pass from the Wankhede', head='Wankhede tour', sub='Churchgate, booked online',
        big='SIX!', card='#2f6db5', ink='#fff', art=[('cricket', 0, 280, .9)])
stub('gateway', 'Gateway of India at dusk', batch=B, palette='sea', name='A harbour boat ticket from the Gateway of India at dusk', head='Gateway of India',
     sub='HARBOUR RIDE, DUSK', rows=['the arch, the light going', 'boats bobbing', 'the Taj across'], band='#2f6db5', paper_='#eef4f6', art=[('boat', 250, 155, .6)], stub_text='HARBOUR')
postcard('sewrifort', 'Climb Sewri Fort above the mudflats', batch=B, palette='sea', name='A field-sketch postcard from Sewri Fort over the mudflats', scene=[('fort', 110, 140, 1.6, '#a08060', '#7a5c40'), ('flamingo', 260, 170, .8), ('flamingo', 300, 176, .7)],
         sky=('#f6d8b0', '#f3e1c4'), ground='#8a7a68', caption='Sewri mudflats', message=['Late afternoon,', 'soft light, the', 'harbour and a line', 'of flamingos.'], to='to us')
stub('regal', 'Matinee at the Regal Cinema', batch=B, palette='red', name='A single-screen matinee ticket from the Regal Cinema in Colaba', head='REGAL',
     sub='COLABA, MATINEE', rows=['balcony, row F', 'Art Deco, 1933', 'an old film, a full house'], band='#7a1a16', paper_='#f6d9a0', ink='#3a1a0a', stub_text='BALCONY', shape='scallop')
postcard('upvan', 'Upvan lake at sunset', batch=B, palette='ghat', name='A ripple postcard of Upvan lake at sunset', scene=[('hills', 120, 140, 1.8, '#5d7d4f', '#3f5d36'), ('sun', 260, 110, .9, '#f6a04d'), ('waves', 180, 196, 1.4, '#ffffff')],
         sky=('#f7b267', '#f4845f'), ground='#5a8fa0', caption='Upvan, Thane', message=['Local to Thane,', 'walked up to the', 'lake. The hills', 'went orange.'], to='to us')
handstamp('kalaghodafest', 'A day at the Kala Ghoda Arts Festival', batch=B, palette='ochre', name='A Kala Ghoda Arts Festival stamp on a hand', stamp_text='KGAF',
          sub='February, nine days', ink='#e0245e')
