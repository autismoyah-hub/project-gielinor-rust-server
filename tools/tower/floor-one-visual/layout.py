"""An original connected Cinderworks location in native X/Z units.

This is an offline authored visual plan. It is not a live map, collision engine
or gameplay system. Coordinates are local to this study (128 units per tile).
"""
AREAS={
 'arrival':dict(name='The Descent',polygon=[[-3100,2680],[-2900,3000],[-2500,2920],[-2220,2580],[-2280,2250],[-2670,2320]],anchor=[-2670,2660],focus=[-2750,2690],story='The stair mouth narrows into a supported service tunnel. A warning bell and supply route lead toward safety.'),
 'sanctuary':dict(name='Bellkeeper Sanctuary',polygon=[[-2470,2460],[-2290,2990],[-1700,3150],[-1040,2780],[-900,2160],[-1390,1700],[-2110,1760]],anchor=[-1690,2370],focus=[-1720,2480],story='A worn bell court, repaired masonry, benches and warm lanterns make a recognizable place of refuge. The workshop flues are visible through its northern threshold.'),
 'workshop':dict(name='The Working Court',polygon=[[-2110,1770],[-1440,1840],[-720,1450],[-570,720],[-900,200],[-1720,190],[-2220,700]],anchor=[-1370,1000],focus=[-1500,790],story='Working benches face an annealing wall. Stock, carts and a paired rail line explain the flow from ore to finished parts.'),
 'waterworks':dict(name='The Lock and Reservoir',polygon=[[-640,1090],[-270,1780],[510,2050],[1410,1710],[1580,1040],[1110,340],[300,190],[-320,490]],anchor=[-45,1190],focus=[180,1070],story='The open reservoir feeds a controlled runoff channel. Pumps, raised service gates and two crossings create an exploration loop between banks.'),
 'quarry':dict(name='The Fracture Gallery',polygon=[[-2290,220],[-1310,230],[-950,-430],[-1110,-1220],[-1740,-1680],[-2610,-1450],[-2910,-650]],anchor=[-1880,-580],focus=[-2200,-870],story='A broken rock face, loaded tubs and a rail curve expose the resource source. The lower service route rejoins the slag channel.'),
 'slag':dict(name='Black Runoff',polygon=[[-930,310],[-150,470],[620,170],[1130,-240],[920,-1160],[400,-1630],[-620,-1450],[-1180,-650]],anchor=[-340,-610],focus=[90,-740],story='Cooled slag, a damaged service bay and the second sluice make the transition toward danger visible. A narrow inspection route follows the water.'),
 'foundry':dict(name='The Last Casting',polygon=[[380,-1450],[1260,-1110],[2060,-1370],[2580,-1960],[2460,-2770],[1860,-3330],[890,-3130],[380,-2450]],anchor=[1210,-2090],focus=[1530,-2380],story='A broad entry opens toward a raised casting ring and an original sluice-crowned creature. Retorts, cradled vessels and return pipes terminate the industrial sequence.')
}
CONNECTORS=[
 [[-2360,2450],[-2110,2510],[-2100,2250],[-2400,2220]],
 [[-1800,1870],[-1450,1870],[-1340,1580],[-1880,1580]],
 [[-840,1190],[-350,1390],[-260,940],[-780,770]],
 [[-1920,390],[-1450,390],[-1420,-80],[-2070,-180]],
 [[-1240,-300],[-810,-260],[-700,-780],[-1320,-890]],
 [[620,620],[1050,420],[1010,-360],[520,-440]],
 [[400,-1250],[930,-1180],[1290,-1600],[430,-1840]]
]
OUTER=[[-3220,3050],[-2630,3370],[-1690,3430],[-850,3030],[-500,2220],[700,2310],[1730,1870],[1860,1050],[1450,260],[1500,-850],[2290,-1250],[2810,-1960],[2670,-2920],[1890,-3570],[660,-3380],[100,-2550],[-200,-1820],[-1690,-1900],[-2840,-1670],[-3160,-700],[-2590,100],[-2510,1510],[-2710,2100],[-3280,2400]]
BRIDGES=[dict(name='reservoir_crossing',center=[300,1010],half_length=355,half_width=112),dict(name='runoff_crossing',center=[395,-720],half_length=355,half_width=112)]
ROUTES=[dict(name='Main descent to the foundry',areas=['arrival','sanctuary','workshop','waterworks','slag','foundry']),dict(name='Ore and exploration loop',areas=['workshop','quarry','slag','waterworks','workshop'])]

def inside(x,z,polygon):
    result=False;j=len(polygon)-1
    for i,(xi,zi)in enumerate(polygon):
        xj,zj=polygon[j]
        if(zi>z)!=(zj>z)and x<(xj-xi)*(z-zi)/(zj-zi)+xi:result=not result
        j=i
    return result

def region(x,z):
    for key,a in AREAS.items():
        if inside(x,z,a['polygon']):return key
    if any(inside(x,z,p)for p in CONNECTORS):return 'connection'
    return None

def channel_center(z):
    if z>=350:return 300
    return 300+max(0,min(1,(350-z)/1350))*180

def water(x,z):
    reservoir=((x-670)/440)**2+((z-1230)/455)**2<1
    return reservoir or(-1640<z<1840 and abs(x-channel_center(z))<132)

def bridge(x,z):return next((b for b in BRIDGES if abs(x-b['center'][0])<b['half_length']and abs(z-b['center'][1])<b['half_width']),None)

def elevation(x,z):
    if region(x,z)=='quarry':return round(max(0,min(1,(-x-1700)/500))*12)
    return 0
