import direct.directbase.DirectStart
from pandac.PandaModules import *
from direct.interval.IntervalGlobal import *
from direct.task.Task import Task
from direct.showbase.DirectObject import DirectObject
from pandac.PandaModules import Vec3
from direct.gui.OnscreenImage import OnscreenImage
import random

model = loader.loadModel("test")
#pointer = models.find('**/pointer')
model.reparentTo(render)
model.setRenderModeWireframe()

model.ls()
print "model type =",type(model)
for child in model.getChildren():
	print child
	print type(child)

run()