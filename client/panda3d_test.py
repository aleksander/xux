import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
import random

base.setFrameRateMeter(True)

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

rbc = RigidBodyCombiner("rbc")
terrain = NodePath(rbc)
terrain.reparentTo(render)

for x in xrange(-50,50):
	for y in xrange(-50,50):
#		tile = random.choice(tiles)
#		tile = tile.copyTo(rbcnp)
#		tile.setPos(x,y,0)
		tile = terrain.attachNewNode('tile')
		tile.setPos(x,y,0)
		random.choice(tiles).instanceTo(tile)
rbc.collect()
#terrain.flattenStrong()

##TODO add FPS

#base.disableMouse()
#camera.setPos(200,200,200)
#camera.lookAt(terrain)
#base.enableMouse()

## Set up the GeoMipTerrain
#terrain = GeoMipTerrain("myDynamicTerrain")
#terrain.setHeightfield("height-field.png")
# 
## Set terrain properties
#terrain.setBlockSize(100)
#terrain.setNear(40)
#terrain.setFar(100)
#terrain.setFocalPoint(base.camera)
# 
## Store the root NodePath for convenience
#root = terrain.getRoot()
#root.reparentTo(render)
#root.setSz(100)

#terrain.setBruteforce(True)
#terrain.setAutoFlatten(GeoMipTerrain.AFMStrong)

## Generate it.
#terrain.generate()
#render.ls()
#for child in render.getChildren():
#  print child

## Add a task to keep updating the terrain
#def updateTask(task):
#  terrain.update()
#  return task.cont
#taskMgr.add(updateTask, "update")

#PStatClient.connect()
render.analyze()
run()
