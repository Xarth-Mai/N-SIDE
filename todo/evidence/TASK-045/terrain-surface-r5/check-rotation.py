"""Check the candidate's actual rotation constants against independent height derivatives."""
from pathlib import Path
import json
import re
import numpy as np

ROOT=Path(__file__).resolve().parents[4]
shader=ROOT/'output/assets/terrain-r5-diagnostics/rotated-pair/game/assets/shaders/terrain-slope.wgsl'
s=shader.read_text()
c, sine=map(float,re.search(r'return vec2\(([0-9.]+) \* uv.x - ([0-9.]+) \* uv.y',s).groups())
a,b,d,e=map(float,re.search(r'let nt = vec3\(([0-9.]+) \* sampled.x \+ ([0-9.]+) \* sampled.y,\s*-([0-9.]+) \* sampled.x \+ ([0-9.]+) \* sampled.y',s).groups())
rotation=np.array([[c,-sine],[sine,c]])
normal_rotation=np.array([[a,b],[-d,e]])
assert np.max(np.abs(rotation.T@rotation-np.eye(2)))<1e-6
assert np.max(np.abs(normal_rotation-rotation.T))<1e-8
assert np.linalg.det(rotation)>0
phase=np.array([.37,.71])
def height(p):
    q=rotation@p+phase
    return np.sin(2*np.pi*q[0])+.4*np.cos(6*np.pi*q[1])
max_error=0.
for x in np.linspace(-12,12,17):
    for y in np.linspace(-12,12,17):
        p=np.array([x,y]);q=rotation@p+phase
        tangent=np.array([-2*np.pi*np.cos(2*np.pi*q[0]), 2.4*np.pi*np.sin(6*np.pi*q[1])])
        actual=normal_rotation@tangent
        eps=1e-5
        expected=-np.array([(height(p+axis*eps)-height(p-axis*eps))/(2*eps) for axis in np.eye(2)])
        max_error=max(max_error,float(np.max(np.abs(actual-expected))))
assert max_error<1e-6
result={'status':'PASS','source':'actual candidate shader constants','checks':['UV rotation preserves metric lengths and handedness','sampled normal xy uses transpose of UV rotation','289 positive/negative coordinates agree with independent finite-difference height gradient'], 'max_gradient_error':max_error, 'runtime_scope':'GPU rendering and final normal/SSAO still require actual captures'}
(ROOT/'todo/evidence/TASK-045/terrain-surface-r5/rotation-check.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
