# 公共地点观察技术验收

从真实小店出生点步行接近入口，读取同一地图公开的用途、开放与到达信息，F／A查看，Esc／B返回原地；没有新增任务、线索、存档或可进入室内的声明

## 实际检查

PASS：修前66项库测试及3项Viewer测试覆盖调度与输入，见r1/tests-final.log；修正门立面后3项窄测读取完整PreparedScene.parts，覆盖小店真实录制位置、摘星台与20cm中间障碍拒绝，见door-after.log。最终Clippy与构建见同目录日志；文档与Skills检查、玩家95／开发158页构建通过，见docs.log和wiki.log

实际GPU首轮FAIL：真实入口前方门立面令末端5cm余量错误拒绝，F未打开，后续脚本最终在410帧等待标题超时退出1；短诊断与完整场景窄测证明实际命中V-04派生立面，目标剩余约7cm。仅将允许触及目标表面末端厚度改为15cm，水平3m、高差1m、朝向与中间墙体检查保持不变；原始失败记录在r1，未将失败改称通过

PASS：最终宽屏1280×720与窄屏480×720各540帧／18秒、17项检查。远处真实按F无响应，近门口打开，持移动与镜头键被隔离；关闭当帧及旧持键不泄漏；模拟手柄断连暂停、键盘继续恢复相同观察，重新连接后能打开／关闭；返回标题与重入清空观察

预期FAIL：60帧独立负例没有打开面板，却断言observation_open=true，实际退出1且只有该状态期待失败，连续截图和资源检查正常；见negative-run和state摘要；第一次负例停在标题，还触发required_assets_ready失败，保留为negative-title，不用它单独证明观察断言

## 看图与边界

实际查看宽屏189—191、229—231连续帧，289、299、319、419、459；窄屏189、199、299、319、419、459，以及宽窄屏199原图。打开与关闭位置未跳变，查看板遮挡层级稳定，保护暂停与恢复画面对应真实状态；文字未裁切，但窄屏字号偏小，下一项TASK-036接实际字号设置与布局

以上为self-audit，未完整播放视频；摘星台本轮为完整真实网格CPU验证，没有从小店再登顶打开面板的GPU录制。Windows、物理手柄、作者体验NOT RUN，G1整体未放行

## 复验

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-observation.json --output output/capture/observation-review
bun run check:docs
bun run docs:build
```

独立审查提出层级、暂停语义、远距真实按键及完整场景检查缺口，已修复并复验；观察时隐藏原Shell，面板固定Z层并说明Tab／Start暂停。最终简化评审 `Lean already. Ship.`
