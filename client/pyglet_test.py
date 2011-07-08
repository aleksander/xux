__docformat__ = 'restructuredtext'
__version__ = '$Id$'

import pyglet

window = pyglet.window.Window()
label = pyglet.text.Label('Hello, world', font_name='Times New Roman', font_size=10, x=window.width//2, y=window.height//2, anchor_x='center', anchor_y='center')
fps_display = pyglet.clock.ClockDisplay()
img = pyglet.image.load('tile1.png')
#sprite = pyglet.sprite.Sprite(img)
igrid = pyglet.image.ImageGrid(image=img, rows=5, columns=5, item_width=320, item_height=320)
for i in igrid:
	i = img
	print(i)
tgrid = pyglet.image.TextureGrid(igrid)


@window.event
def on_draw():
	window.clear()
	fps_display.draw()
	label.draw()
	#sprite.draw()
	tgrid.blit(0,0)

pyglet.app.run()
