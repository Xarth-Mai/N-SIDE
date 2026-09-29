"""Blender-only test fixture, not a game character or approved visual asset

blender -b --python tools/tests/create_character_fixture.py -- tools/tests/fixtures/character.glb
"""

import sys
from pathlib import Path

import bpy

output = Path(sys.argv[sys.argv.index("--") + 1]).resolve()
output.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, 0.85))
body = bpy.context.object
body.name = "FixtureMesh"
body.scale = (0.4, 0.2, 1.7)
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
image = bpy.data.images.new("FixtureTexture", width=4, height=4)
image.generated_color = (0.2, 0.5, 0.7, 1)
image.pack()
material = bpy.data.materials.new("FixtureMaterial")
material.use_nodes = True
texture = material.node_tree.nodes.new("ShaderNodeTexImage")
texture.image = image
material.node_tree.links.new(texture.outputs["Color"], material.node_tree.nodes["Principled BSDF"].inputs["Base Color"])
body.data.materials.append(material)
rig = bpy.data.objects.new("FixtureRig", bpy.data.armatures.new("FixtureSkeleton"))
bpy.context.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
body.select_set(False)
rig.select_set(True)
bpy.ops.object.mode_set(mode="EDIT")
root = rig.data.edit_bones.new("Root")
root.head, root.tail = (0, 0, 0), (0, 0, 0.1)
hips = rig.data.edit_bones.new("Hips")
hips.head, hips.tail, hips.parent = (0, 0, 0.85), (0, 0, 1.2), root
bpy.ops.object.mode_set(mode="OBJECT")
for name, upper in (("Root", False), ("Hips", True)):
    group = body.vertex_groups.new(name=name)
    group.add([v.index for v in body.data.vertices if (v.co.z > 0.85) == upper], 1, "REPLACE")
body.parent = rig
body.modifiers.new("Armature", "ARMATURE").object = rig
hips = rig.pose.bones["Hips"]
hips.rotation_mode = "XYZ"
for name, angle in (("Idle", 0.08), ("Sway", 0.2)):
    rig.animation_data_clear()
    for frame, value in ((1, 0), (16, angle), (31, 0)):
        hips.rotation_euler = (0, value, 0)
        hips.keyframe_insert("rotation_euler", frame=frame)
    rig.animation_data.action.name = name
    rig.animation_data.action.use_fake_user = True
bpy.context.scene.render.fps = 30
bpy.context.scene.frame_start, bpy.context.scene.frame_end = 1, 31
bpy.context.scene.frame_set(1)
bpy.ops.export_scene.gltf(filepath=str(output), export_format="GLB", export_animation_mode="ACTIONS", export_anim_slide_to_zero=True, export_yup=True, export_animations=True, export_skins=True, export_morph=False)
