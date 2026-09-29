// TASK-045 terrain-shape-r3 diagnostic: temporary example, genuine native Ground
// Usage: terrain_r7_probe <before-district.json> <after-district.json>
// Road offsets below are the current geometry.rs bounded-miter formula; no terrain is generated here
use n_side::world::{geometry::Ground, map::Map};
use serde_json::{Value, json};

fn road_offsets(points: &[[f64; 3]], width: f64) -> Vec<[f64; 2]> {
    let normals: Vec<_> = points
        .windows(2)
        .map(|pair| {
            let d = [pair[1][0] - pair[0][0], pair[1][1] - pair[0][1]];
            let length = d[0].hypot(d[1]);
            [-d[1] / length, d[0] / length]
        })
        .collect();
    (0..points.len())
        .map(|index| {
            if index == 0 {
                return normals[0].map(|v| v * width / 2.0);
            }
            if index == points.len() - 1 {
                return normals[index - 1].map(|v| v * width / 2.0);
            }
            let a = normals[index - 1];
            let b = normals[index];
            let dot = 1.0 + a[0] * b[0] + a[1] * b[1];
            // Acute turns receive a bounded miter; this changes only the derived road edge
            let factor = (width / 2.0 / dot.max(0.01)).min(width * 2.0);
            [(a[0] + b[0]) * factor, (a[1] + b[1]) * factor]
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("expected before and after district paths".into());
    }
    let maps = [Map::load(&args[0])?, Map::load(&args[1])?];
    let grounds = [Ground::new(&maps[0])?, Ground::new(&maps[1])?];
    let mut values: Vec<Value> = args
        .iter()
        .map(|p| -> Result<_, Box<dyn std::error::Error>> {
            Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
        })
        .collect::<Result<_, _>>()?;
    for value in &mut values {
        value.as_object_mut().unwrap().remove("terrain");
    }
    if values[0] != values[1] {
        return Err("non-terrain map data changed".into());
    }
    let height_pair = |p: [f64; 2]| grounds.each_ref().map(|g| g.height(p));
    let fixed: Vec<_> = [
        [-151.43885889, 801.78806574],
        [-150.0, 802.803],
        [-148.56114111, 803.81793426],
        [-41.01511846, 754.41628110],
    ]
    .into_iter()
    .map(|p| {
        let h = height_pair(p);
        json!({"point":p,"before":h[0],"after":h[1],"change":h[1]-h[0]})
    })
    .collect();
    let mut rows = Vec::new();
    let mut counts = [0usize; 2];
    for (road_index, road) in maps[1].roads.iter().enumerate() {
        let upper = road.surface.as_ref().is_some_and(|id| {
            maps[1]
                .surfaces
                .iter()
                .any(|s| s.id.as_ref() == Some(id) && s.elevated)
        });
        if road.building.is_some()
            || upper
            || matches!(road.kind.as_str(), "bridge" | "deck" | "lift" | "interior")
        {
            continue;
        }
        let points: Vec<_> = road.nodes.iter().map(|id| maps[1].nodes[id]).collect();
        let offsets = road_offsets(&points, road.width);
        for i in 1..points.len() {
            let a = points[i - 1];
            let b = points[i];
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                for side in [-1.0, 0.0, 1.0] {
                    let p = [0, 1].map(|axis| {
                        a[axis]
                            + (b[axis] - a[axis]) * t
                            + side
                                * (offsets[i - 1][axis]
                                    + (offsets[i][axis] - offsets[i - 1][axis]) * t)
                    });
                    let h = height_pair(p);
                    let grade = a[2] + (b[2] - a[2]) * t;
                    if h.iter().any(|v| !v.is_finite()) {
                        return Err(format!("nonfinite Ground at {p:?}").into());
                    }
                    let inside = (20.0..=320.0).contains(&p[0]) && (570.0..=930.0).contains(&p[1]);
                    counts[usize::from(!inside)] += 1;
                    rows.push(json!({"road":road_index,"from":road.nodes[i-1],"to":road.nodes[i],"point":p,"t":t,"side":side,"inside_changed_control_bounds":inside,"grade":grade,"before":h[0],"after":h[1],"change":h[1]-h[0],"before_grade_error":h[0]-grade,"after_grade_error":h[1]-grade,"absolute_error_increase":(h[1]-grade).abs()-(h[0]-grade).abs()}));
                }
            }
        }
    }
    let partitions: Vec<_> = [true,false].into_iter().enumerate().map(|(i,inside)| {
        let subset: Vec<_> = rows.iter().filter(|r| r["inside_changed_control_bounds"].as_bool()==Some(inside)).collect();
        let change=subset.iter().max_by(|a,b| a["change"].as_f64().unwrap().abs().total_cmp(&b["change"].as_f64().unwrap().abs()));
        let regression=subset.iter().max_by(|a,b| a["absolute_error_increase"].as_f64().unwrap().total_cmp(&b["absolute_error_increase"].as_f64().unwrap()));
        json!({"inside_changed_control_bounds":inside,"samples":counts[i],"maximum_change_sample":change,"maximum_absolute_grade_error_increase_sample":regression})
    }).collect();
    let mut changed: Vec<_> = rows
        .into_iter()
        .filter(|r| r["change"].as_f64().unwrap().abs() > 0.01)
        .collect();
    changed.sort_by(|a, b| {
        b["absolute_error_increase"]
            .as_f64()
            .unwrap()
            .total_cmp(&a["absolute_error_increase"].as_f64().unwrap())
    });
    let mut slopes = [Vec::new(), Vec::new()];
    for y in (704..=776).step_by(4) {
        for x in (132..=216).step_by(4) {
            for (i, g) in grounds.iter().enumerate() {
                let (x, y) = (f64::from(x), f64::from(y));
                let dx = (g.height([x + 0.1, y]) - g.height([x - 0.1, y])) / 0.2;
                let dy = (g.height([x, y + 0.1]) - g.height([x, y - 0.1])) / 0.2;
                slopes[i].push(dx.hypot(dy).atan().to_degrees());
            }
        }
    }
    let angles=slopes.map(|mut s| { s.sort_by(f64::total_cmp); json!({"samples":s.len(),"p50":s[s.len()/2],"p95":s[s.len()*95/100],"maximum":s.last(),"over55":s.iter().filter(|v|**v>55.0).count()}) });
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"scope":"Native Spade Ground; road grade envelope samples, not final stair tread or collision clearance","before":args[0],"after":args[1],"terrain_samples":maps.each_ref().map(|m|m.terrain.samples.len()),"non_terrain_unchanged":true,"fixed_probes":fixed,"road_partitions":partitions,"changed_road_samples_over_1cm":changed.len(),"worst_changed_road_samples":changed.into_iter().take(20).collect::<Vec<_>>(),"cut_window_slope_before_after":angles})
        )?
    );
    Ok(())
}
