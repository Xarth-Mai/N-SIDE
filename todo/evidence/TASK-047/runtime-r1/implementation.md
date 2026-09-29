# CHR-001 真实控制器角色预览

本轮将灰阶造型研究 GLB 接到现有移动控制器，未确认的外观通过显式 `--character-preview` 选项隔离。它不代表角色美术、动作手感或首委托已验收

## 接入契约

- 运行资产 `game/assets/characters/CHR-001/yao-grey-study.glb`，场景名 `Scene`，动作名 `Idle` / `Walk` / `Run`
- 米制 Y-up、+Z 前向、脚底 Y=0，根运动保持 in-place；世界位置与碰撞只由 `PlayerState::step` 提交
- `character::install(app, enabled)` 仅在显式开启时安装；模型附在脚底视觉 root，原胶囊保留至完整依赖、场景和动画目标实际就绪
- 失败保留胶囊并记录错误；加载或实例化超过 30s 报错。截图工具需将 `CharacterStatus.ready/error` 纳入真实运行门槛，不把 fallback 当成功
- 实际碰撞后水平速度选择动作：停止 Idle、行走 Walk、速度高于 4.2m/s Run；方向由实际位移决定，被完全阻挡时不会因按键继续 Run
- 暂停、失焦和阻塞世界的观察界面使用与控制器相同的 active 条件停住动画；恢复继续实际播放时间
- 退出清理沿现有玩家 root 递归销毁，GLB / AnimationGraph handle 随生命周期释放，重新进入重新验证

## 当前表现边界

灰阶研究使用直接动作切换，尚无设计或制作过的混合过渡。Walk / Run 播放速率按现有控制器 3.2 / 5.6m/s 对应 1 倍速，DCC 步幅与实机足滑仍需实际观察和校准。当前没有 Jump clip，空中使用 Idle 占位，不称为跳跃动画

`CharacterStatus` 记录实际 ready、error、动作名、AnimationPlayer seek_time/elapsed、切换数、暂停、蒙皮数和目标数；读取发生在 Bevy AnimationSystems 之后，不改写动画或世界结果

## 源码依据与验证

API 核对本机 `bevy-0.19.1/examples/animation/animated_mesh.rs`、`bevy_animation-0.19.1` 的 AnimationGraph/AnimationPlayer/AnimationSystems，以及 `bevy_asset-0.19.1` 完整依赖状态

- PASS：rustfmt 完成
- NOT RUN：新增角色模块编译、CPU 测试与 GPU 实机，由主 Agent 串行统一执行并补记实际结果
- NOT RUN：作者外观验收；当前模型仍在按真实观察修订
