# 初秋街树

环境包 `AST-003` 的项目原创街树，用于替换 `tree_a` 的块状树冠；地图树点、碰撞和其他植物种类保持原有职责

| 项目 | 内容 |
| --- | --- |
| 可编辑主文件 | [street-tree.blend](street-tree.blend) |
| 初始造型程序 | [build_street_tree.py](build_street_tree.py)，固定 seed `45019` 与明确的主枝端点；重建会覆盖 Blender 编辑，仅在有意重置初始造型时使用 |
| 导出 | [export_street_tree.py](export_street_tree.py) 从主文件导出 `street-tree.glb`，运行派生物为 `game/assets/environment/vegetation/street-tree.glb` |
| 作者与来源 | N:SIDE 原创拓扑、顶点配色及程序绘制树皮，Codex 使用 Blender 制作；没有借用 Kenney 几何、外部贴图或图片／3D 生成服务 |
| 许可 | 项目原始内容，沿用仓库 [MPL-2.0](../../../LICENSE)；不属于环境包中的第三方 CC0 素材 |
| 单位与枢轴 | 米；GLB 为 Y-up；高 6m，底部 Y=0，树干根部原点，实体 scale=1 |
| 最大水平半径 | 约 2.856m，位于 `tree_a` 现有 3.5m 布置限额内 |
| 几何与材质 | 单 Mesh、11,686 三角、2 个材质／primitive；树皮单面，叶片双面且全不透明 |
| 颜色与 UV | [bark-color.png](bark-color.png) 为原创 256×512 sRGB 树皮色图，打包在主文件并嵌入 GLB，UV0 沿枝干轴线展开；叶片通过 `COLOR_0` 保存四组初秋叶色和轻微根尖色差 |
| 透明与细节 | 1,452 片八边浅弯阔叶构成真实几何间隙，每叶 6 三角、每根细枝交错排列 12 叶；叶柄轴向扭转，叶簇覆盖主枝中段至外端，无 cutout／混合 alpha |
| 当前阶段 | 街树样板，CPU 多角度与几何检查通过；实际街景观感、连续移动闪烁及资源成本由运行验收确认 |

主枝从清楚的树干分出，叶冠在高度、方向和密度上错开；暖叶只占少量，保留初秋以绿色为主的整体关系。它提供实际枝叶轮廓与投影，不以整团球形几何代替树冠

近景制作以 [TASK-049 实机自查](../../../todo/evidence/TASK-049/residential-r1/runtime-review.md)中的稀疏粗叶片和纯色树干为输入，保持 6m 米制、根部枢轴、原主枝端点、初秋配色与两种不透明材质。叶片采用圆肩轮廓和交错叶序，减少单叶三角后增加较小叶片，整树保持相同三角数；叶柄的不同扭转与冠内外分布减少水平叶架，保留枝条之间的透空。树皮采用克制的纵向色纹理，细节仍服从中近景轮廓。对照用相同的三向全树与树干近景 CPU 机位，真实 Viewer 近景沿用 `street-tree-detail.json`

在仓库根目录运行，以下命令可直接用于 fish；路径是当前本地 Blender 4.5.14 LTS，其他环境使用同版本 Blender 的实际位置

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/export_street_tree.py
python3 source-assets/environment-kit/vegetation/check_street_tree.py
bun tools/export-environment.ts
```

需要重新查看三面和树干近景检查图时，在 Blender 命令末尾添加 `-- --render --render-dir output/assets/task045-vegetation-r3/after`，仅使用 Cycles CPU、24 samples、720×840；`--render-dir <路径>` 指定本轮目录。它不修改 Blender 主文件，查看后清理临时 PNG。材质无透明纹理，alpha 检查核对所有材质 OPAQUE、顶点 alpha=1 和叶片双面，不能把画面中的几何空隙误判成透明像素

导出后按现有环境清单更新原件 hash，再运行统一环境导出器；源清单及 `appearance.models.tree_a.file` 的接入由环境维护者完成。不会在此维护第二套资产 manifest

当前没有叶风动画、LOD 或树皮法线图；本轮树皮是颜色变化，不声称已有凹凸几何。先检查真实街景中的轮廓与成本，再按实际距离需求扩展，不能把本株 CPU 渲染称为全城植被完成
