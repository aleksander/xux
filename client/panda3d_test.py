import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
import random

tiles = loader.loadModel("2.egg")
tile1 = tiles.find('**/grass')
tile2 = tiles.find('**/Plane')
tile3 = tiles.find('**/Plane_001')
tile4 = tiles.find('**/Plane_002')
#tiles.ls()
tile1.setPos(.0,.0,.0)
tile2.setPos(.0,.0,.0)
tile3.setPos(.0,.0,.0)
tile4.setPos(.0,.0,.0)
tiles = [tile1,tile2,tile3,tile4]
terrain = NodePath('terrain')
terrain.reparentTo(render)
for x in xrange(-50,50):
	for y in xrange(-50,50):
		tile = terrain.attachNewNode('tile')
		tile.setPos(x,y,0)
		random.choice(tiles).instanceTo(tile)
		# tile = random.choice(tiles)
		# tile = tile.copyTo(terrain)
		# tile.setPos(x*1.1,y*1.1,0)
		#terrain.flattenStrong()

base.disableMouse()
camera.setPos(200,200,200)
camera.lookAt(terrain)
base.enableMouse()

run()
