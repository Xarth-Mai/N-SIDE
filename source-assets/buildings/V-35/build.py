"""Author/export BYTE BEAT's north facade; default exports the edited Blender master."""
import argparse
import json
from pathlib import Path
import sys

import bmesh
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MASTER = HERE / "facade.blend"
GLB = ROOT / "game/assets/environment/buildings/v35-byte-beat-facade.glb"
OUTPUT = ROOT / "todo/evidence/TASK-049/v35-building-r1"
TEXTURES = ROOT / "source-assets/environment-kit/materials"
ROLES = {}


# Same metric PBR and disposable export-copy workflow as the project's V-55 source
# The source recipe stays self-contained so another building's edits cannot alter it

def linear(v):
    return v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4


def material(name, color, roughness=.7, metallic=0, texture=None, repeat=(1, 1)):
    mat = bpy.data.materials.new(name); mat.use_nodes = True
    rgba = [linear(v) for v in color] + [1]; mat.diffuse_color = rgba
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = rgba
    shader.inputs["Roughness"].default_value = roughness
    shader.inputs["Metallic"].default_value = metallic
    if texture:
        for suffix, space in (("Color", "sRGB"), ("NormalGL", "Non-Color")):
            node = mat.node_tree.nodes.new("ShaderNodeTexImage")
            path = TEXTURES / f"{texture}_1K-JPG_Color.jpg" if suffix == "Color" else HERE / f"textures/{texture}-NormalGL-scale025.png"
            node.image = bpy.data.images.load(str(path), check_existing=True)
            node.image.colorspace_settings.name = space
            if suffix == "Color":
                mat.node_tree.links.new(node.outputs["Color"], shader.inputs["Base Color"])
            else:
                normal = mat.node_tree.nodes.new("ShaderNodeNormalMap"); normal.inputs["Strength"].default_value = 1
                mat.node_tree.links.new(node.outputs["Color"], normal.inputs["Color"])
                mat.node_tree.links.new(normal.outputs["Normal"], shader.inputs["Normal"])
    mat["meters_per_repeat"] = repeat; ROLES[name] = mat


def metric_uv(mesh):
    uv = mesh.uv_layers.active or mesh.uv_layers.new(name="UVMap")
    for poly in mesh.polygons:
        axis = max(range(3), key=lambda k: abs(poly.normal[k])); axes = ((1, 2), (0, 2), (0, 1))[axis]
        repeat = mesh.materials[poly.material_index]["meters_per_repeat"]
        for loop in poly.loop_indices:
            co = mesh.vertices[mesh.loops[loop].vertex_index].co
            uv.data[loop].uv = (co[axes[0]] / repeat[0], co[axes[1]] / repeat[1])


class Piece:
    def __init__(self, name):
        self.name, self.vertices, self.faces, self.roles = name, [], [], []

    def box(self, left, right, bottom, top, back, front, role):
        assert right > left and top > bottom and front > back
        self.profile([(left, bottom), (right, bottom), (right, top), (left, top)], back, front, role)

    def profile(self, profile, back, front, role):
        offset = len(self.vertices); n = len(profile)
        self.vertices.extend((x, depth, z) for depth in (back, front) for x, z in profile)
        faces = [tuple(range(n)), tuple(range(n, n*2))]
        faces.extend((i, (i+1) % n, (i+1) % n+n, i+n) for i in range(n))
        self.faces.extend(tuple(offset+i for i in face) for face in faces); self.roles.extend([role]*len(faces))

    def finish(self, collection):
        mesh = bpy.data.meshes.new(self.name); mesh.from_pydata(self.vertices, [], self.faces)
        for mat in ROLES.values(): mesh.materials.append(mat)
        names = list(ROLES)
        for poly, role in zip(mesh.polygons, self.roles): poly.material_index = names.index(role)
        bm = bmesh.new(); bm.from_mesh(mesh); bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces)); bm.to_mesh(mesh); bm.free(); mesh.update()
        metric_uv(mesh)
        obj = bpy.data.objects.new(self.name, mesh); collection.objects.link(obj); obj["owner_id"] = "V-35"


