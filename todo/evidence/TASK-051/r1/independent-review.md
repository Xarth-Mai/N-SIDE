# TASK-051 进度保存独立评审

2026-09-30，独立核对冻结后的进度实现、入口生命周期、capture 读值与回归设计；读取时 HEAD 为 `24468bbb91aeadf263da80ebb5b4f7543ed74848`。本记录只评审门前交接的实际观察、一次性路线结果与明确重开，不签署完整委托、作者体验或 G1 放行

## 结论

源码与测试设计评审 PASS：未发现本次范围内应阻塞交付的正确性问题；复杂度评审为 `Lean already. Ship.`。后续已独立核对117项库测试通过日志，以及五次真实运行的状态、日志、脚本和失败文件原字节，1350 帧、35/35 检查通过；完整三点路线与确认后跨进程恢复仍待本轮实测，不据此完成整张任务卡

## 完整路径

生产观察仍从 `capture::input → entry_input → Observation::handle_story_input` 到 `ShopHandoff::observe/choose`；查看资格沿用实际人物、镜头、距离及碰撞视线。新 capture 字段只读取 `ProgressStore.status` 和 `EntryUi.restart_pending()`，五条脚本只发普通按键，没有改写脚点、观察集合、确认计数或判定结果

`Checkpoint` 独立于 ECS，限制为版本 1、`QST-001`、安全入口 `home`、三处稳定地点 ID 与可选 `service_yard` 结果。加载拒绝重复或未知 ID、未知版本／任务／入口、未集齐观察的确认及 `public_steps` 完成结果；阶段由已取得事实派生，确认次数从有效结果重建为 0 或 1，失败选择次数回到会话内的 0。未保存实体、当前位置、面板、输入焦点或不存在的搬货结果

`progress::install` 在启动时完整读取并验证后替换故事资源；标题与重新加载不再无条件清空故事，继续时沿用真实 `PlayerState::from_map` 重新建立安全出生点。标题与暂停的重开操作先进入默认「保留当前进度」的提示，取消／Esc 保留故事；只有独立选择确认才清空并请求加载。旧按钮通过 `restart` 状态过滤，输入释放、失焦与断连门禁仍在故事和菜单动作之前执行

唯一存档写入系统在 `Last` 汇总事实，单个异步任务完成后再写最新快照；`Drop` 先等待在途写入，再落盘尚未提交的最后状态。观察后立即重开不会让旧任务覆盖最终空进度。相同失败值不每帧重试，后续取得不同事实仍可尝试保存；只有成功任务完成或有效文件加载才显示 `Saved`

进度使用独立 `progress.json` 与 `--progress-dir`，Linux 默认位于用户数据目录；默认 capture 不读写真实进度，`--settings-dir` 不启用进度保存，非步行入口不建立进度路径。仅复用设置模块已有的 4096 字节受限读取与同目录临时文件原子替换，没有改变设置文件内容或设置状态机

加载失败将写入路径关闭并保留原字节，本次仍可操作；写入前重新验证现有文件，再保留合法备份、同步临时文件和替换主文件，失败回报 `WriteFailed`。运行中被外部改成坏档或未知版本时也不能由当前会话覆盖。此路径不包含断电恢复、备份自动回滚或多进程写入协调，本评审不将其算作已实现能力

## 回归与证据边界

- 已读四个进度窄测：事实序列化与派生状态、坏档／未来版本／重复与矛盾结果保留、在途保存后重开并退出的最后空槽、普通退出最终快照／合法备份／临时文件冲突／默认 capture 隔离
- 已读入口回归：键鼠与模拟手柄的暂停／重访／标题保留；标题和暂停均核对默认保留、Esc 取消、失焦阻止确认。GPU restore 脚本另声明持续 Enter 不越过重开提示，继续后观察窗关闭，以及明确确认后的空进度
- 已读 capture 字段回归与 Python 参数转发测试：缺少真实入口资源不能满足断言，错误状态或提示值不能通过；`--progress-dir` 只允许步行脚本。测试中的最小 App 和命令构造替身仅覆盖各自 CPU／参数契约，不是可玩场景证据
- 审查中修订一项证据缺口：write-failure 原设计从缺失文件开始，仅能证明首写失败；现以合法空槽和不可替换的备份目标起步，必须在运行前后比较旧主文件字节，才能宣称旧档保留
- 本评审执行 PASS：限定路径的 `git diff --check`；Python 解析五条脚本，确认全部为 `walk-preview`、事件只有 `start/end/keys`、输入及断言区间均在实际帧范围内
- 文档检查初次 FAIL：`bun run check:docs` 扫描 417 个 Markdown 文件、102 个 ID，唯一错误为并行人物 README 指向当时尚未生成的 `todo/evidence/TASK-047/model-r2/review.md`；对应文件完成后本评审复验 PASS，419 个 Markdown 文件、102 个 ID、31 个 Skills 与25 个导入记录
- 本评审没有运行 Cargo、Clippy 或 GPU；运行由主执行者完成，下面只记录独立读取过的实际证据。未知版本的运行时故障注入、完整路线与确认后跨进程恢复尚未包含在这五次运行中

已独立读取同轮[共享库测试日志](../../TASK-045/integration-r3/lib-tests.log)：117项通过、0项失败、1项ignored；四个进度测试、入口重开保护及标题保留测试均明确通过。ignored 为设置跨进程worker，由另一个父测试显式调用，不计作单独通过。并行铺装之后的最终全测和Clippy由主执行者另行记录，不将此前日志冒充最后版本验证

