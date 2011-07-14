import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
from direct.gui.OnscreenImage import OnscreenImage
import random

class tilemap(NodePath):
	def __init__(self, name='tilemap'):
		NodePath.__init__(self, name)
		self.terrain_node = GeomNode(name)
		self.attachNewNode(self.terrain_node)
		self.geoms = []
		self.geoms_count = 0
		self.verts = {}
		self.verts_count = 0
	def add_tile_type(self, tex_file):
		gvd = GeomVertexData('gvd', GeomVertexFormat.getV3t2(), Geom.UHStatic)
		geom = Geom(gvd)
		prim = GeomTriangles(Geom.UHStatic)
		gvwv = GeomVertexWriter(gvd, 'vertex')
		gvwt = GeomVertexWriter(gvd, 'texcoord')
		tex = loader.loadTexture(tex_file)
		#tex.setMagfilter(Texture.FTLinearMipmapLinear)
		#tex.setMinfilter(Texture.FTLinearMipmapLinear)
		rs = RenderState.make(TextureAttrib.make(tex))
		self.terrain_node.addGeom(geom, rs)
		self.geoms.append({'geom':geom,'prim':prim,'vertex':vertex,'texcoord':texcoord,'index':0})
		self.geoms_count += 1
		return self.geoms_count - 1
	def add_tile(self, x, z, tile_type):
		# i = self.geoms[tile_type]['index']
		v = self.geoms[tile_type]['vertex']
		t = self.geoms[tile_type]['texcoord']
		p = self.geoms[tile_type]['prim']
		tmp = [(0,0),(0,1),(1,1),(1,0)]
		vrt = []
		for vert in tmp:
			if (x+vert[0],z+vert[1]) not in self.verts:
				print "({0},{1}) {2}".format(x+vert[0],z+vert[1],self.verts_count)
				v.addData3f(x+vert[0], 0, z+vert[1])
				t.addData2f(vert[0], vert[1])
				vrt.append(self.verts_count)
				self.verts[(x+vert[0],z+vert[1])] = self.verts_count
				self.verts_count += 1
			else:
				vrt.append(self.verts[x+vert[0],z+vert[1]])
			
#		if (x,z+1) not in self.verts:
#			v.addData3f(x, 0, z+1)
#			t.addData2f(0, 1)
#			v2 = self.verts_count
#			self.verts[(x,z+1)] = self.verts_count
#			self.verts_count += 1
#			
#		if (x+1,z+1) not in self.verts:
#			v.addData3f(x+1, 0, z+1)
#			t.addData2f(1, 1)
#			v2 = self.verts_count
#			self.verts[(x,z)] = self.verts_count
#			self.verts_count += 1
#			
#			v.addData3f(x+1, 0, z)
#			t.addData2f(1, 0)
			
#			self.verts[(x,z)] = self.verts_count
#			i = self.verts_count
			
#			p.addVertices(i*4, i*4 + 2, i*4 + 1)
#			p.addVertices(i*4, i*4 + 3, i*4 + 2)
#			self.verts_count += 1

		p.addVertices(vrt[0], vrt[2], vrt[1])
		p.addVertices(vrt[0], vrt[3], vrt[2])

#		else:
#			i = self.verts[(x,z)]
#			p.addVertices(i*4, i*4 + 2, i*4 + 1)
#			p.addVertices(i*4, i*4 + 3, i*4 + 2)
		
	def bake(self):
		for i in xrange(0, len(self.geoms)):
			print self.geoms[i]['prim']
			self.geoms[i]['prim'].closePrimitive()
			self.geoms[i]['geom'].addPrimitive(self.geoms[i]['prim'])

terrain = tilemap()

for i in xrange(0,3):
	terrain.add_tile_type('tile%i.png' % (i+1))

size = 2
for x in xrange(0,size):
	for z in xrange(0,size):
		terrain.add_tile(x, z, random.randint(0,2))

terrain.bake()
terrain.reparentTo(render)
terrain.setPos(-size/2,0,-size/2)
#terrain.setRenderModeWireframe()
terrain.analyze()
base.setFrameRateMeter(True)

run()
