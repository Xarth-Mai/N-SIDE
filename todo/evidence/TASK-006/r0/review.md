# R0 差异复核

按 prompt-skill-review 检查原版正文的小幅链接适配与来源声明：原专业方法、触发范围、作者和许可保留；未安装兄弟入口直接解析到固定上游文件，没有新增假 Skill 或扩大豁免。适配入口的任务源切换留给 R4，当前不混入新旧流程承诺

按 ponytail-review 检查清单门禁、来源比对、宿主探针与图片边界补丁：复用 Bun YAML、Python 标准库、既有 Pillow 与 Git，无新增服务、框架或依赖。独立来源比对与本地完整性承担不同证据责任，无需合并为仅本地检查

Lean already. Ship.

实际回归：32 Bun 门禁测试、8 Python 来源/宿主测试、4 图片端到端测试通过；当前本地 Skills 与文档检查通过。具体上游对象与真实模型行为结果分别保存在 skills.md、upstream.json、host.md 和 host-cases.json；这些有限用例不构成所有隐式请求可靠性的保证

首次 Git whitespace 检查将新增 .patch 文件的空白上下文行（合法的单个空格前缀）识别为尾随空格；使用限定补丁目录的 .gitattributes 保留统一 diff 语法，其余文件继续执行原检查
