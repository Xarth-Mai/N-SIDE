# Windows全可执行目标打包检查

本次仅修改现有Windows工作流与说明，当前Cargo目标为 `n-side` 和启用 `viewer` 特性的 `map_viewer`；步行预览是 `n-side --walk-preview` 参数模式，保留三个启动脚本

用户随后指出上次推送CI未通过，本轮另修复一项全量地形测试的执行预算，保留全部断言与生产实现

## 实现与检查范围

构建保留原MSVC目标、release配置、锁文件及 `--features viewer --bins`。程序清单改从本次 `compiler-artifact` 的bin与executable字段获取，包含fresh产物，去重后复制；缓存旧exe、库与构建脚本不会入包。空清单、缺程序、损坏JSON及Cargo失败均终止打包

三个cmd透传附加参数和程序退出码，Windows工作流上传前检查 `--help`、未知参数失败以及经Viewer启动脚本执行的 `--validate`。原有重复的直接Viewer校验已合并。下载摘要与 `executables.txt` 列出实际程序，摘要同时列出三个入口

解析规则核对了 [Cargo JSON消息契约](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages) 与 [GitHub shell语义](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstepsshell)。Cargo允许过程宏产生额外stdout，按官方建议跳过非 `{` 开头的诊断行；JSON对象解析失败仍报错，不把诊断当产物

## 本地证据

- PASS：`cargo metadata --manifest-path game/Cargo.toml --no-deps --format-version 1 --offline` 核对两个bin目标与viewer前置
- PASS：Bun解析完整workflow YAML，保留手动入口与windows-latest；最初探针误把合法的空workflow_dispatch值当缺字段，改为检查字段是否存在后通过，工作流未因此修改
- PASS：`bun run check:docs`，342 Markdown、92对象ID、31 Skills与25导入记录
- PASS：独立正确性审查；删除重复Viewer校验后，Ponytail复查为 `Lean already. Ship.`
- NOT RUN：本机PowerShell行为探针；用户明确本次无需额外下载运行时重演CI，已停止该验证并删除未执行的临时探针

## 上一次CI失败修复

[原运行36420087864](https://github.com/Xarth-Mai/N-SIDE/actions/runs/36420087864)对应 `e564843968ec782c68509dab73f3d4dec52546c1`，Bun工具测试126通过、1失败；[精简原始日志](previous-ci-failure.log)显示全量地形一致性测试耗时5493.53ms，超过默认5000ms，没有报告断言不匹配

该测试遍历当前全部地形控制点和三角形中心，每个中心通过现有线性查询核对高度。保留完整覆盖，仅以 `node:test` 的单项选项设置30000ms预算，不抬高全局上限，也不把功能回归的耗时当作性能验收

PASS：修前本机定向检查3890.45ms；修后 `bun run test:tools` 的[完整日志](tools-tests.log)记录Python104项通过、Bun127项通过、零失败，该项4113.80ms。独立审查确认只改变单项预算，Ponytail为 `Lean already. Ship.`

## 环境与边界

本机Linux离线Bevy编译尝试因缓存缺少 `ab_glyph v0.2.32` 在下载阶段退出101，未完成编译；本次没有改Rust、地图或运行素材，不将旧构建结果当作本轮重新编译通过

Windows MSVC编译、原生exe运行、cmd参数透传和图形启动本轮均为NOT RUN；工作流已加入命令入口自检，待实际Windows作业运行。本次配置交付依据YAML解析、目标核对、文档检查与代码审查，不据此宣称Windows构建通过

用户已有 `AGENTS.md` 与 `docs/dev/validation/runtime.md` 修改保持原样，不包含在本次提交；本轮不推进玩法路线图或阶段验收

## 收尾

本轮额外PowerShell下载及分段文件位于专用 `/tmp/nside-task042-pwsh`，所有下载进程已结束后清理；离线Cargo临时消息文件一并删除。本轮未生成截图、视频或游戏临时探测程序，未修改既有发布包

Windows后续实际验收使用Actions的 `Build Windows` 手动入口及其上传前检查，核对固定提交与包内清单；本轮仅按用户要求提交推送，不自动触发手动构建
