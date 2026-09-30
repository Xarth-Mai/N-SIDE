# 公共砖面素材接入

2026-10-01，本轮按用户积极使用公共素材的要求补充一组实际缺少的砖面 Color／NormalGL，沿用 AST-003 的唯一资产清单与现有 DDS 导出器，没有修改场景生成、appearance 材质绑定、地图或现有材质

## 选择与来源

现有 `wall_brick` 使用 Concrete034，缺少砖缝结构；选用 [ambientCG Bricks057](https://ambientcg.com/view?id=Bricks057) 的暖棕砖面供镜厅相邻音乐街、店屋侧墙及围墙局部试排。官方页标 2021-04-11 发布、程序生成、约 `1.05 × 1.05 m` 周期；本轮实际读取[官方许可页](https://docs.ambientcg.com/license/)确认素材及原始文件可按 CC0-1.0 使用和分发，作者归属沿用 Lennart Demes / ambientCG，完整 CC0 正文已保存在环境资产包并随运行资产导出

同时取得并查看 [Metal012](https://ambientcg.com/view?id=Metal012) 原始颜色与法线：变化很弱，主要材料辨识仍需粗糙度和反射条件。现有外观契约尚未接粗糙度图，本轮不为这组素材扩张渲染架构，不导入运行资产；官网未标其米制周期，未编造实测尺寸。下载记录和未采用原因保留在 [source-review.json](source-review.json)

两份完整官方 1K-JPG ZIP 均完成 CRC 校验并记录整包 hash；入库的两张 Bricks057 JPG 与包内成员逐字节相同。四个新增源/运行路径以及来源、许可、周期、原件 hash、尺寸和派生参数写入现有 `source-assets/environment-kit/asset-manifest.json`，旧 33 条登记语义保持不变。没有新增并行资产清单或付费依赖

## 实际检查

| 检查 | 实际结果 |
| --- | --- |
| 完整 ZIP CRC、原件与成员一致性、许可与 hash | PASS，见 [来源检查](source-review.json) |
| 既有导出入口 | PASS，35 个运行文件共 84,517,978 bytes，见 [export.log](export.log) |
| 既有只读重算检查 | PASS，见 [export-check.log](export-check.log) |
| 新增 DDS 基础级像素、尺寸、不透明 alpha | PASS，两张均与原 JPG 解码像素逐点相同，见 [pixel-check.json](pixel-check.json) |
| 文档与 Skill 引用检查 | PASS，见 [docs-check.log](docs-check.log)；首次在清理记录落盘前检查出现缺链，原日志保留，补齐后复验通过 |
| 3×3 平铺图实际查看 | self-audit 已完成，颜色与法线对应、砖行连续，无明显边缘断缝，重复图案仍可辨认 |
| Bevy 场景加载、实际材质、光影与连续移动 | NOT RUN，本轮未改绑定，不将文件检查等同于实机场景验收 |
| 作者美术验收 | 未进行 |

导出实际使用 ImageMagick `7.1.2-32 Q16-HDRI`，沿现有无压缩 BGRA8 DDS 和 11 级 mip 链。每张新 DDS 为 5,592,532 bytes，合计增加 11,185,064 bytes；原始周期与纹理分辨率没有被压缩或重新绘制，当前可降低为 512 的需求尚无实机证据

独立 `public_asset_review` 实际核对完整 ZIP、原件成员一致性、源与运行 hash、11 级 mip、不透明 alpha、DDS 基础级像素与唯一登记，并查看两张平铺图。发现复现脚本缺少输出目录创建、清单导出器版本滞后两项小问题，已补上 `mkdir` 并将版本同步为实际值，复核结论为 `Lean already. Ship.`；这项审查不替代后续场景与作者验收

```fish
bun tools/export-environment.ts
bun tools/export-environment.ts --check
python3 -B todo/evidence/TASK-049/public-urban-r1/check.py
```

## 看图结论与下一步

实际看过原始 1024 图及从运行 DDS 生成的两张 3×3 平铺图：暖棕砖与偏浅灰灰缝能够补出原混凝土没有的建造层次；表面有磨损、少量绿灰痕迹，不宜作为镜厅干净主入口的大面积统一饰面。法线砖缝与色图对应，局部边缘响应较强，接入时需要用真实侧光检查凹凸尺度

建议 root 先把这组材质用于一处已有 `wall_brick` 的可达侧墙，按官网 `1.05 × 1.05 m` 设置实际周期，先保留中性 tint，检查真实人眼高度下的砖块尺寸、UV 方向、阴影和远处平铺重复。控制变化范围，保持地图、入口、碰撞和其他商店材质。准备或导出一组素材不代表全城材质已经完成

临时平铺 PNG 在实际查看、hash 与结论记录后清理，清理范围和保留项见 [cleanup.json](cleanup.json)；原始下载包、源 JPG、运行 DDS、复现脚本和机器日志保留

## 后续实际接入

主线程在同一批中完成 `brick_music` 与 V-W10 首层西面三片饰面接入：仅六个三角形，12mm 外贴、2.2×2.55m 门洞、连续米制 UV，原墙碰撞保持。全库测试通过，独立只读审查核对西向法线、绕序、切线和门框覆盖；清单用途同步为当前候选位置

实际 Bevy 150 帧、6 项检查 PASS，已查看第 59、119、149 帧，砖墙与灰泥分工可辨，门窗未被覆盖。脚本见 [music-wall.json](music-wall.json)，run/state/log、已查看 hash 与视觉边界见[集成运行](../blender-integration-r1/runtime-review.md)。本段补充上述素材导入子项的历史 NOT RUN，不回写成当时已经运行；作者美术、近景旧化及多光照验收仍待完成
