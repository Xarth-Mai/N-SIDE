# TASK-047 · 曜帽兜局部精修 r10

2026-10-01，按[下一批美术 brief](../next-art-brief-r1.md)在当前 r9 主文件副本上只修改 `Hood`；固定灯光看图后由 root 接受这一局部薄化改进，随后通过原位 `--update-hood` 写回正式源与运行 GLB，整体仍为 `needs_revision`，未代签作者外观或最终品质

## 改动与保持

帽口上方两圈的 58 个坐标、UV 与 Chest／Neck 权重逐项保持，下部用七圈不等距轮廓替换原三圈，降低最宽处并缩小后背最大突出量。后袋按夹克的实际 REST 评估表面留出距离，侧向开口保留衣领走向；最低圈收窄形成较圆的下缘。厚度仍为 5 mm，未增加细分、模拟、骨骼、纹理或配色

控制网格从 145 顶点／112 四边面变为 261 顶点／224 四边面。当前 GLB 有 24,381 蒙皮顶点、32 骨、三张嵌入图及 Idle／Walk／Run／Jump，实测高度仍为 1.7441905736923218 m

正式生成器新增选择更新入口，与全量构建共用同一 `update_hood`；本轮没有执行全量重建。原始源、运行资产与玲的哈希见[冻结基线](source-baseline.json)，本轮没有修改玲

## 客观检查

| 证据 | 实际结果与范围 |
| --- | --- |
| [候选保持检查](preserved-contract.json) | 14 项 PASS，含候选阶段正式文件未改；Hood 之外的网格、面、UV、权重、骨架 rest 与层级、全部动作曲线、GLB 节点与 inverse bind、四动作全部导出采样、材质与三图字节精确一致 |
| [正式保持检查](promoted-contract.json) | 13 项 PASS，正式 GLB 与看过的最终候选逐字节相同；沿用上述保持项，帽口上两圈不变 |
| [正式 GLB 检查](promoted-glb-check.json) | PASS，实际 Root、身高、四动作、蒙皮与纹理契约通过 |
| [正式 DCC 检查](promoted-dcc-check.json) | PASS，沿用实际源文件的材质、逐帧骨矩阵、循环端点、足底、胸前图形、脸部与 r9 头发检查 |
| [28 姿态检查](promoted-contract.json) | Idle、Walk、Run、Jump 的实际评估蒙皮中，头部、内衫领口、后发与冠顶未检出 BVH 重叠；不等于连续动作的全部几何与视觉验收 |

夹克原有接触仍未完全消除：相同姿态组中的最大 BVH 重叠对由 308 变为 269，主要在帽兜侧面与接缝。几何数量变化会影响重叠对数，因此不将数量下降写成同等程度的视觉改善，也不声称无穿插

正式源 SHA-256：`7f77c2380050a1d479e98f193c47ed2d4ea22fe62ee0989f54779b35c690a76d`；运行 GLB：`ddb1cb5c0064e6e23994de4987458634ae24575e59d0e54b64f8739aa080719d`

## 看图与失败修正

实际查看同光照的侧面、侧后、正背 CPU 图：先拍原版和初步候选六图，修订下缘后分别再拍两轮三图。均采用源文件的灯光与灰阶、360×480、Cycles CPU、2 线程、12 samples；机位、帧数、哈希见[初轮](views-first.json)、[否决轮](views-rejected-sag.json)与[最终轮](views.json)

- 初次直接收薄增加夹克接触，[初次保持检查](initial-contract.json)保留该诊断；把整个低圈推至后背表面又导致领口新增接触，改为仅约束后袋后，领口接触消除
- 首次看图显示侧面硬块已收薄，但背面下缘仍过平；试验抬高两侧边缘，实际图出现两处黑白毛刺，已否决，尽管该轮 DCC／GLB 检查通过
- 最终撤销抬侧缝，只收窄最低圈；实际侧面可见较薄的下垂轮廓，背面下缘较圆，未再看见该毛刺。root 实际查看原版侧面及最终三角度，同意这一局部改进回写
- 背面仍有较平的大面，真实衣褶、侧缝贴合、整套灰阶衣装、脸部个性和最终配色继续待修；本轮不扩大到其他身体部件

以上属于 self-audit 与 root 复看，不是作者审美验收。CPU 图只覆盖 Idle；Bevy 实际走、跑、跳与动作切换由主工作线继续验证，不用该组静态图代替

## 复现与清理

仓库根目录执行，命令可直接用于 fish；先与其他 Blender 工作协调单实例窗口，按项目规则等待进程退出

```fish
env ALSOFT_DRIVERS=null timeout 180 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --update-hood
env ALSOFT_DRIVERS=null timeout 180 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py -- --output todo/evidence/TASK-047/hood-r10/promoted-dcc-check.json
python3 tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --clip Jump --require-texture
```

本次[选择更新补丁](generator.patch)、[候选脚本](candidate.py)和[比较脚本](check.py)保留为复现记录；后续正式更新使用正式生成器中的函数。比较脚本的 `--formal` 读取正式源与导出物，依赖本轮冻结副本，不是对任意未来版本的固定门禁

[清理记录](cleanup.json)：全部 Blender 进程正常退出后，12 张已查看 PNG 共 2,185,317 bytes、这轮生成的两个 `.blend1` 备份与 Python 缓存已删除；保留正式源、候选源、运行文件、参数、哈希、日志与 JSON。视觉文件路径用于追溯，不能再当作当前仍可打开的图片


## 正式游戏续验

2026-10-01，使用本轮正式GLB和R3实际构建，240帧绕摄与540帧真实走跑跳、锁鼠、暂停／恢复全部通过，共36项机器检查。实际观察见[视觉记录](../../TASK-049/blender-integration-r3/visual-review.md)，原运行状态与日志见[捕获清单](../../TASK-049/blender-integration-r3/captures.json)。三次R恢复是既有动作脚本的分段输入，不证明连续路线无重置；闭门路线另用真实胶囊控制器而非该人物模型验证
