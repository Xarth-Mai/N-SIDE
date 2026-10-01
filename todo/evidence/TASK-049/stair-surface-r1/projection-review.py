"""Read actual capture poses and project authored stair treads; no image/render edits."""
import csv
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
MAP = json.loads((ROOT / 'source-assets/district-map/district.json').read_text())
OUT = Path(__file__).parent
summary = []
with (OUT / 'projection-review.csv').open('w') as file:
    writer = csv.writer(file, lineterminator="\n")
    writer.writerow(['view', 'road', 'step_zero_based', 'height_m', 'eye_minus_tread_m', 'camera_distance_m', 'screen_x', 'screen_y', 'projected_tread_vertical_px'])
    for view, roads in [('shop-stair-baseline', [58, 60, 62]), ('shop-stair-surfaces', [60, 62, 64]), ('hill-stair-surfaces', [744, 746, 748])]:
        state = json.loads((ROOT / f'output/blender-integration-r2/{view}/state.json').read_text())
        sample = state['samples'][-1]
        eye = sample['position']
        x, y, z, w = sample['rotation']
        matrix = [[1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w)], [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w)], [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y)]]
        width, height = state['script']['width'], state['script']['height']
        focal = height/2 / math.tan(math.radians(55)/2)
        def project(point):
            delta = [point[i]-eye[i] for i in range(3)]
            camera = [sum(matrix[j][i]*delta[j] for j in range(3)) for i in range(3)]
            return [width/2+focal*camera[0]/-camera[2], height/2-focal*camera[1]/-camera[2]]
        result = {'view': view, 'eye': eye, 'rotation': sample['rotation'], 'horizon_y_at_screen_center': height/2-focal*matrix[1][2]/matrix[1][1], 'roads': []}
        for road_id in roads:
            road = MAP['roads'][road_id]
            a, b = [MAP['nodes'][n] for n in road['nodes']]
            count = max(1, math.ceil(abs(b[2]-a[2])/.17))
            visible = 0
            nearest = None
            for step in range(count):
                elevation = a[2]+(b[2]-a[2])*(step+.5)/count+.025
                world = [[a[0]+(b[0]-a[0])*t, elevation, -(a[1]+(b[1]-a[1])*t)] for t in [step/count, (step+1)/count]]
                pixels = [project(p) for p in world]
                delta = eye[1]-elevation
                row = [view, road_id, step, elevation, delta, math.dist(eye, world[0]), *pixels[0], pixels[1][1]-pixels[0][1]]
                writer.writerow(row)
                visible += delta > 0
                if nearest is None or abs(delta) < abs(nearest['eye_minus_tread_m']):
                    nearest = dict(zip(['view', 'road', 'step_zero_based', 'height_m', 'eye_minus_tread_m', 'camera_distance_m', 'screen_x', 'screen_y', 'projected_tread_vertical_px'], row))
            result['roads'].append({'road': road_id, 'treads': count, 'front_facing': visible, 'back_facing': count-visible, 'nearest_eye_level': nearest})
        summary.append(result)
(OUT / 'projection-review.json').write_text(json.dumps(summary, indent=2)+'\n')
print('PASS: projected three real frame59 poses; wrote projection-review.csv and projection-review.json')
