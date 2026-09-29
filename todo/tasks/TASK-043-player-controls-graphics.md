---
id: TASK-043
type: feature
status: review
milestone: G1
depends_on: [TASK-037]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/design/systems/input.md
  - docs/dev/validation/runtime.md
  - docs/dev/engineering/graphics-settings.md
---

# 基础跳跃、鼠标锁定与画质设置

## 目标与范围

按用户要求给室外步行入口加入基础跳跃、Shift／鼠标右键疾跑与鼠标锁定，设置页提供画质选项卡并暴露当前渲染路径可实际生效的选项；沿用真实场景、碰撞、输入和偏好保存，不增加游戏路线图范围

## 验收条件

跳跃起落、顶碰撞、禁止空中连跳和疾跑碰撞通过状态检查；暂停、观察、失焦与恢复正确隔离输入和释放鼠标；画质选项真实应用、兼容旧设置并跨启动保留；实际运行检查控制变化与画质页可读性，机器结果、视觉自查与未测平台分别记录

## 当前工作与下一步

第2轮，步骤5/6 体验验收；窗口模式／分辨率的15秒保留与恢复、未确认值保存隔离、`--reset-graphics`启动恢复已接入，本地Linux试玩包已更新。91项库测试、3项Viewer测试、真实GPU录制、跨进程恢复、启动重置及故意失败路径完成；原生窗口表现与作者手感仍待真实桌面反馈，不扩大G1放行范围

## 结果与证据

交付、实际命令、输入版本、机器结果、视觉自查和环境限制见[本轮记录](../evidence/TASK-043/r1/review.md)。启动 `./game/target/debug/n-side --walk-preview`，进入街区后跳跃、分别用Shift与右键疾跑，按M释放／恢复，再暂停进入画质页切换TAA、SSAO和第5页垂直同步；重点确认手感、鼠标是否限制在窗口内及菜单释放。原生窗口表现与作者手感确认前保持review；既有AGENTS.md与运行验证文档的用户修改完整保留

显示确认和本地包见[第2轮记录](../evidence/TASK-043/r2/review.md)。修改窗口模式／分辨率可保留或恢复，15秒后自动回退；包内 `run-walk-preview.sh --reset-graphics` 可恢复默认画质。下一步使用实际图形桌面检查窗口变化、焦点恢复和控制手感，尚未执行Windows或物理手柄验收
