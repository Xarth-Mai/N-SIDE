# CHR-001 支撑脚方向与速度修正

本轮以 `e6b398b` 的真实控制器及已冻结的 r5 形体为基础，修正源动作，再从实际交付 GLB 读取轨迹；不改控制器速度、模型几何、32 骨架身份、灰阶贴图、角色材质、输入或暂停规则

## 根因与修改

旧 Walk / Run 的近地段在 GLB 的 +Z 方向移动，和人物真实前进同向；测得近地均速分别约 +0.597 / +1.620 m/s，加上现有 3.2 / 5.6 m/s 控制器速度后会继续滑动。正的播放倍率无法把方向错误修正成支撑，记录见 `exported-foot-tracks.json` 与 `measure.log`；这是 r5 形体冻结时仍保留的旧动作，不是新动作结果

`build.py` 动作段改为可达的双腿 IK、恒速向后的支撑段与抬脚回摆，保留 in-place Root。源逐帧曲线显式使用 LINEAR，以避免默认 Bezier 在帧间越过已求解的足部目标。脚在摆动两端仍沿原支撑方向短暂减速/加速，配合离地高度，避免立即反向的尖点

|Clip|FPS / 周期帧数|周期|名义前进速度|单脚支撑时长|
|---|---|---|---|---|
|Idle|60 / 120|2 s|0|原地|
|Walk|60 / 48|0.8 s|3.2 m/s|16/60 s|
|Run|60 / 40|2/3 s|5.6 m/s|8/60 s|

左右脚相差半周期，Walk 支撑 phase≤1/3，Run 支撑 phase≤1/5；给定项目现有 3.2 m/s 速度，Walk 是轻快小跑实验，不是自然慢走。`character.rs` 已按碰撞后的实际水平速度除以上表名义速度播放，本轮无需新增运行时倍率或状态框架

## 实际检查

|检查|结果与范围|
|---|---|
|旧资产方向失败路径|`measure.py --check` 实际退出 1，正确报告支撑向前；近地段用脚踝高度识别，仍保留其范围|
|最终 GLB 轨迹|`measure.py --fps 120 --gait --check` PASS；实际读取 glTF STEP/LINEAR 与四元数最短弧插值，不读取拟定的目标值代替实际导出|
|Walk 支撑|两脚平均 -3.200000 / -3.200001 m/s；世界空间支撑漂移峰值约 0.283 mm，瞬时速度残差峰值 0.034 m/s|
|Run 支撑|两脚平均 -5.600002 / -5.600003 m/s；世界空间支撑漂移峰值约 0.642 mm，瞬时速度残差峰值 0.077 m/s|
|源动作半帧|120 FPS 取样，Walk 最低鞋底 -1.509 mm，Run -2.537 mm；均小于本轮 5 mm 检查界限，循环首尾所有骨矩阵差为 0|
|几何保留|`preserved-geometry.json` PASS：静态节点、材料、inverse bind、全部网格属性/索引值与嵌入 atlas 字节均与 r5 shape freeze 相同|
|正式 DCC/GLB 结构|由形体负责者对同一最终主文件执行，见 `../model-r5/dcc-check.json` 与该目录最终交付记录|
|真实 Bevy|由 root 串行验证 `walk-character`；当前表不替代 root 的运行状态、日志与看图记录|

120 FPS 采样与平地名义匀速是测量范围，不证明任意帧率、坡道、转向、碰撞变速或过渡状态下足底锁定。支撑漂移按每段实际接地区间内的 `foot_z + body_speed × time` 累积差测量；判定要求平均速度误差小于 0.01 m/s、位置漂移小于 1 mm。中间摆动与足底滚动不在这个支撑条件内

正式 GLB SHA256：`86bc6a8ce380e40204f1baaddd19008d245ffb09248fceef7355121428450b9f`；正式 master SHA256：`76b93aefb0b3ec9360ccde21cbb7771edad4c3944ed1e2e1efd8451548201861`

## 实际看图

用同一 r5 形体、灰阶、CPU Cycles 与固定侧镜头生成 Walk 24 帧和 Run 20 帧，均为 30 FPS 整周期，按行顺序组成接触表。制作 Agent 和独立 art_review Agent 实际逐格查看，root 已收到产物位置；这属于知情视觉自查，不是隔离玩家盲测

支撑与摆动交替、Run 腾空可见，未看到明显反膝或膝部裤子断裂；Run 双掌朝上、上体接近整块竖直起伏、脚底始终平行地面仍明显，运动表现仍是程序步态。保留为修正反向支撑的技术候选，不把这次轨迹 PASS 记作自然走跑、角色外观或人物动画完成

本轮保留既有 Idle / Walk / Run 硬切；跳跃动作、专门过渡、足跟到足尖滚动、斜坡适应尚未制作。Bevy `AnimationTransitions` 的权重推进独立于单个 clip 的 pause，未在本次源动作修复中引入额外暂停处理

## 复现命令

在仓库根目录运行，命令可直接用于 fish

```fish
# 不覆盖正式模型：在内存中的 r5 master 上重算动作、检查半帧并导出临时 GLB
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/animation-r1/prototype.py
# 追加 -- --render 生成两个完整侧视周期，CPU Cycles；查看后清理图片
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/animation-r1/prototype.py -- --render
# 检查正式资产中的真实足轨，输出只写本轮证据
python3 todo/evidence/TASK-047/animation-r1/measure.py --fps 120 --gait --check --output todo/evidence/TASK-047/animation-r1/final-foot-tracks.json
```

正式重建执行 `output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py`；它会重建并覆盖正式主文件和 GLB。本轮实际命令日志为 `build.log`，最终轨迹日志为 `final-measure.log`

`prototype-foot-tracks.json` 是首版 30 FPS 且用近地高度启发式的结果，`refined-foot-tracks.json` 是 60 FPS / Bezier 中间版本的 GLB 结果，`linear-dcc-check.json` 是最终源动作半帧检查。它们用于追溯修正过程，只有 `final-foot-tracks.json` 绑定本次正式交付 GLB

临时图片、周期接触表和探测 GLB 在 root 与 art_review 看完后清理，清理范围与结果见 `cleanup.json`；复现脚本、日志和状态 JSON 保留


## 正式入口复验

Root 在最终 GLB 上执行 `python3 tools/capture.py --script game/capture/walk-character.json --output output/capture/task047-gait-r1 --binary game/target/debug/n-side --character-preview`，540 帧／26 项检查 PASS，实际切换 Idle、Walk、Run 并验证跳跃、暂停及恢复。root 与艺术审查查看连续走跑及暂停前后；未见新网格断裂，Run 手姿与直躯干仍需打磨，暂停菜单遮挡角色时由状态证明确实停止动画。证据与已看帧、清理范围见[集成复验](../../TASK-045/visual-r6/review.md)，不是全方向、坡地或正式动作质量通过
