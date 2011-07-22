import direct.directbase.DirectStart
import procedural

# ico = procedural.IcoSphere(1,6)
# ico.reparentTo(render)
# ico.setRenderModeWireframe()

tk = procedural.TorusKnot()
tk.reparentTo(render)
tk.setRenderModeWireframe()

# t = procedural.Torus()
# t.reparentTo(render)
# t.setRenderModeWireframe()

base.setFrameRateMeter(True)
run()
