import direct.directbase.DirectStart
#from camera import CamFree
import procedural

# # for i in range(0, 4):
# ico = procedural.IcoSphere(1,8)
# ico.reparentTo(render)
# # ico.setRenderModeWireframe()
# # ico.setAntialias(2)
# # ico.setPos(2*i,0,2)
# ico.analyze()
# #ico.writeBamFile("icosphere.bam")

# for i in range(0, 4):
	# for j in range(0, 4):
		# ico = loader.loadModel("icosphere")
		# ico.reparentTo(render)
		# ico.setRenderModeWireframe()
		# ico.setPos(2*i,0,2*j)
		# ico.analyze()

# tk = procedural.TorusKnot()
# # TorusKnot(Radius = 2., SectionRadius = .5, UTile = 3., NumSegCircle = 64, NumSegSection = 16)
# # radius=1., sectionRadius=.2, p=2, q=3, numSegSection=8, numSegCircle=16
# tk.reparentTo(render)
# tk.setRenderModeWireframe()

t = procedural.Torus()
t.reparentTo(render)
t.setRenderModeWireframe()

base.setFrameRateMeter(True)

#base.disableMouse()
#base.oobe()
#camera.setPos(0,-2,0)
#camera.lookAt(terrain)
# base.camLens.setNear(.01)
# #base.enableMouse()

# CamFree()

run()
