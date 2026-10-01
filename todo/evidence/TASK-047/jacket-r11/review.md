# TASK-047 · 曜上衣大轮廓候选 r11

2026-10-01，从正式 r10 主文件的字节副本制作，仅修改 `Jacket_ContinuousShoulders` 的衣身侧线与袖筒松量。先在 `output/yao-jacket-r11/` 完成独立候选；root 实际复看后接受此局部增量，随后原位更新正式主文件与运行 GLB，人物仍为 `needs_revision`，未获得作者外观验收

## 范围与源

[冻结输入](source-baseline.json)记录 r10 主文件、运行 GLB、正式生成器与玲资产的哈希。候选复制自实际 `.blend`，没有执行全量生成；[candidate.py](candidate.py)复用正式生成器的导出函数，只对夹克现有网格进行位移，不新建全身模型

夹克控制网格保持 519 顶点、456 面，286 个顶点发生位移；衣身侧面最大横向增加 29 mm，袖筒中段相对其原截面中心增加 22%—47% 松量，两端逐步收束。这些是本候选试验值，不写成项目统一比例。衣长、上方三层肩领、连续袖窿、拉链边界、帽兜贴靠的上背区域及袖口最后两圈保持原坐标

原拓扑、UV、所有权重及分组、修改器、材质和三张贴图保持；头脸、r9 发组、r10 Hood、内衫、裤鞋、手部与玲未修改。骨架与四动作不因衣服变宽而重算；没有新配色、配饰、衣骨、模拟或外部角色素材

## 检查

| 证据 | 实际结果 |
| --- | --- |
| [保持比较](preserved-contract.json) | 17 项 PASS，其他源网格逐坐标／面／UV／权重一致；32 骨 rest／层级、动作曲线、GLB 节点、inverse bind、四动作完整导出采样、材质及三个嵌入图字节完全相同；夹克锚点／面／UV／权重／组不变；正式文件哈希不变 |
| [GLB 检查](glb-check.json) | PASS，32 骨、24,381 蒙皮顶点、Idle／Walk／Run／Jump、3 张嵌入图；高度 1.7441905736923218 m、Root 原地 |
| [DCC 检查](dcc-check.json) | PASS，实际候选主文件通过既有材质、骨矩阵、足底、循环、Jump、脸部／发组和胸前图形贴合检查 |
| [28 个姿态](preserved-contract.json) | Idle 5、Walk 7、Run 9、Jump 7 个实际评估姿态；帽兜最大 BVH 重叠对 269→269，袖口两侧各 68→68，T 恤 190→180；头、发组、领口与手掌保持 0 |

BVH 数量是接触诊断，原有帽兜／衣领接缝及衣片穿插没有因此全部解决；不将数量下降解释为同比视觉改善，也不声称覆盖连续动作中的所有自相交

候选运行 GLB SHA-256 为 `0f02f5eb635b356ab25325829bbdb51fbe021f723fe162abe9e4090486cb87db`，其他精确哈希由保持比较记录。候选材质、骨架与动作契约通过不意味着新衣装已获作者认可

## 实际看图

实际查看[六张固定机位对照](views.json)：原版与候选的正面、前三分之四和背面。640×480、Cycles CPU、2 线程、12 samples、相同源灯光；全身约 300 像素高，用来判断常用显示尺寸下的大形，未通过放大局部或换光强化差异

- 正面与三分之四中，袖筒较原版有明确松量，中段宽、袖口收紧；衣身侧线更饱满，内衫与门襟维持原关系，变化在原尺寸下可见
- 背面能看到上衣形成较宽短的体积，帽口位置与上背接缝保持；身体、头和手没有被整体放大
- 当前体积仍圆滑，袖部呈偏厚的宽松外套感，缺少有方向的大衣褶和明确裁片；T 恤下摆仍有外扩，裤装仍偏直，脸部个性与配色未完成。本批不把“变宽”写成整套潮流服装或目标日漫画风已经成立

另实际查看[五张动作姿态](motion-views.json)：Walk 8、Run 6／26、Jump 13 正面／侧前与 Jump 21 背面。袖口持续包住手腕，抬臂和弯肘时未见新增破洞、拉散或明显穿过手掌；Run 肘弯仍有偏硬的体积折转，动作自然度不由本次衣形更改解决。这是静态姿态自查，不是完整视频观看或真实 Bevy 连续动作验收