两份评审记录完成后再运行 `bun run check:docs`，PASS：421个Markdown文件、102个ID、31个Skills与25个导入记录；限定文档差异的 `git diff --check` 通过

## 五次真实运行复核

已读取[运行摘要](runtime-summary.json)及其五个输出目录内的 `run.json`、`state.json`、`runtime.log`、`script.json`，逐一重算报告和脚本 SHA-256，与摘要完全一致；五次均为 Linux Vulkan 正式入口，进程退出 0、状态 PASS、全部帧保存、有限变换检查通过。共同二进制 SHA-256 为 `e138c1b0b44ad880b19440aa48173bcee869878b9c1060a56855e7050fe94c5b`，本页列出的源码冻结 hash 复核仍一致

| 运行 | 帧与检查 | 独立核对的实际范围 |
| --- | --- | --- |
| `task051-write-r1` | 300 帧，6/6 PASS | 仅在真实打开04观察后记录信息并显示 Saved；返回标题保留04，计数为0 |
| `task051-restore-r1` | 150 帧，11/11 PASS | 第二个进程从同一隔离目录读取04；持续 Enter 不越过重开提示；取消保留原信息，继续回到 `[100,28.045998,-255]` 并着地；暂停默认保留，明确确认后重开并保存空槽 |
| `task051-write-failure-r1` | 300 帧，6/6 PASS | 从合法空槽读取 Saved，实际观察后因备份目标为目录报 `Is a directory (os error 21)`，状态变为 WriteFailed；标题保留本次观察 |
| `task051-corrupt-r1` | 300 帧，6/6 PASS | 损坏槽触发 JSON 解析错误并保持 ReadFailed；仍能真实观察04、关闭观察并返回标题 |
| `task051-portrait-r1` | 300 帧，6/6 PASS | 720×1280、125% 字号下重复坏档流程；坏档状态、观察和标题信息一致 |

独立读取[失败文件记录](failure-fixtures.json)和当前文件，精确比较原字节：write-failure 的116字节合法空槽来自第二进程的真实重开结果，前后 SHA-256 均为 `a70f6219ff0b99fa415a99ac32ed5510d4a6c3f4919afddbd5d9adcc232319f9`；corrupt 的7字节 `{broken` 前后均为 `be302461cda285db3ed2d45fb767b23e08ca8808b35a638009937df22b332d73`。`task051-live/progress.json` 当前为空槽，其 `.bak` 仍保存04，符合显式重开保留上一份合法数据的行为

主执行者报告实际查看 write289、restore18／29／106、write-failure289、corrupt289、portrait289，并通过画面自查；本独立评审者没有查看这些画面，只核对上述源码、状态、日志和文件，因此不将主执行者的画面观察写成独立视觉评审。完整帧的编码成功也不表示本评审完整播放了视频

未修改生产代码、任务状态、`AGENTS.md` 或运行验证规则；未创建截图、原始帧、视频或临时探测可执行文件，无本评审视觉产物待清理

## 冻结输入

SHA-256 按文件原始字节计算；后续若修改这些文件，应检查对应结论是否仍有效

| 文件 | SHA-256 |
| --- | --- |
| `game/src/progress.rs` | `14bc878fe2d4e4ab305c1b70b0d7ea33d26730389adbf3e3d689eee025b577e9` |
| `game/src/app.rs` | `93766302d5b9e9a63962d881e3a83ecd01106a4d1b1ab2522527a1367b735fdb` |
| `game/src/capture.rs` | `697096439eb0dc62c40b30daaf8f5961c92ca4e74bdab9a002f821517ebd9f4c` |
| `game/src/settings.rs` | `c14bf37702aef351976078681fc9dec66220405da5e4dfa72cd3a80887b21bbf` |
| `game/src/story.rs` | `2331505ac634513e90ed415967e3625f21373b7f4fda005b2a23c441f291777b` |
| `game/src/lib.rs` | `67132b2ced69afabe5037310576fcb2d32af38066a8b6cbde75a01ac5fbfb4e0` |
| `tools/capture.py` | `f29ccb6f549af8c2c2c0762f90fb870c5648e024125104239b0ffc45e02365b1` |
| `tools/tests/test_capture.py` | `6584268ec79653a39cf18e65f9c510af8989d399414ec9da3f72c50890b2699c` |
| `game/capture/prologue-save-confirmed.json` | `37ad3603af7ef66a2969df03483bbf336938fb2fb44a2f3a0c8cb8f7d0ef8467` |
| `game/capture/prologue-save-corrupt.json` | `acf4de5c338f15887d75dcad15ea49705298ee0bc4fe7dab48bcfcbd3283299d` |
| `game/capture/prologue-save-restore.json` | `3ef9be4e5da8d225d3a73fcd910e47b9426690e6faab4a72ba814366a11b1dd5` |
| `game/capture/prologue-save-write-failure.json` | `c490b4fe51863fc75056e49564b1495701d94c4f42cd74d769c03c2c30bb5a35` |
| `game/capture/prologue-save-write.json` | `22c70c6c5bcfd2d3cf070d162254507326aaf714f8b6d5bd68485898f166f7b6` |
