# Third-party development Skills and methods

本清单记录 N:SIDE 实际保留的开发 Skills 与改编方法。完整文件映射和 SHA-256 校验值见 [manifest.json](third_party/skills/manifest.json)，项目差异见 [.agents/skills/nside/SKILL.md](.agents/skills/nside/SKILL.md)

原版 Skill 目录逐文件保留，仅增加 `UPSTREAM.md`；项目改编明确标注来源。下面的许可适用于相应开发材料，不改变 N:SIDE 其他代码或资产的许可证。字体、模型、声音、生成服务与编码器的条款单独记录在所属资产包或依赖中

## chrisgliddon/bevy-skills

- Author / attribution: Chris Gliddon
- Upstream commit: [`b1b4da5744ebbd5c526342b2351967411cd5ca61`](https://github.com/chrisgliddon/bevy-skills/tree/b1b4da5744ebbd5c526342b2351967411cd5ca61)
- License: MIT; [LICENSE](third_party/skills/bevy-skills/LICENSE)

| Upstream | Local | Mode |
| --- | --- | --- |
| `skills/bevy-core-concepts` | [`.agents/skills/bevy-core-concepts`](.agents/skills/bevy-core-concepts/SKILL.md) | 原版加已登记补丁 |
| `skills/bevy-ecs-components` | [`.agents/skills/bevy-ecs-components`](.agents/skills/bevy-ecs-components/SKILL.md) | 原样 Skill |
| `skills/bevy-ecs-queries` | [`.agents/skills/bevy-ecs-queries`](.agents/skills/bevy-ecs-queries/SKILL.md) | 原样 Skill |
| `skills/bevy-ecs-systems` | [`.agents/skills/bevy-ecs-systems`](.agents/skills/bevy-ecs-systems/SKILL.md) | 原样 Skill |
| `skills/bevy-assets` | [`.agents/skills/bevy-assets`](.agents/skills/bevy-assets/SKILL.md) | 原样 Skill |
| `skills/bevy-custom-assets` | [`.agents/skills/bevy-custom-assets`](.agents/skills/bevy-custom-assets/SKILL.md) | 原样 Skill |
| `skills/bevy-save-load` | [`.agents/skills/bevy-save-load`](.agents/skills/bevy-save-load/SKILL.md) | 原版加已登记补丁 |
| `skills/bevy-cameras` | [`.agents/skills/bevy-cameras`](.agents/skills/bevy-cameras/SKILL.md) | 原版加已登记补丁 |
| `skills/bevy-rendering` | [`.agents/skills/bevy-rendering`](.agents/skills/bevy-rendering/SKILL.md) | 原版加已登记补丁 |
| `skills/bevy-animation` | [`.agents/skills/bevy-animation`](.agents/skills/bevy-animation/SKILL.md) | 原样 Skill |
| `skills/bevy-ui` | [`.agents/skills/bevy-ui`](.agents/skills/bevy-ui/SKILL.md) | 原版加已登记补丁 |
| `skills/bevy-testing` | [`.agents/skills/bevy-testing`](.agents/skills/bevy-testing/SKILL.md) | 原版加已登记补丁 |
| `skills/bevy-diagnostics-profiling` | [`.agents/skills/bevy-diagnostics-profiling`](.agents/skills/bevy-diagnostics-profiling/SKILL.md) | 原版加已登记补丁 |

保留 13 个 Bevy 0.19 基础参考及其完整资源；实际 API 以本地锁文件、源码和测试为准。原 `compatibility` 字段提及其他宿主，不安装 Claude 配置。未采用的可选兄弟引用已直接修为固定版本线上链接，9 个改动文件保留显著修改说明及逐文件补丁，其余原文件保持原样

## qiuaoru-coder/game-design-agent-skills

- Author / attribution: qiuaoru-coder; methodology: 六边形老闪（张鹏）
- Upstream commit: [`29e8c759ca26c6bd4f337744b647dc728bf322b6`](https://github.com/qiuaoru-coder/game-design-agent-skills/tree/29e8c759ca26c6bd4f337744b647dc728bf322b6)
- License: MIT; [LICENSE](third_party/skills/game-design-agent-skills/LICENSE), [THIRD_PARTY_NOTICES.md](third_party/skills/game-design-agent-skills/THIRD_PARTY_NOTICES.md)

| Upstream | Local | Mode |
| --- | --- | --- |
| `gameplay-mechanism-designer` | [`.agents/skills/gameplay-mechanism-designer`](.agents/skills/gameplay-mechanism-designer/SKILL.md) | 原样 Skill |
| `game-design-reality-check` | [`.agents/skills/game-design-reality-check`](.agents/skills/game-design-reality-check/SKILL.md) | 原样 Skill |

机制方法论归属六边形老闪（张鹏）。保留上游完整 `THIRD_PARTY_NOTICES.md`；其中关于改编和再分发授权的表述为维护者公开声明，本项目未持有独立授权原件，也未复制原始 PDF

## abagames/agentic-gamedev-skills

- Author / attribution: abagames
- Upstream commit: [`24a4cdce3b629f123162c0bdcf61647eeb85f8db`](https://github.com/abagames/agentic-gamedev-skills/tree/24a4cdce3b629f123162c0bdcf61647eeb85f8db)
- License: MIT; [LICENSE](third_party/skills/agentic-gamedev-skills/LICENSE)

| Upstream | Local | Mode |
| --- | --- | --- |
| `.agents/skills/stress-testing-game-concepts` | [`.agents/skills/stress-testing-game-concepts`](.agents/skills/stress-testing-game-concepts/SKILL.md) | 原样 Skill |
| `.agents/skills/implementing-gameplay-invariants` | [`third_party/skills/agentic-gamedev-skills/references/implementing-gameplay-invariants`](third_party/skills/agentic-gamedev-skills/references/implementing-gameplay-invariants/SKILL.md) | 原样参考，不启用 |
| `.agents/skills/directing-game-visuals` | [`third_party/skills/agentic-gamedev-skills/references/directing-game-visuals`](third_party/skills/agentic-gamedev-skills/references/directing-game-visuals/SKILL.md) | 原样参考，不启用 |
| `.agents/skills/maximizing-game-feel` | [`third_party/skills/agentic-gamedev-skills/references/maximizing-game-feel`](third_party/skills/agentic-gamedev-skills/references/maximizing-game-feel/SKILL.md) | 原样参考，不启用 |
| `.agents/skills/evaluating-gameplay-balance` | [`third_party/skills/agentic-gamedev-skills/references/evaluating-gameplay-balance`](third_party/skills/agentic-gamedev-skills/references/evaluating-gameplay-balance/SKILL.md) | 原样参考，不启用 |
| `.agents/skills/gating-intent-legibility` | [`third_party/skills/agentic-gamedev-skills/references/gating-intent-legibility`](third_party/skills/agentic-gamedev-skills/references/gating-intent-legibility/SKILL.md) | 原样参考，不启用 |

## gamedev-skills/awesome-gamedev-agent-skills

- Author / attribution: Abhishek Barali and the awesome-gamedev-agent-skills contributors
- Upstream commit: [`44888f28ff918357ad82c4473352c60a1c5bde5b`](https://github.com/gamedev-skills/awesome-gamedev-agent-skills/tree/44888f28ff918357ad82c4473352c60a1c5bde5b)
- License: Apache-2.0; [LICENSE](third_party/skills/awesome-gamedev-agent-skills/LICENSE), [NOTICE](third_party/skills/awesome-gamedev-agent-skills/NOTICE)

| Upstream | Local | Mode |
| --- | --- | --- |
| `skills/disciplines/create-game-assets` | [`.agents/skills/create-game-assets`](.agents/skills/create-game-assets/SKILL.md) | 原版加已登记补丁 |
| `skills/disciplines/game-ui-ux` | [`.agents/skills/game-ui-ux`](.agents/skills/game-ui-ux/SKILL.md) | 原样 Skill |
| `skills/disciplines/dialogue-systems` | [`.agents/skills/dialogue-systems`](.agents/skills/dialogue-systems/SKILL.md) | 原样 Skill |
| `skills/disciplines/audio-design` | [`.agents/skills/audio-design`](.agents/skills/audio-design/SKILL.md) | 原样 Skill |
| `skills/disciplines/level-design` | [`.agents/skills/level-design`](.agents/skills/level-design/SKILL.md) | 原样 Skill |
| `skills/disciplines/game-ai` | [`.agents/skills/game-ai`](.agents/skills/game-ai/SKILL.md) | 原样 Skill |
| `skills/disciplines/` | [`.agents/skills/nside`](.agents/skills/nside/SKILL.md) | 基于上游改编 |

保留完整 Apache-2.0 LICENSE 与 NOTICE；`create-game-assets/scripts/asset_report.py` 有已登记补丁，修复精确颜色数量为上限加一时漏报的问题，文件内保留原作者、版本、许可及修改说明。其余五个专业目录及该目录的未改文件保持原样；项目适配在独立的 N:SIDE 文件中维护

## poorvith-mp/skills-gamedev

- Author / attribution: Poorvith M P
- Upstream commit: [`e8b87e9086fbf2322b1c216c2d2de85954bf4015`](https://github.com/poorvith-mp/skills-gamedev/tree/e8b87e9086fbf2322b1c216c2d2de85954bf4015)
- License: MIT; [LICENSE](third_party/skills/skills-gamedev/LICENSE)

| Upstream | Local | Mode |
| --- | --- | --- |
| `skills/blender-modeling/; skills/blender-animation/; skills/blender-tooling/` | [`.agents/skills/nside-blender-pipeline`](.agents/skills/nside-blender-pipeline/SKILL.md) | 基于上游改编 |
| `skills/blender-modeling` | [`third_party/skills/skills-gamedev/references/blender-modeling`](third_party/skills/skills-gamedev/references/blender-modeling/SKILL.md) | 原样参考，不启用 |
| `skills/blender-animation` | [`third_party/skills/skills-gamedev/references/blender-animation`](third_party/skills/skills-gamedev/references/blender-animation/SKILL.md) | 原样参考，不启用 |
| `skills/blender-tooling` | [`third_party/skills/skills-gamedev/references/blender-tooling`](third_party/skills/skills-gamedev/references/blender-tooling/SKILL.md) | 原样参考，不启用 |
| `skills/narrative-design` | [`third_party/skills/skills-gamedev/references/narrative-design`](third_party/skills/skills-gamedev/references/narrative-design/SKILL.md) | 原样参考，不启用 |
| `skills/playtesting` | [`third_party/skills/skills-gamedev/references/playtesting`](third_party/skills/skills-gamedev/references/playtesting/SKILL.md) | 原样参考，不启用 |
| `skills/level-design` | [`third_party/skills/skills-gamedev/references/level-design`](third_party/skills/skills-gamedev/references/level-design/SKILL.md) | 原样参考，不启用 |
| `skills/tech-art` | [`third_party/skills/skills-gamedev/references/tech-art`](third_party/skills/skills-gamedev/references/tech-art/SKILL.md) | 原样参考，不启用 |
| `skills/game-audio` | [`third_party/skills/skills-gamedev/references/game-audio`](third_party/skills/skills-gamedev/references/game-audio/SKILL.md) | 原样参考，不启用 |

## htdt/godogen

- Author / attribution: Alex Ermolov
- Upstream commit: [`0b725bca053769a4727f76c332bf1f7b42e146ab`](https://github.com/htdt/godogen/tree/0b725bca053769a4727f76c332bf1f7b42e146ab)
- License: MIT; [LICENSE.md](third_party/skills/godogen/LICENSE.md)

方法改编：`prompts/runtime.md`、`engines/bevy.md` → [运行验收](docs/production/runtime-validation.md)、[Viewer capture](game/src/bin/map_viewer/capture.rs)与[执行入口](tools/capture.py)；`asset-gen/rembg.md` 等 → [资产工具审查](docs/production/asset-tool-review.md)与 [asset_preview.py](tools/asset_preview.py)。本地代码连接现有 Bevy/资产路径；没有复制发布器、空项目 scaffold 或多供应商生成器

## MengTo/Skills

- Author / attribution: Meng To
- Upstream commit: [`a965851e27dc179e693fde1bee94457a64e1a7a5`](https://github.com/MengTo/Skills/tree/a965851e27dc179e693fde1bee94457a64e1a7a5)
- License: MIT; [LICENSE](third_party/skills/mengto-skills/LICENSE)

| Upstream | Local | Mode |
| --- | --- | --- |
| `agent-skills/codex/video-to-superprompt/; agent-skills/game-development/build-hybrid-game-assets/` | [`.agents/skills/nside-reference-analysis`](.agents/skills/nside-reference-analysis/SKILL.md) | 基于上游改编 |
| `agent-skills/codex/video-to-superprompt` | [`third_party/skills/mengto-skills/references/video-to-superprompt`](third_party/skills/mengto-skills/references/video-to-superprompt/SKILL.md) | 原样参考，不启用 |
| `agent-skills/game-development/author-game-levels` | [`third_party/skills/mengto-skills/references/author-game-levels`](third_party/skills/mengto-skills/references/author-game-levels/SKILL.md) | 原样参考，不启用 |
| `agent-skills/game-development/design-game-encounters` | [`third_party/skills/mengto-skills/references/design-game-encounters`](third_party/skills/mengto-skills/references/design-game-encounters/SKILL.md) | 原样参考，不启用 |
| `agent-skills/game-development/create-game-vfx` | [`third_party/skills/mengto-skills/references/create-game-vfx`](third_party/skills/mengto-skills/references/create-game-vfx/SKILL.md) | 原样参考，不启用 |
| `agent-skills/game-development/build-game-audio-feedback` | [`third_party/skills/mengto-skills/references/build-game-audio-feedback`](third_party/skills/mengto-skills/references/build-game-audio-feedback/SKILL.md) | 原样参考，不启用 |
| `agent-skills/game-development/build-hybrid-game-assets` | [`third_party/skills/mengto-skills/references/build-hybrid-game-assets`](third_party/skills/mengto-skills/references/build-hybrid-game-assets/SKILL.md) | 原样参考，不启用 |

## 改编与未采用范围

- **bevy-skills / defer**：`skills/bevy-capture/` → `docs/production/runtime-validation.md` — Keep one capture path using existing Bevy screenshot/Viewer facilities; do not add bevy_capture or a second encoder framework
- **bevy-skills / defer**：`skills/bevy-physics/`, `skills/bevy-vfx/`, `skills/bevy-audio/`, `skills/bevy-input-actions/` → `.agents/skills/nside/SKILL.md` — Revisit when choosing actual physics, VFX, audio or input backends; no automatic Rapier/Hanabi/Seedling dependency
- **bevy-skills / reject**：`skills/bevy/` → `.agents/skills/nside/SKILL.md` — The upstream all-skills router assumes the full collection; use the existing N:SIDE progress owners plus project specialist adapter
- **game-design-agent-skills / reject**：`game-gmt-review/` → `.agents/skills/game-design-reality-check/` — Narrow legacy review overlaps the selected reality check
- **awesome-gamedev-agent-skills / defer**：`skills/disciplines/shader-programming/`, `skills/disciplines/physics-tuning/`, `skills/disciplines/game-feel/`, `skills/disciplines/camera-systems/` → `.agents/skills/nside/SKILL.md` — Install only when a concrete task needs these workflows; Bevy camera and project art references cover current work
- **skills-gamedev / adapt**：`skills/blender-modeling/`, `skills/blender-animation/`, `skills/blender-tooling/` → `.agents/skills/nside-blender-pipeline/` — Use asset-specific budgets and GLB export; preserve rigs and sources; no universal Ctrl+A, fixed skeleton, 60 FPS or add-on framework
- **skills-gamedev / extract**：`skills/narrative-design/`, `skills/playtesting/`, `skills/level-design/`, `skills/game-audio/`, `skills/tech-art/` → `.agents/skills/nside/references/content-and-review.md` — Preserve original references, retain existing narrative/review owners, one level-design Skill; reject universal metrics, mandatory LOD counts and middleware
- **agentic-gamedev-skills / extract**：`.agents/skills/implementing-gameplay-invariants/`, `.agents/skills/directing-game-visuals/`, `.agents/skills/maximizing-game-feel/`, `.agents/skills/evaluating-gameplay-balance/`, `.agents/skills/gating-intent-legibility/` → `.agents/skills/nside/references/content-and-review.md`, `docs/production/runtime-validation.md` — Use state contracts, captured proof and bounded feedback; preserve narrative uncertainty; label nonisolated review self-audit and omit arcade score gates
- **mengto-skills / adapt**：`agent-skills/codex/video-to-superprompt/`, `agent-skills/game-development/build-hybrid-game-assets/` → `.agents/skills/nside-reference-analysis/` — Keep media inspection, temporal sampling, visual layers and asset representation; replace web architecture with project Bevy implementation brief
- **mengto-skills / extract**：`agent-skills/game-development/author-game-levels/` → `.agents/skills/nside/references/content-and-review.md` — Keep stable anchors, separated geometry and state; reject mandatory flat gameplay plane, which conflicts with existing slopes, steps and bridges
- **mengto-skills / extract**：`agent-skills/game-development/design-game-encounters/`, `agent-skills/game-development/create-game-vfx/`, `agent-skills/game-development/build-game-audio-feedback/` → `.agents/skills/nside/references/content-and-review.md` — Use event-role clarity, cleanup and recovery checks; no browser/mobile architecture or implemented-combat claims
- **godogen / adapt**：`prompts/runtime.md`, `engines/bevy.md` → `docs/production/runtime-validation.md` — Reuse the real scene/input path, wait for asynchronous screenshots and record assertions separately from observed visuals; keep locked Bevy
- **godogen / selective-methods**：`asset-gen/rembg.md`, `asset-gen/motion.md`, `asset-gen/tools/` → `docs/production/assets.md`, `docs/production/asset-tool-review.md`, `tools/asset_preview.py` — Keep source-image approval, alpha-edge contrast, asset scale and numeric QA where current assets support them; no second generation workflow
- **godogen / reject**：`publish.sh`, `setup.md`, `AGENTS.md`, `asset-gen/tools/asset_gen.py` → `docs/production/runtime-validation.md`, `docs/production/assets.md` — No scaffold, destructive publisher, external agent framework or paid-provider defaults

## 更新与独立分发

本地完整性由 `bun tools/validate-skills.mjs` 检查；固定上游内容由 `python3 tools/check_skill_upstream.py --output output/skills/upstream.json` 独立比较，检查完整目录、原文件、补丁基线与精确差异、许可及固定链接目标。后者读取 GitHub 固定 Git 对象，或 `--checkouts` 提供的独立 Git checkout，不以本地 hash 自洽替代上游证据

更新选中目录时固定新的完整 commit，比较原文件与项目适配的差异，再刷新 manifest 校验值及本清单。不要将上游示例占位符写成项目配置；本地文件、锚点、宿主 metadata、工具依赖与固定上游引用分别校验，不保留断链豁免。独立打包任何 Skill 时一并携带对应 LICENSE、NOTICE、方法归属说明和 UPSTREAM 信息
