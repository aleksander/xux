import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
from direct.gui.OnscreenImage import OnscreenImage
import random

textures = []
for i in xrange(0,3):
	t = Texture('tile'+str(i+1))
	t.load('tile'+str(i+1)+'.png')
	textures.append(t)
	textures[i].setMagfilter(Texture.FTNearest)
	textures[i].setMinfilter(Texture.FTNearest)
print textures

terrain = NodePath("terrain")

for x in xrange(0,100):
	for z in xrange(0,100):
		gvd = GeomVertexData('name', GeomVertexFormat.getV3t2(), Geom.UHStatic)
		vertex = GeomVertexWriter(gvd, 'vertex')
		texcoord = GeomVertexWriter(gvd, 'texcoord')
		vertex.addData3f(x, 0, z)
		texcoord.addData2f(0, 0)
		vertex.addData3f(x, 0, z+1)
		texcoord.addData2f(0, 1)
		vertex.addData3f(x+1, 0, z+1)
		texcoord.addData2f(1, 1)
		vertex.addData3f(x+1, 0, z)
		texcoord.addData2f(1, 0)
		# prim = GeomLinestrips(Geom.UHStatic)
		# prim.addVertices(0, 1, 2, 3)
		# prim.addVertex(0)
		# prim.closePrimitive()
		prim = GeomTriangles(Geom.UHStatic)
		prim.addVertices(0, 2, 1)
		prim.closePrimitive()
		prim.addVertices(0, 3, 2)
		prim.closePrimitive()
		geom = Geom(gvd)
		geom.addPrimitive(prim)
		geomnode = GeomNode('gnode')
		geomnode.addGeom(geom)
		nodepath = NodePath("np")
		nodepath.attachNewNode(geomnode)
		nodepath.reparentTo(terrain)
		nodepath.setTexture(random.choice(textures))

# terrain = render.attachNewNode(node)
terrain.setPos(-50,0,-50)
# terrain.setTexture(textures[0])
terrain.reparentTo(render)
print "================================= BEFORE ========"
render.analyze()
terrain.flattenStrong()
print "================================= AFTER ========="
render.analyze()
terrain.ls()

# terrain.setRenderModeWireframe()
# terrain.ls()

# # base.disableMouse()
# # base.camera.setPos(50,50,50)
# # base.camera.lookAt(pointer)

base.setFrameRateMeter(True)

run()
