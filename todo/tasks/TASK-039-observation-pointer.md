---
id: TASK-039
type: feature
status: done
acceptance:
  role: codex
  revision: 5512fbccb72c143cf75305a97b0adcf34638460c566d5f2641d9c548e86fda3f
  record: todo/evidence/TASK-039/r1/review.md
milestone: G1
depends_on: [TASK-035]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# 公共观察鼠标返回与输入隔离

## 目标与范围

公共观察增加真实可点击返回按钮，沿用键鼠和手柄的关闭及暂停保护

## 验收条件

真实UI布局命中鼠标按钮后关闭；空白点击无效；关闭帧、持键和暂停恢复不泄漏玩法输入

## 当前工作与下一步

第1轮，步骤6/6 完成；1280×720与480×720／125%字号的20秒原生指针录制通过，空白误点反例按预期返回非零退出；鼠标、键盘和模拟手柄重开及暂停恢复继续使用同一门槛

## 结果与证据

[公共观察鼠标返回验收](../evidence/TASK-039/r1/review.md)保留宽窄屏脚本、原始失败断言、连续帧自查与源码hash；仅公共观察返回按钮完成本次原生picking验收，其他Shell鼠标路径、Windows实机及作者体验另验
