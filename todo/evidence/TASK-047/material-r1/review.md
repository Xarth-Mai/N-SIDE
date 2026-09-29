# CHR-001 灰阶材质 r1

基线为 `b948cabe717c1c6081055267a4b0c063b7de855d`，本轮由 `character_shading` 制作单材质响应，root 负责真实 Bevy 接入与 capture。灰阶底色仍是形体候选，角色配色和最终人物美术尚未验收

## 实际改动

- 保留原始 base color 的完整 PNG 字节，新增共用 UV 的 1K 粗糙度/金属度图与 1K 小幅缝线法线图；源图及纹理生成算法为项目原创，未调用外部生成服务
- glTF 的 G 通道提供粗糙度，B 全为零；数据图为 Non-Color，base color 为 sRGB，全部图打包进主文件与 GLB。现有 UV 对夹克、棉衫、裤装、皮肤、头发、鞋面和共用白色区域分别采样
- 法线只沿已有夹克、裤装和鞋面图集接缝起伏，皮肤与头发无毛孔、发丝或随机凹凸；幅度直接写入 RGB，`normalTexture.scale` 为 1
- `Principled BSDF` Specular IOR Level 使用 0.24，导出 `KHR_materials_specular.specularFactor` 为约 0.48；本地 Bevy 0.19.1 loader 转为 reflectance 0.24。由 root 移除旧角色接入对 roughness / reflectance 的强制覆盖，既有 StandardMaterial 扩展继续处理实际光照与投影
- 没有新增材质框架、palette、几何、关节、动画或控制器修改

## 同条件观察：self-audit

CPU Cycles 使用同一个 r6 模型、Idle 第 1 帧、原主文件三盏面积灯与 world、720×960、32 samples、2 threads，取三分之四整身和上身两机位。参数及图片哈希保留在各 `*-render.json`；实际图片是 DCC 渲染，不是游戏画面

第一候选 A 的头发粗糙度 0.38、Specular 0.35 使冠顶出现亮白窄面，视觉偏塑料，未采用。候选 B 将头发粗糙度提高至 0.52、Specular 降至 0.24，皮肤设为 0.64、鞋面 0.62；衣料保持 0.76—0.88，白色共用区 0.86

本人实际查看 baseline、A、B 及最终打包主文件的整身与上身，最终主文件与 B 候选表现一致：B 的冠顶高光较 A 收敛、发束明暗与衣料形成可见区别，肩袖和裤装保持哑光，未见新纹理破边或闪亮缝线。整身尺度下收益主要在发束，皮肤和鞋面差异较小，不能把本轮称为完整角色贴图绘制

`art_review` 已实际查看 baseline / A / B / final 各两图共 8 张，认可拒绝 A、采用 B 进入真实运行验证；指出本轮主要改善发束反光，全身衣料、皮肤和鞋材质差异仍轻，最终质量保持 `needs_revision`。这是协作自查，不是隔离玩家评审或作者验收

仍需保留的质量缺口：灰阶脸与发束的塑形仍偏建模样板；当前 UV 重用不支持精准逐裁片缝制图，也不能把共用 `white` 的眼白与鞋底独立处理。本轮没有添加湿润眼层、次表面散射或写实发丝。真实运动中的切线法线、阴影分段和高光必须由 Bevy capture 另行判断

## 数值验证

| 实际命令 / 入口 | 结果 | 范围 |
| --- | --- | --- |
| `build.py`，Blender 4.5.14 LTS，CPU 2 threads | PASS | 重建实际 master 与 GLB，1 primitive、32 joints、原三个 clip |
| `check.py` | PASS | 三张源图输入与色彩空间、8 区粗糙度、零金属 B、法线幅度；原 211 帧骨架/接地/循环及脸发检查 |
| `check-contract.py` | PASS，15 项 | 对实际 b948cab GLB 的所有几何/法线/UV/weights/indices、节点、inverse binds、全 clip 采样与原 base PNG 逐值/逐字节对比；检查导出的真实 G/B/normal 数据 |
| `tools/validate_character.py` | PASS | 实际 GLB 高度、32 骨、权重、三个 clip 和必需嵌入纹理 |
| `build.py --export-existing` | PASS | 从可编辑主文件再次导出的 GLB 逐字节相同 |
| `repro-bad-colorspace.py` | 预期 FAIL，exit 1 | 仅隔离副本将 surface 改为 sRGB，真实 `check.py` 拒绝：`CHR001_GreyStudy_Surface: data texture must be linear`；正式资产哈希不变 |

Blender exporter 对 metallic 与 roughness 两个输入发出“多个图像节点参与纹理采样器”警告；核对本地 `io_scene_gltf2/blender/exp/material/texture.py:174`，该处按输入 socket 数量报警，本文件两个 socket 实际追到同一 ImageTexture。实际 GLB 的 sampler 和 G/B 通道已检查，未压下日志或将警告当作另一个纹理成功

最终文件哈希及大小见 `delivery.json`；GLB 为 `220370b6ee7a073a0c04edb1c865d9e70129d3f58b6163c07d81590ccecd68a0`。本页的技术 PASS 不替代 root 的实际 Bevy 结果或作者的美术验收

## 复现命令

仓库根目录执行，fish 可直接使用；前两条只导出和检查现有主文件，不重建几何

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 -B todo/evidence/TASK-047/material-r1/check-contract.py
python3 -B tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/material-r1/render.py -- final
```

重建本轮基线对照时，先 `mkdir -p output/characters/CHR-001/material-r1/baseline`，再使用 `git show b948cab:source-assets/characters/CHR-001/model/yao-grey-study.blend > output/characters/CHR-001/material-r1/baseline/yao-grey-study.blend`，然后将 render 参数改为 `baseline`。坏色彩空间复现使用本目录 `repro-bad-colorspace.py`，预期返回 1

根Agent已完成正式GPU环绕及540帧动作检查，结果见[第8轮集成](../../TASK-045/visual-r8/review.md)。本轮两张最终CPU图也已由根Agent查看后清理，完整剩余清理记录见同批 `cleanup.json`；正式贴图与源模型保持
