from pandac.PandaModules import Point3, EggData, StringStream, loadEggData, NodePath
import direct.directbase.DirectStart
import cStringIO
from random import randint
egg = cStringIO.StringIO()
ryad, sum_pol = 10, 100
egg.writelines('<Texture> 1 {"Tiles/1.png" <Scalar> uv-name { UVTex }}'
               '<Texture> 2 {"Tiles/2.png" <Scalar> uv-name { UVTex }}'
               '<Texture> 3 {"Tiles/3.png" <Scalar> uv-name { UVTex }}'
               '<VertexPool> Plane {')
def PolygonPos(i): 
    return Point3((i%ryad) - 3.5, int(i/ryad) - 3.5, 0)
Index0, Index1, Index2, Index3 = 0, 1, 2, 3
for i in range(sum_pol):
    Pos = PolygonPos(i)
    X = Pos[0]
    Y = Pos[1]
    egg.writelines('<Vertex> '+str(Index0)+' {'+str(X-0.5)+' '+str(Y-0.5)+' 0 <UV> UVTex { 0 0 }}')
    egg.writelines('<Vertex> '+str(Index1)+' {'+str(X+0.5)+' '+str(Y-0.5)+' 0 <UV> UVTex { 1 0 }}')
    egg.writelines('<Vertex> '+str(Index2)+' {'+str(X+0.5)+' '+str(Y+0.5)+' 0 <UV> UVTex { 1 1 }}')
    egg.writelines('<Vertex> '+str(Index3)+' {'+str(X-0.5)+' '+str(Y+0.5)+' 0 <UV> UVTex { 0 1 }}')
    Index0, Index1, Index2, Index3 = Index0+4, Index1+4, Index2+4, Index3+4
egg.writelines('}')
Index0, Index1, Index2, Index3 = 0, 1, 2, 3
for i in range(sum_pol):
    egg.writelines('<Polygon> {<TRef> { '+str(randint(1,3))+' } <VertexRef> { '+str(Index0)+' '+str(Index1)+' '+str(Index2)+' '+str(Index3)+' <Ref> { Plane }}}')
    Index0, Index1, Index2, Index3 = Index0+4, Index1+4, Index2+4, Index3+4
Create_egg = EggData()
Create_egg.read(StringStream(egg.getvalue()))
egg.close()
#Create_egg.writeEgg("Territor.egg") # write file
Territor = render.attachNewNode(loadEggData(Create_egg))
run()