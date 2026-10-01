# 建筑共享法线修复

基线为 R4 `5cd3037`；Root 完成 R5 反射 A/B、确认输入冻结后解除源修冻结。本轮复用同一法线函数，将7种已使用组合集中到 AST-003 材料目录，原位编辑四个既有 Blender 主文件并导出 V-15、V-A08、V-55；V-35 运行 GLB 保持完整原字节。源工程复读、几何保护与逐像素检查 PASS，三个修复资产的 R5 capture 均 PASS；Root 已实际查看代表帧，结论限定于下述脚本与观察范围，作者验收仍待完成

## 唯一共享源

[共享烘焙入口](../../../../source-assets/environment-kit/materials/bake_normals.py) 维护一份 `scaled`／`baked_normal`，列出本轮7个已知组合，使用现有 Pillow；[V-35 旧入口](../../../../source-assets/buildings/V-35/bake_normals.py) 仅转发兼容历史命令。准备阶段的两份候选脚本已移除，不保留第二份待维护函数

| 资产 | 目标材质 | 烘入原强度 | 处理 |
| --- | --- | --- | --- |
| V-15 | Plaster001、Concrete034、WoodSiding009 | 0.35 | 3张新 PNG |
| V-A08 立面 | Plaster、Concrete | 0.28 | 2张新 PNG；独立屋檐包保持 |
| V-55 | Plaster、Concrete | 0.25 | 复用 V-35 两张已有 PNG |
| V-35 | Plaster、Concrete | 已烘入0.25 | 两张 PNG 原字节迁入共享路径，主文件只更新 packed 图片来源路径；运行 GLB 不重新导出 |

共享路径为 `source-assets/environment-kit/materials/{stem}-NormalGL-scale{025|028|035}.png`，共7张，实际新增5张。原授权 JPG 保留，CC0-1.0 许可、原图与衍生 SHA、脚本关系登记于 [AST-003 manifest](../../../../source-assets/environment-kit/asset-manifest.json) 的 `derived_normals`；这些图仍内嵌于建筑 GLB，不增加独立运行导出条目。海报及字体保持各自既有许可，未将其标成 CC0

本机 Bevy 0.19.1 glTF loader 未应用 `normalTexture.scale`，既有不同源强度因而按完整法线读取。公式从原始 RGB 解码 `n = rgb/127.5-1`，仅将 x/y 乘原强度、保留 z 后归一化并量化到 RGB PNG；不做 sRGB 转换。Blender Normal Map Strength 与导出 scale 均为1，当前9个目标槽（7个新修、V-35既有2个）全部符合合同。版本差异有 R4 归因证据，不能据此声称其他版本相同，单级 PNG 也不证明远距采样稳定

## 实际执行与保护

[commands.json](commands.json) 记录真实命令、执行环境与退出状态；[bake.json](bake.json) 记录7张图的原图／衍生哈希。准备阶段 [candidate-check.json](candidate-check.json) 已在内存重现 V-35 两张 PNG 的完整字节，正式迁移后仍一致

[单资产源修脚本](candidates/patch_source.py) 在打开前检查四个冻结源哈希，只替换所列法线与强度；保存前检查网格、UV、法线、FONT、变换、其他材质输入／连线及非目标图片。四份 `V-*-source-repair.json` 记录实际保存前后语义和源哈希；没有执行任何 `--rebuild`、风格参数修改或新增几何

首次 V-15 误在沙箱运行，保存保护通过后卡在 PulseAudio 退出；宿主确认仅本轮 PID 35404 后发送 SIGTERM，exit143，保留 [初次日志](V-15-source-repair.log)，不把落盘记为进程成功。其余源修完成后，最终在宿主无音频、2线程 Blender 中重新打开四个保存主文件，exit0，见 [source-reopen.log](source-reopen.log) 与 [source-reopen.json](source-reopen.json)

