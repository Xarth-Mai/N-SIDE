# 街树模型 r1 检查

本轮实际制作一株 6m 的项目原创阔叶街树，替换目标为既有 `tree_a`；原树是 Kenney `tree_detailed.glb`，6m、402 三角，仍保留原件与历史来源

## 实际执行

- Blender 4.5.14 LTS 背景模式、4 CPU threads 制作、保存可编辑 `.blend` 并导出 GLB；Cycles CPU 24 samples、720×840 分别从 0°／120°／240° 检查，原始日志见 `blender.log`、`blender-final.log`、`export.log`
- `python3 source-assets/environment-kit/vegetation/check_street_tree.py`：PASS，见 `geometry-check.json`；检查实际顶点和索引、finite、归一法线、UV／COLOR_0、两种材质、透明模式、叶片净空、米制边界和半径
- 从保存后的 `.blend` 再次导出得到逐字节相同的 GLB，见 `export-check.json`
- 将真实 GLB 中叶材质改为 `BLEND` 的坏样本交给同一检查入口，实际返回退出码 1 和 `tree must use opaque materials`；坏样本已删除
- 最终主文件与运行 GLB 逐字节一致，清单待整合条目见 `integration-entry.json`；未编辑共享 appearance、环境清单、场景与任务卡

## 实际看图 self-audit

首稿的尖菱形叶片呈竹叶式尖锐轮廓，已改为六边阔叶，减少中脊折痕并打散竖向叶冠分布；最终三张 CPU 画面均已实际查看。枝干从躯干延伸到叶簇，不再由块状绿色球团围住树干；三方向都能辨认分枝、叶间天空与有限的暖叶，底部接地

四组叶色由顶点颜色共用一个叶材质，将五个材质合为树皮／叶片两个 primitive。颜色未烘焙固定太阳阴影；初稿几何地面检测还发现倾斜根部截面低于 0，已按实际最低顶点统一归一至 0，最终高度恰为 6m

## 覆盖边界

这次是 CPU 模型自查，不是 Bevy GPU 运行或作者美术验收。现有树点与 `tree_a` 3.5m 布置限额保持不变，导出模型半径约 3.156m；新增模型 8,782 三角，比原 402 三角明显增加，实际多树场景的载入、帧时间和移动闪烁需主 Agent 串行 GPU 采样。没有制作风动画、LOD、树皮纹理、其他树种或树林，不把单株改进写为 README 目标已经达到

运行接入：`appearance.models.tree_a.file` 改为 `environment/vegetation/street-tree.glb`，scene=0、scale=1。统一源清单加入 `integration-entry.json` 的条目，同时正确标注项目原创许可，不能继承第三方环境素材包的全包 CC0 描述

临时三角度图在完成实际查看及文字记录后按项目要求清理；源 `.blend`、导出 GLB、制作／导出／检查程序保留，清理统计见 `cleanup.json`
