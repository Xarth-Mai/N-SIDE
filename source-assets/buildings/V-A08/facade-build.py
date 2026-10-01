"""Export the editable V-A08 facade master; --rebuild replaces it with the initial design."""
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
RUNTIME = ROOT / "game/assets/environment/buildings/v-a08-facade.glb"
TEXTURES = ROOT / "source-assets/environment-kit/materials"
OUTPUT = ROOT / "output/buildings/V-A08/facade-r1"
FACES = {"S": ((-9, -6.5), (1, 0), (0, -1), 18), "E": ((9, -6.5), (0, 1), (1, 0), 13),
         "N": ((9, 6.5), (-1, 0), (0, 1), 18), "W": ((-9, 6.5), (0, -1), (-1, 0), 13)}
ROLES = {}


def linear(v):
    return v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4


def material(name, rgb, roughness=.7, metallic=0, texture=None, repeat=(1, 1)):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    rgba = [linear(v) for v in rgb] + [1]
    mat.diffuse_color = rgba
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = rgba
    shader.inputs["Roughness"].default_value = roughness
    shader.inputs["Metallic"].default_value = metallic
    if texture:
        for suffix, space in (("Color", "sRGB"), ("NormalGL", "Non-Color")):
            node = mat.node_tree.nodes.new("ShaderNodeTexImage")
            node.image = bpy.data.images.load(str(TEXTURES / f"{texture}_1K-JPG_{suffix}.jpg"), check_existing=True)
            node.image.colorspace_settings.name = space
            if suffix == "Color":
                mat.node_tree.links.new(node.outputs["Color"], shader.inputs["Base Color"])
            else:
                normal = mat.node_tree.nodes.new("ShaderNodeNormalMap")
                normal.inputs["Strength"].default_value = .28
                mat.node_tree.links.new(node.outputs["Color"], normal.inputs["Color"])
                mat.node_tree.links.new(normal.outputs["Normal"], shader.inputs["Normal"])
    mat["meters_per_repeat"] = repeat
    ROLES[name] = mat
    return mat


def point(face, u, height, depth):
    origin, along, out, _ = FACES[face]
    return (origin[0] + along[0]*u + out[0]*depth, origin[1] + along[1]*u + out[1]*depth, height)


class Piece:
    """One editable architectural group; its simple cuboids share material slots."""
    def __init__(self, name):
        self.name, self.vertices, self.faces, self.roles = name, [], [], []

    def box(self, face, left, right, bottom, top, back, front, role):
        assert right > left and top > bottom and front > back
        base = len(self.vertices)
        self.vertices.extend(point(face, u, z, d) for d in (back, front) for z in (bottom, top) for u in (left, right))
        for corners in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
            self.faces.append(tuple(base+i for i in corners))
            self.roles.append(role)

    def finish(self, collection):
        if not self.vertices:
            return
        mesh = bpy.data.meshes.new(self.name)
        mesh.from_pydata(self.vertices, [], self.faces)
        for mat in ROLES.values():
            mesh.materials.append(mat)
        keys = list(ROLES)
        for poly, role in zip(mesh.polygons, self.roles):
            poly.material_index = keys.index(role)
        bm = bmesh.new()
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        bm.to_mesh(mesh)
        bm.free()
        mesh.update()
        uv = mesh.uv_layers.new(name="UVMap")
        for poly in mesh.polygons:
            axis = max(range(3), key=lambda k: abs(poly.normal[k]))
            axes = ((1, 2), (0, 2), (0, 1))[axis]
            repeat = mesh.materials[poly.material_index]["meters_per_repeat"]
            for loop in poly.loop_indices:
                co = mesh.vertices[mesh.loops[loop].vertex_index].co
                uv.data[loop].uv = (co[axes[0]]/repeat[0], co[axes[1]]/repeat[1])
        obj = bpy.data.objects.new(self.name, mesh)
        collection.objects.link(obj)
        obj["owner_id"] = "V-A08"
        return obj