def window(name, left, right, bottom, top, collection):
    p = Piece(name); center = (left+right)/2
    p.box(left, right, bottom, top, .024, .046, "Glass")
    for a, b in ((left, left+.10), (right-.10, right), (center-.035, center+.035)):
        p.box(a, b, bottom, top, .047, .34, "Graphite")
    for a, b in ((bottom, bottom+.08), (top-.08, top)):
        p.box(left, right, a, b, .047, .34, "Graphite")
    p.box(left-.055, right+.055, bottom-.10, bottom, .07, .43, "Concrete")
    # Shallow interior-tone bars remain behind the projecting frame
    p.box(left+.10, right-.10, bottom+(top-bottom)*.68, bottom+(top-bottom)*.68+.035, .047, .075, "Paper")
    p.finish(collection)


def showcase(left, right, name, collection):
    p = Piece(name)
    p.box(left, right, .24, 2.69, .024, .055, "Graphite")
    for a, b in ((left, left+.12), (right-.12, right)):
        p.box(a, b, .24, 2.69, .056, .77, "Graphite")
    for a, b in ((.24, .36), (2.57, 2.69)):
        p.box(left, right, a, b, .056, .77, "Graphite")
    p.box(left+.12, right-.12, 2.53, 2.57, .61, .75, "Paper")
    p.box(left, right, .12, .24, .02, .81, "Concrete")
    p.finish(collection)


def machine(center, variant, collection):
    p = Piece(f"V35_Showcase_Arcade_{variant}"); x = center; low = .36
    p.profile([(x-.65,low),(x+.65,low),(x+.70,1.38),(x+.59,2.26),(x+.39,2.45),(x-.39,2.45),(x-.59,2.26),(x-.70,1.38)], .08, .45, "Mauve" if variant % 2 == 0 else "Paper")
    p.box(x-.54,x+.54,1.36,2.23,.451,.478,"Graphite")
    p.box(x-.47,x+.47,1.43,2.14,.479,.49,"Glass")
    # Anonymous rhythm lanes, not a title screen or a playable mechanic
    for j in range(4):
        a = x-.34+j*.205
        p.box(a,a+.11,1.47,1.52+(j%3)*.20,.491,.498,"Mauve" if j%2 else "Lime")
    p.box(x-.64,x+.64,1.16,1.30,.32,.64,"Graphite")
    for j in range(4):
        a=x-.43+j*.25
        p.box(a,a+.16,1.305,1.34,.45,.60,"Lime" if j%2 else "Mauve")
    p.box(x-.38,x+.38,.54,.83,.451,.468,"Graphite")
    for j in range(5):
        a=x-.28+j*.12; p.box(a,a+.048,.59,.76,.469,.475,"Paper")
    p.box(x-.065,x+.065,.92,1.04,.451,.468,"Graphite")
    p.finish(collection)


