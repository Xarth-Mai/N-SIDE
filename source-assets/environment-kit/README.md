# 街区环境素材

| 项目 | 内容 |
| --- | --- |
| 资产 ID | `AST-003` |
| 状态 | `exported`，已生成运行派生物并通过 Bevy 加载与六镜头渲染检查，样板美术验收待作者确认 |
| 主文件 | 本目录中的 GLB、JPG、道路色板和原始许可文件，逐文件见 [asset-manifest.json](asset-manifest.json) |
| 作者 | Kenney、ambientCG；模型米制归一与植物材质调整由 N:SIDE 制作 |
| 许可 | CC0 1.0，保留 [CC0 正文](licenses/CC0-1.0.txt)、[Nature Kit 包内声明](licenses/kenney-nature.txt)及[City Kit Roads 包内声明](licenses/kenney-roads.txt) |
| 获取日期 | 2026-09-25 |
| 使用位置 | 首个 3D Map Viewer 的街树、花盆灌木、街灯、铺装与墙面；导视空杆已导出、尚未放置 |
| 运行派生物 | `game/assets/environment/`，由导出命令生成，运行期间无需联网 |

## 来源与取得范围

| 来源 | 取得内容 | 证据 |
| --- | --- | --- |
| [Kenney Nature Kit](https://kenney.nl/assets/nature-kit) | `tree_detailed.glb`、`tree_oak.glb`、`plant_bushDetailed.glb` | 原包 `License.txt` 标 Nature Kit 2.1 与 CC0；所选 GLB 内嵌纯色材质，无外部纹理依赖 |
| [Kenney City Kit Roads](https://kenney.nl/assets/city-kit-roads) | `light-curved.glb`、`road-sign-empty.glb`、依赖的 `Textures/colormap.png` | 原包 `License.txt` 标 City Kit Roads 2.1 与 CC0；保留色板相对路径 |
| [ambientCG Concrete034](https://ambientcg.com/view?id=Concrete034) | 1K JPG Color、NormalGL，各 1024×512 | 官方产品页与[许可页](https://docs.ambientcg.com/license/)确认 CC0，可在项目中分发原始文件 |
| [ambientCG PavingStones092](https://ambientcg.com/view?id=PavingStones092) | 1K JPG Color、NormalGL，各 1024×1024 | 同上 |
| [ambientCG Asphalt012](https://ambientcg.com/view?id=Asphalt012) | 1K JPG Color、NormalGL，各 1024×1024 | 同上 |

所选文件通过官方 ZIP 的 HTTP Range 提取，ZIP CRC 校验通过，下载链接、包内成员路径及原件 SHA-256 记录在清单；仓库只保留实际选用文件，未保存完整包，因此没有完整 ZIP 的哈希

`road-sign-empty.glb` 是导视空杆，文字和牌面由项目原创；Quaternius 与 Poly Haven 候选尚未下载

## 米制、枢轴与材质

| 运行模型 | 原件几何高度 | 导出高度 | 三角面数 |
| --- | --- | --- | --- |
| `nature/tree_detailed.glb` | 1.332417 | 6 m | 402 |
| `nature/tree_oak.glb` | 1.226240 | 5.5 m | 196 |
| `nature/plant_bushDetailed.glb` | 0.360411 | 0.8 m | 104 |
| `roads/light-curved.glb` | 0.675 | 4.5 m | 92 |
| `roads/road-sign-empty.glb` | 0.475 | 2.3 m | 42 |

运行 GLB 使用默认场景 `Scene0`、Y 轴向上、地面枢轴，默认实体缩放为 `1`；树木与道具造型本身保留原件，导出只修正场景根节点的缩放与底部高度，灯臂沿原模型的负 Z 方向延伸

Nature Kit 原件的场景节点含 `y=-0.05` 偏移，植物材质 `metallicFactor=1`；导出将底部置于 `y=0`，植物改为非金属、粗糙度 `1`，树叶与树皮按清单中的线性颜色调整为较克制的绿色与棕色

颜色图按 sRGB 加载，OpenGL 法线图按线性数据加载；JPG 主文件保持原件，运行纹理从主文件导出为完整 11 级 mipmap 的无压缩 BGRA8 DDS，原始分辨率不变，首次接入用固定粗糙度，未引入位移贴图或专用 shader

DDS 使用 ImageMagick 的默认缩小过滤，不作颜色空间转换；颜色图 mip 暂按编码值平均，法线 mip 按通道平均并由标准 PBR shader 在采样后归一化，保留首次验证通过的简洁导出链

Concrete034 的一周期覆盖 1.1 m × 0.55 m，PavingStones092 为 1.55 m × 1.55 m，均来自官方产品页；Asphalt012 页面未标尺寸，首轮暂定 4 m × 4 m，属于表现参数，需在真实镜头中校准

## 导出与检查

在仓库根目录执行：

```sh
bun tools/export-environment.mjs
bun tools/export-environment.mjs --check
```

导出需要 Bun 与 ImageMagick，本次验证版本为 `ImageMagick 7.1.2-31 Q16-HDRI`；[官方 DDS 格式说明](https://imagemagick.org/formats/)列出压缩与 mipmap 选项

导出先校验全部原件 SHA-256，再执行 GLB 根节点与植物材质调整；每张 JPG 使用 `magick <source> -alpha on -define dds:compression=none dds:-` 导出，附加不透明 alpha 明确选用 BGRA8，避免 RGB24 转码的通道解释差异，纹理未经 BC 压缩

导出器检查 DDS 头部的尺寸、像素掩码、完整 mip 层数和数据字节数，其余文件逐字节复制；生成的 `game/assets/environment/manifest.json` 记录派生物大小与 SHA-256，`--check` 重算派生结果并逐字节比较，不写文件

原件与导出参数为唯一编辑入口；新增素材时先记录来源、许可、原件哈希和尺寸，再更新清单与运行派生物

2026-09-25 已运行 DDS 导出及 `--check`，15 个文件共 28,054,454 bytes 校验通过；6 张纹理均含从原始尺寸至 1×1 的 11 级 mip，Concrete 法线图的第 0 级解码像素与 JPG 主文件一致，旧运行 JPG 已移除

上述结果验证文件完整性、mip 数据与可复现导出，最新 GPU 材质效果、相机近景和场景性能由 Viewer 验收记录承接
