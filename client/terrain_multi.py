import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
from direct.gui.OnscreenImage import OnscreenImage
import random

models = loader.loadModel("models.egg")
pointer = models.find('**/pointer')
pointer.reparentTo(render)

gvd = GeomVertexData('name', GeomVertexFormat.getV3t2(), Geom.UHStatic)
vertex = GeomVertexWriter(gvd, 'vertex')
texcoord = GeomVertexWriter(gvd, 'texcoord')
size = 100
vertex.addData3f(size/2, size/2, 0)
texcoord.addData2f(1, 1)
vertex.addData3f(size/2, -size/2, 0)
texcoord.addData2f(1, 0)
vertex.addData3f(-size/2, -size/2, 0)
texcoord.addData2f(0, 0)
vertex.addData3f(-size/2, size/2, 0)
texcoord.addData2f(0, 1)

prim = GeomTriangles(Geom.UHStatic)
prim.addVertices(0, 2, 1)
prim.addVertices(0, 3, 2)

geom = Geom(gvd)
geom.addPrimitive(prim)
node = GeomNode('gnode')
node.addGeom(geom)
terrain = render.attachNewNode(node)

# tiles = [VBase3D(1.0,0.0,0.0), VBase3D(0.0,1.0,0.0), VBase3D(0.0,0.0,1.0)]
# pnmi = PNMImage(100, 100)
# for x in xrange(0,100):
	# for y in xrange(0,100):
		# pnmi.setXel(x, y, random.choice(tiles))

tex = Texture()
tex.load('tile1.png')
tex.setMagfilter(Texture.FTNearest)
tex.setMinfilter(Texture.FTNearest)
ts = TextureStage('ts1')
terrain.setTexture(ts, tex)
terrain.setTexScale(ts, 100, 100)

tex = Texture()
tex.load('mask.png')
tex.setMagfilter(Texture.FTNearest)
tex.setMinfilter(Texture.FTNearest)
ts = TextureStage('ts2')
terrain.setTexture(ts, tex)
# terrain.setTexScale(ts, .1, .1)

# base.disableMouse()
# base.camera.setPos(50,50,50)
# base.camera.lookAt(pointer)
base.setFrameRateMeter(True)

print base.win.getGsg().getMaxTextureStages()

run()