# 地表、住宅、街树与人物的第 4 轮集成

基线 `dbe5ad3`，按完整游戏目标继续优先制作地图、画面与人物；6 路子 Agent 分担造型、地表、建筑、植被、视觉复查和运行安排，root 串行编译与 GPU 验证。工作区原有 `AGENTS.md` 与 `docs/dev/validation/runtime.md` 修改未覆盖或纳入提交

## 实际交付

- [自然地面](../terrain-surface-r1/README.md)：取得 ambientCG Ground037 原始 Color／NormalGL，核对完整 ZIP CRC、来源 CC0 许可、原件 hash 与官方约 2.1m 周期，沿既有 DDS 导出和标准 PBR 加载；先看源图再导入。固定坐标调色增加地块变化，陡坡主轴投影修复近退化 UV，地形位置和碰撞保持
- [住宅](../facade-detail-r2/README.md)：样板 8→10 栋，上层中梃、交错护栏／空调、檐口和贴墙落水管；实际测试统计1,415盒构件、16,980碰撞三角，保留入口、邻栋与地面约束，没有增加可进入房间
- [街树](../vegetation-r3/review.md)：叶片1,089→1,452，改善水平分层与冠内空洞；三角面仍11,686，顶点13,099→14,914，GLB增加79,864 bytes，未以三角不变冒充零成本
- [曜模型](../../TASK-047/model-r3/review.md)：眉眼、口线、耳廓、发组、肩袖和衣服叠层局部制作；补侧身与动作检查后修复背腰穿插，保持32骨、原Idle／Walk／Run采样与灰阶图集。32,822三角，仍是needs_revision技术候选，未批准形象或配色

原件／代码与最终运行资产 hash 见 [final-inputs.json](final-inputs.json)，人物主文件重导出与合同逐项比较见其模型记录；环境素材仍由 AST-003 唯一清单维护

## 验证与真实失败

实际执行 `cargo fmt --manifest-path game/Cargo.toml --check`、锁版本全目标 Clippy、工程测试与所有 binary 构建。最终 [全测试](all-tests-final.log)120 library tests 与3 Viewer tests PASS，1个设置子进程工作入口按设计ignored，由父测试调用；[Clippy](clippy-final.log)无warning，[构建](build-final.log)PASS。[world窄测](world-tests-final.log)42项通过，8584控制点保留，16,130地形三角UV投影面积比最小0.586863，高于1/√3

`bun tools/export-environment.ts` 与只读 `--check` 均通过，27文件／56,853,670 bytes；现有静态环境变换测试、街树真实GLB检查通过。一次在街树源文件施工中启动的导出被hash检查拒绝，完成定稿与清单更新后正常导出，没有豁免。随后一次过早启动的街树基线 capture 因地面DDS尚未导出退出1、0帧，错误明确定位两个文件；[失败日志](task045-tree-before-r3/runtime.log)保留，不计作有效基线

首次地面 capture 状态PASS但实际画面FAIL：小店台地与下山陡坡出现拉伸竖条。改为按真实三角面主轴投影后重新构建和录制；近处台地拉伸改善，远处与掠视坡面仍有规律细纹。隔离项目副本仅将terrain.normal_texture置空，其他资源链接同一正式资产，90帧诊断仍有细纹，因此未采用关闭法线。副本与正式状态分开，未将诊断配置写回主资源

## 实际运行与看图

同一 Linux／RX6650XT Vulkan 环境，沿用真实 Viewer、正式游戏和既有输入脚本；[各次摘要](summary.json)保存二进制hash、状态与成本范围，各目录保留命令、完整日志、脚本和状态摘要。11次运行中10次状态PASS，共2880帧；包括基线、失败画面版本、修复复验与诊断，不能全部称为最终品质通过

| 复验 | 帧／时长 | 机器检查 | 实际观察 |
| --- | --- | --- | --- |
| `task045-terrain-final` | 540／18s | 8 PASS | 小店抬头148–149与原版149、失败版149对照；近台地不再拉成长条，山腰重复细纹仍在 |
| `task045-cut-final-r1` | 90／3s | 6 PASS | 下山29、43–44转头，近地面可辨苔土，左侧掠视坡面仍显规律重复 |
| `task045-cut-no-normal-r1` | 90／3s | 6 PASS | 同机位29，条纹仍存在；不是最终配置 |
| `task045-facade-after-r2` | 150／5s | 6 PASS | 与修改前59对照，连续61–62由root查看、60–63由独立Agent查看；窗中梃与立管可辨，无此镜头可见的批量跳位或遮门 |
| `task045-tree-after-r3` | 150／5s | 6 PASS | 59与连续61–62，冠内更连贯、叶面朝向分散、树皮可辨，仍有多边形薄片感 |
| `task047-model-r3-runtime` | 540／18s | 26 PASS | 141–142步行、159停步、209疾跑、314转向；真实模型和动作加载，背腰与袖口未见原锯齿，脸部细节另有DCC近景 |

Viewer住宅与树录制在最终地形UV修订前完成，建筑／树资产之后未变；最终地形另录山体与下山复验，正式人物使用最终二进制和最终角色GLB。所有看图为self-audit，非盲测、作者审美批准或物理手柄实机验收，Windows此次NOT RUN

## 成本与下一步

同机位山体基线资源为4939实体／3342 mesh／41 image／51 material，增加住宅与地表后的记录为4958／3361／43／51；首末稳定。这是混合批次总量，不是单个资产GPU成本。两张DDS合计11,185,064 bytes共享，16个tree_a实例三角仍186,976，顶点增加29,040。capture的帧间隔包括GPU读回与PNG写入，不用其结果宣布原生FPS或性能预算达标

保留局部改善，TASK-045／047／049继续active。下一批按实际问题推进地表平铺抑制、台地挡土与成组院落、林缘，以及人物脸部／眼部材质和头发硬边层次；当前大片简化坡面与外壳、片状树叶、人物通用脸和服饰材质仍不足以达到README目标，不增加游戏完成数

完成实际查看后按项目规则清理本轮视觉产物，保留源文件、正式资产、复现脚本、参数、日志和hash；删除清单见 [cleanup.json](cleanup.json)，制作阶段的CPU清理另见人物与街树记录

最终文档检查427 Markdown／102 IDs、31 Skills／25 imports通过；任务生成与只读检查46卡片通过。`bun run docs:build` 的55地图测试、player101页与dev168页均通过，保留既有dev大chunk提示。独立代码与造型复查无新增阻断，ponytail-review结论为 `Lean already. Ship.`。Root已清理2968个本轮视觉／临时归档文件，共2,524,837,247 bytes；capture剩余3,969,580 bytes为日志与状态，CPU制作清理另有两份记录