def make_master():
    district=json.loads((ROOT/"source-assets/district-map/district.json").read_text())
    building=next(b for b in district["buildings"] if b["id"]=="V-35")
    assert building["polygon"]==[[-402,113],[-370,113],[-370,153],[-402,153]] and building["elevation"]==12 and building["height"]==11
    assert [f["z"] for f in building["design"]["floors"]]==[12,18] and building["design"]["canopy"]==1.5
    assert district["nodes"]["game_entry"]==[-386,153,12] and district["nodes"]["game_service_entry"]==[-386,113,12]
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene=bpy.context.scene; scene.name="Scene0"; scene.unit_settings.system="METRIC"; scene.unit_settings.scale_length=1
    collection=bpy.data.collections.new("V35_EXPORT_North"); scene.collection.children.link(collection)
    material("Plaster",(.80,.79,.75),.87,texture="Plaster001",repeat=(2,2))
    material("Concrete",(.62,.64,.63),.86,texture="Concrete034",repeat=(1.1,.55))
    material("Graphite",(.16,.145,.205),.53,.3)
    material("Mauve",(.80,.43,.68),.6,.08)
    material("Lime",(.64,.76,.33),.6)
    material("Glass",(.15,.24,.29),.28,.12)
    material("Paper",(.91,.89,.80),.73)
    windows=[(x-1.95,x+1.95,7.0,9.55) for x in (-12.6,-7.6,-2.55,2.55,7.6,12.6)]
    windows += [(-15.35,-14.05,.90,2.48),(14.05,15.35,.90,2.48)]
    openings=windows+[(-13.25,-4.15,.12,2.69),(4.15,13.25,.12,2.69),(-1.1,1.1,0,2.60)]
    p=Piece("V35_North_WallAndPlinth")
    levels=sorted({-.18,0,.6,3.16,6,10.78}|{z for _,_,low,high in openings for z in (low,high)})
    for low,high in zip(levels,levels[1:]):
        cursor=-16
        intervals=sorted((a,b) for a,b,z0,z1 in openings if z0<=(low+high)/2<=z1)
        for left,right in intervals+[(16,16)]:
            if left>cursor: p.box(cursor,left,low,high,.024,.12,"Concrete" if high<=.6 else "Graphite" if high<=3.16 else "Plaster")
            cursor=max(cursor,right)
    p.finish(collection)
    for i,(left,right,low,high) in enumerate(windows): window(f"V35_North_Window_{i}",left,right,low,high,collection)
    showcase(-13.25,-4.15,"V35_West_ShallowShowcase",collection); showcase(4.15,13.25,"V35_East_ShallowShowcase",collection)
    for i,x in enumerate((-11.65,-8.7,-5.75,5.75,8.7,11.65)): machine(x,i,collection)
    p=Piece("V35_PublicDoor")
    for a,b in ((-1.10,-1),(1,1.10)): p.box(a,b,0,2.60,.024,.28,"Graphite")
    p.box(-1.1,1.1,2.48,2.60,.024,.28,"Graphite")
    p.box(-1,1,.02,2.48,.024,.049,"Glass")
    p.box(-.032,.032,.02,2.48,.050,.105,"Graphite")
    for a,b in ((-.21,-.175),(.175,.21)): p.box(a,b,.94,1.28,.106,.18,"Paper")
    p.box(-1,1,.04,.38,.05,.08,"Mauve"); p.finish(collection)
    p=Piece("V35_North_FloorEdgesAndCanopy")
    p.box(-16,16,5.89,6.08,.12,.24,"Graphite")
    p.box(-16,16,10.78,11,.024,.29,"Concrete")
    p.box(-16,16,2.97,3.16,.12,.77,"Graphite")
    p.box(-16,16,2.82,3.02,.77,1.5,"Graphite")
    p.box(-15.98,15.98,2.86,2.90,1.5-0.016,1.5,"Mauve")
    for x in (-13,-9,-5,0,5,9,13):
        p.box(x-.18,x+.18,2.803,2.82,.92,1.31,"Paper")
        p.box(x-.045,x+.045,2.64,2.82,.12,1.37,"Graphite")
    for x in (-15.86,15.86): p.box(x-.12,x+.12,-.18,10.78,.12,.28,"Concrete")
    p.finish(collection)
    p=Piece("V35_North_IdentityPanels")
    # Original sign occupies x +-3.08, z 3.17..4.83; it is retained by the game
    for left,right in ((-13.25,-4.15),(4.15,13.25)):
        p.box(left,right,3.32,4.64,.121,.185,"Graphite")
        for j in range(8):
            x=left+.46+j*1.1; top=3.62+(.28,.53,.72,.46)[j%4]
            p.box(x,x+.44,3.48,top,.186,.20,"Mauve" if j%3 else "Lime")
        p.box(left+.25,right-.25,3.40,3.43,.186,.20,"Paper")
    # Sparse vertical louvres relate the stair zones to the calmer second storey
    for side in (-1,1):
        for j in range(5):
            x=side*(14.95+j*.19); p.box(x-.025,x+.025,6.42,10.40,.125,.36,"Graphite")
    p.finish(collection)
    p=Piece("V35_East_WaitingEdge")
    # A shallow leaning edge beside the showcases leaves the public lane and door open
    for x in (5.05,8.25,11.45): p.box(x-.05,x+.05,0,.87,1.04,1.14,"Graphite")
    p.box(5,11.5,.86,.93,.93,1.24,"Mauve")
    for left,right in ((5,6.2),(7.65,8.85),(10.3,11.5)):
        p.box(left,right,.012,.029,1.20,1.40,"Lime")
    p.finish(collection)
    scene["owner_id"]="V-35"; scene["pivot_map_xyz"]=[-386,153,12]
    scene["source_building"]=json.dumps(building,ensure_ascii=False)
    scene["door_nodes"]=json.dumps({e["node"]:district["nodes"][e["node"]] for e in building["design"]["entries"]})
    scene["retained_sign"]="source-assets/district-scene/byte-beat.svg; business_signs unchanged"
    scene["replaced_faces"]="North facade and source 1.5 m north canopy only; retain shell and other three faces"
    scene["window_count"]=len(windows); scene["display_machine_count"]=6
    for image in bpy.data.images:
        if image.filepath: image.pack()
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))


