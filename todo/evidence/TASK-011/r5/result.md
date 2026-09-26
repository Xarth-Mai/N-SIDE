# R5 回归、真实工作流与交付

输入基线为审查提交 a0deb64e1447d84ab7161e494ee1748de279090c，实施按 R0—R4 已分别提交。最终输入哈希见 input-files.json；源设定、对象 ID、叙事关系、Rust源码与运行资产保留，本轮未推进后续游戏任务，也未推送远端

## 已确认问题与结果

| 问题 | 实际落地与复验 |
| --- | --- |
| F01 | schema/version/必需字段/项目策略/双向文件覆盖检查，缺失与损坏清单不再成功；真实 Bun 负例通过 |
| F02 | 脚本、工具依赖、锚点、metadata、第三方参考和方法落点纳入检查；补丁与原文区分，固定上游独立复核 PASS |
| F03 | Apache 来源颜色上限加一修复，保留修改声明/许可/NOTICE/补丁；N-1/N/N+1/N+2 回归通过 |
| F04 | 非空预期、请求/返回 cwd 集合与唯一性检查；真实 Codex 双 cwd发现及模型显式/隐式/反向调用完成 |
| F05 | player/dev 同一发布规则用于构建、导航、搜索、附件、数据及开发服务；真实构建和 HTTP 负例通过 |
| F06 | 活动状态仅由 Markdown 任务卡编辑，Bun 生成看板；旧 JSON/看板/日志归档、旧生成器退出 |
| F07 | 项目制作手册具有真实输入、前提、步骤、交付、检查和失败处理；现有 Logo 任务走通新入口与长期回写 |

## 最终实际检查

- `bun run test:tools`：103 Python、79 Bun 测试全部 PASS，日志见 tests.log；包含缺失清单/引用、颜色边界、来源篡改、宿主集合、任务取消/重开/无环/证据以及生成只读与稳定性
- `bun run check:docs`：最终 PASS，Skills31项/25登记迁入；Markdown数量随后增加本交付记录，最终命令日志见 final-checks.log
- `bun run check:narrative`：PASS，1文件22状态；`bun run check:story-design`：PASS，91场所/12街坊/35角色/18委托/64引用建筑；`bun run check:templates`：PASS，1文件2状态
- `bun run docs:build`：PASS，真实 VitePress player88页/380文件、dev143页/597文件；build.log保留输出。两次 `bun tools/wiki.mjs check` 再次核对实际产物 PASS
- [R3 HTTP检查](../../TASK-009/r3/wiki-http.json)：24项真实请求 PASS，覆盖两站、搜索、旧URL、JSON、编码路径与@fs。R5没有改变发布代码；R3另有真实失败构建复现，未将模拟 fixture 当成完整构建
- `python3 tools/probe_skills.py --output output/skills/r5-host/discovery.json`：PASS，当前实际宿主分别在根目录和game发现31个已启用入口，缺失0、错误0；完整结果见 discovery.json
- `python3 tools/check_skill_upstream.py --checkouts /tmp/nside-source-checkouts.json --output todo/evidence/TASK-011/r5/upstream.json`：PASS，7来源、168原始文件/许可、10补丁、20固定兄弟链接；6来源读独立Git对象，MengTo读GitHub固定tree/blob。原始作者、许可证、NOTICE和方法归属继续保留
- [最终迁移核对](migration-final.json)：198个基线文件有实际落点、61旧项有明确处理；旧JSON原字节归档，8个结构数据的允许变化之外内容未变。逐文件/逐段/逐旧项规则见 [R1索引](../../TASK-007/r1/migration-map.json)，执行快照见 [R4映射](../../TASK-010/r4/migration-executed.json)

## 真实模型与资产小任务

模型沿用实际宿主配置，CLI 0.156.1，独立 ephemeral turn。原始 JSONL、stderr和完整回答在 output/skills/r5-host/；[host-cases.json](host-cases.json)保留原始请求、命令轨迹、线程ID、事件哈希、正文及退出码

| 用例 | 实际观察 | 结果 |
| --- | --- | --- |
| 根目录显式 n-side-guide | 读真实卡片、区分技术迁移与游戏、只读查询；首轮猜错TASK-010文件名 | 状态判断通过、链接失败，原回答保留 |
| 同一显式请求复验 | guide补充使用实际读取路径；读取真实卡片并输出存在的全部链接 | turn.completed、CLI exit0，PASS |
| game目录隐式capture审查 | 读取nside、bevy-testing及实际FreeCamera/capture源码；指出已有检查的范围与缺口 | turn.completed、exit0，PASS；未运行游戏 |
| 百科一句润色反例 | 仅返回一句，命令0，不开任务、不改状态 | turn.completed、exit0，PASS |
| 显式work-loop TASK-019 | 读任务与create-game-assets，先看图，再跑报告和预览；实际检查并回写品牌说明 | turn.completed、exit0，PASS；保持active交根会话复核 |

[TASK-019资产证据](../../TASK-019/r5/result.md)完整记录真实命令、源hash、尺寸、alpha、三底色预览及self-audit。根会话又查看同一预览、核对输入hash后完成该技术任务。长期约定回写source-assets/branding/README.md，既有资产working/许可unverified状态未改变。流程试运行没有修改Logo图片或伪造新玩法

prompt-skill-review：保留指定任务优先、只读查询、已定设计、技术自主与真实体验边界；静态契约与实际代表请求均已复验。首轮链接问题已明确修订并同请求复验，不以单次成功宣称所有自然语言都可靠

ponytail-review：R5只修真实观察到的一条路径输出问题，复用既有图片工具与资产包；没有新增调度框架、平行台账或空工具。Lean already. Ship.

## 限制与下一项

- CUA创建浏览器失败，环境枚举为 apps=[]、browsers=[]；浏览器实际点击/地图交互 NOT RUN。实际HTTP、构建产物及组件单测已执行，不能替代人工浏览体验；临时预览已关闭
- 未修改 Rust、Bevy版本、运行资产或capture行为，本次纯文档/工具验收没有新跑游戏、GPU录像、Windows或手柄操作，也没有替作者或试玩者验收
- 真实调用是有限样本。原R0隐式调用曾有CLI exit143，R5相应路径exit0；历史异常仍保留，不能保证宿主将来所有退出路径稳定
- 下一项游戏工作仍为 TASK-012《最后的玩具》机制走查，等待作者对已交付操作因果与恢复假设判断。本轮到R5交付结束，不继续推进路线图

## 后续日常入口

以下命令在仓库根执行，可直接用于fish

```fish
bun run tasks:list
bun run tasks:check
bun run check:docs
bun run docs:build
bun run docs:dev:player
```

执行指定任务使用 n-side-work-loop，读取其specs与专业Skill；交付后先做适用检查，需要真实运行时用[运行验证](../../../../docs/dev/validation/runtime.md)的capture入口。结果写入本任务evidence，长期决定回写docs或原资产包，完成后运行tasks:sync与tasks:check；仅查进度不修改卡片
