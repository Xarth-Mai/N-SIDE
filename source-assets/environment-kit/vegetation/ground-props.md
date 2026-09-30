# 林缘草与石块

草、石块的 Blender 主文件是 `grass-forest.blend` 与 `rock-forest.blend`，米制、Z-up 源坐标、底面枢轴；导出 GLB 为 Y-up，各使用一个不透明材质。几何与草贴图由 N:SIDE 原创，沿用 MPL-2.0；石块内嵌的 ambientCG Rock043L 颜色和 OpenGL 法线保持 CC0-1.0，来源与 hash 见[环境清单](../asset-manifest.json)

| 资产 | 规格 |
| --- | --- |
| 草 | 60 片弯曲叶、4 个不等基簇，1,320 三角，高 0.35m、最大水平半径约 0.29m，64×256 sRGB 颜色图，双面无 alpha 裁切 |
| 石 | 不规则压平底面，320 三角，高 0.65m、最大水平半径约 0.745m，颜色按 sRGB、法线按线性，约 1.8m 贴图周期 |

编辑主文件后执行下列导出；省略 `--export-existing` 会按造型脚本重建两份主文件，手工编辑后只使用导出模式。源 GLB hash 变更后更新环境清单，再运行既有导出器

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/environment-kit/vegetation/ground-props.py -- --export-existing
bun tools/export-environment.ts
bun tools/export-environment.ts --check
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib understory_keeps_actual_roots_patch_openings_and_scene_clearance -- --nocapture
```

空间检查读取运行 GLB 的真实几何，草石按平面底部顶点验证贴地，叶片和石块侧面仍计入完整水平避让与观景视线范围。运行时沿用既有 `grass`、`rock` 逻辑键；新造型不改变布点算法、道路和建筑 ID。制作、失败诊断及实机自查见[本轮记录](../../../todo/evidence/TASK-049/ground-props-r1/review.md)
