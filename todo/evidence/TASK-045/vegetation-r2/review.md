# 街树叶簇与树皮 r2

针对 [TASK-049 近景自查](../../TASK-049/residential-r1/runtime-review.md)的稀疏多边形叶片与纯色树干，实际修改原创街树主文件、造型／导出／检查程序及运行 GLB；源文件仍在 `source-assets/environment-kit/vegetation/`，本记录是 CPU 制作 self-audit，真实街景结果由根 Agent 的本轮运行证据承接

## 制作与主文件保护

修改前 vegetation 目录工作区干净；先复制原源文件至本轮临时目录，用原 `.blend` 重导出，再运行复制的原始固定 seed 造型程序。两路 GLB 都与原件 `bcf0c94df78f5ebdd6215386d194b8d1f49307cbb9ce5d4b2ddad4e94b7deaf9` 逐字节一致，确认没有被生成程序遗漏的手工几何／材质变更后再重建，见 [master-check.json](master-check.json)与 [inputs.json](inputs.json)

保留 11 条明确的主枝端点、6m 高、根部枢轴、初秋四组叶色、实体 scale=1 与两材质结构。每根细枝改为交错 9 叶，单叶由六边改为圆肩八边浅弯几何，叶长范围从 0.30–0.46m 调为 0.28–0.40m，补轻微根尖色差；细枝侧数从 5 降为 4，树冠仍由真实枝叶与空隙组成

新增 256×512 原创程序绘制树皮色图 `bark-color.png`，低对比纵纹随沿枝干轴展开的 UV0 分布。纹理使用 sRGB 色值，打包进 `.blend` 并嵌入 GLB，不增加外部运行依赖；没有使用第三方图像或生成服务，原创内容继续沿用 MPL-2.0。未增加法线图，纵纹是颜色信息，不声称已有凹凸几何

第一次候选减少到每主枝 9 组细枝，实际看图发现枝间空隙扩大、树皮 sRGB 色值过暗，因此撤回该叶簇减量并修正色值；失败候选与已查看图 hash 见 [iteration-1.json](iteration-1.json)

## 实际检查

```sh
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/build_street_tree.py
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/export_street_tree.py -- --render
python3 source-assets/environment-kit/vegetation/check_street_tree.py
python3 source-assets/environment-kit/vegetation/check_street_tree.py game/assets/environment/vegetation/street-tree.glb
```

- PASS：Blender 4.5.14 LTS 背景制作与主文件保存，最终 GLB 从保存后的 `.blend` 重导出逐字节一致，见 [build.log](build.log)、[after-render.log](after-render.log)及 [export-check.json](export-check.json)
- PASS：实际顶点／索引、归一法线、有限数据、UV0／COLOR_0、两种材质、不透明模式、叶片双面／净空、米制边界、半径、嵌入 PNG 尺寸与树皮 UV 非退化检查，见 [geometry-check.json](geometry-check.json)
- PASS：把实际导出 GLB 的叶材质改为 BLEND，以及把树皮 UV 全部压为零的两个坏样本均被同一检查器拒绝，退出码 1；坏样本已删除，结果见 [export-check.json](export-check.json)
- PASS：最终源 GLB 与运行 `game/assets/environment/vegetation/street-tree.glb` 逐字节一致；清单条目供根 Agent 合并，见 [integration-entry.json](integration-entry.json)，未由本 Agent 修改共享清单、世界场景或任务卡

## 同机位 CPU 看图

前后均用 Cycles CPU、4 threads、24 samples、720×840、AgX、相同区域光与地面；全树为 0°／120°／240° 三向正交机位，另有树干近景。具体参数在 [inputs.json](inputs.json)，前后各四张均已实际查看，图像 hash 见 [before-views.json](before-views.json)与 [after-views.json](after-views.json)

最终叶簇中的叶片较小、叶肩较圆，沿细枝的交错叶序可辨；三个方向仍能看清主干、分叉与冠部空隙，没有用球形体积填满树冠。树皮近景能看到低对比纵纹，颜色没有第一次候选的明显发黑；全树尺度下纹理较克制。叶片仍为双面薄片几何，斜向会呈细边，本次静态 CPU 图不证明移动闪烁、全城性能或作者美术验收通过

根 Agent 另外实际查看前后 `tree-0` 与 `bark-detail`，确认叶肩改善和树皮纹理出现，同时指出树冠仍显稀疏分层，后续需要继续制作叶簇层次；本轮不宣称概念目标品质已经达到

## 增量与边界

| 项目 | 原版 | 最终候选 | 增量 |
| --- | --- | --- | --- |
| 三角面／实例 | 8,782 | 11,686 | +2,904，约 +33.07% |
| 叶片 | 847 | 1,089 | +242 |
| GLB 文件 | 409,052 bytes | 709,372 bytes | +300,320 bytes |
| primitive／材质 | 2／2 | 2／2 | 0 |
| 内嵌图像 | 0 | 1，256×512 PNG，60,358 bytes | 单份共享纹理 |
| 高度／最大半径 | 6m／3.155431m | 6m／2.992843m | 保持 3.5m 放置限额 |

12,000 三角是本轮候选检查上限，代替未经实际性能验证的旧 10,000 暂定值，不是已获批准的全城运行预算。总树点为 72；准确 `tree_a` 实例数需要按真实 Ground 高程与既有分配路径统计，不能把 72 全部算作本模型。全城几何增量为 `tree_a 实例数 × 2,904`，GPU 时间、显存、原生 FPS 和移动稳定性由根 Agent 实机对照记录，CPU 图片不作性能结论

来源、代码、主文件及最终导出 hash 见 [outputs.json](outputs.json)。完成自查与根 Agent 查看后已清理本轮 12 张 CPU PNG 及临时 baseline 副本，删除前确认无进程打开该目录内文件，输出剩余 0 bytes；数量、字节与删除文件 hash 见 [cleanup.json](cleanup.json)。源纹理、Blender 主文件、运行 GLB、日志、JSON、复现程序与文字结论保留

文档检查 `python3 -B tools/validate_docs.py --root .` PASS，419 Markdown files／102 IDs，见 [docs-check.log](docs-check.log)；本轮受影响文件 `git diff --check` PASS。共享环境导出、全仓检查与真实 GPU 对照由根 Agent 统一运行

## 真实街景补证

Root 合并清单后运行 `bun tools/export-environment.ts`，25 文件／45,588,742 bytes 导出 PASS；同一 `street-tree-detail.json` 前后各 150 帧、6 项机器检查 PASS，真实 Viewer 关键帧59及后续横移62已查看。叶肩由尖锐大片改为更圆的小叶，纵向树皮纹理加载可见；树冠仍稀疏分层、叶片薄边可辨，未达到最终日漫街景品质，详见[集成运行摘要](../integration-r3/runtime-summary.json)

沿用 `map_viewer --validate` 对实际 `PreparedScene.props` 按型号计数，`tree_a` 为16实例，几何从140,512增至186,976三角，增加46,464三角；其他类型不套用本模型面数。独立静态纹理文件60,358 bytes共享，不将实例数乘纹理存储。诊断命令与日志见[实际场景计数](../integration-r3/scene-validation.log)

两次捕获处于同一RX6650XT、相同150帧机位和参数；总场景对象为4,983→5,013、mesh资产3,386→3,416、image40→41、material51→51，每次首末计数一致。整体变化同时包含陈列与铺装，不能作为街树单独的draw call或GPU成本。捕获帧间隔p95为38.09→37.71ms，包含截图回读与PNG工作，未测原生FPS、GPU毫秒或显存；不据此建立性能预算
