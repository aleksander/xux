import direct.directbase.DirectStart
import procedural

# model = procedural.IcoSphere(1,6)
# model = procedural.TorusKnot()
# model = procedural.Torus()
# model = procedural.Tetrahedron()
# model = procedural.Octahedron()
# model = procedural.Dodecahedron()
model = procedural.LimpetTorus()
# model = procedural.TwistedPseudosphere()

model.reparentTo(render)
model.setRenderModeWireframe()

#base.camLens.setFov(7.)
base.setFrameRateMeter(True)
run()
