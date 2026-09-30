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
