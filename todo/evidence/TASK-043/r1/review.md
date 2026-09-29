# 基础操作与画质设置交付记录

## 输入与范围

基线为 main@8d24201a98f11f07c3b74bfeef7b528ab9560664，核对并接续本任务已有未提交实现；最终文件 hash、锁文件和新编译二进制见 inputs.json，运行命令、脚本与二进制 hash、逐项断言见 runs.json。AGENTS.md 与运行验证文档原有用户修改保留，本轮未提交、未推送

正式步行入口新增空格／手柄西键跳跃，左／右 Shift、鼠标右键及手柄左摇杆按下疾跑；默认锁定鼠标，M切换，解锁时左键拖动转镜头。暂停、观察与失焦释放鼠标并隔离输入，恢复先等待旧输入释放。沿用真实胶囊碰撞、地形、场景与设置文件，没有新增玩法系统或角色动画

设置沿用现有组件和保存逻辑，新增19项画质／显示选项、分页与恢复默认，包括MSAA、FXAA、SMAA、TAA、SSAO、CAS、阴影、Bloom、雾、曝光、色调映射、视距、FOV、VSync、窗口模式和分辨率。开启SSAO时MSAA切为TAA，选择MSAA时关闭SSAO；按设备能力跳过不支持的采样数。FSR未接入，CAS只是锐化；美术灯光数据和材质参数仍由场景管理

## 实际检查

| 检查 | 实际命令或记录 | 结果与范围 |
| --- | --- | --- |
| Rust格式 | `cargo fmt --manifest-path game/Cargo.toml --all -- --check` | PASS |
| Rust测试 | `cargo test --manifest-path game/Cargo.toml --locked --features viewer`，tests-verified.log | PASS：库87项、Viewer3项；1项标为ignored的重启worker由父测试在独立进程调用 |
| 两个可执行入口 | `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins`，build-verified.log | PASS；本机rustc 1.98.1 |
| 跳跃与疾跑 | walk-controls.json，task043-verified-controls | PASS：540帧／18秒，20项检查；1秒步行3.200m，两种疾跑各5.597m；跳跃最高脚点28.946m，空中再按和持续按住不会续跳 |
| 抗锯齿与SSAO切换 | walk-graphics.json，task043-verified-graphics | PASS：810帧／27秒，22项检查；实际相机组件及渲染切换，旧时域组件正确退出 |
| 持久化 | walk-graphics-restore.json，task043-verified-restore | PASS：第二进程恢复实际相机设置；原设置文件保护由CPU测试覆盖 |
| 其他画质选项 | walk-graphics-options.json，task043-options | PASS：同一生产代码的已有650帧运行记录，核对二进制hash相同；窗口三项只证明菜单／组件路径 |
| 125%字体 | walk-graphics-large.json，task043-verified-large | PASS：设置全部页面与返回进入街区，11项检查 |
| 窄画幅 | graphics-portrait.json，task043-verified-portrait | PASS：720×960、125%字号，11项检查，逐页导航 |
| 失败路径 | controls-expected-failure.json，task043-expected-failure | EXPECTED FAIL：故意要求9次跳跃，只有该断言失败，游戏和外层均退出1 |
| 当前新编译版本 | task043-current-build-smoke | PASS：180帧，加载真实城市、恢复画质并打开设置，5项检查 |
| 文档与Skills | `bun run check:docs`，docs-check-verified.log | PASS；最终回写后再校验 |
| Wiki | `bun run docs:build`，wiki-build-verified.log | PASS：player／dev独立构建及发布内容边界 |
| 代码审查 | 独立控制与画质只读审查 | 无阻断缺陷；修正了文档中GPU回退方式表述，复杂度结论为 Lean already. Ship. |

GPU运行使用Linux／Radeon RX 6650 XT／Vulkan；当前构建烟测前的所有复验二进制SHA相同，之后仅增加窗口组件断言，没有修改生产行为。原始参数、状态采样与日志仍在runs.json列出的output目录

## 修复与失败记录

接续实现包含Bevy 0.19.1动态退出TAA的局部修复：引擎保留的背景运动向量pipeline／bind-group在失去MotionVectorPrepass后会引发渲染目标不匹配；仅清理无该组件视图的过期缓存，保留版本依据和CPU回归。初次task043-graphics失败日志保留，后续同路径切换已经通过

大字脚本最初只打开标题设置，未进入世界，因此required_assets_ready诚实返回FAIL；本轮补充真实确认进入街区和等待就绪，未降低生产断言。沙箱内首次GPU启动因无可见适配器失败，改在授权的本机GPU环境执行后通过，两份记录分开保留

## 画面自查

SELF-AUDIT：实际查看控制路线起跳前、空中、落地、疾跑终点与暂停帧；角色保持胶囊占位，跳跃与接地变化可见，没有把基础动作称为正式角色动画。对照19项设置页、TAA／SSAO状态、阴影关闭后返回场景、跨启动设置页，以及1280×720和720×960的125%字号，未见所查看页面裁字、焦点遮挡或返回信息丢失

连续录制由真实输入与逐帧状态检查覆盖，人工观察采用关键帧／连续帧抽查；不是隔离盲测，也不代表作者已认可手感、最终画面或性能预算。日志仍有现有ICU4X中日文分词模型提示，所查看中文正常显示，本轮未更换文本依赖

## 限制与下一步

原生窗口尝试记录在native/run.json：独立Xvfb缺少Radeon Vulkan所需DRI3，呈现模式列表为空，游戏在初始化阶段退出。鼠标实际约束、窗口尺寸／无边框表现及物理VSync时序均NOT RUN；不将此环境失败推断为真实桌面不可用。窗口PresentMode开关与物理分辨率组件由CPU测试验证，刷新和鼠标抓取仍须桌面操作

Windows运行、物理手柄、作者手感均NOT RUN。TASK-043保留review；下一步用实际桌面的步行入口检查跳跃／疾跑、M锁定、暂停释放及画质第5页窗口设置，不新增G1验收或继续其他路线图工作

## 产物清理

所有游戏、Xvfb、FFmpeg及编译进程已结束；查看与结论记录后清理本任务output目录内成功和失败PNG帧、关键帧及MP4，源码、运行日志、参数、状态JSON、设置样例、发布包与既有资料保留。逐目录数量与占用见cleanup.json
