# 调查记录重入时恢复选中焦点

原样板保留了第二条记录内容，却在重开工作台时把焦点设回第一条，再次确认就误切内容。修复从已有 `selected_record` 恢复焦点，其他页面仍从首个控件开始；未增加状态或改动版式

## 复现与检查

旧行为在[回归](test-red.log)真实FAIL（focus实际0、期望1），同一测试修后[PASS](test-green.log)。`cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib ui::tests::menu_flow_keeps_focus_and_sample_information_separate -- --exact` 可直接在fish执行

编译、capture脚本契约、fmt、clippy、文档与Skills校验PASS，日志同目录，归档文本只清理行尾空白；原日志留在output。原600帧输入与15项脚本断言逐项保留，追加重开、返回、选择与再次确认，见[capture保留检查](capture-preservation.json)

```sh
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/ui-signal.json --output output/capture/record-focus
```

本次相同命令实际输出到 `output/ui-signal/2026-09-27/r3-focus/main/`，28秒／840帧／25项状态检查PASS，MP4编码PASS。输入文件、场景源和二进制hash见[provenance](provenance.json)及[run](run.json)。场景包含并行TASK-025尚未提交的地形源，此处只验收UI行为，城市验收独立记录

## 画面自查

实际查看599、659、749、779、780、809帧。659第一次重入与779再次重入均在第二条显示亮色焦点和「当前」，右侧保持空记录；809确认后没有跳回第一条。779/780连续帧没有内容跳变；菜单恢复线索入口。原流程的大字号菜单允许滚动，当前焦点可见，未把裁剪区外标题当作内容丢失

这是读取过设计和源码后的self-audit。未整段播放MP4，视觉结论限于上述帧；鼠标实操、物理手柄、Windows、作者手感NOT RUN。脚本模拟手柄输入继续经过现有真实UI路径，没有正式线索或任务状态接入

最小实现复用已存在选择状态与录制器，没有新组件、依赖或框架。复杂度复查：`Lean already. Ship.`
