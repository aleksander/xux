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
	# loader.loadTexture(...)
	t = Texture('tile'+str(i+1))
	t.load('tile'+str(i+1)+'.png')
	textures.append(t)
	textures[i].setMagfilter(Texture.FTNearest)
	textures[i].setMinfilter(Texture.FTNearest)
print textures

terrain = NodePath("terrain")

gvd = GeomVertexData('name', GeomVertexFormat.getV3t2(), Geom.UHStatic)
vertex = GeomVertexWriter(gvd, 'vertex')
texcoord = GeomVertexWriter(gvd, 'texcoord')
geom = Geom(gvd)
prim = GeomTriangles(Geom.UHStatic)
#for i in dir(geom):
#	print i," - ",type(i)
size = 100
i = 0
for x in xrange(0,size):
	for z in xrange(0,size):
		vertex.addData3f(x, 0, z)
		texcoord.addData2f(0, 0)
		vertex.addData3f(x, 0, z+1)
		texcoord.addData2f(0, 1)
		vertex.addData3f(x+1, 0, z+1)
		texcoord.addData2f(1, 1)
		vertex.addData3f(x+1, 0, z)
		texcoord.addData2f(1, 0)
		prim.addVertices(i, i+2, i+1)
		prim.addVertices(i, i+3, i+2)
		i += 4
prim.closePrimitive()
geom.addPrimitive(prim)
geomnode = GeomNode('gnode')
geomnode.addGeom(geom)
terrain.attachNewNode(geomnode)
terrain.setPos(-size/2,0,-size/2)
terrain.setTexture(textures[0])
terrain.reparentTo(render)
# print "================================= BEFORE ========"
# render.analyze()
# # terrain.flattenStrong()
# print "================================= AFTER ========="
# render.analyze()
# terrain.ls()

#terrain.setRenderModeWireframe()
# terrain.ls()

base.setFrameRateMeter(True)

run()
