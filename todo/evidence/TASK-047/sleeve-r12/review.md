# 曜 r12 袖筒裁片候选

本轮先在 `output/yao-sleeve-r12/` 制作受控候选，Root 实际看图后接受去圆鼓技术修订，随后按授权原位提升到正式源与运行 GLB；[输入冻结记录](source-baseline.json) 保留修改前版本，不是作者造型批准，也不单独构成美术里程碑

## 问题与改动

r11 正面与背面袖筒呈连续圆鼓形，关节处缺少衣料方向；本轮只改 `Jacket_ContinuousShoulders` 两侧袖筒中间五圈，共 160 个控制顶点，将前后截面收浅，肘外侧引入斜向转折

保持衣身 231 个顶点、两侧肩缝及过渡环、袖口末两环、拓扑、UV、权重不变；其他网格、三张内嵌纹理、32 骨及 Idle／Walk／Run／Jump 四动作不变，没有新增配色、装饰物或骨骼

## 实际看图

- 实际查看同机位 `before/candidate-front`、`quarter`、`back` 六张 CPU 图，640×480 输出中人物约 300 像素高；正面和背面肘部不再同样圆鼓，斜向转折能看见，侧前方的袖筒厚度有所减少
- 整体全身差异较轻，衣身仍大面积灰阶光滑，不能据这次局部收型宣称日漫服装完成；本次也没有改变肩、帽兜或背部裁片关系
- 实际查看 Walk8、Run6、Run26、Jump13、Jump21 的 before/candidate 十张同机位 CPU 图，手掌、袖口、腋下、T 恤未观察到新增穿插；Walk8 和 Jump13 的外侧折转较明显，Run6／26 全身差异很轻，Jump21 背面仍以平滑的大衣身为主
- Root 已实际查看同机位 before/candidate-quarter 及 candidate-run6／jump13，认定肘部减圆、方向变化可见但全身改善较轻，授权作为小幅技术修订接入，不能作为大美术成果计数

## 机器检查

- [保持检查](preserved-contract.json)：18 项 PASS，全部正式输入 hash 未变，只允许的五圈发生位移；原有其他网格、骨架、动作曲线、图片、UV、材质、权重以及 GLB 导出样本一致
- 同一检查覆盖四动作 28 个取样姿态；头部、发块、领口、手掌 overlap 为 0，帽兜／T 恤／袖口原有接触最大值保持 269／180／68，数值仅用于发现回归，不代表视觉质量或全部连续帧无穿插
- [GLB 检查](glb-check.json)：PASS，24381 个蒙皮顶点、32 骨、四动作、三纹理，身高 1.7441905736923218 m 保持不变
- [既有 DCC 检查](dcc-check.json)：PASS，实际输入为 `output/yao-sleeve-r12/candidate.blend`，见 [调用日志](dcc-check.log)；沿用验证器的 `asset` 文字标签仍为 r7，不表示本轮检查了旧主文件
- [正式提升检查](promoted-contract.json)：21 项 PASS，正式 GLB 与已看候选逐字节相同；更新函数从 r11 冻结主文件重新产生同样的控制顶点，重复调用保持坐标；全部非目标网格／图／动作及 CHR-002 保持
- [文档与 Skills 检查](docs-check-final.log)：PASS，599 Markdown、129 IDs、31 Skills 与 25 项导入；首轮共享工作区缺失后场页面造成的断链已随该工作线交付消失，原始失败日志保留
- Bevy 运行：R5 `yao-motion-retry` 的新网格 540 帧真实路径 PASS，27 项原生／28 项封装检查通过；运行与视觉范围见下节，不沿用 r11 的实机结果

## R5 真实运行续验

[实际运行](../../TASK-049/blender-integration-r5/captures/yao-motion-retry/run.json)采用 1280×720、30 FPS、540 帧，共 18 秒，退出码 0；[原生状态](../../TASK-049/blender-integration-r5/captures/yao-motion-retry/state.json.gz)逐帧保存四动作与真实控制器状态，27 项原生检查和新增角色加载检查共 28 项均通过。脚本覆盖行走、两侧 Shift／右键疾跑、三次跳跃、暂停恢复与失焦；脚本手柄输入不等于实体手柄验收

此次角色 GLB hash 为 `3fb220c6f073f003399acf25e0083aafb2f1bb18027064f49183bfbfe61370db`，与 r12 正式提升一致；实际执行二进制 hash 以该 run.json 中的 `284df06444f9c362ad9bb208dfd58963ed27316e98afa8a15aa3a8ed2fef0d0d` 为准，后续建筑批次重建的二进制不覆盖这次运行来源

art_review 的[实际画面自查](../../TASK-049/blender-integration-r5/visual-review.md)查看 Run 190–192、Jump 68–70 和落地 79：袖筒、袖口、下摆及正面腋下未见新增穿洞或分离；肘外侧转折可见，但正常全身差异仍轻，灰阶衣身、发面与人物识别度仍低于目标。79 帧已恢复着地 Idle，不称作空中跳跃

首次 `yao-motion` 因误用 Viewer 的 `--aa` 参数在启动前失败；保留失败记录，retry 才是本次运行依据。视频为 `NOT RUN: --no-video`，程序内 `visual_review=NOT RUN` 保持不变，由独立文字记录承载实际看图结论；有限帧自查不证明所有姿态均无穿插，整体继续 `needs_revision`，作者外观验收未执行

## 复现与交付边界

候选脚本 [candidate.py](candidate.py) 从 `output/yao-sleeve-r12/before.blend` 原位副本修改，复用冻结生成器的既有 GLB exporter；[render.py](render.py) 固定相机、灯光、CPU 双线程和 12 samples，不将摄影机设置保存回主文件；[check.py](check.py) 复用 r9／r11 保持与姿态比较，`--formal` 验证实际主文件和运行 GLB

正式 `build.py` 的 `--update-jacket` 与完整构建共用函数，r11 输入跳过旧松量增幅只作本次袖筒更新，r12 输入直接返回；本次只从已有 `.blend` 原位更新，避免全量重建覆盖手工内容或复制候选文件导致图片相对路径改变

```fish
env ALSOFT_DRIVERS=null output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --update-jacket
python3 -B tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441905736923218 --height-tolerance 0.00001 --clip Idle --clip Walk --clip Run --clip Jump --require-texture
```

固定比较脚本依赖本轮已冻结的 before/candidate 临时副本，清理后不再直接运行；要重建旧输入可按 source-baseline 的 commit 及各 hash 恢复，不将历史清理后的媒体路径当成现存截图

[恢复核对](baseline-restore.json) 已确认五个冻结输入与基线提交一致；[清理记录](cleanup.json) 保存 16 张已看图、候选副本与本轮 `.blend1` 备份的路径/hash，共 22 个文件、21,869,121 字节已删除，`output/yao-sleeve-r12/` 已无残留。正式源、贴图、GLB、脚本、日志和 JSON 保留

本轮脚本只保留本次位移、复用比较与固定摄影三项职责，没有引入衣物框架、依赖或新增配置层；按 `ponytail-review` 自查：Lean already. Ship.
