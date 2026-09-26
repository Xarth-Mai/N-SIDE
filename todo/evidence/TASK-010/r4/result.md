# R4 任务唯一源与入口切换

输入为 R3 `a860f07` 加本记录的 input-files.json 所列工作区内容哈希；迁移输入与目标的完整对照见 migration-executed.json。198 个基线文件均有实际落点，61 个旧工作项逐项保留处理方式、原状态与理由；近期重编 7 项游戏任务，远期留成果门槛，不复制原分母

## 实际交付

新源为 todo/tasks 下的任务卡，Bun 原生 YAML 解析。todo/README.md 生成当前看板；roadmap.md 只保留 G0—G4 与本轮技术管线的成果门槛。已接受体验基线写入方向，M0-02 继续为 TASK-012 review，不新增作者批准或游戏完成数

旧 JSON 原字节归档，旧看板、工作日志与已结项城市/Viewer记录移至 archive/legacy；有效街区决策回写 docs/dev/decisions/district-baseline.md。删除旧 Python 进度生成器与其测试，全部活动命令、CI、AGENTS、guide/work-loop/review改读任务卡

## 实际检查

- `bun run tasks:sync` 与 `bun run tasks:check`：PASS，13 项任务；完整 schema、ID、引用/锚点、依赖和证据核对
- `bun run test:tools`：PASS，Python 与 Bun 全部结果见 tests.log，包含否定实验完成、缺失/损坏字段、重复ID、循环、取消依赖、重开与证据路径、check/list只读、sync稳定、空任务源失败
- `bun run check:docs`：PASS，235 Markdown、83登记身份；Skills31项、迁入25项通过
- 仓库活动入口检索未残留旧 roadmap 命令或 demo-progress 编辑源；归档和迁移证据中的旧名称保留历史含义

## 指令契约复核

| 要求 | 接入与静态检查 |
| --- | --- |
| R-STATE 单一状态源 | 三个项目Skill、AGENTS、模板、Bun命令与CI一致读取任务卡 |
| R-SCOPE 指定工作优先 | guide保持只读查询，work-loop只执行授权任务，旧主题从归档续接 |
| R-EVIDENCE 真实完成依据 | review核对输入版本和真实记录，技术完成与作者体验/里程碑放行分开 |
| R-CONTENT 长期事实归属 | 百科事实、开发规格、制作方法与本轮证据分别维护 |
| R-LIMIT 不扩张授权 | 未替换引擎/设定，不安装服务，不启动未来游戏任务，不推送 |

prompt-skill-review：旧六步/JSON字段与新schema的冲突已删除；正常查询、制作、评审及反向百科请求保持各自边界。静态契约检查没有发现未处理的高影响冲突；实际新入口模型行为在R5复验，本记录不提前宣布通过

ponytail-review：任务卡直接由Bun YAML读取，状态规则集中在手册；保留3个职责稳定Skill，仅替换失效接口。旧生成器和重复活动源已退出，没有增加管理平台或另一套数据库。Lean already. Ship.
