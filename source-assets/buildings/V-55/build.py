"""Author/export V-55's two-face workshop facade; the .blend remains the editable master."""
import argparse
import json
from pathlib import Path
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MASTER = HERE / "facade.blend"
OUTPUT = ROOT / "output/buildings/V-55"
GLB = ROOT / "game/assets/environment/buildings/v55-workshop-facade.glb"
TEXTURES = ROOT / "source-assets/environment-kit/materials"
FONT = ROOT / "source-assets/ui-kit/fonts/NotoSansSC-VF.ttf"
FACES = {"W": ((-6, 8), (0, -1), (-1, 0), 16), "S": ((-6, -8), (1, 0), (0, -1), 12)}
ROLES = {}


def linear(v):
    return v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4


def material(name, color, roughness=.7, metallic=0, texture=None, repeat=(1, 1)):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    rgba = [linear(v) for v in color] + [1]
    mat.diffuse_color = rgba
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = rgba
    shader.inputs["Roughness"].default_value = roughness
    shader.inputs["Metallic"].default_value = metallic
    if texture:
        for suffix, space in (("Color", "sRGB"), ("NormalGL", "Non-Color")):
            tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
            tex.image = bpy.data.images.load(str(TEXTURES / f"{texture}_1K-JPG_{suffix}.jpg"), check_existing=True)
            tex.image.colorspace_settings.name = space
            if suffix == "Color":
                mat.node_tree.links.new(tex.outputs["Color"], shader.inputs["Base Color"])
            else:
                normal = mat.node_tree.nodes.new("ShaderNodeNormalMap")
                normal.inputs["Strength"].default_value = .25
                mat.node_tree.links.new(tex.outputs["Color"], normal.inputs["Color"])
                mat.node_tree.links.new(normal.outputs["Normal"], shader.inputs["Normal"])
    mat["meters_per_repeat"] = repeat
    ROLES[name] = mat


def point(face, u, z, depth):
    origin, along, outward, _ = FACES[face]
    return (origin[0] + along[0] * u + outward[0] * depth,
            origin[1] + along[1] * u + outward[1] * depth, z)


def metric_uv(mesh):
    uv = mesh.uv_layers.active or mesh.uv_layers.new(name="UVMap")
    for poly in mesh.polygons:
        axis = max(range(3), key=lambda k: abs(poly.normal[k]))
        axes = ((1, 2), (0, 2), (0, 1))[axis]
        repeat = mesh.materials[poly.material_index]["meters_per_repeat"]
        for loop in poly.loop_indices:
            co = mesh.vertices[mesh.loops[loop].vertex_index].co
            uv.data[loop].uv = (co[axes[0]] / repeat[0], co[axes[1]] / repeat[1])


class Piece:
    def __init__(self, name):
        self.name, self.vertices, self.faces, self.roles = name, [], [], []

    def box(self, face, left, right, bottom, top, back, front, role):
        assert right > left and top > bottom and front > back
        offset = len(self.vertices)
        self.vertices.extend(point(face, u, z, d) for d in (back, front) for z in (bottom, top) for u in (left, right))
        for ids in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
            self.faces.append(tuple(offset + i for i in ids))
            self.roles.append(role)

    def finish(self, collection):
        mesh = bpy.data.meshes.new(self.name)
        mesh.from_pydata(self.vertices, [], self.faces)
        for mat in ROLES.values():
            mesh.materials.append(mat)
        names = list(ROLES)
        for poly, role in zip(mesh.polygons, self.roles):
            poly.material_index = names.index(role)
        bm = bmesh.new(); bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        bm.to_mesh(mesh); bm.free(); mesh.update()
        metric_uv(mesh)
        obj = bpy.data.objects.new(self.name, mesh)
        collection.objects.link(obj)
        obj["owner_id"] = "V-55"


def label(name, body, face, u, z, depth, height, max_width, role, collection, font):
    curve = bpy.data.curves.new(name, "FONT")
    curve.body = body; curve.font = font; curve.align_x = "CENTER"
    curve.size = height; curve.extrude = .002; curve.resolution_u = 2
    curve.offset = .006 if height > .2 else .002
    obj = bpy.data.objects.new(name, curve); collection.objects.link(obj)
    _, along, outward, _ = FACES[face]
    obj.matrix_world = Matrix(((along[0], 0, outward[0], 0), (along[1], 0, outward[1], 0), (0, 1, 0, 0), (0, 0, 0, 1)))
    obj.location = point(face, u, z, depth)
    curve.materials.append(ROLES[role]); obj["owner_id"] = "V-55"
    bpy.context.view_layer.update()
    local = [Vector(p) for p in obj.bound_box]
    actual_height = max(p.y for p in local) - min(p.y for p in local)
    width = max(p.x for p in local) - min(p.x for p in local)
    scale = min(height / actual_height, max_width / width)
    obj.scale = (scale, scale, scale)


