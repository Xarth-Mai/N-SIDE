---
id: TASK-028
type: feature
status: done
milestone: G1
depends_on: []
acceptance:
  role: codex
  revision: e519ba672aa7f7eec43a7b83fef0511e21c3ba3bb9776aeda7d351474eacb4d5
  record: todo/evidence/TASK-028/r1/review.md
specs:
  - docs/dev/engineering/index.md
  - docs/dev/design/systems/input.md
  - docs/dev/validation/runtime.md
---

# 正式入口与城市场景进出生命周期

## 目标与范围

将默认游戏入口接到既有真实世界运行时，完成标题、异步加载、固定镜头街区预览、返回与再次进入；复用现有城市主数据、资产、白天成像、UI tokens 与同一 capture 实现。作为人物尺度实验的运行基础，不代替 TASK-014 的角色、碰撞与跟随镜头验收

## 验收条件

- 正式入口经真实键鼠或脚本手柄输入进入城市，所有必需资产及模型实例化完成后才进入世界状态
- 返回标题后移除场景实体与加载状态，再次进入的源对象数量一致；取消加载不使旧结果重新进入场景
- 真实加载失败有诊断与恢复入口；capture 成功与代表性失败路径均保留明确退出码
- 实际查看标题、加载、街区与重入的连续画面；原 Viewer 和街区信号样板仍可使用同一 capture
- 默认无 viewer 特性的编译、适用测试与文档检查通过；实物手柄、Windows 与人物游玩独立记录

## 当前工作与下一步

第1轮，步骤6/6 完成；正式入口540帧／18秒／11项机器检查通过，键盘和脚本手柄两次进入各4572个场景实体，返回归零，加载取消有效。缺地图与不可能断言均实际返回1，原Viewer UI的840帧／25项回归通过；实际看图修正提示换行后复录，未代签人物游玩与物理设备验收

## 结果与证据

[实际检查与画面自查](../evidence/TASK-028/r1/review.md)保留命令、状态、失败探针、输入hash与关键帧；原始连续截图和视频位于 `output/game-entry/2026-09-28/r1/`。后续可复用入口做中性代理人物实验，TASK-014与G1整体尚未验收
