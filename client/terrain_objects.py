import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
import random

print type(base)
print type(cpMgr)

models = loader.loadModel("models.egg")
tile1 = models.find('**/tile1')
tile2 = models.find('**/tile2')
tile3 = models.find('**/tile3')
tile4 = models.find('**/tile4')
pointer = models.find('**/pointer')
#tiles.ls()
tile1.setPos(.0,.0,.0)
tile2.setPos(.0,.0,.0)
tile3.setPos(.0,.0,.0)
tile4.setPos(.0,.0,.0)
pointer.setPos(.0,.0,.0)
tiles = [tile1,tile2,tile3,tile4]

rbc = RigidBodyCombiner("rbc")
terrain = NodePath(rbc)
terrain.reparentTo(render)
pointer.reparentTo(render)

for x in xrange(-50,50):
	for y in xrange(-50,50):
		tile = terrain.attachNewNode('tile')
		tile.setPos(x,y,0)
		random.choice(tiles).instanceTo(tile)
		# tile = random.choice(tiles)
		# tile = tile.copyTo(terrain)
		# tile.setPos(x*1.1,y*1.1,0)
		#terrain.flattenStrong()
rbc.collect()

class cameraHandler(DirectObject):
	def __init__(self):
		base.disableMouse()
		self.mx,self.my=0,0
		self.dragging=False
		self.j1 = render.attachNewNode('cam_j1')
		self.j2 = self.j1.attachNewNode('cam_j2')
		self.j2.setZ(5)
		self.j3 = self.j2.attachNewNode('cam_j3')
		self.j3.setY(-40)
		self.accept("mouse3",self.drag,[True])
		self.accept("mouse3-up",self.drag,[False])
		self.accept("wheel_up", self.adjustCamDist,[0.9])
		self.accept("wheel_down", self.adjustCamDist,[1.1])
		taskMgr.add(self.dragTask,'dragTask')
	def drag(self,bool):
		self.dragging=bool 
	def adjustCamDist(self,aspect):
		self.j3.setY(self.j3.getY()*aspect) 
	def turnCamera(self,tx,ty):
		self.j1.setH(self.j1.getH()+tx)
		self.j2.setP(self.j2.getP()-ty)
		if self.j2.getP()<-80:
			self.j2.setP(-80)
		if self.j2.getP()>-10:
			self.j2.setP(-10)
	def dragTask(self,task):
		if base.mouseWatcherNode.hasMouse():
			mpos = base.mouseWatcherNode.getMouse()  
			if self.dragging:
				self.turnCamera((self.mx-mpos.getX())*100,(self.my-mpos.getY())*100)
			else:
				if self.my>0.8:
					aspect=-(1-self.my-0.2)*5
					self.j1.setY(self.j1,aspect)
				if self.my<-0.8:
					aspect=(1+self.my-0.2)*5
					self.j1.setY(self.j1,aspect)
				if self.mx>0.8:
					aspect=-(1-self.mx-0.2)*5
					self.j1.setX(self.j1,aspect)
				if self.mx<-0.8:
					aspect=(1+self.mx-0.2)*5
					self.j1.setX(self.j1,aspect)
			self.mx=mpos.getX()
			self.my=mpos.getY()                    
		vDir=Vec3(self.j3.getPos(render))-Vec3(base.camera.getPos(render))
		vDir=vDir*0.2
		base.camera.setPos(Vec3(base.camera.getPos())+vDir)
		base.camera.lookAt(self.j2.getPos(render))
		return task.cont

camera = cameraHandler()

class mouseControl(DirectObject):
	def __init__(self):
		self.picker = CollisionTraverser()  
		self.pickerQ = CollisionHandlerQueue()  
		pickerCollN = CollisionNode('mouseRay')  
		pickerCamN = base.camera.attachNewNode(pickerCollN)  
		pickerCollN.setFromCollideMask(BitMask32.bit(1))  
		pickerCollN.setIntoCollideMask(BitMask32.allOff())  
		self.pickerRay = CollisionRay()  
		pickerCollN.addSolid(self.pickerRay)  
		self.picker.addCollider(pickerCamN, self.pickerQ)  
		self.accept('mouse1',self.pick)  
	def pick(self):
		if base.mouseWatcherNode.hasMouse():
			mpos = base.mouseWatcherNode.getMouse()
			self.pickerRay.setFromLens(base.camNode, mpos.getX(), mpos.getY())
			self.picker.traverse(render)
			print self.pickerQ.getNumEntries()
			for i in xrange(self.pickerQ.getNumEntries()):
				entry=self.pickerQ.getEntry(i)
				print 'entry '+str(entry.getSurfacePoint(render))

mouse = mouseControl()

base.setFrameRateMeter(True)
run()
