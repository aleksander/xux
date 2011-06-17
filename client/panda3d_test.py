import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
import random

tiles = loader.loadModel("models.egg")
tile1 = tiles.find('**/tile1')
tile2 = tiles.find('**/tile2')
tile3 = tiles.find('**/tile3')
tile4 = tiles.find('**/tile4')
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

#TODO add FPS

base.disableMouse()
camera.setPos(200,200,200)
camera.lookAt(terrain)
base.enableMouse()

run()
