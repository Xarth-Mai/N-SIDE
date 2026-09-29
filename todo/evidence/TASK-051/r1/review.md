# TASK-051 门前交接进度保存第1轮记录

已验证单点观察跨进程保存与继续、标题和暂停的明确重开、读写失败保留原文件，以及竖屏大字号下的失败流程。随后完成完整三点路线写入新槽与独立进程恢复，门前交接保存技术验收通过，TASK-051 结束；完整序章和作者体验另行验收

## 实现与证据输入

版本化 `progress.json` 只保存三处真实取得的观察和一次性配送路线结果，独立于设置文件；继续从既有安全出生点建立玩家，阶段和确认计数从合法事实派生。重新开始需要独立确认，取消保留信息；坏档关闭本次写入，写失败保持当前可操作会话与旧主文件。长期职责见[事实、认知与恢复契约](../../../../docs/dev/design/systems/state-and-recovery.md)及[人物尺度步行实验](../../../../docs/dev/engineering/player-preview.md)

[独立评审](independent-review.md)已核对完整生产路径与回归设计，未发现阻塞问题，复杂度结论为 `Lean already. Ship.`；其中保存源码和五条原始脚本的冻结 SHA-256。下列五次运行共用二进制 `e138c1b0b44ad880b19440aa48173bcee869878b9c1060a56855e7050fe94c5b`，记录 Git HEAD 为 `24468bbb91aeadf263da80ebb5b4f7543ed74848` 且包含未提交改动，不能把 HEAD 单独当作运行输入的完整身份

[运行摘要](runtime-summary.json)保留真实子进程命令、二进制／脚本 hash、全部检查、性能范围、报告与关键帧 hash；原始 `run.json`、`state.json`、`runtime.log`、留存脚本位于对应 `output/capture/task051-*-r1` 目录。独立评审已重算五次报告与脚本 hash，均与摘要一致，已复核源码仍与独立评审冻结输入一致

## 已完成的真实运行

环境为 Linux `7.2.8-1-cachyos-x86_64`、glibc 2.44、Vulkan，正式 `n-side --walk-preview` 使用真实场景与输入。所有运行均为30fps固定模拟，进程退出0、视频编码 PASS；1350帧全部保存，35/35项机器检查通过

| 运行 | 画幅与帧数 | 检查 | 实际行为 |
| --- | --- | --- | --- |
| write | 1280×720，300帧 | 6/6 PASS | 空槽起步，真实靠近并观察04后保存；返回标题仍持有04 |
| restore | 1280×720，150帧 | 11/11 PASS | 第二进程读取04；长按不穿透重开提示、Esc取消、继续安全出生、暂停中默认保留与明确确认重开；末尾成功保存空槽 |
| write-failure | 1280×720，300帧 | 6/6 PASS | 读取已有合法空槽；观察后真实备份替换失败，显示 WriteFailed，旧槽和本次会话观察分别保留 |
| corrupt | 1280×720，300帧 | 6/6 PASS | 损坏JSON触发ReadFailed，实际观察和返回标题仍可进行；不伪报保存成功 |
| portrait | 720×1280，300帧 | 6/6 PASS | 125%字号重复坏档与返回标题流程，状态和输入检查通过 |

write 第189帧已到真实04观察范围但尚无观察记录，第229帧打开面板后得到04并显示 `saved`，第289帧标题仍保留。restore 第9帧读取04，第18帧持续 Enter 时仍停在重开提示的保留项，第25帧取消后焦点回到原项；第59帧从 `[100,28.045998,-255]` 着地恢复且观察窗关闭，零异常恢复；第139帧仅在明确重开后变为空记录并显示 `saved`

这五次运行没有配送确认输入，因此只支持观察保存、恢复、清空和失败保护；它们不证明已确认路线的跨进程幂等。完整三点路线与 `prologue-save-confirmed.json` 的本轮执行尚待补入

## 隔离目录与失败原文件

write 与 restore 依次使用 `output/progress/task051-live`，第二进程的正常重开产生合法空槽；其主文件为116字节，备份仍保存此前的04。write-failure 把该真实空槽放入 `output/progress/task051-write-failure`，并以同名目录占用 `progress.json.bak`，运行日志记录实际 `Is a directory (os error 21)`，未通过改写游戏结果制造失败

corrupt 与 portrait 共用 `output/progress/task051-corrupt`，只将该隔离目录的槽内容设为7字节 `{broken`；两次启动均记录 `key must be a string at line 1 column 2`。portrait 另用 `output/settings/task051-large` 中的版本1设置文件启用125%字号，保存进度与字号使用独立显式目录，其余运行没有传入设置目录，capture保持设置隔离

[失败文件记录](failure-fixtures.json)保存原内容、来源与前后比较；独立评审重新读取文件并作精确字节比较，结果均相等

| 文件 | 前后共同 SHA-256 | 结果 |
| --- | --- | --- |
| `output/progress/task051-write-failure/progress.json` | `a70f6219ff0b99fa415a99ac32ed5510d4a6c3f4919afddbd5d9adcc232319f9` | 116字节合法空槽保留 |
| `output/progress/task051-corrupt/progress.json` | `be302461cda285db3ed2d45fb767b23e08ca8808b35a638009937df22b332d73` | 7字节损坏槽保留 |

## 实际命令

以下对应 `run.json` 已执行的子进程命令，仅将同一仓库绝对路径缩为根目录相对路径；原始完整参数保存在运行摘要。目录名记录既有证据，重新执行需建立新输出目录并按上述先后关系准备独立进度目录

