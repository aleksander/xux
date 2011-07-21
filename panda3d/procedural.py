from pandac.PandaModules import *
from math import *

def empty(prefix):
	path = NodePath(prefix + '_path')
	node = GeomNode(prefix + '_node')
	path.attachNewNode(node)

	gvd = GeomVertexData('gvd', GeomVertexFormat.getV3(), Geom.UHStatic)
	geom = Geom(gvd)
	gvw = GeomVertexWriter(gvd, 'vertex')
	node.addGeom(geom)
	prim = GeomTriangles(Geom.UHStatic)
	return (gvw, prim, geom, path)

def IcoSphere(radius, subdivisions):
	(gvw, prim, geom, path) = empty('ico')

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
	
	return path
	
########################################################################

Vec3_ZERO = Vec3(0,0,0)

def getRotationTo(src, dest, fallbackAxis = Vec3_ZERO):
	# Quaternion q;
	# Vector3 v0 = *this;
	# Vector3 v1 = dest;
	# v0.normalise();
	# v1.normalise();
	q = Quat()
	v0 = Vec3(src)
	v1 = Vec3(dest)
	v0.normalize()
	v1.normalize()

	# Real d = v0.dotProduct(v1);
	d = v0.dot(v1)

	# if (d >= 1.0f)
	# {
		# return Quaternion::IDENTITY;
	# }
	if d >= 1.0:
		return Quat(1,0,0,0)

	# if (d < (1e-6f - 1.0f))
	if d < (1.0e-6 - 1.0):
		# if (fallbackAxis != Vector3::ZERO)
		# {
			# // rotate 180 degrees about the fallback axis
			# q.FromAngleAxis(Radian(Math::PI), fallbackAxis);
		# }
		if fallbackAxis != Vec3_ZERO:
			q.setFromAxisAngleRad(pi, fallbackAxis)
		# else
		# {
			# // Generate an axis
			# Vector3 axis = Vector3::UNIT_X.crossProduct(*this);
			# if (axis.isZeroLength()) // pick another if colinear
				# axis = Vector3::UNIT_Y.crossProduct(*this);
			# axis.normalise();
			# q.FromAngleAxis(Radian(Math::PI), axis);
		# }
		else:
			axis = Vec3(1,0,0).cross(src)
			if axis.almostEqual(Vec3.zero()):
				axis = Vec3(0,1,0).cross(src)
			axis.normalize()
			q.setFromAxisAngleRad(pi, axis)
	# else
	# {
		# Real s = Math::Sqrt( (1+d)*2 );
		# Real invs = 1 / s;

		# Vector3 c = v0.crossProduct(v1);

		# q.x = c.x * invs;
		# q.y = c.y * invs;
		# q.z = c.z * invs;
		# q.w = s * 0.5f;
		# q.normalise();
	# }
	else:
		s = sqrt((1 + d) * 2)
		invs = 1 / s
		c = v0.cross(v1)
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
	tY = quat.xform(Vec3(0,1,0))
	quat2 = getRotationTo(tY, projectedY)
	q = quat2 * quat
	return q

########################################################################

def TorusKnot(mRadius=1., mSectionRadius=.2, mP=2, mQ=3, mNumSegSection=64, mNumSegCircle=64):
	(gvw, prim, geom, path) = empty('tk')

	offset = 0

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

		q = computeQuaternion(direction)

		for j in range(0, mNumSegSection+1):
			alpha = pi*2 * j / mNumSegSection
			vp = q.xform(Vec3(cos(alpha), sin(alpha), 0)) * mSectionRadius
			gvw.addData3f(v0+vp)

			if i != mNumSegCircle * mP:
				prim.addVertices(offset+mNumSegSection+1,offset+mNumSegSection,offset)
				prim.addVertices(offset+mNumSegSection+1,offset,offset+1)
			offset += 1

	prim.closePrimitive()
	geom.addPrimitive(prim)
	
	return path

########################################################################

def Torus(mNumSegSection=64, mNumSegCircle=64, mRadius=1.0, mSectionRadius=0.2):
	(gvw, prim, geom, path) = empty('t')

	deltaSection = (pi*2 / mNumSegSection)
	deltaCircle = (pi*2 / mNumSegCircle)

	offset = 0

	for i in range(0, mNumSegCircle+1):
		for j in range(0, mNumSegSection+1):
			v0 = Vec3(mRadius + mSectionRadius * cos(j * deltaSection), mSectionRadius * sin(j * deltaSection), 0.0)
			q = Quat()
			q.setFromAxisAngleRad(i*deltaCircle, Vec3(0,1,0))
			v = q.xform(v0)
			
			gvw.addData3f(v)

			if i != mNumSegCircle:
				prim.addVertices(offset + mNumSegSection + 1,offset,offset + mNumSegSection)
				prim.addVertices(offset + mNumSegSection + 1,offset + 1,offset)
			offset += 1

	prim.closePrimitive()
	geom.addPrimitive(prim)

	return path
