# R0 Skills 门禁与来源复验

本轮输入为 `a0deb64e1447d84ab7161e494ee1748de279090c` 及交接包 F01/F02；2026-09-26 在真实 Bun 1.4.2、Python 3.14.7 环境执行，未用 JSON frontmatter 替代 Bun YAML

## 修复与范围

- `manifest.json` schema 2 将必需入口保存在 `policy.required_skills`，原生目录保存在 `local_skills`，专业与适配目录仍由 `skills` 登记；不以固定 31 项数值作为通过条件
- 检查缺失/空/损坏清单、字段类型、来源与路径重复、必需入口、双向文件覆盖、原文件 hash、完整许可文件、项目声明、宿主 metadata、具体命令路径与已登记工具依赖、本地链接及锚点；`third_party` 原始参考同样受检
- 20 处可选兄弟断链改为固定上游 URL，9 个 Bevy 文件显著标注修改；删除断链豁免，未安装技能仍不安装也不占位
- F03 的 Apache 脚本修复独立登记为第 10 个补丁；原始 hash、准确 diff、原因与当前 hash 全部保留；未修改文件继续逐字节比较
- 通用方法/触发条件保持原义，仅改变不可用本地链接的解析方式；Apache LICENSE/NOTICE、qiuaoru 方法论归属及七来源完整 commit 保留

## 实际结果

| 命令 | 结果与证明范围 |
| --- | --- |
| `bun test tools/tests/validate-skills.test.mjs` | PASS，32 项真实 Bun 回归，含坏清单、空目录、未登记脚本、命令缺失、坏锚点、metadata、第三方参考断链与错误豁免 |
| `bun tools/validate-skills.mjs` | PASS，当前 31 个有效入口、25 项专业/适配登记；这是本地完整性与工具依赖检查 |
| `python3 -B -m unittest discover -s tools/tests -p test_skill_upstream.py` | PASS，5 项使用临时真实 Git 对象的回归；篡改原文并同步本地 hash 仍失败，补丁不解释全部变更会失败，来源不可读为 NOT RUN 且总退出非零 |
| `python3 tools/check_skill_upstream.py --checkouts /tmp/nside-source-checkouts.json --output todo/evidence/TASK-006/r0/upstream.json` | PASS，独立比较 7 来源、168 个原始文件/许可、10 个补丁基线及精确 diff、20 个线上链接的固定 tree 目标；原始目录树与登记文件全集相符 |
| `python3 -B tools/validate_docs.py --root .` | PASS；已将 patched 原版文件交回专用 Skills 检查，项目格式规则继续检查本地适配及来源说明 |

[完整来源结果](upstream.json)记录每个来源的 commit、方法、文件数与问题；6 个来源读取上轮独立克隆的固定 Git 对象，MengTo 从 GitHub 固定 tree 与 raw blob 读取并核对 Git blob SHA。首次沙箱网络失败记录为 NOT RUN，获准的宿主联网重跑后才得到 PASS

## 后续可复验入口

在仓库根目录运行以下命令，fish 可直接执行；无 checkout 参数时逐来源读取固定 GitHub 对象，网络失败单列 NOT RUN 并返回非零

```fish
bun run check:skills
python3 tools/check_skill_upstream.py --output output/skills/upstream.json
```

`--checkouts` 可读取 `{source-id: "/absolute/independent/git/checkout"}` JSON，工具要求每个目录确实包含清单中的完整 commit，不读取其工作树作为来源。更新上游时先核对完整目录，再审查补丁能否删除或重做；仅改 manifest hash 不能通过独立原样证明

## 证据边界

本地 hash 检查有意不声称能防止原文与清单一起被改动，独立来源命令专门处理这一边界。改编入口不声称逐字节原样，独立检查只核对它们指向的真实上游路径；其提示词契约由专项审查与宿主调用验证。这里没有运行游戏，也不提高游戏路线图完成度

机器检查能确认引用路径、锚点、已登记依赖与固定来源一致，不能认证人的署名身份、法律授权原件或模型对全部自然语言的理解；真正宿主发现与代表性调用另见本轮宿主证据
