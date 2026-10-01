"""Read the saved shade master and re-export it without changing source data."""
import hashlib
import json
from pathlib import Path
import runpy
import bpy

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
SOURCE = ROOT / 'source-assets/environment-kit/street-furniture/roof-shade'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
master, glb = SOURCE/'roof-shade.blend', SOURCE/'roof-shade.glb'
before = {'master':sha(master),'glb':sha(glb)}
bpy.ops.wm.open_mainfile(filepath=str(master))
objects = list(bpy.data.collections['RoofShade_EXPORT'].objects)
assert len(objects) == 28 and all(o.type == 'MESH' for o in objects)
assert len([o for o in objects if o.name.startswith('wood shade slat ')]) == 10
bounds = {}
for obj in objects:
    points = [obj.matrix_world @ vertex.co for vertex in obj.data.vertices]
    bounds[obj.name] = {'min':[min(p[i] for p in points) for i in range(3)], 'max':[max(p[i] for p in points) for i in range(3)],
                        'materials':[m.name for m in obj.data.materials]}
for name in ('west foot plate','east foot plate'):
    assert abs(bounds[name]['min'][2]) < 1e-6
assert min(b['min'][2] for b in bounds.values()) >= -1e-6
assert set(m.name for m in bpy.data.materials) == {'RoofShade_Metal','RoofShade_Wood'}
images = [image for image in bpy.data.images if image.users]
assert len(images) == 2 and all(image.packed_file for image in images)
for image in images:
    path = Path(bpy.path.abspath(image.filepath))
    assert path.read_bytes() == image.packed_file.data
    assert image.colorspace_settings.name == ('Non-Color' if 'NormalGL' in image.name else 'sRGB')
normal = bpy.data.materials['RoofShade_Wood'].node_tree.nodes['Principled BSDF'].inputs['Normal'].links[0].from_node
assert normal.type == 'NORMAL_MAP' and normal.inputs['Strength'].default_value == 1
runpy.run_path(str(SOURCE/'build.py'))['export']()
after = {'master':sha(master),'glb':sha(glb)}
assert before == after
report = {'status':'PASS','blender':bpy.app.version_string,'before':before,'after':after,'source_objects':28,
          'source_groups':bounds,'packed_images':{image.name:hashlib.sha256(image.packed_file.data).hexdigest() for image in images},
          'export_from_saved_master':'byte-identical','source_modified':False,'visual_review':'NOT RUN; no new media generated'}
(HERE/'source-check.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