def wall(face, holes, collection):
    length = FACES[face][3]
    p = Piece(f"VA08_{face}_PlasterCladding")
    heights = sorted({0, .6, 3.62, 4, 7.2, 10.02} | {z for _, _, low, high, _ in holes for z in (low, high)})
    for bottom, top in zip(heights, heights[1:]):
        mid = (bottom+top)/2
        intervals = sorted((left, right) for left, right, low, high, _ in holes if low <= mid <= high)
        cursor = 0
        for left, right in intervals + [(length, length)]:
            if left > cursor:
                role = "Concrete" if top <= .6 else "Plaster"
                p.box(face, cursor, left, bottom, top, .025, .18, role)
            cursor = max(cursor, right)
    p.finish(collection)


def window(face, u, base, width, height, index, collection, guard=False, shade=False):
    p = Piece(f"VA08_{face}_Window_{index:02d}")
    left, right = u-width/2, u+width/2
    low, high = base, base+height
    # Opaque double panes sit in an actual exterior reveal; no fake interior room
    p.box(face, left+.07, u-.035, low+.06, high-.06, .033, .045, "Glazing")
    p.box(face, u+.035, right-.07, low+.06, high-.06, .033, .045, "Glazing")
    for a,b in ((left, left+.065), (right-.065, right), (u-.035, u+.035)):
        p.box(face, a,b, low,high, .045,.225,"Metal")
    for a,b in ((low,low+.065),(high-.065,high)):
        p.box(face,left,right,a,b,.045,.225,"Metal")
    p.box(face,left-.065,right+.065,low-.09,low,.05,.30,"Concrete")
    p.box(face,left-.065,right+.065,high,high+.055,.12,.285,"Metal")
    # Small catches and a stable lower mullion give ordinary scale at the street
    p.box(face,u-.05,u+.05,low+.55,low+.78,.23,.26,"Metal")
    if guard:
        for z in (low-.015, low+.67):
            p.box(face,left-.06,right+.06,z,z+.045,.31,.355,"Metal")
        for j in range(9):
            x=left+(right-left)*j/8
            p.box(face,x-.014,x+.014,low,low+.67,.32,.35,"Metal")
        for x in (left,right):
            p.box(face,x-.025,x+.025,low+.6,low+.645,.18,.35,"Metal")
    if shade:
        p.box(face,left+.085,u-.07,high-.87,high-.08,.09,.105,"WarmPanel")
        p.box(face,left+.06,u-.045,high-.92,high-.865,.09,.16,"Metal")
    p.finish(collection)


def entry(face, u, width, name, collection):
    p=Piece(f"VA08_{name}_Door")
    left,right=u-width/2,u+width/2
    for a,b in ((left-.10,left),(right,right+.10)):
        p.box(face,a,b,0,2.48,.028,.245,"Metal")
    p.box(face,left-.10,right+.10,2.35,2.48,.028,.245,"Metal")
    # Closed facsimiles retain the original public/resident/service dimensions
    if name=="Public":
        for a,b in ((left,u-.025),(u+.025,right)):
            p.box(face,a,b,.05,2.30,.035,.055,"Glazing")
        p.box(face,u-.035,u+.035,.03,2.33,.06,.12,"Metal")
    else:
        p.box(face,left,right,.025,2.35,.035,.07,"WarmPanel")
        for z in (1.67,1.81,1.95):
            p.box(face,left+.12,right-.12,z,z+.035,.07,.075,"Metal")
    p.box(face,right-.24,right-.19,.94,1.20,.08,.13,"Metal")
    p.finish(collection)


