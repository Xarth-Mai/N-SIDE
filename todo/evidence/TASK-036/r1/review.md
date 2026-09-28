# 会话设置与探索观察综合验收

标题和暂停页提供文字100／125%、镜头100／65%，设置实际作用于入口、地点HUD、公共信息面板和键盘／鼠标／右摇杆镜头输入。返回来源菜单恢复焦点；本会话回标题和重入保留选择，未加入磁盘设置保存

## 自动与运行检查

PASS：71项库测试、3项Viewer测试，覆盖旧输入恢复、完整259节点CPU往返、观察及设置；默认features全目标check、viewer全目标Clippy -D warnings、fmt、正式入口build、3项Python capture工具测试。实际命令和输出见tests-final、default-check、clippy-final、build、capture-tools日志

PASS：真实Linux Vulkan／RX6650XT GPU录制4条：walk-settings宽窄屏各600帧／20秒17项检查；walk-observation-settings宽窄屏各780帧／26秒21项检查。进入设置、切换字号与镜头、返回原焦点、进入街区、再次设置、回标题与重新进入均由真实输入触发。65%档一秒镜头实际转角1.040000rad，100%档1.599996rad，不只检查设置变量

综合路径从小店走到门口→打开公共信息→Tab暂停→设置125%／65%→返回原暂停焦点→继续原观察→关闭且持有移动／镜头输入→释放后真实转镜头→模拟手柄开关查看→标题与重入。四条录像所有状态断言、连续帧和视频生成通过，raw在output/settings/2026-09-28/r1，当前源hash见provenance，实际binary/hash/命令见各run

路线驱动读取同一真实镜头倍率，同时调整限速和鼠标逆映射，默认1.0保持旧行为；两倍率CPU检查通过。本轮没有重新录制整个往返，也没有将设置短录制称为全山GPU回归

PASS：文档／Skills检查、52项地图检查、玩家95页与开发158页Wiki构建；任务生成重复无漂移，tasks:check未改文件，见docs、wiki和handoff-checks记录

## 实际看图

原图查看窄屏设置69、HUD149、综合观察379。联系表查看宽屏69、99、109、149、199、229、299、309、349、509、599；综合窄屏189、199、239、259、299、319、329—331、379、389—391、399、469、519、559、629、699。125%长中文换行后保留全文及返回提示，地点标签为右上暂停预留空间，未见覆盖；暂停恢复与关闭的连续帧没有位置跳变，手柄提示与实际输入一致

这是self-audit，未完整播放视频。Windows、实体手柄、真实桌面焦点切换、作者手感与短路线节奏NOT RUN；角色仍是代理胶囊、场景仍需山坡折面与美术精修，设置未持久化，G1整体不据此放行

## 复验与下一步

以下从仓库根执行，capture输出使用新目录

```fish
cargo run --manifest-path game/Cargo.toml --locked --bin n-side -- --project-root . --walk-preview
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-settings.json --output output/capture/settings-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-observation-settings.json --output output/capture/observation-settings-review
bun run tasks:check
```

五项技术交付已分别留证：TASK-032完整上山、TASK-033原路下山与几何修复、TASK-034地点HUD、TASK-035公共查看、TASK-036设置与综合恢复。本轮止于第五项；下一轮优先处理TASK-014作者实际控制反馈与TASK-023已记录近坡精修，再按既有任务依赖继续，未启动新任务或自动批准G1

独立只读复查已处理路线固定倍率和重复getter建议；最终 `Lean already. Ship.`，无剩余实现阻断
