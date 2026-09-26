# R0：真实 Codex 宿主发现与代表性模型行为

日期为 2026-09-26；审查输入为 `a0deb64e1447d84ab7161e494ee1748de279090c` 加本轮工作区修改，宿主为 Codex CLI `0.156.1`。本记录与模拟 RPC 单元测试分开：发现使用真实 `app-server`，行为使用实际 `codex exec` 新模型 turn；模型沿用宿主配置，未指定新模型或外部服务

## F04 修复与回归

`tools/probe_skills.py` 从 schema v2 的 `policy.required_skills`、`local_skills` 与 `skills` 建立预期集合，集合必须非空，名字与路径唯一，必需入口必须已登记且文件存在。请求 cwd 非空且唯一，返回 cwd 必须逐一匹配请求集合且无重复；缺失、禁用的 Skill 和宿主错误不能 PASS

```fish
python3 -B -m unittest discover -s tools/tests -p test_probe_skills.py -v
python3 tools/probe_skills.py --output output/skills/r0-host/discovery.json
```

- PASS：3 个测试，包括空 expected、空 required policy、缺失声明/文件、重复声明、重复请求 cwd、重复返回 cwd、缺少或额外 cwd、必需入口缺失/禁用、正常双 cwd 结果，以及没有 CLI 时 NOT RUN
- PASS：真实 `skills/list` 返回仓库根和 `game/`，各有 31 项已启用、缺失 0、错误 0；数字是本轮观察结果，检查器没有写死 31
- 首次受外层沙箱限制，宿主无法初始化自身 SQLite 状态目录；按用户授权在正常宿主权限下重试后通过。此处记录环境失败，不把第一次失败删除为不存在

## 实际模型行为

四个请求各开启独立 `codex exec --ephemeral --sandbox read-only --json`，没有复用当前对话的解释。原始 prompt、JSONL 事件、命令输出、最终正文及摘要保存在忽略目录 `output/skills/r0-host/`；`behavior-summary.json` 记录事件 SHA-256、线程 ID、命令数与 CLI 退出码

| 用例 | cwd | 实际观察 | 结果 |
| --- | --- | --- | --- |
| 显式 `$nside $bevy-testing`，审查现有 capture 与失败脚本 | 根目录 | 实际读取两项 Skill 和真实实现，说明 FreeCamera 调用链与 1000 m 失败条件；明确游戏未运行 | PASS，turn.completed，CLI exit 0 |
| 未点名 Skill，请求分析 Esc 禁用时输入阻断证据 | game | 主动读取 nside 和 bevy-testing，定位真实脚本与最大区间位移检查；指出旋转和逐帧 enabled 未覆盖，未虚报运行成功 | 行为 PASS，turn.completed；CLI 最终 exit 143，进程退出异常不计为工具完整成功 |
| 普通百科一句同义润色 | 根目录 | 仅返回一句正向正文，命令调用 0，没有启动制作、修改进度或调用验收 | PASS，turn.completed，CLI exit 0 |
| 显式 `$n-side-guide` 只读查询 | game | 先找仓库根，读真实进度与记录；区分本轮技术迁移和游戏进度，不把文档迁移计入游戏完成 | PASS，turn.completed，CLI exit 0 |

隐式用例的完整 agent_message 与 turn.completed 已保存在 JSONL；最后正文从事件原样另存为 `implicit-game.answer-from-events.md`，没有伪造缺失的 CLI `--output-last-message` 文件。stderr 有可用模型列表刷新超时，但实际模型 turn 完成；本轮不据此宣称所有宿主退出路径稳定

四份原始输入及事件摘要已跟踪在 [host-cases.json](host-cases.json)，可在仓库根用 fish 复跑第一项只读用例

```fish
mkdir -p output/skills/r0-host-recheck
jq -r '.cases[0].prompt' todo/evidence/TASK-006/r0/host-cases.json | codex exec --ephemeral --sandbox read-only --json -C . --output-last-message output/skills/r0-host-recheck/answer.md - > output/skills/r0-host-recheck/events.jsonl 2> output/skills/r0-host-recheck/stderr.log
```

## 证据边界

这是四个真实模型请求的有限行为样本，不是盲测、所有 Skill 的全量执行或未来路由保证。发现检查只证明宿主可见和启用，不能替代上述模型行为；模型行为也不能证明游戏运行、视觉品质或作者验收

只读查询观察到的是迁移中旧进度源仍活动的瞬间，最终任务卡切换后应重新检查新入口。已发现的 capture 覆盖边界留给关联工程任务，不在本次 F04 中改写游戏控制逻辑

协议依据为 [Codex App Server 官方文档](https://learn.chatgpt.com/docs/app-server)，并核对本机 `codex exec --help`、`codex app-server --help` 与 CLI 版本；没有用模拟 RPC 或静态读取冒充真实调用
