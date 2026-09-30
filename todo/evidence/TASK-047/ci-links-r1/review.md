# CI 历史证据链接修复

## 原因与修复

2026-09-30，GitHub CI [36711267401](https://github.com/Xarth-Mai/N-SIDE/actions/runs/36711267401) 与 [36707779587](https://github.com/Xarth-Mai/N-SIDE/actions/runs/36707779587) 均在文档检查失败；角色运行时独立评审的四个链接指向本机 output，干净检出没有这些临时路径

修复 [independent-review.md](../runtime-r1/independent-review.md) 的四个链接，改为已提交的 character-r3 与 character-negative 原始 run/state JSON。四份持久记录与本机原件逐字节一致，评审结论不变，未修改检查器或重跑角色录像

## 验证

验收输入为 eccb1411cdea26aad456780ba4dfb9954c975107 加上述四个链接变更；使用 git archive 导出无 output 的干净副本，不混入并行未提交内容

- 负例：修复前 `python3 -B tools/validate_docs.py --root .` 退出 1，精确复现上述四个缺失链接
- 正例：只加入链接修复，同命令退出 0，468 Markdown 文件、102 IDs 通过
- 完整失败步骤：干净副本执行 `bun run check:docs` 退出 0，31 Skills 与 25 项导入检查通过；Bun 依赖复用本机已安装版本
- `git diff --check` 通过；不涉及游戏行为，游戏构建与 GPU 重录不适用

现有 CI 在推送后的干净检出继续执行完整检查；这里记录的是本地已执行的修复验证，不预先宣称远端通过

本轮未生成图片、视频或探测可执行文件；临时干净副本在检查后清理，复现日志留在本地 output/ci-repair/repro.json

## 远端后续回归修复

[36714200681](https://github.com/Xarth-Mai/N-SIDE/actions/runs/36714200681) 已通过原来失败的文档步骤，随后在工具测试暴露故事图的历史硬编码：目录已包含 17 条支线，测试仍要求 12 条；QST-002 还已关联 QST-027 的开始条件，旧测试只允许尾声重访这一项

修复 `tools/tests/story-graph.test.ts`，按权威目录逐角色类别核对投影的全部任务 ID，并明确核对 QST-027 的开始门槛与 QST-010 的可选来信重访。保持剧情、任务数据、发布过滤和主线依赖语义不变

验收输入为 465070c302aba2be7d2506fc1afcdb401bd5e8ad 加测试修复，另建干净 git archive 副本并复用已安装 Bun 依赖；故事图窄测 12 项通过，完整 `bun run test:tools` 为 Python 114 项、Bun 131 项全部通过，`bun run check:types` 通过。没有从并行叙事工作区复制新内容
