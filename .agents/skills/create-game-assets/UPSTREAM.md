# Upstream

Selected files from [gamedev-skills/awesome-gamedev-agent-skills at 44888f28ff918357ad82c4473352c60a1c5bde5b](https://github.com/gamedev-skills/awesome-gamedev-agent-skills/tree/44888f28ff918357ad82c4473352c60a1c5bde5b/skills/disciplines/create-game-assets)

- Original path: `skills/disciplines/create-game-assets`
- Author: Abhishek Barali and the awesome-gamedev-agent-skills contributors
- License: Apache-2.0; [LICENSE](../../../third_party/skills/awesome-gamedev-agent-skills/LICENSE), [NOTICE](../../../third_party/skills/awesome-gamedev-agent-skills/NOTICE)
- Unpatched files are byte-for-byte copies; patched files and exact diffs are registered in `third_party/skills/manifest.json`; this `UPSTREAM.md` is added by N:SIDE
- N:SIDE entrypoints, dependency choices and data ownership are in [the project adaptation](../nside/SKILL.md)
- Original templates and examples are reference material, not N:SIDE configuration or new task records
- If distributed separately, include the linked license and notice files with the Skill

## Local patches

- `scripts/asset_report.py`: Reject exact color counts above --max-colors (F03)
