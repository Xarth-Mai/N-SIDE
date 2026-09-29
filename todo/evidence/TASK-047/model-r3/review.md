# TASK-047 · 曜眉眼、发组与衣服轮廓 r3

2026-09-30，基线 `dbe5ad3`，沿用 CHR-001 实际年龄 20、外观目标约 18 和未批准灰阶；实际查看镜厅预览左侧人物海报、r3 黑白造型稿、r2 主文件四个 CPU 机位后制作，输入哈希见 `input-hashes.json`

## 实际修改

- 收下颌与眼睑，眉线两端变细、下眼线仅保留外眼角，增加虹膜相对眼白的面积；固定笑弧换成短中性口线，缩小耳廓并用皮肤同值内耳弧替代深色硬角
- 刘海重新分为主扫束、反向短束与窄侧束，去掉成排同宽发片和两侧重复冠发；发束增加沿曲线采样，发帽按实际高度对应头皮椭球，保留可编辑网格
- 收外套肩袖、帽沿与袖口体积，肘部、膝部和裤脚改为局部受压折形；T 恤低两环用略方宽松截面包住双腿裤腰，保留短外套与较长内衫叠层
- 32 骨、节点变换、inverse bind、全部 Idle / Walk / Run 动作采样、标准材质和灰阶图集逐项保持 r2 相同，未改设定、配色、动作速度与世界位移责任

## 制作中的真实问题与修复

`first` 的发帽前缘露成横向硬带，袖口收小后出现相交锯齿；`second` 只抬高发帽环线又造成帽面穿出刘海根。根因是提高 Z 后仍沿用低处的前后半径，本轮改为同一头皮椭球上的连续环，并加入环线高度次序断言，最终同机位不再出现突出的横带

袖口改为覆盖实际袖末端轮廓，最终全身与动作机位未再看到原有锯齿。首次正面下摆看似修好，补侧身与跑步后在背腰看到穿裤锯齿；这批图保留在 `hem-before` 直至复验，随后修订下摆的完整横截面，`final` 的侧身、步行与跑步图已消除该处交叉

## 实际检查与看图

主文件原灯光、灰阶、Standard 视图、CPU Cycles 24 samples、720×960，脸部正面／三分之四／侧面、全身正面与侧身、Walk 第 8 帧、Run 第 6 帧；相机与动作参数保存在各 `*-views.json`，由 `render.py` 重现，未为通过观察改光照

建模者实际查看全部迭代；root 与独立 `art_review` 查看四个核心机位，认可收肩袖、口线、主副刘海、耳部与穿插修复可以保留。独立复查的结论仅为 self-audit，不是作者形象批准。脸部仍偏圆平、眼圈有贴片感、侧后发帽下缘仍有连续切边，衣裤褶皱与材质仍简化，T 恤下摆略硬外翻，硬边赛璐璐和角色个性继续 `needs_revision`

以下命令在仓库根目录实际执行，fish 可直接使用；第一次 GLB 命令漏传必填 `--height`，退出 2，修正后按下列完整命令通过

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/model-r3/render.py -- final
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/model-r3/render.py -- final --motion
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
python3 todo/evidence/TASK-047/model-r3/check_preserved.py
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
```

- `dcc-check.json`：PASS，Idle 61、Walk 31、Run 21 帧的骨矩阵与循环端点，鞋底接地距离及五官贴面；动作仍 in-place
- `glb-check.json`：PASS，1.7441906 m、32 骨、三个 clip、嵌入图、法线、UV、权重与 inverse bind；32,822 三角面，比 r2 增加 3,640，主要来自发帽细分和发束采样，未把增加面数解释为质量验收
- `preserved-contract.json`：PASS，独立读取提交中的 r2 GLB 与当前 GLB 比较实际动画采样和骨架数据；一网格、一 primitive、一材质与原灰阶 PNG 保持
- `export-existing.log` 与 `delivery.json`：PASS，从保存的可编辑主文件再次导出的 GLB 字节一致；Python 编译检查与 scoped `git diff --check` 通过
- 本子任务未运行 Cargo、GPU、Windows 或手柄；root 接续同一真实控制器的 Bevy capture，当前 CPU 单帧与数值检查不替代运行验证或作者的审美判断

## 交付与清理

保留可编辑主文件、图集、GLB、复现脚本、参数、日志、哈希与结论。本轮临时图、基线副本和 `.blend1` 已在确认没有 Blender 使用进程、完成 root 与独立查看后清理，保留文件哈希、数量与字节于 `cleanup.json`；source 图集和正式资产保持，output 仅剩模型参数与重导出校验摘要

Root 已用最终GLB运行真实540帧／18秒，26项状态断言PASS，实际查看步行、疾跑、转向及连续帧；范围与运行日志见[集成复验](../../TASK-045/visual-r4/review.md)，不代替人物美术验收
