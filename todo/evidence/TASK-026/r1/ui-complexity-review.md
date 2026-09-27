# UI 工程差异复杂度复核

范围：`game/src/ui.rs`、Viewer 接入、同一 capture 的键盘／模拟手柄输入和状态断言、结构化 UI tokens

按 ponytail-review 检查现有流程复用、原生 UI 组件、抽象与依赖：本轮沿用同一世界加载、相机、截图和检查器；界面使用 Bevy Node、Text、Button、InputFocus、Grid、ScrollPosition，未增加依赖、平行录制器、通用主题框架或假任务系统。小型状态与颜色测试保留，捕获断言只覆盖本轮真实操作

布局、字号、颜色、圆角与滑入时长中已接入的字段由 tokens 提供；`reference_canvas`、`spacing` 是候选规范记录，运行时仍有局部布局尺寸，并非 tokens 的全部字段已被消费。数值未获视觉验收

Lean already. Ship.
