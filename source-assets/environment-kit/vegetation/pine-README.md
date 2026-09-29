# 院落与林缘针叶树

`AST-003` 的原创针叶树样板，用于改善近景 `tree_pine` 的层叠实心圆锥外观；稳定树点、碰撞和空间设计仍由地图与世界模块负责

| 项目 | 内容 |
| --- | --- |
| 可编辑主文件 | [pine-street.blend](pine-street.blend) |
| 造型与导出 | [pine-build.py](pine-build.py)，Blender 4.5 LTS，固定 seed `49031`；默认按源程序重建，`--export` 仅从主文件导出 |
| 检查 | [pine-check.py](pine-check.py)，检查实际 GLB 顶点、UV、法线、颜色、材质、枢轴、尺度与包络 |
| 源与运行派生物 | [pine-street.glb](pine-street.glb) → `game/assets/environment/vegetation/pine-street.glb`，由现有环境导出器统一接入 |
| 来源与许可 | N:SIDE 原创树干、分枝与针叶几何，Codex 使用 Blender 制作，沿用仓库 [MPL-2.0](../../../LICENSE)；没有复制 Kenney 几何，也未使用外部图片／3D 生成服务 |
| 尺度与枢轴 | 米制，GLB Y-up，6.5m 高，根节点变换为 identity，底部 Y=0，树干根部原点 |
| 水平包络 | 最大半径 1.690000m，低于原运行模型 1.694578m 的最大水平半径及布置检查的 2.3m 限额；包络按根部水平圆定义，不声称逐点包含于原多面锥体 |
| 几何 | 单 Mesh、2 primitive、6,510 三角、17,134 导出顶点；17 主枝、68 次枝、426 针叶簇、5,112 片尖细针叶 |
| 材质与纹理 | 树皮单面，针叶双面且全不透明；复用原街树 [bark-color.png](bark-color.png) 的 256×512 sRGB 色图，UV0 沿枝干轴线，针叶为四组低饱和常绿顶点色 |
| 当前阶段 | 原创近景样板，CPU 三向形体与数值检查完成；连续移动、背面照明和实际成本由 Bevy 运行验收确认 |

树冠以不等高、不等角度的弯曲分枝承接针叶簇，沿枝中段与外端形成有空隙的偏心冠组。针叶使用真实不透明三角几何；没有堆叠圆锥、球体冠层或透明贴片。它是偏疏枝的修剪型树形，不以这一株代表全部林缘生态

源树皮位图由现有街树程序原创生成，本资产只读取并打包，不改写街树或位图。每个 GLB 内嵌一次图像，实例引用同一加载模型；同图像嵌入不同 GLB 不保证引擎跨模型自动去重，不把“源图复用”写成已测 GPU 内存共享

## 导出与检查

在仓库根目录执行，以下命令可直接用于 fish；主文件有手工编辑时使用 `--export`，不要用默认重建覆盖

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/pine-build.py -- --export
python3 source-assets/environment-kit/vegetation/pine-check.py
bun tools/export-environment.ts
```

需要按源程序有意重建时省略 `--export`。同机位 CPU 检查在命令末尾追加 `--render-dir output/assets/task049-pine-r1/after`；三个正交机位沿树旋转 0°／120°／240°，720×840、Cycles CPU、24 samples。渲染不写回主文件，临时 PNG 查看后按项目规则清理

导入沿用共享环境清单，登记原件 hash、源主文件、纹理依赖与许可；该资产不维护另一个 manifest。现有导出器无需额外归一化，因为 GLB 已使用米制、Y-up 和接地原点

## 当前边界

6,510 三角是本轮候选的实测值，7,000 三角是本样板检查上限，不是已批准的全城预算。原版为 204 三角，实例数由实际地图派生并随林群制作变化；当前 138 个实例合计 898,380 三角；必须结合近景收益、远景轮廓和实际运行成本决定应用范围

当前没有叶风、LOD 或专用针叶着色器；针叶在近景仍有偏锐平面轮廓，远景细线覆盖与转动闪烁待实机观察。模型制作、检查和接入不代替最终美术验收
