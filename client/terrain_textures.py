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
#terrain.setPos(0,30,-5)

tiles = [VBase3D(1.0,0.0,0.0), VBase3D(0.0,1.0,0.0), VBase3D(0.0,0.0,1.0)]
pnmi = PNMImage(100, 100)
for x in xrange(0,100):
	for y in xrange(0,100):
		#myImage.setXelVal(0, 0, gray * 255)
		pnmi.setXel(x, y, random.choice(tiles))
#img = OnscreenImage(pnmi)
#img.setName('123cv')
#img.setPos(-0.5, 0, 0.02)

tex = Texture()
tex.load(pnmi)
tex.setMagfilter(Texture.FTNearest)
tex.setMinfilter(Texture.FTNearest)
#tex.setMinfilter(Texture.FTLinear)

terrain.setTexture(tex)

# base.disableMouse()
# base.camera.setPos(50,50,50)
# base.camera.lookAt(pointer)
base.setFrameRateMeter(True)

run()