## 复现

以下为候选阶段命令；先按冻结输入恢复 `output/yao-jacket-r11/before.blend`、`before.glb` 和 `generator-before.py`，只在确认 Blender 空闲的窗口运行，命令适用于 fish。正式回写后的保持检查使用下节 `--formal`，候选阶段“正式文件未改”条件只适用于回写前

```fish
env ALSOFT_DRIVERS=null timeout 180 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/jacket-r11/candidate.py
env ALSOFT_DRIVERS=null timeout 240 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/jacket-r11/check.py
env ALSOFT_DRIVERS=null timeout 180 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py -- --master output/yao-jacket-r11/candidate.blend --output todo/evidence/TASK-047/jacket-r11/dcc-check.json
python3 -B tools/validate_character.py output/yao-jacket-r11/yao-grey-study.glb --root-node Root --height 1.7441905736923218 --height-tolerance 0.00001 --clip Idle --clip Walk --clip Run --clip Jump --require-texture
env ALSOFT_DRIVERS=null timeout 240 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/jacket-r11/render.py
env ALSOFT_DRIVERS=null timeout 240 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/jacket-r11/render.py -- --candidate-only --motion
```

所有本轮 Blender 进程正常退出，未发生音频退出错误，未增加后台残留。未运行 Cargo、GPU、新一轮游戏或作者体验验收；主工作线完成指定图像复看后，11 张候选对照与动作 PNG 按要求清理，精确路径与哈希保留在视图清单。候选源、冻结输入、脚本、日志与 JSON 保留用于复现

限定脚本的精简复查沿用已有导出与比较函数，没有新增通用衣服框架或依赖，结论为 `Lean already. Ship.`；仅表示脚本复杂度检查，不是衣服外观放行

[候选文档检查](docs-check.log) PASS：585 篇 Markdown、129 个 ID，31 Skills／25 imports；三个 Python 源编译通过，限定范围 `git diff --check` 未报错

## 正式回写

root 实际查看原版与候选的三分之四／背面，以及候选 Run 6／Jump 13，明确接受此局部大形增量用于 R4。将同一位移函数加入正式生成器的 `--update-jacket` 入口及全量建模焊接后的末端；从原正式主文件原位更新，保留其打包图像及源路径，不直接复制候选主文件覆盖

对象 `jacket_shape_revision = 11` 只用于阻止重复执行时累加位移；全量重建创建新对象后应用同一函数。后续手工编辑使用 `--export-existing`，不清除标记强行再跑位移；本轮没有为此新增通用衣服系统

[正式保持检查](promoted-contract.json) 20 项 PASS：沿用所有冻结契约，正式 GLB 与已查看候选逐字节相同，正式函数能从冻结网格重新得到相同坐标，重复调用正式函数不再改变坐标。主文件 SHA-256 为 `08da019a48c32725fe0b33fe20f615659105ad16b3c6a0a2e824398f9e739849`；运行 GLB 仍为 `0f02f5eb635b356ab25325829bbdb51fbe021f723fe162abe9e4090486cb87db`

```fish
env ALSOFT_DRIVERS=null timeout 180 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --update-jacket
env ALSOFT_DRIVERS=null timeout 240 output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/jacket-r11/check.py -- --formal
```

正式清单与模型 README 已更新，配色未定、圆滑衣面、偏硬肘弯、脸部个性等不足继续保留；新的 Bevy 运行证据由主工作线补充，源／导出相同不代替实机观察

[清理记录](cleanup.json)：已查看的 11 张 PNG 与原位保存产生的一份 r10 `.blend1` 共 7,741,229 bytes 已删除；备份字节先与保留的 `before.blend` 比对一致，正式资产、冻结输入与候选源仍保留，候选目录无剩余视觉文件。[正式文档检查](promoted-docs-check.log) PASS，四个 Python 源编译及资产清单两项哈希一致性检查通过，限定 diff 未报错；正式函数的重复应用保护属于必要数据保持，精简复查结论仍为 `Lean already. Ship.`
