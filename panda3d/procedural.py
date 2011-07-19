from pandac.PandaModules import *
from math import *

def IcoSphere(radius, subdivisions):
	ico_path = NodePath('ico_path')
	ico_node = GeomNode('ico_node')
	ico_path.attachNewNode(ico_node)

	gvd = GeomVertexData('gvd', GeomVertexFormat.getV3(), Geom.UHStatic)
	geom = Geom(gvd)
	gvw = GeomVertexWriter(gvd, 'vertex')
	ico_node.addGeom(geom)
	prim = GeomTriangles(Geom.UHStatic)
	
	verts = []
	
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

	faces = [
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

	size = 60

	# Step 2 : tessellate
	for subdivision in range(0,subdivisions):
		size*=4;
		newFaces = []
		for i in range(0,size/12):
			i1 = faces[i*3]
			i2 = faces[i*3+1]
			i3 = faces[i*3+2]
			i12 = len(verts)
			i23 = i12+1
			i13 = i12+2
			v1 = verts[i1]
			v2 = verts[i2]
			v3 = verts[i3]
			# make 1 vertice at the center of each edge and project it onto the sphere
			vt = v1+v2
			vt.normalize()
			verts.append(vt)
			vt = v2+v3
			vt.normalize()
			verts.append(vt)
			vt = v1+v3
			vt.normalize()
			verts.append(vt)
			# now recreate indices
			newFaces.append(i1)
			newFaces.append(i12)
			newFaces.append(i13)
			newFaces.append(i2)
			newFaces.append(i23)
			newFaces.append(i12)
			newFaces.append(i3)
			newFaces.append(i13)
			newFaces.append(i23)
			newFaces.append(i12)
			newFaces.append(i23)
			newFaces.append(i13)
		faces = newFaces
	
	for i in range(0,len(verts)):
		gvw.addData3f(VBase3(verts[i]))
	for i in range(0, len(faces)/3):
		prim.addVertices(faces[i*3],faces[i*3+1],faces[i*3+2])

	prim.closePrimitive()
	geom.addPrimitive(prim)
	
	return ico_path

########################################################################

def TorusKnot(mRadius=1., mSectionRadius=.2, mP=2, mQ=3, mNumSegSection=8, mNumSegCircle=16):
	tk_path = NodePath('tk_path')
	tk_node = GeomNode('tk_node')
	tk_path.attachNewNode(tk_node)

	gvd = GeomVertexData('gvd', GeomVertexFormat.getV3(), Geom.UHStatic)
	geom = Geom(gvd)
	gvw = GeomVertexWriter(gvd, 'vertex')
	tk_node.addGeom(geom)
#	prim = GeomTriangles(Geom.UHStatic)
	prim = GeomLines(Geom.UHStatic)

	offset = 0

	# for i in range(0, mNumSegCircle * mP):
	for i in range(0, mNumSegCircle*mP+1):
		phi = pi*2 * i / mNumSegCircle
		x0 = mRadius * (2 + cos(mQ * phi / mP)) * cos(phi) / 3.
		y0 = mRadius * sin(mQ * phi / mP) / 3.
		z0 = mRadius * (2 + cos(mQ * phi / mP)) * sin(phi) / 3.

		phi1 = pi*2 * (i + 1) / mNumSegCircle
		x1 = mRadius * (2 + cos(mQ * phi1 / mP)) * cos(phi1) / 3.
		y1 = mRadius * sin(mQ * phi1 / mP) / 3.
		z1 = mRadius * (2 + cos(mQ * phi1 / mP)) * sin(phi1) / 3.

		v0 = Vec3(x0,y0,z0)
		v1 = Vec3(x1,y1,z1)
		direction = v1-v0
		direction.normalize()

		gvw.addData3f(x0,y0,z0)
		gvw.addData3f(x1,y1,z1)
		prim.addVertices(i*2, i*2+1)

		# Quaternion getRotationTo(const Vector3& dest, const Vector3& fallbackAxis = Vector3::ZERO) const
		def getRotationTo(src, dest, fallbackAxis = Vec3(0,0,0)):
			# Based on Stan Melax's article in Game Programming Gems
			q = Quat()
			# Copy, since cannot modify local
			v0 = Vec3(src)
			v1 = Vec3(dest)
			v0.normalize()
			v1.normalize()

			d = v0.dot(v1) #dotProduct(v1);
			# If dot == 1, vectors are the same
			if d >= 1.0:
				return Quat(1,0,0,0) #Quaternion::IDENTITY;
			if d < (1e-6 - 1.):
				if fallbackAxis != Vec3(0,0,0):
					# rotate 180 degrees about the fallback axis
					q.setFromAxisAngle(pi, fallbackAxis)
				else:
					# Generate an axis
					axis = Vec3(1,0,0).crossProduct(src)
					if axis.almostEqual(Vec3.zero()): # pick another if colinear
						axis = Vec3(0,1,0).crossProduct(src)
					axis.normalize()
					q.setFromAxisAngle(pi, axis)
			else:
				s = sqrt((1 + d) * 2)
				invs = 1 / s

				c = v0.cross(v1)

				# q.x = c.x * invs
				# q.y = c.y * invs
				# q.z = c.z * invs
				# q.w = s * .5
				q.setI(c.x * invs)
				q.setJ(c.y * invs)
				q.setK(c.z * invs)
				q.setR(s * .5)
				q.normalize()
			return q

		def computeQuaternion(direction):
			# Quaternion quat = Vector3::UNIT_Z.getRotationTo(direction);
			quat = getRotationTo(Vec3(0,0,1), direction)
			projectedY = Vec3(0,1,0) - direction * Vec3(0,1,0).dot(direction)
			tY = quat * Vec3(0,1,0)
			quat2 = getRotationTo(tY, projectedY)
			q = quat2 * quat
			return q

		q = computeQuaternion(direction)

		# for j in range(0, mNumSegSection+1)
		for j in range(0, mNumSegSection):
			alpha = pi*2 * j / mNumSegSection
			vp = Vec3(cos(alpha), sin(alpha), 0)
			vp = q * vp
			vp = vp * mSectionRadius
			gvw.addData3f(v0+vp)

			if i != mNumSegCircle * mP:
				prim.addVertices(offset+mNumSegSection+1,offset+mNumSegSection,offset)
				prim.addVertices(offset+mNumSegSection+1,offset,offset+1)
				# buffer.index(offset + mNumSegSection + 1);
				# buffer.index(offset + mNumSegSection);
				# buffer.index(offset);
				# buffer.index(offset + mNumSegSection + 1);
				# buffer.index(offset);
				# buffer.index(offset + 1);
			offset += 1

	prim.closePrimitive()
	geom.addPrimitive(prim)
	
	return tk_path
