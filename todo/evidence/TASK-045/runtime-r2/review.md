# 天空、街树接入后的室外路线复验

2026-09-30，读取本轮真实 capture 的 run、state、script 与 runtime 日志，并使用 view_image 独立检查原始 PNG；本记录是有上下文的视觉自查（self-audit），没有真人操作或作者审美反馈

## 输入与机器结果

主 Agent 实际执行 `python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-shop-exterior.json --output output/capture/task045-r2-sky-tree-shop`，本目录保留原始 [run.json](run.json)、[script.json](script.json)、[state.json](state.json)、[runtime.log](runtime.log) 与 [FFmpeg 日志](ffmpeg.log)，核对其退出码为 0、status 为 PASS，视频编码成功；本次文档收束没有再次运行 Cargo 或 GPU

| 项目 | 实测 |
| --- | --- |
| 构建标识 | run 记录 HEAD `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3`，工作树非干净，二进制 SHA-256 `3d4ffc583adabeef3e8c0e32e7d9a1f78417e1df5bb200efd121ebbf0f4096c1` |
| 脚本 | SHA-256 `057d3f4870916d2651be013a62a6097d1167dabd4e1fc3f64e6adbf5d4925ab6`，walk-preview / shop，1280 × 720、30 Hz、450 帧 |
| 画质 | 状态记录 FOV 55°、曝光 EV100 9.7、MSAA4、阴影 4096、雾开启，SSAO／Bloom／接触阴影关闭 |
| 设备 | runtime 记录 Linux、AMD Radeon RX 6650 XT、RADV / Vulkan、Mesa 26.2.3-arch3.1 |
| 状态断言 | 9/9 PASS：必需资源、450 帧保存、有限 Transform、窗外停留、返回室外、路线支撑与画质、三段人物位移 |
| 实际位移 | 接近 6.400 m、横移 6.191 m、返回 6.188 m，三段期望均为 5—7 m |

新增三段位移条件已经在真实运行中通过，承接[独立审查 R1](../../TASK-049/integration-review-r1/review.md)要求的复验，不能由静止出生点满足。所有原始机器报告的 `visual_review: NOT RUN` 保留原样，后续实际看图结论在下节记录，不把生成报告自动改写为视觉通过

场景与天空的后续 CPU 结果读取 [13 项场景测试](../../TASK-049/integration-r1/cpu-r2-0.log)及[3 项视觉测试](../../TASK-049/integration-r1/cpu-r2-1.log)，包括住宅边界、公共外皮、植被避让、天空连续方向和相机组合；这些是主 Agent 运行日志，本次只核对，不声称重新运行。原创街树的 Blender 源、847 片阔叶、6 m、8,782 三角、2 材质及 OPAQUE 检查由[资产制作记录](../vegetation-r1/review.md)承接；25 个环境运行文件含 3 份许可，混合 CC0 与原创 MPL-2.0，不能写成新增 25 种资源

## 实际画面自查

[visual-review.json](visual-review.json)记录实际查看的帧号、原 PNG 字节与 SHA-256：关键帧 125、199、330、445，连续帧 136—140（横移）和 315—319（转向后的稳定段），共 14 张；没有使用视频播放工具，未声称逐帧看完全部 450 帧

- 125 与 445：月台杂货中文店名完整，门框、雨棚、窗框、浅搁板和木饰板能与灰泥墙区分，阴影没有吞没窗内商品图形；中央胶囊占位角色遮住部分门口，当前画面不能作为正式人物表现验收
- 136—140：真实横移中窗框、雨棚、招牌和铺地连续移动，未见抽查帧间突然丢失或跳位；仅覆盖这一短段，不能排除整条路线的所有细节闪烁
- 199 与 330：斜视能看见窗框厚度、雨棚支架及路面高差；坡道／平台接合处仍可见细长暗缝，具体几何成因本轮未诊断，后续需同机位检查修复
- 315—319：转向后画面保持稳定，没有额外相机漂移；招牌仍受右上 HUD 和构图边缘遮挡，不能把这一近景当作完整街景构图完成
- 本组帧的天空占比很小，未出现清楚可辨的新 tree_a 街树近景；199 左缘仍可见其他块状树冠，不能把它误认成新街树。方向天空的较大视野见[N站复验](../../TASK-049/runtime-r2/review.md)，新街树的真实枝叶、投影和移动闪烁仍需专项镜头

## 成本、结论与下一步

本次状态记录可用作后续相同条件采样的单次参考：观测到 loading→ready 为 0.062742 s，ready 时资源计数稳定为 5,019 entities、44 images、52 materials、3,387 meshes；世界 capture 帧间隔 415 项，p50 25.664 ms、p95 29.219 ms、最大 54.612 ms。指标包含截图回读与 PNG 工作，不是 GPU 帧时或原生 FPS；load 数值不包含观察前准备，资源计数不等于 GPU 字节。没有同仪表、同构建配置的受控旧版重跑，增量性能比较为 NOT RUN

PASS：机器路线与加载检查、上述短段视觉自查记录完成。需要继续制作：铺装接缝、店面纵深、街景构件与植被镜头覆盖。NOT RUN：新街树专项实机视觉、同条件前后成本比较、完整采购街往返品质、真人手感、作者审美、Windows 与物理手柄；没有据此接受 README 概念图目标或 G2

文档收束与 JSON 解析检查 PASS，命令和共同日志见[本轮文档检查](../../TASK-049/runtime-r2/review.md#成本结论与下一步)；任务看板同步和视觉产物清理由主 Agent 统一处理

下一步从同一正式入口补街树近景与住宅沿街路线，实际检查枝叶、树影、入口遮挡、地面接缝和移动闪烁，再修复并复验。任务保持 active

本次收束未生成新截图或视频，也未删除主 Agent 的视觉产物；原 `output/capture/task045-r2-sky-tree-shop/` 的 PNG、关键帧与 MP4 已查看并记录 hash，交主 Agent 在本轮统一清理后补清理记录，日志、脚本、状态与本文保留

root 已统一清理本次输出中的 PNG 连续帧、关键帧及录屏，日志与状态保留；数量和字节见 [cleanup.json](cleanup.json)
