# 街区外观与原创招牌

| 项目 | 内容 |
| --- | --- |
| 资产 ID | `AST-004` |
| 主文件 | [appearance.json](appearance.json)、本目录五个招牌 SVG |
| 作者与来源 | N:SIDE 原创外观绑定和招牌图形；公开环境素材来源见[环境素材包](../environment-kit/README.md) |
| 使用位置 | `world::scene` 生成的外墙、道路、店面和绿植 |
| 派生文件 | `game/assets/environment/signs/*.png` |
| 状态 | `exported`，真实镜头检查记录于 Viewer 任务 |

`appearance.json` 通过材质角色、模型 ID 和地图建筑 ID 绑定表现，不复制空间坐标。颜色使用 sRGB，颜色纹理按 sRGB 加载，OpenGL 法线纹理按线性数据加载，`tile_meters` 定义纹理一周期的实际米数。所有材质与贴图为必需资源；只有模型显式标记 `optional: true` 时允许带 Warning 省略

`shopfronts` 对应小店、烘焙、生鲜、早餐和干货店，建筑用途来自地图；新招牌只修改 SVG 主文件。字形使用系统 DejaVu Sans 栅格化，源 SVG 保持可编辑，项目不分发字体文件

导出和核对：

```sh
bun tools/export-district-scene.mjs
bun tools/export-district-scene.mjs --check
```

导出依赖 Bun、ImageMagick 与 DejaVu Sans，运行 Viewer 只使用已导出的 PNG。脚本以固定画布栅格化并排除 PNG 时间元数据，`--check` 在临时目录重算并比较字节，五个招牌均为 1024 × 128

## 规则生成的构件

外墙、天井、入口、楼层和道路采用地图原值。窗格、檐口、雨棚、门框、花盆和空调简体按建筑立面与入口推导，花盆避开入口 1.6 m；它们属于外观规则，后续可由美术替换。每个生成网格和模型实体保留源对象路径

小店样板的花盆搭配已归一为 0.8 m 的公开灌木模型

植树点采用地图原值，树底优先对应所在地面平台，其余使用地形插值。灯具沿足够长的主路、街道和岸边路段推导，并排除建筑和水面；表现规则与相机控制相互独立
