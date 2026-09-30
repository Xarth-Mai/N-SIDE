# TASK-047 · 曜 r7 基线与哲本地参考模型

2026-09-30，本轮开始时以 `465070c302aba2be7d2506fc1afcdb401bd5e8ad` 的曜 r7 为输入，已完成六个 CPU 近景的实际观察；随后作者将方向改为以《绝区零》对应角色为参照，并允许查找玩家分享模型，因此暂停手修曜，转入哲模型来源核对与本地导入

## 交付与边界

曜的源脚本、检查脚本、Blender 主文件、运行 GLB 和三张贴图均与 `input-hashes.json` 字节相同，继续为灰阶 `needs_revision`，实际年龄 20／外观约 18 的目标未改，也未批准配色

本地已取得哲的 [官方发布包](https://activity.hdslb.com/blackboard/static/20240704/40370b1512d054e187721bf3fb452961/3QEUENLp4E.zip)，来源页缓存 `output/assets/zzz-reference/official-models.html` 标题为「绝区零UP主激励计划」，其中标题「哲」指向同一下载 URL。ZIP 共 6,193,943 字节，SHA256 为 `6616ada891c18c110cfdfb7f36cafbaaebe5b24683a22f3d6f5fd9b58c2cdc68`，保留在 `output/assets/zzz-reference/wise/`

包内 `readme【一定要看】.txt` 与 `哲.pmx` 内嵌说明一致：模型制作者为观海子，权利归 miHoYo；允许修正物理、权重和表情、改色及适度修改衣装，禁止二次配布、拆件改造其他模型及商用。原模型、贴图和本地转换品均留在 Git 忽略的 `output/`，维持哲／Wise 及原权利人署名，不作为曜的正式资产，也不进入提交或发布

| 本地文件 | 实际内容 |
| --- | --- |
| `output/assets/zzz-reference/wise/official-package/哲.pmx` | PMX 2.0，22,118 顶点、28,901 三角、15 材质、288 源骨；独立手机有 976 顶点、934 三角、4 材质、6 骨 |
| `output/assets/zzz-reference/wise/wise-official-local.blend` | 原 MMD 节点导入副本，保留源网格、UV、蒙皮、52 个 shape keys 和原署名，9 张图打包；导入器增加辅助骨后共有 332 骨，无动作 |
| `output/assets/zzz-reference/wise/wise-official-preview.blend` | 使用原底图的独立 Principled 预览副本，15 材质、3 张实际使用底图；其网格、UV、rest 骨层级和所有权重与原导入版完全相同 |
| `output/assets/zzz-reference/wise/import-reference.py`、`preview-reference.py`、`check-reference.py` | 本地复现与窄检查脚本，未成为项目正式 PMX 管线 |

## 来源调查

| 发布页 | 实际检查结果 |
| --- | --- |
| [LunaEagle / Sketchfab](https://sketchfab.com/3d-models/wise-9e8b5c84f2da41af96a799d860765d40) | 页面标 CC BY 4.0、22.4k 三角与 11.9k 顶点，说明仅为 Zenless Zone Zero；正常下载按钮弹出登录，未下载，无法确认发布者拥有原模型或转授权 |
| [Deliquecent-Skull / DeviantArt](https://www.deviantart.com/deliquecent-skull/art/ZZZ-Wise-1163776416) | 明确来自游戏的 GLB、嵌入贴图、限教育用途、非商用且权利归 miHoYo；公开 MEGA 链接被浏览器站点安全策略阻止，未绕过，包内检查 NOT RUN |
| [NekoPixil / DeviantArt](https://www.deviantart.com/nekopixil/art/%5BFBX%5D-Wise-%28Zenless-Zone-Zero%29-3D-Model-%7C-DL-1084867247) | 声称绑定 FBX／PNG、修正骨方向并归 miHoYo；下载需要登录，未取得包 |
| [purinmods / DeviantArt](https://www.deviantart.com/purinmods/art/Wise---DL-1254858651) | 说明是旧 Beta 哲、FBX；下载需要登录，未取得包 |

## 工具、失败与修复

使用 [MMD Tools v4.5.14](https://github.com/MMD-Blender/blender_mmd_tools/releases/tag/v4.5.14)，源码与 GPL-3.0-or-later 许可保存在 `output/tools/blender_mmd_tools-4.5.14/`。实际阅读 README、manifest、LICENSE、注册与 PMX 导入入口后执行；README 声明支持 Blender 4.2—5.2，当前执行版本为 Blender 4.5.14 LTS

扩展和所带 OpenCC wheel 只在 `output/tools` 解包，通过当前 Blender 进程的模块路径与临时偏好启用，未保存全局偏好。最初直接从 wheel 导入导致 OpenCC 找不到其目录资源，解包该随附 wheel 后修复；直接注册时缺少共享 toon 目录，改为当前进程启用扩展并使用其随附 toon01。首个贴图检查过早读取 `has_data`，改为实际读取尺寸与像素后通过，全部失败日志保留

第一次实际渲染暴露头发主体透明，原 MMD 和单纯换 Principled 均复现。检查证明 `髮` 与 `髮+` 各有 6,174 个面且坐标范围完全相同，后者为 MMD 加色高光层；Cycles 的透明射线跳过共面基底。仅在独立预览副本中，将 `髮+` 改用同一原头发底图，使两层均为不透明底色；不移动、删除或重建几何，原 MMD 材质副本仍保留。该预览没有复原 MMD toon、sphere 加色或游戏专用着色

## 检查与实际观察

| 检查 | 结果 |
| --- | --- |
| ZIP、PMX 头部、原始说明 | PASS，见 [来源核对](source-audit.json) |
| Blender 实际导入与贴图读取 | PASS，见 [导入数值](dcc-import.json) 与 [实际日志](import-wise.log) |
| 原导入版和预览版的拓扑、UV、骨 rest／层级、权重逐项比较 | PASS，共同指纹 `4b1678de9ce0eb0174b5bc0d84061ec369eec780bd0ad623d4c2a4ee3a7d9188` |
| 有限几何／骨矩阵／52 个形态、绑定组、最多四个骨权重及归一化 | PASS，权重和最大误差 `1.564621925354004e-7`，见 [机器检查](machine-check.json)；`mmd_edge_scale` 与 `mmd_vertex_order` 按导入器源码作为非变形组单独核对 |
| 四个最终静态视角 | PASS：实际查看全身正面、脸部正面、三分之四与侧面；CPU Cycles、2 threads、16 samples、480×640、Standard，同组前后机位与灯光保持 |
| GLB 导出、项目 `validate_character.py`、动作重定向、真实 Bevy 运行、物理 | NOT RUN；本轮交付为本地 `.blend` 参考，无 Idle／Walk／Run，不能按现有 32 骨角色契约声称可直接接入 |

视觉记录属于 self-audit。曜 r7 的宽平发根、放射冠顶与均匀尖叶轮廓确实可复现；哲模型的发根更集中，前侧长短组束相互承接，侧后发尾向外分开，眼睑、嘴角和颊部能表达更明确的表情。最终哲预览可作为形体和服装层次的本地观察输入，眼周、鼻下与下巴仍带原贴图明暗与普通 PBR 阴影，不等于《绝区零》游戏成像。两个模型的身高、材质和镜头尺度不同，没有把它们当同像素测量或已验收游戏画面

## 复现与收尾

以下为本地已执行成功的命令，扩展与模型均需在上述忽略目录中存在

```sh
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --factory-startup --disable-autoexec --python-exit-code 1 --python output/assets/zzz-reference/wise/import-reference.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --factory-startup --disable-autoexec --python-exit-code 1 --python output/assets/zzz-reference/wise/preview-reference.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --factory-startup --disable-autoexec --python-exit-code 1 --python output/assets/zzz-reference/wise/check-reference.py
```

[清理记录](cleanup.json)记录临时截图、基线副本与中间 `.blend1` 的清理；保留原模型参考包、两份本地 `.blend`、工具源码与许可、复现脚本、日志和 JSON。`source-audit.json` 核对正式曜资产未变，人物与共享任务看板未修改
