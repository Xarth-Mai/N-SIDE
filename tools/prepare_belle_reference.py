"""Prepare the inspected Ghost73 Belle glTF for a local N:SIDE comparison

Run with Blender --python-exit-code 1 --python this_file -- --source ... --output ...
Third-party models are supplied separately; outputs stay in ignored output/
This is a specific rig conversion, not a general humanoid retargeter
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys

import bpy
from mathutils import Matrix, Quaternion

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--source", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
source, output = args.source.resolve(), args.output.resolve()
if not source.is_file() or source.suffix != ".gltf":
    parser.error("--source must be the extracted Belle .gltf file")
if not output.is_relative_to(ROOT / "output") or output.exists():
    parser.error("--output must be a new directory inside this repository's output/")
data = json.loads(source.read_text())
dependencies = [source]
for item in [*data.get("buffers", []), *data.get("images", [])]:
    dependency = (source.parent / item["uri"]).resolve()
    if not dependency.is_relative_to(source.parent) or not dependency.is_file():
        parser.error(f"Missing or nonlocal reference dependency: {item['uri']}")
    dependencies.append(dependency)
output.mkdir(parents=True)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(source))
scene = bpy.context.scene
rigs = [obj for obj in scene.objects if obj.type == "ARMATURE"]
if len(rigs) != 1 or len(rigs[0].data.bones) != 148 or bpy.data.actions:
    raise ValueError("Expected the inspected 148-joint Belle rig without source animation")
rig = rigs[0]
meshes = [obj for obj in scene.objects if obj.type == "MESH"
          and any(mod.type == "ARMATURE" and mod.object == rig for mod in obj.modifiers)]
if not meshes:
    raise ValueError("No meshes bound to the reference rig")
for obj in list(scene.objects):
    if obj not in meshes and obj != rig:
        bpy.data.objects.remove(obj, do_unlink=True)
points = [obj.matrix_world @ vertex.co for obj in meshes for vertex in obj.data.vertices]
low = min(point.z for point in points)
height = max(point.z for point in points) - low
scale = 1.65 / height
transform = Matrix.Diagonal((scale, scale, scale, 1.0))
transform.translation.z = -low * scale
for obj in [rig, *meshes]:
    if max(abs(obj.matrix_world[row][col] - (row == col))
           for row in range(4) for col in range(4)) > 1e-5:
        raise ValueError(f"Expected identity object transform: {obj.name}")
    if obj.type == "MESH":
        obj.data.transform(transform, shape_keys=True)
        if obj.data.shape_keys and any((point.co - vertex.co).length > 1e-5
                for point, vertex in zip(obj.data.shape_keys.key_blocks[0].data, obj.data.vertices)):
            raise ValueError(f"Shape-key Basis no longer matches scaled geometry: {obj.name}")
    else:
        obj.data.transform(transform)
rig.name = "ReferenceBelle"
bpy.ops.object.select_all(action="DESELECT")
rig.select_set(True)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
root = rig.data.edit_bones.new("Root")
root.head, root.tail = (0, 0, 0), (0, 0, .1)
rig.data.edit_bones["ValveBiped.Bip01_Pelvis"].parent = root
bpy.ops.object.mode_set(mode="OBJECT")

motion_file = ROOT / "source-assets/characters/CHR-002/model/ling-grey-study.blend"
with bpy.data.libraries.load(str(motion_file), link=False) as (_, imported):
    imported.objects = ["CHR002_Rig"]
    imported.actions = ["Idle", "Walk", "Run"]
motion = imported.objects[0]
scene.collection.objects.link(motion)
actions = {action.name: action for action in imported.actions}
prefix = "ValveBiped.Bip01_"
mapping = {"Hips": prefix + "Pelvis", "Spine": prefix + "Spine", "Chest": prefix + "Spine2",
           "Neck": prefix + "Neck1", "Head": prefix + "Head1"}
for side in ["L", "R"]:
    for src, target in [("Clavicle", "Clavicle"), ("UpperArm", "UpperArm"),
                        ("Forearm", "Forearm"), ("Hand", "Hand"), ("Thigh", "Thigh"),
                        ("Shin", "Calf"), ("Foot", "Foot"), ("Toe", "Toe0")]:
        mapping[src + "." + side] = prefix + side + "_" + target
for src, target in mapping.items():
    if src not in motion.pose.bones or target not in rig.pose.bones:
        raise ValueError(f"Missing mapped joint: {src} -> {target}")
pairs = sorted(mapping.items(), key=lambda pair: len(rig.pose.bones[pair[1]].parent_recursive))
# SFM tails are axis markers; use shoulder, elbow and wrist joint positions
rest_align = {name: Quaternion() for name in mapping}
for side in ["L", "R"]:
    for parent, child in [("UpperArm", "Forearm"), ("Forearm", "Hand")]:
        name, child_name = parent + "." + side, child + "." + side
        target_direction = (rig.data.bones[mapping[child_name]].head_local
                            - rig.data.bones[mapping[name]].head_local)
        source_direction = motion.data.bones[child_name].head_local - motion.data.bones[name].head_local
        rest_align[name] = target_direction.rotation_difference(source_direction)
    rest_align["Hand." + side] = rest_align["Forearm." + side].copy()

scene.render.fps = 60
made = []
pose_samples = 0
for name, frames in [("Idle", 120), ("Walk", 48), ("Run", 40)]:
    motion.animation_data.action = actions[name]
    actions[name].name = "Source_" + name
    rig.animation_data_clear()
    action = bpy.data.actions.new(name)
    rig.animation_data_create()
    rig.animation_data.action = action
    for frame in range(1, frames + 2):
        scene.frame_set(frame)
        for bone in rig.pose.bones:
            bone.matrix_basis = Matrix.Identity(4)
        bpy.context.view_layer.update()
        for src, target in pairs:
            sp, tp = motion.pose.bones[src], rig.pose.bones[target]
            rotation = (sp.matrix.to_quaternion() @ sp.bone.matrix_local.to_quaternion().inverted()
                        @ rest_align[src] @ tp.bone.matrix_local.to_quaternion())
            position = tp.parent.matrix @ tp.parent.bone.matrix_local.inverted() @ tp.bone.head_local
            if src == "Hips":
                position += (sp.matrix.translation - sp.bone.head_local) * (1.65 / 1.641)
            desired = rotation.to_matrix().to_4x4()
            desired.translation = position
            tp.rotation_mode, tp.matrix = "QUATERNION", desired
            tp.keyframe_insert("rotation_quaternion", frame=frame, group=target)
            if src == "Hips":
                tp.keyframe_insert("location", frame=frame, group=target)
            bpy.context.view_layer.update()
        pelvis = rig.pose.bones[mapping["Hips"]].head.z
        if (rig.pose.bones[mapping["Head"]].head.z < pelvis + .3
                or any(rig.pose.bones[mapping["Foot." + side]].head.z > pelvis - .2 for side in ["L", "R"])
                or rig.pose.bones["Root"].matrix.translation.length > 1e-6):
            raise ValueError(f"Reference pose failed head/pelvis/feet/root checks: {name} frame {frame}")
        pose_samples += 1
    action.use_fake_user = True
    made.append(action)
bpy.data.objects.remove(motion, do_unlink=True)
for action in actions.values():
    bpy.data.actions.remove(action)
for material in bpy.data.materials:
    if material.use_nodes:
        bsdf = next((node for node in material.node_tree.nodes if node.type == "BSDF_PRINCIPLED"), None)
        if bsdf:
            bsdf.inputs["Specular IOR Level"].default_value = .5
scene.name = "Scene"
rig.animation_data.action = made[0]
scene.frame_set(1)
bpy.ops.object.select_all(action="DESELECT")
for obj in [rig, *meshes]:
    obj.select_set(True)
bpy.context.view_layer.objects.active = rig
for image in bpy.data.images:
    if image.source == "FILE":
        image.pack()
bpy.ops.wm.save_as_mainfile(filepath=str(output / "reference.blend"))
glb = output / "reference.glb"
bpy.ops.export_scene.gltf(filepath=str(glb), use_selection=True, export_format="GLB", export_yup=True,
                         export_animations=True, export_animation_mode="ACTIONS",
                         export_anim_slide_to_zero=True, export_skins=True, export_morph=False)
# Local overlay uses the existing character slot, without replacing tracked assets
project = output / "project"
assets = project / "game/assets"
characters = assets / "characters"
characters.mkdir(parents=True)
(project / "source-assets").symlink_to(ROOT / "source-assets", target_is_directory=True)
for entry in (ROOT / "game/assets").iterdir():
    if entry.name != "characters":
        (assets / entry.name).symlink_to(entry, target_is_directory=entry.is_dir())
for entry in (ROOT / "game/assets/characters").iterdir():
    if entry.name != "CHR-002":
        (characters / entry.name).symlink_to(entry, target_is_directory=entry.is_dir())
(characters / "CHR-002").mkdir()
shutil.copy2(glb, characters / "CHR-002/ling-grey-study.glb")
(output / "prepare.json").write_text(json.dumps({
    "status": "PASS", "blender": bpy.app.version_string,
    "source": str(source), "dependencies": {path.name: sha(path) for path in dependencies},
    "motion_sha256": sha(motion_file),
    "glb_sha256": sha(glb), "source_height": height, "height_m": 1.65,
    "pose_samples": pose_samples, "mapping": mapping,
    "identity": "Ghost73 Belle reference; CHR-002 is only the preview slot",
    "scope": "local comparison, not a redistribution approval or accepted N:SIDE character",
}, indent=2) + "\n")