def make_master():
    m=json.loads((ROOT/"source-assets/district-map/district.json").read_text())
    b=next(b for b in m["buildings"] if b["id"]=="V-A08")
    assert b["polygon"]==[[70,271],[88,271],[88,284],[70,284]]
    assert abs(b["height"]-10.4)<1e-6 and [round(f["z"]-b["elevation"],5) for f in b["design"]["floors"]]==[0,4,7.2]
    assert [m["nodes"][e["node"]][:2] for e in b["design"]["entries"]]==[[88,277.5],[88,273],[70,277]]
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    bpy.context.scene.unit_settings.system="METRIC"
    bpy.context.scene.unit_settings.scale_length=1
    collection=bpy.data.collections.new("V_A08_EXPORT_Facade")
    bpy.context.scene.collection.children.link(collection)
    material("Plaster",(.83,.82,.76),.87,texture="Plaster001",repeat=(2,2))
    material("Concrete",(.66,.67,.63),.86,texture="Concrete034",repeat=(1.1,.55))
    material("Metal",(.20,.24,.245),.55,.55)
    material("Glazing",(.25,.37,.40),.31,.15)
    material("WarmPanel",(.67,.65,.56),.82)
    material("Ochre",(.68,.43,.22),.75)
    openings={face:[] for face in FACES}
    windows=[]
    index=0
    for face,centers,width in (("S",[2.25,6.75,11.25,15.75],2.65),("E",[2.15,6.5,10.85],2.6),("W",[2.15,6.5,10.85],2.3)):
        for level in (4,7.2):
            for bay,u in enumerate(centers):
                index+=1
                bottom=level+.82; height=1.79
                openings[face].append((u-width/2,u+width/2,bottom,bottom+height,"window"))
                windows.append((face,u,bottom,width,height,index,face=="E" and bay!=1,level==4 and bay%2==0))
    # The north facade is an actual one-metre maintenance gap, not a new balcony frontage
    for face,centers,width,height in (("S",[2.25,6.75,11.25,15.75],2.65,1.5),("E",[10.55],3.0,1.85),("W",[2.3,10.6],2.3,1.25)):
        for u in centers:
            index+=1
            openings[face].append((u-width/2,u+width/2,.9,.9+height,"window"))
            windows.append((face,u,.9,width,height,index,False,False))
    for face,u,width,name in (("E",6.5,1.8,"Public"),("E",2,1.3,"Resident"),("W",7,1.3,"Service")):
        openings[face].append((u-width/2-.10,u+width/2+.10,0,2.48,"door"))
        entry(face,u,width,name,collection)
    for face,holes in openings.items():
        wall(face,holes,collection)
    for face,u,bottom,width,height,index,guard,shade in windows:
        window(face,u,bottom,width,height,index,collection,guard,shade)
    for face in FACES:
        p=Piece(f"VA08_{face}_LayerAndCorner")
        length=FACES[face][3]
        # Horizontal reveals track existing floor datums; no new floor or roof level
        for z in (3.87,7.07):
            p.box(face,0,length,z,z+.13,.185,.235,"Concrete")
        for u in (0,length-.22):
            p.box(face,u,u+.22,0,10.02,.185,.255,"Concrete")
        p.finish(collection)
    for face,u in (("E",6.5),("S",6.75),("W",10.85)):
        p=Piece(f"VA08_{face}_Condenser")
        p.box(face,u-.48,u+.48,4.16,4.67,.20,.64,"WarmPanel")
        for z in (4.26,4.34,4.42,4.50,4.58):
            p.box(face,u-.40,u+.40,z,z+.025,.643,.657,"Metal")
        for x in (u-.31,u+.31):
            p.box(face,x-.025,x+.025,4.05,4.12,.08,.64,"Metal")
            p.box(face,x-.025,x+.025,4.05,4.40,.08,.13,"Metal")
        # Insulated services run below and beside the opening, never through the pane
        p.box(face,u+.48,u+1.58,4.44,4.495,.20,.25,"WarmPanel")
        p.box(face,u+1.53,u+1.58,4.44,5.15,.20,.25,"WarmPanel")
        p.finish(collection)
    p=Piece("VA08_E_EntryCanopies")
    for left,right,depth in ((4.7,8.3,1.35),(.85,3.15,.7)):
        p.box("E",left,right,2.84,2.98,.18,depth,"Metal")
        p.box("E",left+.08,right-.08,2.83,2.845,.24,depth-.08,"WarmPanel")
        p.box("E",left,right,2.99,3.10,depth-.045,depth,"Ochre")
    p.finish(collection)
    p=Piece("VA08_E_Rainwater")
    for u in (.43,12.57):
        p.box("E",u-.045,u+.045,.07,10.02,.26,.35,"Metal")
        for z in (.4,3.6,6.8,9.6):
            p.box("E",u-.085,u+.085,z,z+.055,.19,.37,"Metal")
        p.box("E",u-.05,u+.05,9.94,10.06,.25,.45,"Metal")
    p.finish(collection)
    scene=bpy.context.scene
    scene["owner_id"]="V-A08"
    scene["pivot_map_xyz"]=[79,277.5,b["elevation"]]
    scene["source_building"] = json.dumps(b,ensure_ascii=False)
    scene["door_nodes"] = json.dumps({e["node"]:m["nodes"][e["node"]] for e in b["design"]["entries"]})
    scene["window_count"]=len(windows)
    for image in bpy.data.images:
        if image.filepath:
            image.pack()
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))


