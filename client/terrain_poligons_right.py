import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
from direct.gui.OnscreenImage import OnscreenImage
import random

from pandac.PandaModules import *
#loadPrcFileData("editor-startup", "show-frame-rate-meter #t")
import direct.directbase.DirectStart
from random import randint

node = GeomNode('gnode')
geoms = []
for i in xrange(3):
	gvd = GeomVertexData('gvd', GeomVertexFormat.getV3t2(), Geom.UHStatic)
	geom = Geom(gvd)
	prim = GeomTriangles(Geom.UHStatic)
	vertex = GeomVertexWriter(gvd, 'vertex')
	texcoord = GeomVertexWriter(gvd, 'texcoord')
	tex = loader.loadTexture('tile%i.png' % (i+1))
	tex.setMagfilter(Texture.FTLinearMipmapLinear)
	tex.setMinfilter(Texture.FTLinearMipmapLinear)
	geoms.append({'geom':geom,'prim':prim,'vertex':vertex,'texcoord':texcoord,'index':0,'gvd':gvd,'texture':tex})

size = 100
for x in xrange(0,size):
	for z in xrange(0,size):
		t_img = random.randint(0,2)
		i = geoms[t_img]['index']
		geoms[t_img]['vertex'].addData3f(x, 0, z)
		geoms[t_img]['texcoord'].addData2f(0, 0)
		geoms[t_img]['vertex'].addData3f(x, 0, z+1)
		geoms[t_img]['texcoord'].addData2f(0, 1)
		geoms[t_img]['vertex'].addData3f(x+1, 0, z+1)
		geoms[t_img]['texcoord'].addData2f(1, 1)
		geoms[t_img]['vertex'].addData3f(x+1, 0, z)
		geoms[t_img]['texcoord'].addData2f(1, 0)
		geoms[t_img]['prim'].addVertices(i*4, i*4 + 2, i*4 + 1)
		geoms[t_img]['prim'].addVertices(i*4, i*4 + 3, i*4 + 2)
		geoms[t_img]['index'] += 1

for i in xrange(3):
	geoms[i]['prim'].closePrimitive()
	geoms[i]['geom'].addPrimitive(geoms[i]['prim'])
	node.addGeom(geoms[i]['geom'])
	node.setGeomState(i, node.getGeomState(i).addAttrib(TextureAttrib.make(geoms[i]['texture'])))

terrain = render.attachNewNode(node)
terrain.setPos(-size/2,0,-size/2)
#terrain.setRenderModeWireframe()
terrain.analyze()
base.setFrameRateMeter(True)

run()