首次保存后复读发现 V-A08／V-55 全材质指纹不同，已停止导出并只读对照备份。精确差异仅为各自一个零引用、无 fake user 的默认 `Material` 被 Blender 自动清除；全部对象、使用中材质及非目标图相同。[诊断明细](reopen-diagnostic.json) 和 [初次失败](source-reopen-initial.log) 保留，最终复读先验证旧默认材质 users=0，再仅允许这两项消失，未放宽在用材质或几何检查

[export_saved.py](export_saved.py) 依次调用三个既有普通导出入口，宿主单进程、2线程，三次 exit0；[export-commands.json](export-commands.json) 保留命令。V-15 原入口会输出旧任务目录，本轮将新报告另存 `V-15-*.json` 并恢复历史报告原字节，没有重写旧证据

[before-semantics.json](before-semantics.json) 与 [after-semantics.json](after-semantics.json) 由 [semantic_snapshot.py](semantic_snapshot.py) 实际解析四个 GLB。前后全部几何 accessor、UV、法线、索引、非目标材质值、颜色／海报图片字节、节点／场景变换与纹理绑定均相同；[runtime-preservation.json](runtime-preservation.json) 保存逐项保护结果，V-35 整个运行 GLB SHA 仍为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`

| 资产 | 修复后运行 SHA-256 | 实际检查 |
| --- | --- | --- |
| V-15 | `b3617d3c57d8e466ce4e6ca3e5fe4dd3f2b55d596f39c43a3cbdd5405d313bdd` | [V-15-check.json](V-15-check.json)，12,362三角、10材质、7图，海报原字节与正面UV通过 |
| V-A08 | `12e0fdef03f7a2ac2fcd22f2661302b0c29bc1932a0ac83d6ea7d8a02af1e09a` | [V-A08-check.json](V-A08-check.json)，6,060三角、6材质、4图 |
| V-55 | `597bc0b96477295efd6a743e2ebf3884ad8b94305e9ba2e93d232e03b258cff4` | [V-55-check.json](V-55-check.json)，7,362三角、6材质、4图 |
| V-35 | 原字节保持 | [V-35-check.json](V-35-check.json)，3,612三角、7材质、4图 |

四包系统 Python 检查实际 exit0，核对 AST-003 原授权输入哈希、全部颜色原字节、共享法线 PNG 原字节及逐像素公式、scale1，同时保留各自几何／法线／UV／净空检查。源修检查证明局部资产数据保持，三个新 GLB 的真实画面由下述 R5 复验覆盖；V-35 原字节保持，原 R4 图像证据仍适用

[清单检查](manifest-check.json) 核对四包当前 source／runtime／脚本与7种原图、衍生图记录；[环境导出只读检查](environment-check.log) PASS，37文件共95,470,486 bytes，证明新增源派生元数据没有改变既有独立环境运行导出。文档检查见 [docs-check-final.log](docs-check-final.log)

## R5 实机复验

`cinema-normal` 150帧、`va08-normal` 120帧、`v55-normal` 120帧均 PASS，各为6项原生／6项包装检查，实际脚本与状态由 [R5 整合记录](../blender-integration-r5/review.md)归档。Root 实际查看 `cinema-normal` 第59帧、`va08-normal` 第49帧、`v55-normal` 第49帧，所看近远材质表现稳定，暂未见新增明显颗粒块；这属于限定机位的画面自查，独立连续帧观察由 [R5 画面记录](../blender-integration-r5/visual-review.md)继续记录

本次脚本覆盖当前修复 GLB 的加载、渲染与相机输入，不代表任意距离过滤稳定、人物真实通行或作者美术验收。源工程、贴图与四个运行 GLB 自冻结后保持字节不变

## 保留与清理

本轮未生成临时截图、渲染帧或视频；7张正式共享 PNG 是源资产，保留。`output/assets/normal-scale-shared-r1/` 中四个 before 主文件与四个 before GLB 共约59MiB，供保存后语义对照及回退，属于明确保留的源基线；无本轮 `.blend1`。原图、许可、正式资产、复现脚本、日志与 JSON 均保留
