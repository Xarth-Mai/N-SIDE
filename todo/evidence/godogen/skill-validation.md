# 第三方 Skill 接入验证

本记录覆盖开发资料接入，不改变 Demo 工作包的验收状态。来源、固定版本、许可与改编范围见 [THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md)，逐文件校验值以 [manifest.json](../../../third_party/skills/manifest.json) 为准

## 接入范围

- 22 个原版 Skill：13 个 Bevy 0.19 基础参考、qiuaoru 两个设计工作流、abagames 规则压力测试、awesome 六个专业工作流，共 99 个上游原文件逐字节保留
- 3 个本地适配：`nside`、`nside-blender-pipeline`、`nside-reference-analysis`，修改来源在文件内显著说明
- 19 个上游方法目录保留在 `third_party/skills/` 供改编与比较，不加入 `.agents/skills/` 的启用集合；避免两个 `level-design` 同名启用
- 7 个来源保留完整许可；Apache-2.0 完整 NOTICE 与 qiuaoru 的第三方方法论归属同时保留
- 20 条原版可选兄弟 Skill 链接没有通过扩装整套仓库解决，各 `UPSTREAM.md` 提供固定 commit 的外部替代路径；所有 Skill 自身 `references/`、脚本、模板与原 `agents/` 配置完整保留

原版 Bevy 元数据中的 `compatibility: opencode,claude-code,cursor` 作为来源内容保留，没有迁入 Claude 配置或执行器。原模板中的示例和占位字段供参考，项目适配要求写回已有设计、资产包与任务记录

## 实际检查

| 检查 | 结果 | 能证明什么 |
| --- | --- | --- |
| 读取 manifest，逐一对比本地文件 SHA-256 | PASS，172 个文件 | 已记录的原文、适配文件、参考资料与许可未漂移 |
| MengTo 固定 git tree 与下载文件的 Git blob hash 对照 | PASS，19 个文件 | 原始文件属于 commit `a965851e27dc179e693fde1bee94457a64e1a7a5`，commit 与 blob ID 分开记录 |
| `bun run check:skills` | PASS，31 个 Skill，25 个登记接入项 | 名称、必需路径、元数据、引用和登记完整性检查 |
| `bun run check:docs` | PASS，初查 185 个 Markdown、71 个 ID；添加本记录后复查 186 个 Markdown、71 个 ID；Skill 检查同时通过 | 本轮检查时点的文档结构和链接有效 |
| 本机 Codex 0.156.1 `app-server` 的 `skills/list`，cwd 为仓库根及 `game/` | PASS，31 个 Skill 均启用，包括全部 13 个 Bevy Skill | 真实运行时能发现；证据存于忽略的 `output/skills/discovery.json` |
| `python3 .agents/skills/bevy-rendering/scripts/audit_renderer_features.py game/Cargo.toml` | 退出 0，有一个 WARNING | 当前仍启用 Bevy 默认 features；没有据此擅自裁剪依赖 |
| Skill Creator 自带 `quick_validate.py` | 12/25 PASS；13 个 Bevy 均只因 `compatibility` 键被旧白名单拒绝 | 该检查器限制，与上述实际 Codex 发现结果分别记录；没有改原文来绕过它 |

`quick_validate.py` 检查的是当前环境 Skill Creator 附带版本，其允许字段为 `allowed-tools`、`description`、`license`、`metadata`、`name`；未将其失败写成通过

渲染 features 原版审查器实际输出：

```text
Renderer feature audit: game/Cargo.toml
WARNING: Bevy default features remain enabled; explicit profile features do not trim them.
```

项目当前的额外 `jpeg`、`dds` 与 `viewer` feature 不等于选择了精简 renderer profile；这个提示没有证明性能故障，也不构成升级或替换插件的理由

## 语义检查与行为证据

按 `prompt-skill-authoring` 的契约核对和 `prompt-skill-review` 静态审查，已处理以下差异：原版资产 manifest 不成为第二台账，Bevy 插件示例不自动成为依赖，Ink/Yarn 不替换既有叙事格式，Blender 静态与绑定动画变换分开，未校准的预算/通过率不成为本项目硬门槛，街机持续读数不提前公开剧情真相，非隔离看图标为 `self-audit`

MengTo `author-game-levels` 明确要求单平面并禁止台阶、坡道和桥梁；这些规则与 N街区既有空间冲突，因此只摘取稳定锚点、系统分层和恢复检查，没有启用其原版关卡 Skill

一条只读行为探针要求根据 `nside` 与 `create-game-assets` 处理“现有小店招牌的同风格变体”。参与过资产工作的代理返回了沿用 `shop.svg`、AST-004 README、已有 Bun 导出、实际尺寸预览、按需生图和真实镜头验收的步骤，没有提出第二台账或自动开通服务；其回答引用 `tools/capture.py` 时，该入口仍在并行落地；随后已由运行验收工作线建立并实际运行。此探针自身没有调用捕获工具，属于上下文知情自查，未执行资产修改，不能证明独立路由质量、工具调用成功或游戏体验通过

未执行原版所有代码示例的逐一编译、Blender 实作、模型间提示词效果对照或 31 个 Skill 的全面行为评测。实际发现、静态有效与真实任务效果分别记录

另起一个未读本轮审查结论的新上下文执行[前向使用探针](skill-forward-probe.md)，实际读取 5 个 Skills、现有素材、实现和录像，运行导出／尺寸报告后形成双招牌制作与视频拆解 brief。它沿用现有 SVG、资产包、世界源数据和 capture，没有采用额外服务、全局台账、单平面关卡或第二套录制器；还找到 8:1 源图与 10:1 牌面的比例检查项，没有修改资产

该探针实际看图范围为开发中的 `viewer-tour-final`，后续版本只核对状态，边界已在其记录说明；交付版本 `761f325` 的实际看图另见[运行证据](runtime-validation.md)。这是一项局部指令使用检查，不代表 31 个 Skills 全面行为验收或盲测