def wall(face, openings, collection):
    length = FACES[face][3]
    p = Piece(f"V55_{face}_PlasterAndPlinth")
    levels = sorted({-.55, 0, .6, 3, 6, 8.83} | {z for _, _, low, high in openings for z in (low, high)})
    for low, high in zip(levels, levels[1:]):
        intervals = sorted((a, b) for a, b, z0, z1 in openings if z0 <= (low + high) / 2 <= z1)
        cursor = 0
        for left, right in intervals + [(length, length)]:
            if left > cursor:
                p.box(face, cursor, left, low, high, .024, .12, "Concrete" if high <= .6 else "Plaster")
            cursor = max(cursor, right)
    p.finish(collection)


def window(face, u, low, width, height, name, collection, louvers=False):
    p = Piece(name)
    left, right, high = u - width / 2, u + width / 2, low + height
    for a, b in ((left + .06, u - .03), (u + .03, right - .06)):
        p.box(face, a, b, low + .055, high - .055, .024, .039, "Glass")
    for a, b in ((left, left + .055), (u - .03, u + .03), (right - .055, right)):
        p.box(face, a, b, low, high, .04, .19, "Graphite")
    for a, b in ((low, low + .055), (high - .055, high)):
        p.box(face, left, right, a, b, .04, .19, "Graphite")
    p.box(face, left - .045, right + .045, low - .075, low, .08, .25, "Concrete")
    if louvers:
        for i in range(6):
            a = left + .13 + i * .16
            p.box(face, a, a + .035, low + .06, high - .06, .19, .34, "Indigo")
        for a, b in ((low + .05, low + .095), (high - .095, high - .05)):
            p.box(face, left + .07, left + 1.015, a, b, .19, .27, "Graphite")
    p.finish(collection)


def entry(u, width, name, collection):
    p = Piece(f"V55_W_{name}Door")
    left, right = u - width / 2, u + width / 2
    for a, b in ((left - .09, left), (right, right + .09)):
        p.box("W", a, b, 0, 2.48, .024, .20, "Graphite")
    p.box("W", left - .09, right + .09, 2.35, 2.48, .024, .20, "Graphite")
    p.box("W", left, right, .02, 2.35, .024, .044, "Glass" if name == "Public" else "Graphite")
    if name == "Public":
        p.box("W", u - .026, u + .026, .02, 2.35, .045, .095, "Graphite")
        p.box("W", left, right, .02, .43, .045, .06, "Indigo")
    p.box("W", right - .23, right - .19, .92, 1.19, .055, .105, "Paper")
    depth = .75 if name == "Public" else .45
    p.box("W", left - .36, right + .36, 2.64, 2.77, .13, depth, "Indigo" if name == "Public" else "Graphite")
    p.box("W", left - .3, right + .3, 2.642, 2.66, .22, depth - .06, "Paper")
    p.finish(collection)


def display(collection):
    p = Piece("V55_W_ShallowWorkSamples")
    left, right, low, high = 5.3, 10.7, .72, 2.30
    p.box("W", left, right, low, high, .022, .044, "Graphite")
    for a, b in ((left, left + .08), (7.96, 8.04), (right - .08, right)):
        p.box("W", a, b, low, high, .043, .29, "Indigo")
    for a, b in ((low, low + .09), (high - .08, high), (1.37, 1.43)):
        p.box("W", left, right, a, b, .043, .29, "Indigo")
    # Customer objects are not invented: these are anonymous packaging and cloth samples
    for i, u in enumerate((5.8, 6.47, 7.16, 8.45, 9.16, 9.83)):
        bottom = .815
        height = (.35, .48, .29, .26, .37, .24)[i]
        p.box("W", u - .20, u + .20, bottom, bottom + height, .075, .255, "Paper")
        if i < 3:
            p.box("W", u - .025, u + .025, bottom, bottom + height + .002, .256, .261, "Indigo")
            p.box("W", u - .20, u + .20, bottom + height * .45, bottom + height * .45 + .04, .257, .264, "Indigo")
        else:
            for z in range(2):
                h = bottom + .075 + z * .08
                p.box("W", u - .205, u + .205, h, h + .016, .08, .264, "Indigo" if i == 4 else "Graphite")
    for i, u in enumerate((5.91, 6.85, 8.63, 9.57)):
        p.box("W", u - .28, u + .28, 1.58, 2.12, .06, .082, "Paper" if i % 2 == 0 else "Indigo")
        p.box("W", u - .055, u + .055, 2.09, 2.17, .08, .108, "Paper")
        for j in range(5):
            a = u - .205 + j * .09
            p.box("W", a, a + .045, 1.63, 1.646, .083, .087, "Graphite")
    p.finish(collection)


