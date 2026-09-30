# Belle 本地适配 r4

当前结果为本地验证候选，整体仍 `needs_revision`；第三方原包和转换结果保持在忽略目录，仅动作来源为项目自有 CHR-002 Idle / Walk / Run

- r2 使用 bone head→tail 做全身方向对齐，但 SFM 的 Spine / Spine2 / Head1、UpperArm / Forearm 等骨 tail 是轴向标记，并不指向实际下一个关节，结果躯干与腿水平翻转
- r3 保留 r1 的髋部、躯干、下肢 rest 语义，只用 UpperArm→Forearm 和 Forearm→Hand 的关节 head 方向求左右上肢 rest 修正，手沿用前臂校正以保留导入 wrist offset
- r3 实际看图发现脸部沉入领口，诊断表明头部 Mesh Z=1.395–1.596m，未同步变换的 shape-key Basis 及 evaluated Z=1.151–1.316m
- r4 对 Mesh 使用 `transform(..., shape_keys=True)`，保持 Armature 原调用，实际六视图中脸部位置与五官恢复

`pose-check-r1.json` 与 `pose-check-r2.json` 是修复前逐帧数值；r1 肢段方向误差最高 32.80°，r2 最高 92.12°且 Head / Pelvis / Foot 高度关系失败

`pose-check-r4.json` 为最终逐帧检查，Idle / Walk / Run 分别检查 121 / 49 / 41 帧：矩阵有限、Head 高于 Pelvis 至少 0.498m、Pelvis 高于双脚至少 0.557m、Root 位移为 0、首尾矩阵差为 0、两段双侧上肢方向与源动作误差为 0°、shape-key Basis 与 Mesh 顶点差为 0

`render-pose-r4.json` 记录 Cycles CPU、16 samples、480×640 下实际查看的 Idle 正面/三分之四、Walk 三分之四、Run 两个相反相位三分之四与侧面；身体竖直、站立手臂下垂、走跑腿与肘正常运动，所有六图已查看后清理，日志及复现脚本保留

尚未完成：手指动作映射（当前张开，跑步部分手掌向上）、布料/头发次级动作、表情动作、引擎运行与美术验收；本轮没有修改正式角色、玩法或剧情

最终 GLB SHA256：`4d77e7dc09629322b95cb2a6807b8841d0845000be467f7763b4ac79de009e19`

修复后的 `adapt.py` 已交给 root 接手规范工具；此文件为本地诊断记录，不代表第三方模型可重新分发
