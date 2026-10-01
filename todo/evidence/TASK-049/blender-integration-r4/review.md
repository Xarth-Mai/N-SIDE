# Blender 接入 R4：整合验收

基线 `d7222a21786ad79c4c686748bb4fad13aaedd5c1` 上的未提交批次，接入 BYTE BEAT 的 V-35 北立面与曜上衣 r11，沿用现有场景、控制器、碰撞和捕获入口。7组真实 capture 均通过机器检查，共1,470帧、66项原生检查、68项包装器检查；首次 V-35 画面存在黑点，机器通过没有被当作视觉通过，归因和修复记录保留。作者品质验收和任务放行未由本报告代签

## 本批变化与输入

- V-35 原创立面按 `[-386,153,12]` 唯一锚点接入，只接管北面泛型外观与1.5m雨棚，保留原楼体、其余三面、南后勤门和 BYTE BEAT 招牌；[整合边界](scene-integration.md)记录真实 GLB 的3,612三角面、7材质和4图，以及实际门体投影支撑的0.55m门前站距
- 曜 r11 调整连续夹克衣身与袖筒，沿用原骨骼、权重、UV和动画；[20项保存契约](../../TASK-047/jacket-r11/promoted-contract.json)通过，四动画的实际运行结果见下表
- [runtime-inputs.json](runtime-inputs.json)保留初次源与二进制快照及正式法线修复 hash；地图保持 `a2f15c59246a57c1250ee76db54e0a12fa5ce766f17f15dc1535e5895128038f`，Viewer 为 `1690bfe27f125cbe590f1bb0adeec389e158cebc9fb155adb1fe834885776035`，正式入口为 `20c24de20fa8859db7d2dd33e148ad07b12fd180c1a37ed7c057b2d5ac4092cb`
- 原 V-35 GLB 为 `cca2fe302db5fce06565f67a03d12d8b3205f963988e4278c156a26a8288bd1b`，正式法线修复为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`；曜运行 GLB 为 `0f02f5eb635b356ab25325829bbdb51fbe021f723fe162abe9e4090486cb87db`

## 从实际问题到复验

1. 首次 `v35-facade` 与 `v35-entry` 的资产、相机、截图检查 PASS；实际看图发现新浅墙密集黑椒点，材质视觉为 FAIL／需修订，详见 [画面自查](visual-review.md)
2. 本机 Bevy 0.19.1 glTF loader 忽略 `normalTexture.scale`，源制作的0.25强度未在引擎中应用；[normal-off 隔离记录](normal-isolation.json)只去掉 Plaster 法线输入，几何、其他材质、光照不变。150帧机器 PASS，同机位第59帧黑点消失而窗框阴影保留，支持法线链归因
3. [正式源修](normal-scale-r1/source-repair.md)将 Plaster／Concrete 法线的 x/y 乘0.25后归一化，保存无损 PNG，并将 Blender Strength／GLB scale 设为1；原 CC0 JPG 保留。[源保留](normal-scale-r1/source-preservation.json)与[运行导出保留](normal-scale-r1/runtime-preservation.json)分别核对几何、UV、节点和其余材质不变，修复没有重做场景或弱化碰撞
4. `v35-entry-final` 与 `v35-facade-final` 使用正式修复 GLB，分别150／120帧、各6项机器检查 PASS。[实际自查](visual-review.md)检查入口第59帧及40–42连续帧、总览第49帧及31–33连续帧：首轮黑椒点消退，窗框／腰线／雨棚层次仍在，所看片段没有可辨的大块颗粒跳动或表皮覆盖切换，支持本次修复保留

法线 PNG 仍只有一级 mip，本次强度修复没有解决所有距离下的纹理过滤；暗窗玻璃、设备细节和外场密度仍需后续制作。曜所看动作未见新增明显破洞或跳位，但袖部、布料裁片、脸发和配色仍未完成，未批准参考成品品质

## 实际运行与技术检查

全部录制为1280×720、30fps、MSAA4，脚本与命令见 [复现入口](scripts/README.md)及[captures.json](captures.json)。角色包装器各增加一项实际 CHR-001 身份检查，因此原生总数与包装器总数不同

| 运行 | 帧数／时长 | 原生／包装检查 | 证据用途 |
| --- | --- | --- | --- |
| v35-facade | 120／4s | 6／6 PASS | 初次立面横移，保留黑点视觉问题 |
| v35-entry | 150／5s | 6／6 PASS | 初次门前横移，保留黑点视觉问题 |
| yao-orbit | 240／8s | 9／10 PASS | 正式 CHR-001 的 Idle 动画与真实镜头环绕 |
| yao-motion | 540／18s | 27／28 PASS | 实际 Idle／Walk／Run／Jump及原有暂停恢复 |
| v35-normal-off | 150／5s | 6／6 PASS | 只关闭 Plaster 法线的诊断资产，不是最终交付 |
| v35-entry-final | 150／5s | 6／6 PASS | 正式0.25强度烘焙法线的入口复验 |
| v35-facade-final | 120／4s | 6／6 PASS | 相同正式资产的广角立面复验 |

Viewer 证明真实渲染和 FreeCamera 输入，不证明人物通行；闭门接近由实际 GLB 三角形、Ground 支承与 capsule 窄测覆盖。角色动作脚本实际包含3次 R 恢复输入，用于分段动作检查，不是无恢复连续路线；本轮没有重复录制 R3 长路线，模拟手柄不等于实体设备验收

| 已执行检查 | 结果与范围 |
| --- | --- |
| Rust lib 完整测试 | [141 PASS、0 FAIL、1 ignored](lib-tests.log)，含 V-35 真实 GLB 接近／闭门命中、保留立面、既有小店与登高路线；发生于法线修复前，修复后的几何保留另有逐项对照，不写成重跑完整测试 |
| Viewer 测试与开发构建 | [3项 PASS](viewer-tests.log)，[构建完成](build.log)；同一二进制用于七组运行 |
| V-35 源修与最终导出 | [烘焙逐像素核对](normal-scale-r1/source-bake-check.json)、[最终 GLB 检查](normal-scale-r1/runtime-check.json)、[保存后重新打开 Blender 检查](normal-scale-r1/source-check.log)均 PASS |
| 源码与来源复核 | [code-review.md](code-review.md)记录独立窄测、manifest hash、法线修复与角色保存契约复核，结论 `Lean already. Ship.` |
| 文档／Skills | [587份Markdown、129个ID、31 Skills、25 imports PASS](docs.log)，本汇总落盘前快照，最终文档复验由主任务追加 |
| 环境导出 | [37文件／95,470,486字节 PASS](export-check.log)，以该检查器覆盖的既有环境导出集合为范围，V-35另由上述资产检查覆盖 |
| 地图与双 Wiki 构建 | [61项地图检查、player 128页／679文件、dev 223页／1,046文件 PASS](wiki.log)；dev大于500kB的chunk提示仍在，非构建失败；最终报告变动后的检查由主任务收尾 |

视频均明确 `NOT RUN: --no-video`；画面自查使用真实关键帧和连续帧，审阅者知道设计与源码，属于 self-audit。最终 run.json／state.json 内工具生成的 `visual_review: NOT RUN` 原样保留，人工观察单独记录

## 非媒体归档与清理交接

[captures.json](captures.json)索引7组共28个原文件，原始2,986,839字节，归档989,641字节；`yao-orbit/state.json` 与 `yao-motion/state.json` 超过500,000字节，使用 `gzip mtime=0`、空filename压缩，其余原字节保存。每份记录同时保留原始／解压 SHA-256与存储 SHA-256，已逐份读回核对；没有复制 PNG 或视频

可从仓库根只读查看压缩角色状态，命令支持 fish：

```fish
gzip -dc todo/evidence/TASK-049/blender-integration-r4/captures/yao-motion/state.json.gz | jq '{status, checks: (.checks | length), samples: (.samples | length)}'
```

Root 确认宿主无本轮游戏或 Blender 进程后，已清理两处本轮输出目录的 1528 个媒体文件，共 1,644,274,071 字节；[cleanup.json](cleanup.json)保留逐件 hash，正式源图、模型、回退源工程与运行日志仍保留。归档读回核对28文件、2份gzip解压与mtime、7组计数及报告引用均 PASS；最终文档检查由主任务追加

## 收尾检查

最终 `bun run tasks:sync`／`tasks:check` 47任务卡 PASS；[check:docs](docs-final.log)595份Markdown、129ID、31Skills／25imports PASS；源码 `git diff --check` PASS。新增材质经验后双 Wiki 已[重新构建通过](asset-docs-build-final.log)，[首次构建](asset-docs-build.log)因系统 Node 缺 `libsimdjson.so.33` 失败，改用本机现有 Codex bundled Node 的 PATH 后通过，没有安装或升级依赖。输出目录余量为 capture 3.0MB、法线诊断8KB，仅日志／状态等非媒体

提交前差异与现有最小实现复查：Lean already. Ship.