def make_master():
    district = json.loads((ROOT / "source-assets/district-map/district.json").read_text())
    b = next(b for b in district["buildings"] if b["id"] == "V-55")
    assert b["polygon"] == [[148, 198], [160, 198], [160, 214], [148, 214]] and b["elevation"] == 24 and b["height"] == 9
    assert [f["z"] for f in b["design"]["floors"]] == [24, 27, 30]
    assert [district["nodes"][e["node"]] for e in b["design"]["entries"]] == [[148, 202, 24], [148, 210, 24]]
    bpy.ops.object.select_all(action="SELECT"); bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene; scene.unit_settings.system = "METRIC"; scene.unit_settings.scale_length = 1
    collection = bpy.data.collections.new("V55_EXPORT_Workshop"); scene.collection.children.link(collection)
    material("Plaster", (.82, .81, .76), .87, texture="Plaster001", repeat=(2, 2))
    material("Concrete", (.66, .67, .63), .86, texture="Concrete034", repeat=(1.1, .55))
    material("Graphite", (.18, .21, .24), .52, .45)
    material("Indigo", (.39, .39, .68), .5, .22)
    material("Glass", (.23, .34, .38), .29, .12)
    material("Paper", (.92, .84, .66), .8)
    openings = {face: [] for face in FACES}
    windows = []
    for face, centers in (("W", (2, 6, 10, 14)), ("S", (2, 6, 10))):
        for floor in (3, 6):
            for i, u in enumerate(centers):
                width = 2.55
                openings[face].append((u - width / 2, u + width / 2, floor + .68, floor + 2.45))
                windows.append((face, u, floor + .68, width, 1.77, f"V55_{face}_Workroom_{floor}_{i}", face == "W" and i == 0))
    for u in (3, 9):
        openings["S"].append((u - 1.6, u + 1.6, .78, 2.35))
        windows.append(("S", u, .78, 3.2, 1.57, f"V55_S_Workroom_Ground_{u}", False))
    openings["W"].append((5.3, 10.7, .72, 2.30))
    for u, width, name in ((12, 1.8, "Public"), (4, 1.3, "Service")):
        openings["W"].append((u - width / 2 - .09, u + width / 2 + .09, 0, 2.48))
        entry(u, width, name, collection)
    for face in FACES:
        wall(face, openings[face], collection)
        p = Piece(f"V55_{face}_FloorEdges")
        for z in (2.97, 5.97):
            p.box(face, 0, FACES[face][3], z, z + .075, .12, .175, "Graphite")
        p.box(face, 0, FACES[face][3], 8.83, 9, .02, .19, "Concrete")
        p.finish(collection)
    for face, u, low, width, height, name, louvers in windows:
        window(face, u, low, width, height, name, collection, louvers)
    display(collection)
    p = Piece("V55_SW_CornerReturn")
    p.box("W", 15.82, 16.18, -.55, 9, -.02, .18, "Concrete")
    p.finish(collection)
    p = Piece("V55_W_Identification")
    p.box("W", 5.18, 10.82, 2.37, 2.91, .121, .18, "Indigo")
    p.box("W", 3.20, 4.80, 2.37, 2.58, .202, .216, "Graphite")
    p.box("W", 13.75, 15.55, .74, 2.34, .121, .17, "Indigo")
    # A fold/cut pattern identifies wrapping and repair without inventing a brand
    for u in (13.97, 15.33):
        p.box("W", u - .015, u + .015, .98, 2.09, .171, .18, "Paper")
    for z in (.98, 2.09):
        p.box("W", 13.97, 15.33, z - .015, z + .015, .171, .18, "Paper")
    for i in range(8):
        a = 14.03 + i * .155
        p.box("W", a, a + .072, 1.16, 1.186, .171, .18, "Paper")
    for u in (14.43, 14.77):
        p.box("W", u, u + .03, 1.18, 1.9, .171, .18, "Paper")
    p.finish(collection)
    font = bpy.data.fonts.load(str(FONT)); font.pack()
    label("V55_PurposeTitle", "包装 / 修补", "W", 8, 2.39, .184, .39, 4.9, "Paper", collection, font)
    label("V55_ServiceLabel", "收件", "W", 4, 2.385, .22, .15, 1.2, "Paper", collection, font)
    label("V55_SampleLabel", "样品", "W", 14.65, 1.42, .185, .28, 1.08, "Paper", collection, font)
    scene["owner_id"] = "V-55"; scene["pivot_map_xyz"] = [154, 206, 24]
    scene["source_building"] = json.dumps(b, ensure_ascii=False)
    scene["door_nodes"] = json.dumps({e["node"]: district["nodes"][e["node"]] for e in b["design"]["entries"]})
    scene["window_count"] = len(windows); scene["replaced_faces"] = "West and South; retain original structural shell and East/North generic details"
    for image in bpy.data.images:
        if image.filepath: image.pack()
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))


