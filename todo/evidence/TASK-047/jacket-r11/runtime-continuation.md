# TASK-047 · 上衣 r11 的 R4 运行接续

2026-10-01，承接[源模型制作与正式回写](review.md)以及 [R4 捕获清单](../../TASK-049/blender-integration-r4/captures.json)。本接续读取归档 run／压缩 state 和既有实际看图结论，没有重新运行游戏或代替作者评审

## 实际输入与机器结果

运行基线为 `d7222a21786ad79c4c686748bb4fad13aaedd5c1` 上的 R4 未提交整合，正式 `n-side` 二进制 SHA-256 为 `20c24de20fa8859db7d2dd33e148ad07b12fd180c1a37ed7c057b2d5ac4092cb`；实际加载 CHR-001 r11 GLB 为 `0f02f5eb635b356ab25325829bbdb51fbe021f723fe162abe9e4090486cb87db`，与本轮正式导出及复看候选一致。其他运行输入由 [R4 输入记录](../../TASK-049/blender-integration-r4/runtime-inputs.json)维护

| 真实路径 | 帧数与时长 | 客观结果 | 原始记录 |
| --- | --- | --- | --- |
| `yao-orbit` | 240／240 帧，30 FPS，8 秒，1280×720 | PASS，进程退出 0，9 项引擎检查／10 项包装器检查；实际 CHR-001 加载、Idle 推进、锁鼠绕摄与原地接地通过 | [run](../../TASK-049/blender-integration-r4/captures/yao-orbit/run.json) · [state.gz](../../TASK-049/blender-integration-r4/captures/yao-orbit/state.json.gz) |
| `yao-motion` | 540／540 帧，30 FPS，18 秒，1280×720 | PASS，进程退出 0，27 项引擎检查／28 项包装器检查；实际 Idle／Walk／Run／Jump、输入及暂停恢复路径通过 | [run](../../TASK-049/blender-integration-r4/captures/yao-motion/run.json) · [state.gz](../../TASK-049/blender-integration-r4/captures/yao-motion/state.json.gz) |

绕摄仅观察到 Idle；四动作来自 motion。motion 包含脚本明确输入的三次 R 恢复与三次跳跃，不能称为零复位的连续游玩路线。模拟输入不等于实体手柄操作，本轮没有编码或观看视频；原 run／state 的自动 `visual_review` 字段保持原状，实际看图另记于下节来源

## 实际画面结论

[R4 画面自查](../../TASK-049/blender-integration-r4/visual-review.md#曜上衣-r11-实机自查)记录主工作线实际查看：环绕侧面 89、背面 139 及连续 137–139；Walk 141、Run 连续 190–192、Jump 正面 69 和背侧 392。已查看样本的路径与代表帧哈希保留在该记录中

这些画面支持保留本轮局部增量：正常画幅中能读出较宽松的衣身与袖筒，帽兜、袖口和手腕保持相邻；所看连续段没有新增大块破洞、袖筒拉散或突然跳位

人物仍为 `needs_revision`：袖筒有夸张的圆筒膨起感，背部仍是大片灰面，裁片关系与受力折形不足；裤形、脸发、最终配色及作者形象接受尚未完成。没有本次运行 r10 的同机位 A/B，源坐标变化不等于同比画质提升；这些采样帧也不能证明完整动作不存在自相交

## 下一动作与边界

第 16 轮完成制作、保持检查与真实引擎接续，TASK-047 继续 active。下一轮先针对袖筒圆筒感与背部大面制作有限的裁片／受力方向造型对照，以同尺寸正背面和弯肘姿态判断改动是否可读；保留已验证的领口、袖窿、门襟、帽兜接触区、袖口、32 骨及四动作，不把缩小圆筒或增加几条细纹直接等同于日漫服装完成

正式配色继续未定；作者外观验收、实体设备试玩和完整动作品质均未由本批放行。运行视觉媒体的清理由 R4 主工作线统一记录，本接续没有新增截图或缓存
