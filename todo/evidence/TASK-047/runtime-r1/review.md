# 首个蒙皮人物的真实运行检查

## 交付与边界

CHR-001 的 AST-008 灰阶制作候选通过 `--character-preview` 接入正式 `n-side` 入口，同一控制器、碰撞、镜头与暂停状态驱动角色；默认步行实验仍保留中性代理。候选 GLB 为 `fee9414871bfbc7d82fdd0ad8eac78a8be37cf5c3f65936e9dc7baebf60b0d61`，32 骨、1 蒙皮、34 个实际动画目标，Idle / Walk / Run 由真实解析后的水平速度与着地状态选择

脸颊与颈部连成锥形、五官贴片感、膨胀袖体和衣边锯齿仍需修订，未获作者美术验收；灰阶不确定角色配色。跳跃仍用 Idle 姿态，未制作 Jump、混合过渡、足部 IK 或正式赛璐璐着色。技术接入不表示 README 视觉目标已完成

## 实际命令与结果

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib player_checks_require_real_snapshots -- --nocapture
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-character.json --output output/capture/task047-character-r3
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/character-failure.json --output output/capture/task047-character-negative
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-observation-settings.json --output output/capture/task050-character-large-reentry
```

输出目录须未存在；以上是实际本轮命令。首次误填不存在的 `game/target/debug/game` 被包装器拒绝，改用 Cargo 的实际 `n-side` 后执行成功，原 setup 失败记录留在 `output/capture/task047-character-r1/run.json`

- PASS：全库 CPU 检查 109 passed / 1 ignored，`all-tests.log`；随后只改动画断言，补充冻结、speed=0、区间中段漂移与片段重启负测，`animation-assertions.log` 再次通过，构建 `build-r3.log` 通过
- PASS：540 帧 / 18 秒走跑、跳跃控制与暂停录制，原始 [run](../character-r3/run.json) 和 [state](../character-r3/state.json)；播放持续推进与暂停零漂移的实测值见 [measurements](animation-measurements.json)
- 预期 FAIL：90 帧负向录制中，资产就绪、90 张落盘和有限 Transform 三项通过；要求 19 帧内推进 10 秒的动画断言失败，原生及包装器退出 1，见 [negative](../character-negative/run.json)。这是门禁有效证据，不能算功能失败或将负例写成普通 PASS
- PASS：780 帧 / 26 秒观察大字、菜单暂停、标题清理与重入录制，21 项断言通过，见 [large-reentry](../../TASK-050/large-reentry-r1/run.json)。采样 629 在标题中 skin/targets 清零且 ready=false，679 重入后 skin=1 / targets=34 / ready=true；观察期间 clip_time 与 elapsed 均冻结

独立审查先发现只检查 elapsed 下界会漏掉冻结，再确认 Bevy `elapsed` 即使 speed=0 仍会增加。最终检查结合区间 elapsed 增量、实际 seek_time 有变化、片段与切换计数一致；暂停对每帧同时检查两种时间漂移。它证明这条运行路径的片段有播放与停播，不能自动判断步态自然或足滑大小

## 实际画面观察

root 查看了原始站姿、走路、疾跑与暂停关键帧，并用 139 / 145 / 151 和 189 / 195 / 201 的连续时段接触表核对左右脚与摆臂变化，图像哈希存于各运行的 `observed-images.json`。角色朝向会转到真实运动方向、腿臂发生变化、胶囊已隐藏；观察窗打开与暂停后画面保持。没有把播放计时器代替看图

当前角色仍偏灰塑料感、衣服缺乏硬边赛璐璐层次；从不同步速和抬阶画面仍需专门量测脚滑及脚底穿插。相机和服装比例尚未作者体验验收。Windows、实物手柄、原生鼠标锁定以及最终动画混合为 NOT RUN

## 后续与清理

下一轮先修角色脸颈、衣料与日漫材质，并以真实近景和步态对照复验；同时继续序章三点路线，保持玩法状态来自实际操作。视觉产物查看后按根 AGENTS 的清理约定移除，日志、脚本、状态 JSON、版本哈希与源模型保留，实际清理结果见同目录 cleanup.json
