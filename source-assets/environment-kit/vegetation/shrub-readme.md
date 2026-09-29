# 院落阔叶灌木

AST-003 的原创近景灌木，供既有 `shrub` 模型槽替换块状叶片；保留 0.8 m 高度、地面枢轴与旧实例的水平包络，不新增地图点位

| 项目 | 内容 |
| --- | --- |
| 主文件 | [shrub-courtyard.blend](shrub-courtyard.blend) |
| 造型程序 | [shrub-build.py](shrub-build.py)，固定 seed `49031`；重建会覆盖主文件中的人工编辑，仅在有意重置时使用 |
| 导出 / 检查 | [shrub-export.py](shrub-export.py)、[shrub-check.py](shrub-check.py) |
| 导出物 | [shrub-courtyard.glb](shrub-courtyard.glb)，对应 `game/assets/environment/vegetation/shrub-courtyard.glb` |
| 作者 / 许可 | N:SIDE 原创分枝、拓扑与顶点配色，Codex 使用 Blender 制作，沿用项目 [MPL-2.0](../../../LICENSE)；没有复用 Kenney 几何、外部图片或生成服务 |
| 单位 / 轴向 | 米；GLB 为 Y-up；高 0.8 m，底部 Y=0，根部原点，实体 scale=1 |
| 包络 | 最大水平半径约 0.568525 m，处于被替换模型的 0.669339 m 包络及布置器 0.7 m 限额内；冠幅略收窄，现有花盆与地面承托保持原高度 |
| 网格 / 材质 | 单 Mesh、5,824 三角、1 个 primitive / 材质；全不透明、非金属、双面，粗糙度 0.95 |
| 叶片 / 颜色 | 1,022 片浅弯六边叶片，每叶 4 三角；枝叶使用 `COLOR_0`，四组绿色与棕枝共享一个材质，UV0 保留，无纹理依赖 |
| 动作 | 静态，未制作风动或骨骼 |
| 状态 | 几何与 CPU 自查完成，实际 Bevy 近景、连续移动与作者美术验收待完成 |

主枝由低位根丛向外生长，低侧枝覆盖裸茎，中部密于顶端，叶片按枝向交错并保留疏密和真实空隙。顶端的新梢较疏，不采用完整球块填充；平面色块与清楚的小叶轮廓服务日漫画面的阅读，未以烘焙光照代替真实受光

在仓库根目录运行，以下命令可用于 fish

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --threads 2 --python source-assets/environment-kit/vegetation/shrub-export.py
python3 source-assets/environment-kit/vegetation/shrub-check.py
```

需要四角度 CPU 查看时，在导出命令末尾加 `-- --render-dir output/assets/task049-shrub-r1/after`。它读取主文件、导出 GLB，再用临时地面和灯光渲染，不把检查场景保存回主文件；`--old-model` 仅加载现有 Kenney 运行灌木进行同机位对照，不写本资产。检查图查看后清理，模型与导出程序保留

环境清单与统一导出由根 Agent 接入，本目录不另设资产 manifest。此次成本增量按旧灌木 104 三角计算，每实例增加 5,720 三角；若替换当前 15 个实例，合计增加 85,800 三角，材质和纹理成本仍需由实际引擎资源统计核对，不能以 CPU 渲染推断运行性能