def export():
    collection=bpy.data.collections["V_A08_EXPORT_Facade"]
    bpy.ops.object.select_all(action="DESELECT")
    copies=[]
    for source in collection.objects:
        obj=source.copy(); obj.data=source.data.copy(); bpy.context.scene.collection.objects.link(obj)
        obj.select_set(True); copies.append(obj)
    bpy.context.view_layer.objects.active=copies[0]
    bpy.ops.object.join()
    obj=bpy.context.object
    obj.name="V_A08_Facade"
    # All source groups are already at the shared origin; only the export copy is joined
    RUNTIME.parent.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(RUNTIME),export_format="GLB",use_selection=True,export_yup=True,export_animations=False,export_cameras=False,export_lights=False,export_extras=False,export_materials="EXPORT")
    bpy.data.objects.remove(obj,do_unlink=True)


def render():
    OUTPUT.mkdir(parents=True,exist_ok=True)
    scene=bpy.context.scene
    scene.render.engine="CYCLES"; scene.cycles.device="CPU"; scene.cycles.samples=28
    scene.render.threads_mode="FIXED";scene.render.threads=6
    scene.render.resolution_x=1200;scene.render.resolution_y=900;scene.render.resolution_percentage=100
    scene.view_settings.view_transform="AgX"
    scene.world.color=(.30,.30,.30)
    # Source map body and unchanged eaves are preview context only, never saved/exported here
    bpy.ops.mesh.primitive_cube_add(size=1,location=(0,0,5.2));bpy.context.object.dimensions=(18,13,10.4)
    roof_mat=bpy.data.materials.new("ReviewBodyOnly");roof_mat.diffuse_color=(.25,.30,.28,1);bpy.context.object.data.materials.append(roof_mat)
    with bpy.data.libraries.load(str(HERE/"roof-eaves.blend"),link=False) as (data_from,data_to):
        data_to.objects=["V_A08_RoofEaves"]
    roof=data_to.objects[0];bpy.context.scene.collection.objects.link(roof);roof.location.z=10.4
    bpy.ops.object.light_add(type="AREA",location=(17,-20,23));light=bpy.context.object
    light.data.energy=7500;light.data.size=13
    light.rotation_euler=(Vector((0,0,5))-light.location).to_track_quat("-Z","Y").to_euler()
    bpy.ops.object.camera_add();camera=bpy.context.object;scene.camera=camera;camera.data.type="PERSP";camera.data.lens=43
    for name,pos,target in (("southeast",(29,-30,16),(0,0,4.8)),("east-entry",(25,-3.5,4),(8,0,3.9)),("window-detail",(14,-5,8.5),(9,-.2,5.7)),("southwest",(-27,-30,13),(0,0,4.6))):
        camera.location=pos;camera.rotation_euler=(Vector(target)-camera.location).to_track_quat("-Z","Y").to_euler()
        scene.render.filepath=str(OUTPUT/f"{name}.png");bpy.ops.render.render(write_still=True)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rebuild",action="store_true")
    parser.add_argument("--render",action="store_true")
    args=parser.parse_args(sys.argv[sys.argv.index("--")+1:] if "--" in sys.argv else [])
    if args.rebuild:
        make_master()
    else:
        bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    export()
    if args.render:
        render()


if __name__=="__main__":main()
