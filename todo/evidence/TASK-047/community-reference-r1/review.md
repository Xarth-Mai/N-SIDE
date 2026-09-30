# 社区铃模型：来源、真实接入与重定向实验

2026-09-30，输入游戏提交 `62f909f02fabb433727c4be5fc8c5a6a716e8a1e`；作者允许查找并改编社区素材，以哲、铃作为两位主角的具体造型参照。本轮保留第三方原角色身份，未将其改称项目原创资产，也未修改人物设定

## 已取得的模型与来源

| 包 | 实际取得内容 | 来源与范围 |
| --- | --- | --- |
| 官方哲 | PMX、纹理、骨骼与表情，无走跑动作 | 见[实际导入与条款核对](../model-r8/official-wise.md)，观海子制作、miHoYo 权利，包内禁止二次配布及商业使用，本地比较 |
| 官方铃 | `铃.pmx`、手机、耳机和纹理 | [官方活动页](https://www.bilibili.com/blackboard/activity-vl9IMaaeyZ.html)对应[公开 ZIP](https://activity.hdslb.com/blackboard/static/20240704/40370b1512d054e187721bf3fb452961/o1kaensO5D.zip)，与哲同类条款；完整文件表、hash 与包内原文见[来源记录](official-belle-source.json)，本轮未做 PMX 导入 |
| Ghost73 玩家铃 | glTF、bin、7 张独立图片，148 个 joints，无动画 | [公开发布页](https://sketchfab.com/3d-models/belle-by-ghost73-449252f59ef0451589622b333c7f4e9b)标记 CC Attribution，描述提供[MediaFire ZIP](https://www.mediafire.com/file/h6i5fpmj7gzdu7e/belle+by+ghost73.zip/file)，通过正常公开下载按钮取得 |

社区包 SHA-256 为 `dfbe6c815be831c19be3af5ef5b1f7e5066bd9c6ce1adbf3f323ff2b2afc6924`，5,693,805 字节，原包位于 `output/assets/zzz-reference/belle-player-share/belle-by-ghost73.zip`。包中没有 README 或独立许可文本；页面发布者为 Ghost73，内部对象名包含 `Belle_SFM` 与 `rstar`，只能说明曾经过 SFM 路径，不能据此确认全部制作链。保留 HoYoverse 原角色、Ghost73 分享者及上述未核实项，不把页面许可标签当作原游戏角色权利的独立证明

哲的玩家版已找到但尚未下载，具体页面、登录要求及 MEGA 访问限制见哲的来源核对。本轮没有新购服务或使用付费下载

## 实际接入方式

Blender 4.5.14 导入社区 glTF，移除无蒙皮的导入辅助球体，网格与 rest 骨一起转换到脚底原点及 1.65 m 高度，保留原 UV 和七张嵌入图片。在原骨架上增加统一 `Root`，使用项目现有 CHR-002 的原创 `Idle`、`Walk`、`Run` 做重定向；源包不含这些动作

通过隔离的 `output/assets/zzz-reference/belle-player-share/local-project/` 覆盖 CHR-002 的模型路径，复用正式二进制、真实世界、控制器、角色加载和动作选择。`CHR-002` 在本次状态 JSON 中代表复用的技术槽位，画面中仍是第三方铃模型，不是已定稿的月城玲；正式角色资产的字节未被覆盖

## 迭代与证据

- 首次 sandbox 运行因无法建立 GPU 退出，记录于 [belle-orbit/run.json](belle-orbit/run.json)，不记作图形验收通过
- 原生 GPU 环绕检查通过，见 [run](belle-orbit-native/run.json) 与 [state](belle-orbit-native/state.json)；实际查看帧 59、204，贴图与人物可辨，但双臂仍保留导入 A-pose，视觉结论为需要修改
- r2 尝试按所有骨的 head-tail 方向对齐；数据契约与 18 秒控制器断言通过，见 [contract](contract-r2.json)、[run](belle-walk-r2/run.json) 和 [state](belle-walk-r2/state.json)。实际查看帧 59、159、209 后判定视觉 FAIL：多个 SFM 骨尾仅是局部轴标记，不能当作解剖关节方向，全骨对齐使身体横躺、动作扭曲。此结果证明动画时间、有限 Transform 和 in-place 根检查不能代替姿态验收

## 最终修复与实机复验

r3 仅按真实肩、肘、腕位置修正两段上肢，保留躯干与腿的原 rest 语义；r4 同步缩放 Mesh 的 shape keys，修复面部 Basis 仍留在原尺寸、实际脸部沉入领口的问题。[逐帧姿态检查](pose-check-r4.json)覆盖 211 帧，头高于骨盆、脚低于骨盆、根静止、上肢方向与源动作一致、循环矩阵闭合、形态键 Basis 与网格一致均通过；此前 r1／r2 的失败数值保留，详见 [DCC 记录](retarget-r4.md)

已落地 [prepare_belle_reference.py](../../../../tools/prepare_belle_reference.py)，从原 glTF 直接导入、转换、导出并建立隔离资源根，不依赖某次手工保存的中间 blend。正常执行退出 0，重复使用同一个输出目录退出 2，保留 [拒绝覆盖日志](final-existing-output-negative.log)。工具输出与 r4 GLB hash 精确一致：`4d77e7dc09629322b95cb2a6807b8841d0845000be467f7763b4ac79de009e19`；原输入、七张贴图、bin 和正式角色未变，完整依赖 hash 见 [prepare](final-prepare.json)，最终 GLB 预检见 [contract](final-contract.json)

真实 Linux／RADV 运行使用既有 `n-side` 二进制与两段未修改 capture 脚本。18 秒行走、Shift／右键疾跑、跳跃、暂停与失焦恢复共 27 项检查通过，8 秒环绕共 10 项通过；两个视频均实际编码成功，但本轮以关键帧与逐帧图像检查，未把生成视频等同完整人工回放

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python tools/prepare_belle_reference.py -- --source output/assets/zzz-reference/belle-player-share/model/belle.gltf --output output/assets/belle-reference-r1
python3 tools/validate_character.py output/assets/belle-reference-r1/reference.glb --root-node Root --height 1.65 --clip Idle --clip Walk --clip Run --require-texture
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-002 --project-root output/assets/belle-reference-r1/project --script game/capture/walk-character.json --output output/visual-r10/belle-final-walk
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-002 --project-root output/assets/belle-reference-r1/project --script game/capture/character-orbit.json --output output/visual-r10/belle-final-orbit
```

以上为本次实际命令，重跑需为两个输出各取新目录。最终 [walk run](belle-final-walk/run.json)／[state](belle-final-walk/state.json) 和 [orbit run](belle-final-orbit/run.json)／[state](belle-final-orbit/state.json)包含版本、参数、检查与日志；没有修改 Rust，未额外重编相同二进制，Windows 运行 NOT RUN

## 画面自查与剩余工作

root 实际查看最终环绕 204 帧、走路 159 帧、跑步 209 帧以及连续 140—143 帧，确认站立竖直、双臂下垂、脸部位置和五官恢复、腿部随步态运动；[已看帧 hash](viewed-frames.json)与机器检查分开保留。上游模型的轮廓、服饰与贴图可用于实际比例比较，当前整体仍 needs_revision：手指保持导入姿态、部分跑步掌心向上，缺乏布料／头发次级动作、表情和空中专用动作，未做完整足滑量测或作者审美验收

社区来源、可再分发范围与最终人物改编仍未收口；本轮不把第三方文件提交进正式资产或发布包，也不以技术接入代替完整游戏人物完成。建筑参照由 [TASK-049](../../TASK-049/reference-r1/building-references.md)维护

已按项目要求清理本轮截图、连续帧、视频和 `.blend1` 临时备份，共 2,179 个文件、2,215,297,275 字节，剩余视觉中间产物 0，见 [清理记录](cleanup.json)。原包、源纹理、参考 blend／GLB、脚本、日志、状态 JSON 和文字结论保留；图像不再作为有效文件链接

提交前专项审查：工具及文档无额外框架、没有重复 capture，`ponytail-review` 结论为 `Lean already. Ship.`；本轮长期结论已回写美术方向与 Blender 接入说明

检查收尾：`bun run check:docs` PASS（530 Markdown、129 IDs；31 Skills／25 imports），`bun run tasks:check` PASS（47 卡），`bun run docs:build` PASS（58 项地图测试；player 128 页／679 文件、dev 223 页／1,046 文件），日志见 [文档检查](docs-check.log) 与 [双站构建](wiki-build.log)。构建保留已有大 chunk 提示，未作为失败；`git diff --check` PASS
