# 地点 HUD 技术验收

实际位置来自district同一数据源的公共到达节点；32m水平和4m高度范围内选最近地点，超出显示Null Site，暂停隐藏，标题与重载清理。显示名称不生成委托线索，代理仍为灰盒

## 检查与画面

PASS：最终宽屏1280×720和窄屏480×720各24秒／720帧，11项检查；真实移动离开小店→未知区→暂停持有输入→继续原路回店→键盘接管→标题清理，全部由正式入口执行，run/script/state摘要见同目录。r1为早期录制，r2补齐最终capture参数分组与挡墙修补后的相同路径；源hash见provenance.json，binary与环境见run

PASS：63项库测试和3项Viewer测试见tests.log，最终Clippy见TASK-033/r1/wall-crossing-clippy.log；文档与Skills检查及玩家95／开发158页构建见r1/docs.log和wiki.log

实际看r2宽窄屏59、149、329、349、649、719帧的联系表，以及两种尺寸149原图。地点从小店到Null Site再回店、设备提示变化、暂停隐藏、标题清理与状态一致；文字未裁切，窄屏的实际字号仍偏小，TASK-036继续做真实125%设置及窄屏重排。看图属于self-audit，未播放完整视频，不等于手柄手感或作者视觉放行

NOT RUN：Windows、物理手柄、作者实际路线体验；G1不据此放行

## 复验

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-places.json --output output/capture/places-review
bun run check:docs
bun run docs:build
```

独立实现审查发现早期hash与后来参数分组不一致，已保留r1并对最终代码重新capture；无剩余阻断，简化审查 `Lean already. Ship.`