```fish
game/target/debug/n-side --project-root . --capture output/capture/task051-write-r1/script.json --output output/capture/task051-write-r1 --walk-preview --progress-dir output/progress/task051-live
game/target/debug/n-side --project-root . --capture output/capture/task051-restore-r1/script.json --output output/capture/task051-restore-r1 --walk-preview --progress-dir output/progress/task051-live
game/target/debug/n-side --project-root . --capture output/capture/task051-write-failure-r1/script.json --output output/capture/task051-write-failure-r1 --walk-preview --progress-dir output/progress/task051-write-failure
game/target/debug/n-side --project-root . --capture output/capture/task051-corrupt-r1/script.json --output output/capture/task051-corrupt-r1 --walk-preview --progress-dir output/progress/task051-corrupt
game/target/debug/n-side --project-root . --capture output/capture/task051-portrait-r1/script.json --output output/capture/task051-portrait-r1 --walk-preview --settings-dir output/settings/task051-large --progress-dir output/progress/task051-corrupt
```

## 画面、检查与未测范围

主执行者报告已实际查看 write289、restore18／29／106、write-failure289、corrupt289、portrait289，并通过画面自查；本草稿的独立评审者没有直接查看画面，仅核对状态、日志、文件和报告 hash。当前记录不宣称完整播放视频、实体手柄试玩或作者理解与手感已获验收

独立检查 PASS：冻结输入 hash、报告与脚本 hash、失败文件原字节、限定差异格式与脚本区间。`bun run check:docs` 曾因并行人物证据链接未完成失败，两份评审记录完成后复验 PASS：421个Markdown文件、102个ID、31个Skills、25个导入记录

已读取同轮[共享库测试日志](../../TASK-045/integration-r3/lib-tests.log)：117项通过、0项失败、1项ignored，其中四个进度测试、重开确认及标题保留明确通过；ignored 为设置跨进程worker，由父测试调用。并行铺装修复后的 `lib-tests-final.log` 仍由主执行者复验，本草稿不预先确认其结果。已读[Wiki构建日志](../../TASK-045/integration-r3/docs-build.log)：地图55项测试通过，玩家站101页、开发站168页均PASS，开发站仅有超过500kB的bundle提示

主执行者另报告 fmt PASS，以及 `python3 -B -m unittest discover -s tools/tests -p test_capture.py` 的3项测试PASS；先前 `python3 -B -m unittest tools.tests.test_capture` 入口产生ImportError，随后改用上述发现入口完成。这里明确保留报告来源，本独立评审未重复执行这些命令；Clippy仍待主执行者统一检查

NOT RUN：集齐待选阶段独立进程恢复、未知版本真实GPU失败路径、Windows实机、实体手柄、断电恢复、备份自动回滚、多进程写入协调、完整序章及作者／玩家体验。已实施的窄测覆盖未知版本与矛盾数据校验，不将其写成真实GPU覆盖

当前截图、连续PNG、关键帧与视频仍供主执行者收尾验证，待本轮完整路径和结论登记后统一清理；本次文档评审没有生成新的视觉产物。源码、脚本、日志、状态JSON、版本与参数和文字结论继续保留，视觉清理结果由主执行者在本记录收尾时补齐

## 最终完整路线与独立恢复

在 `output/progress/task051-complete` 空目录运行 `prologue-route.json --character-preview`，2672帧／约89秒、15项检查PASS；真实角色连续步行到04、29、28，错误台阶选择一次后选择服务院，确认只提交一次并实际落盘。root查看894台阶、2545待选、2578错误反馈、2671最终记录帧；同一控制器完成路线，未传送、改写观察或伪造结果

再以最终构建运行 `prologue-save-confirmed.json --character-preview`，90帧、5项检查PASS；第二进程在标题读取三处信息和有效路线，从家门口着地继续，确认计数仍为1、会话内错误计数清零，未重新提交。root实际查看89帧，HUD显示已确认配送路线和保存反馈。最终两次与此前五次合计4112帧、55项机器检查全部通过，二进制各自hash在同一[摘要](runtime-summary.json)中；没有把输入脚本声明当作实际结果

```fish
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/prologue-route.json --progress-dir output/progress/task051-complete --output output/capture/task051-complete-route-r1
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/prologue-save-confirmed.json --progress-dir output/progress/task051-complete --output output/capture/task051-confirmed-r1
```

最终验收输入见[文件hash](accepted-inputs.json)；root统一运行全target测试118项库测、3项Viewer测试PASS，Clippy `-D warnings`与最终所有bin构建PASS，证据在[集成记录](../../TASK-045/integration-r3/review.md)。最后的标准库与ECS参数分组lint修正没有改变保存逻辑，最终恢复进程使用修正后实际构建；前面的运行保留原始二进制hash

技术验收不等于作者已认可叙事节奏或完整游戏完成，本轮不继续增加新剧情系统，回到地图与人物画面的优先制作

本轮视觉清理已完成：确认无游戏、录屏或Blender进程使用文件后，root删除已查看的截图、连续PNG、关键帧与视频，以及本轮24468bb临时素材快照。正式模型、贴图、源工程、参数脚本、日志、状态JSON及文字结论保留；数量、字节与归属见 `todo/evidence/TASK-045/integration-r3/cleanup.json`，街树CPU图另见其 `cleanup.json`
