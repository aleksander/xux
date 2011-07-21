import direct.directbase.DirectStart
#from camera import CamFree
import procedural

# ico = procedural.IcoSphere(1,6)
# ico.reparentTo(render)
# ico.setRenderModeWireframe()
# # ico.setAntialias(2)
# # ico.setPos(2*i,0,2)
# # ico.analyze()
# # ico.writeBamFile("icosphere.bam")

# tk = procedural.TorusKnot()
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
