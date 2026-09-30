"""Build explicitly, or export the editable V-A08 roof master with local Blender."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import bmesh
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MASTER = HERE / "roof-eaves.blend"
RUNTIME = ROOT / "game/assets/environment/buildings/v-a08-roof-eaves.glb"
PROFILE = [(0, 0), (.28, -.062), (.33, -.12), (.42, -.12), (.45, -.09), (.45, -.35), (-.03, -.35)]


def linear(value):
    return value / 12.92 if value <= .04045 else ((value + .055) / 1.055) ** 2.4


def create_master():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    bpy.context.scene.unit_settings.system = "METRIC"
    bpy.context.scene.unit_settings.scale_length = 1
    appearance_path = ROOT / "source-assets/district-scene/appearance.json"
    appearance = json.loads(appearance_path.read_text())
    materials = []
    for name in ("roof", "metal", "trim"):
        spec = appearance["materials"][name]
        material = bpy.data.materials.new(name)
        material.use_nodes = True
        shader = material.node_tree.nodes.get("Principled BSDF")
        rgba = [linear(channel) for channel in spec["color"][:3]] + [spec["color"][3]]
        shader.inputs["Base Color"].default_value = rgba
        shader.inputs["Roughness"].default_value = spec["roughness"]
        shader.inputs["Metallic"].default_value = spec["metallic"]
        material.diffuse_color = rgba
        materials.append(material)
    corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)]
    vertices = [(sx * (9 + offset), sy * (6.5 + offset), z) for offset, z in PROFILE for sx, sy in corners]
    faces = []
    for ring in range(len(PROFILE)):
        next_ring = (ring + 1) % len(PROFILE)
        for corner in range(4):
            next_corner = (corner + 1) % 4
            faces.append((4 * ring + corner, 4 * ring + next_corner, 4 * next_ring + next_corner, 4 * next_ring + corner))
    mesh = bpy.data.meshes.new("V_A08_FoldedEavesMesh")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new("V_A08_RoofEaves", mesh)
    bpy.context.collection.objects.link(obj)
    for material in materials:
        mesh.materials.append(material)
    for face in mesh.polygons:
        face.material_index = [0, 1, 1, 1, 2, 2, 2][face.index // 4]
    editable = bmesh.new()
    editable.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(editable, faces=list(editable.faces))
    if editable.calc_volume(signed=True) < 0:
        bmesh.ops.reverse_faces(editable, faces=list(editable.faces))
    editable.to_mesh(mesh)
    editable.free()
    mesh.update()
    uv = mesh.uv_layers.new(name="UVMap")
    for face in mesh.polygons:
        origin = mesh.vertices[face.vertices[0]].co
        along = (mesh.vertices[face.vertices[1]].co - origin).normalized()
        across = face.normal.cross(along).normalized()
        for loop_index in face.loop_indices:
            relative = mesh.vertices[mesh.loops[loop_index].vertex_index].co - origin
            uv.data[loop_index].uv = (relative.dot(along), relative.dot(across))
    obj["owner_id"] = "V-A08"
    obj["asset_id"] = "AST-V-A08-ROOF"
    obj["pivot_map_xyz"] = [79, 277.5, 40.421515]
    obj["appearance_sha256_at_build"] = hashlib.sha256(appearance_path.read_bytes()).hexdigest()
    obj["material_source"] = "source-assets/district-scene/appearance.json: roof, metal, trim; sRGB converted to linear"
    obj["profile_offsets_heights"] = json.dumps(PROFILE)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))


def export():
    bpy.ops.object.select_all(action="DESELECT")
    obj = bpy.data.objects["V_A08_RoofEaves"]
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    RUNTIME.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(RUNTIME), export_format="GLB", use_selection=True,
                             export_animations=False, export_yup=True, export_cameras=False,
                             export_lights=False, export_materials="EXPORT", export_extras=False)


def preview(output):
    output.mkdir(parents=True, exist_ok=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 24
    scene.render.resolution_x = 960
    scene.render.resolution_y = 720
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "AgX"
    scene.world.color = (.28, .28, .28)
    # A temporary measured body locates the eave; it is never saved or exported
    body_mat = bpy.data.materials.new("ReviewContextOnly")
    body_mat.diffuse_color = (.53, .50, .43, 1)
    bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, -5.2))
    body = bpy.context.object
    body.name = "ReviewContext_NotExported"
    body.dimensions = (18, 13, 10.4)
    body.data.materials.append(body_mat)
    bpy.ops.object.light_add(type="AREA", location=(10, -14, 18))
    light = bpy.context.object
    light.data.energy = 5500
    light.data.shape = "DISK"
    light.data.size = 12
    light.rotation_euler = (Vector((0, 0, -2)) - light.location).to_track_quat("-Z", "Y").to_euler()
    bpy.ops.object.camera_add()
    camera = bpy.context.object
    scene.camera = camera
    camera.data.type = "ORTHO"
    for name, position, target, scale in (
        ("overall", (25, -29, 14), (0, 0, -3), 30),
        ("gutter-corner", (14, -13, 4), (8.55, -6.1, -.15), 3.6),
        ("street-under-eave", (14, -13, -3.5), (8.8, -6.25, -.25), 4.3),
    ):
        camera.location = position
        camera.rotation_euler = (Vector(target) - camera.location).to_track_quat("-Z", "Y").to_euler()
        camera.data.ortho_scale = scale
        scene.render.filepath = str(output / f"{name}.png")
        bpy.ops.render.render(write_still=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rebuild", action="store_true", help="Replace the editable master from the initial recipe")
    parser.add_argument("--render-output", type=Path)
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    if options.rebuild:
        create_master()
    elif MASTER.is_file():
        bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    else:
        parser.error("master missing; use --rebuild explicitly for initial construction")
    export()
    if options.render_output:
        preview(options.render_output)


if __name__ == "__main__":
    main()