def export():
    bpy.ops.object.select_all(action="DESELECT"); copies=[]
    for source in bpy.data.collections["V35_EXPORT_North"].objects:
        obj=source.copy(); obj.data=source.data.copy(); bpy.context.scene.collection.objects.link(obj)
        obj.select_set(True); copies.append(obj)
    bpy.context.view_layer.objects.active=copies[0]
    bpy.ops.object.transform_apply(location=True,rotation=True,scale=True); bpy.ops.object.join()
    obj=bpy.context.object; obj.name="V35_ByteBeatNorthFacade"
    bpy.ops.export_scene.gltf(filepath=str(GLB),export_format="GLB",use_selection=True,export_yup=True,
                             export_animations=False,export_cameras=False,export_lights=False,export_extras=False,export_materials="EXPORT")
    bpy.data.objects.remove(obj,do_unlink=True)


def render():
    scene=bpy.context.scene; scene.render.engine="CYCLES"; scene.cycles.device="CPU"; scene.cycles.samples=16
    scene.render.threads_mode="FIXED"; scene.render.threads=2
    scene.render.resolution_x=960; scene.render.resolution_y=540; scene.render.resolution_percentage=100
    scene.view_settings.view_transform="AgX"; world=bpy.data.worlds.new("ReviewWorld"); world.use_nodes=True
    world.node_tree.nodes["Background"].inputs["Color"].default_value=(.38,.43,.5,1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value=.65; scene.world=world
    # Temporary source shell, ground and the unchanged runtime sign: not saved or exported
    bpy.ops.mesh.primitive_cube_add(size=1,location=(0,-20,5.5)); bpy.context.object.dimensions=(32,40,11)
    bpy.ops.mesh.primitive_plane_add(size=110,location=(0,0,-.20))
    sign=bpy.data.materials.new("REVIEW_ONLY_ExistingByteBeatSign"); sign.use_nodes=True
    texture=sign.node_tree.nodes.new("ShaderNodeTexImage"); texture.image=bpy.data.images.load(str(ROOT/"game/assets/environment/signs/byte-beat.png"))
    sign.node_tree.links.new(texture.outputs["Color"],sign.node_tree.nodes["Principled BSDF"].inputs["Base Color"])
    mesh=bpy.data.meshes.new("REVIEW_ONLY_ExistingSign"); mesh.from_pydata([(-3,.365,3.25),(3,.365,3.25),(3,.365,4.75),(-3,.365,4.75)],[],[(3,2,1,0)])
    mesh.materials.append(sign); uv=mesh.uv_layers.new()
    for loop in mesh.loops: uv.data[loop.index].uv=((1,0),(0,0),(0,1),(1,1))[loop.vertex_index]
    obj=bpy.data.objects.new(mesh.name,mesh); scene.collection.objects.link(obj)
    bpy.ops.object.light_add(type="AREA",location=(-15,18,25)); light=bpy.context.object; light.data.energy=11000; light.data.size=14
    light.rotation_euler=(Vector((0,0,4))-light.location).to_track_quat("-Z","Y").to_euler()
    bpy.ops.object.camera_add(); camera=bpy.context.object; scene.camera=camera; camera.data.lens=42
    for name,pos,target in (("north-street",(0,41,12),(0,0,4.8)),("northwest",(-27,32,12),(0,0,4.3)),("equipment",(-11,10,2.3),(-8.6,.2,1.7))):
        camera.location=pos; camera.rotation_euler=(Vector(target)-camera.location).to_track_quat("-Z","Y").to_euler()
        scene.render.filepath=str(OUTPUT/f"{name}.png"); bpy.ops.render.render(write_still=True)


def main():
    parser=argparse.ArgumentParser(description=__doc__); parser.add_argument("--rebuild",action="store_true"); parser.add_argument("--render",action="store_true")
    args=parser.parse_args(sys.argv[sys.argv.index("--")+1:] if "--" in sys.argv else [])
    if args.rebuild: make_master()
    else: bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    export()
    if args.render: render()


if __name__=="__main__": main()
