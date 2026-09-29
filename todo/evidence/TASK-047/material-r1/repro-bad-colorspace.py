"""Inject a bad data color space into an isolated master and run the real check."""
from pathlib import Path
import runpy
import sys

import bpy

ROOT = Path(__file__).resolve().parents[4]
source = ROOT / 'source-assets/characters/CHR-001/model'
bpy.ops.wm.open_mainfile(filepath=str(source / 'yao-grey-study.blend'))
bpy.data.images['CHR001_GreyStudy_Surface'].colorspace_settings.name = 'sRGB'
bad = ROOT / 'output/characters/CHR-001/material-r1/bad-color-space.blend'
bad.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=str(bad))
sys.argv = ['check.py', '--', '--master', str(bad), '--output', str(bad.with_suffix('.json'))]
runpy.run_path(str(source / 'check.py'), run_name='__main__')
raise AssertionError('Corrupted surface color space unexpectedly passed')
