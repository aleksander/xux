import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
from direct.gui.OnscreenImage import OnscreenImage
import random
from pandac.PandaModules import *
import direct.directbase.DirectStart
from random import randint
from math import sqrt

def IcoSphere(radius, subdivs):
	ico_path = NodePath('ico_path')
	ico_node = GeomNode('ico_node')
	ico_path.attachNewNode(ico_node)

	gvd = GeomVertexData('gvd', GeomVertexFormat.getV3(), Geom.UHStatic)
	geom = Geom(gvd)
	gvw = GeomVertexWriter(gvd, 'vertex')
	ico_node.addGeom(geom)
	prim = GeomTriangles(Geom.UHStatic)
	
	verts = []
	# faces = []
	
	phi = .5*(1.+sqrt(5.))
	invnorm = 1/sqrt(phi*phi+1)

	verts.append(Vec3(-1,  phi, 0) * invnorm)   #0
	verts.append(Vec3( 1,  phi, 0) * invnorm)   #1
	verts.append(Vec3(0,   1,  -phi) * invnorm) #2
	verts.append(Vec3(0,   1,   phi) * invnorm) #3
	verts.append(Vec3(-phi,0,  -1) * invnorm)   #4
	verts.append(Vec3(-phi,0,   1) * invnorm)   #5
	verts.append(Vec3( phi,0,  -1) * invnorm)   #6
	verts.append(Vec3( phi,0,   1) * invnorm)   #7
	verts.append(Vec3(0,   -1, -phi) * invnorm) #8
	verts.append(Vec3(0,   -1,  phi) * invnorm) #9
	verts.append(Vec3(-1,  -phi,0) * invnorm)   #10
	verts.append(Vec3( 1,  -phi,0) * invnorm)   #11

	firstFaces = [
		0,1,2,
		0,3,1,
		0,4,5,
		1,7,6,
		1,6,2,
		1,3,7,
		0,2,4,
		0,5,3,
		2,6,8,
		2,8,4,
		3,5,9,
		3,9,7,
		11,6,7,
		10,5,4,
		10,4,8,
		10,9,5,
		11,8,6,
		11,7,9,
		10,8,11,
		10,11,9
	]

	int size = 60;

	# Step 2 : tessellate
	for subdiv in subdivs:
		size*=4;
		newFaces = []
		for i in size/12:
			i1 = firstFaces[i*3]
			i2 = firstFaces[i*3+1]
			i3 = firstFaces[i*3+2]
			i12 = len(verts)
			i23 = i12+1
			i13 = i12+2
			v1 = Vec3(verts[i1])
			v2 = Vec3(verts[i2])
			v3 = Vec3(verts[i3])
			# make 1 vertice at the center of each edge and project it onto the sphere
			vertices.push_back((v1+v2).normalisedCopy());
			vertices.push_back((v2+v3).normalisedCopy());
			vertices.push_back((v1+v3).normalisedCopy());
			# now recreate indices
			newFaces.push_back(i1);
			newFaces.push_back(i12);
			newFaces.push_back(i13);
			newFaces.push_back(i2);
			newFaces.push_back(i23);
			newFaces.push_back(i12);
			newFaces.push_back(i3);
			newFaces.push_back(i13);
			newFaces.push_back(i23);
			newFaces.push_back(i12);
			newFaces.push_back(i23);
			newFaces.push_back(i13);
		faces.swap(newFaces);
	
	# for i in range(0,len(verts)):
		# gvw.addData3f(VBase3(verts[i]))
	# for i in range(0, len(firstFaces)/3):
		# prim.addVertices(firstFaces[i*3],firstFaces[i*3+1],firstFaces[i*3+2])

	# prim.closePrimitive()
	# geom.addPrimitive(prim)
	
	return ico_path

########################################################################

ico = IcoSphere(1,1)

ico.reparentTo(render)
#terrain.setPos(-size/2,0,-size/2)
ico.setRenderModeWireframe()
ico.analyze()
base.setFrameRateMeter(True)

run()
