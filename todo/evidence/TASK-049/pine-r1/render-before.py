"""Render the existing normalized Kenney pine with the new asset's CPU cameras."""

from pathlib import Path
import runpy

import bpy

root = Path(__file__).resolve().parents[4]
helpers = runpy.run_path(str(root / "source-assets/environment-kit/vegetation/pine-build.py"), run_name="pine_helpers")
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.ops.import_scene.gltf(filepath=str(root / "game/assets/environment/nature/tree_pineRoundA.glb"))
helpers["render"](root / "output/assets/task049-pine-r1/before")