def export():
    collection = bpy.data.collections["V55_EXPORT_Workshop"]
    bpy.ops.object.select_all(action="DESELECT")
    copies = []
    for source in collection.objects:
        obj = source.copy(); obj.data = source.data.copy(); bpy.context.scene.collection.objects.link(obj)
        obj.select_set(True); copies.append(obj)
    bpy.context.view_layer.objects.active = copies[0]
    bpy.ops.object.convert(target="MESH")
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.join(); obj = bpy.context.object; obj.name = "V55_WorkshopFacade"
    bm = bmesh.new(); bm.from_mesh(obj.data)
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if f.calc_area() < 1e-9], context="FACES")
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces)); bm.to_mesh(obj.data); bm.free(); obj.data.update()
    for poly in obj.data.polygons: poly.use_smooth = False
    metric_uv(obj.data)
    GLB.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(GLB), export_format="GLB", use_selection=True, export_yup=True,
                              export_animations=False, export_cameras=False, export_lights=False, export_extras=False, export_materials="EXPORT")
    bpy.data.objects.remove(obj, do_unlink=True)


def render():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    scene = bpy.context.scene; scene.render.engine = "CYCLES"; scene.cycles.device = "CPU"; scene.cycles.samples = 12
    scene.render.threads_mode = "FIXED"; scene.render.threads = 2
    scene.render.resolution_x = 640; scene.render.resolution_y = 480; scene.render.resolution_percentage = 100
    scene.view_settings.view_transform = "AgX"; scene.world.color = (.3, .3, .3)
    # Source shell and ground are review context only, neither saved nor exported
    bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, 4.5)); obj = bpy.context.object; obj.dimensions = (12, 16, 9)
    mat = bpy.data.materials.new("REVIEW_ONLY_Shell"); mat.diffuse_color = (.40, .42, .38, 1); obj.data.materials.append(mat)
    bpy.ops.mesh.primitive_plane_add(size=60); bpy.context.object.location.z = -.12
    bpy.ops.object.light_add(type="AREA", location=(-15, -18, 23)); light = bpy.context.object
    light.data.energy = 7000; light.data.size = 12
    light.rotation_euler = (Vector((0, 0, 4)) - light.location).to_track_quat("-Z", "Y").to_euler()
    bpy.ops.object.camera_add(); camera = bpy.context.object; scene.camera = camera; camera.data.lens = 44
    for name, pos, target in (("southwest", (-24, -26, 13), (0, 0, 3.9)), ("west-street", (-23, -2, 6), (-6, 0, 4)), ("display", (-12, -.2, 2.4), (-6, 0, 1.5))):
        camera.location = pos; camera.rotation_euler = (Vector(target) - camera.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = str(OUTPUT / f"{name}.png"); bpy.ops.render.render(write_still=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rebuild", action="store_true")
    parser.add_argument("--render", action="store_true")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    if args.rebuild: make_master()
    else: bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    export()
    if args.render: render()


if __name__ == "__main__": main()
