# R3 全量文档与手册迁移

输入：R2 提交 `0cba1e6`、R1 逐文件迁移表及原审查基线。输出：player 百科/指南、dev 方向/规格/工程/制作/验收/手册/决定分层；旧正文来源仍由 Git 与迁移表追溯

## 实际检查

- `python3 -B tools/validate_docs.py --root .`：PASS，218 Markdown、83 个登记身份，包含对象 catalog；引用锚点按实际 VitePress 规则核对
- `python3 -B tools/validate_story_design.py --root .`：PASS，91 场所、12 街坊、35 角色、18 委托、64 个被叙事引用的建筑；此数量不是全城建筑数或运行内容数
- `python3 -B tools/validate_narrative.py --root .`：PASS，1 个叙事文件、22 个状态
- `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'`：134 项 PASS
- `bun test tools/tests/validate-skills.test.mjs`：35 项 PASS；包括真实 Bun YAML、缺失与损坏输入、命令和来源覆盖、宿主 metadata、实际 VitePress 锚点及迁入方法落点
- `bun tools/validate-skills.mjs`：31 个 Skills、25 项迁入/适配 PASS，未重新修改原版文件
- [结构数据对照](structured-data.json)：8 个 JSON/CSV 的逐字节或允许字段归一后对照 PASS；仅路径与 approved→accepted 映射变化，对象 ID、关系与叙事状态未变
- [正文保留检查](player-prose.json)：原玩家设定中的 673 段长正文，逐行归一链接后均仍在新文档中；未发现缺失段落。该检查补充人工职责走查，不代替对标题、短文和表格的语义审查
- 完整双构建、真实开发服务器、搜索/数据/旧 URL/越界检查见 [Wiki 实测](wiki.md)，构建日志和结果分别留存

## 内容分工走查

《最后的玩具》的完整因果、人物后续、现实物件可追踪规则保留在百科；操作、信息、固定夹/运输/恢复假设与资源要求留在开发规格；作者尚待判断的内容在下一阶段任务卡续接，不提前批准。潜梦世界事实与游戏检查点分开，主线证据的制作要求移入叙事手册

开发手册补齐输入、前提、步骤、交付、检查与失败处理：机制/状态、资产家族与尺度/alpha/颜色空间、Blender/动画、UI焦点和返回、声音/VFX生命周期、内容加载与退出、参考分析及自查/隔离评审。新方法页保留上游版本、作者与许可证，manifest 的采用记录同步指向实际落点

## 评审与边界

prompt-skill-review：本阶段 Skills 仅更正读取路径、归属说明与来源哈希；已定设计、真实反馈与只读查询边界保留。状态模型的行为切换在 R4 完成，R3 不声称新任务流程已上线

ponytail-review：已移除 Wiki 废弃全量导出入口，保持现有 VitePress/Bun 与单一渲染器；没有新增站点框架、文档副本台账或游戏系统。Lean already. Ship.

未修改 Rust、运行资产或玩法。本轮纯文档迁移未新跑游戏录像、Windows或手柄体验，均不宣称通过；R2 已验证现有资产脚本与导出。R4 接续归档旧状态源、任务卡和全部路由/CI切换
