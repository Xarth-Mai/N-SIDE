# 地图与人物画面第 3 轮集成

用户将目标调整为完整游戏，当前先推进地图、画面、人物建模贴图、骨骼与动作。根目标、项目方向与路线图已同步；本轮同时收口已在制的门前交接保存，不启动新的剧情系统。用户原有 `AGENTS.md`、`docs/dev/validation/runtime.md` 改动保留且不纳入本轮提交

## 实际资产与修复

小店新增两架固定室外陈列，复用五种材质并批处理，4脚支撑、窗口／门口净空、米制UV及0.32m人物碰撞有真实检查。街树新增圆肩交错叶、轴向UV和原创sRGB树皮纹理；曜灰模重做脸颈连接与五官贴面，骨架与三段动画数据保持一致。分别见[门前陈列](../street-details-r1/review.md)、[街树 r2](../vegetation-r2/review.md)和[人物 r2](../../TASK-047/model-r2/review.md)

门前黑缝经过两次成因定位：首次按道路／平台相邻面裁掉不存在土体的挡墙，机器9项PASS但实机199、330帧仍有黑线，视觉FAIL；真实camera投影进一步定位为道路高出平台25mm的重复铺装边。第二版让同标高平台拥有铺装，独立审查发现公园草地也会吃掉步道，随后只让实际铺装平台覆盖道路并增加五条真实花园步道回归。最终GPU同路线450帧、9项检查PASS，root实际查看199、330及邻近帧，原道路边缘细长黑线已消失；平台外沿与门前陈列保持可见，CPU碰撞检查未被放宽。定位与各版本见[铺装记录](../paving-r1/review.md)

## 已执行检查

- PASS：最终全库 `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib` 为118 passed／0 failed／1 ignored；ignored是子进程恢复worker，由宿主测试调用，包括所有target的118项库测与3项Viewer测试，见[最终日志](all-targets-tests.log)
- PASS：`cargo fmt --manifest-path game/Cargo.toml --check`；Python capture包装器3项测试；统一环境导出25文件／45,588,742 bytes
- PASS：`bun run docs:build`，地图55项检查与玩家101页／开发168页真实产物构建；开发站有既有500kB bundle提示，见[构建日志](docs-build.log)
- PASS：模型／街树DCC、GLB、重复导出一致性，独立复核骨架与动画采样不变；源码复杂度审查最终 `Lean already. Ship.`，第一次铺装视觉失败及公园范围问题均保留在记录中
- NOT RUN：Windows本机、原生手柄、作者审美与G2放行；当前不会把资产技术检查计成完整角色或城市品质完成

## 运行与自查

[机器摘要](runtime-summary.json)保留二进制／脚本hash、实际参数、完整检查条件与结果、状态和日志hash。大文件在各次output目录，PNG与视频完成实际查看后按项目约定清理；记录和参数用于重放，不保留临时可视文件。图像是实际引擎输出，固定时间步与seed不保证跨GPU像素一致

街树同机位前后各150帧、6项检查PASS；root查看59与后续62帧，圆肩叶片及纵纹在真实材质中可见，冠层仍稀疏分层，薄片轮廓仍需精修。全部环境变化同时包含铺装和陈列，计数变化与capture吞吐已记录，不将其当作单棵树的GPU成本或原生FPS

小店首轮450帧、9项检查PASS；199与330帧确认两架陈列、标签、承重腿和门口空隙，连续移动没有物件整体跳失；同一图中黑线仍在，已据此重开铺装修复。人物540帧／18秒、26项检查PASS；root查看141—142连续步行、159步行、209疾跑、314回转，仍有灰模服饰与头发简化感，最终美术未通过

正常启动保存、跨进程继续、取消与确认重开、写失败／坏档保留及720×1280的125%中文错误页面分别在[TASK-051](../../TASK-051/r1/review.md)记录，属于在制功能收口，不扩大本轮画面优先范围

## 后续

继续精修脸眉神态、发束层次和衣服轮廓，按街区补齐外观家族、入口、道具与植被；围绕真实太阳、天空、材质和阴影提高昼夜层次。当前仍有大面积简化山坡、少量通用树与建筑背侧细节不足，未达到README预览品质，TASK-045／047／049继续active

工程检查补充：首次Clippy发现Rust1.98对常量chunk和偶数判断的新建议，以及既有ECS参数数量告警；按标准库数组切片、is_multiple_of与SystemParam元组修正，没有新增依赖或框架。首次数组切片改写暴露测试闭包切片类型不匹配，使用局部闭包自动强制转换后再次检查。最终 `cargo clippy --manifest-path game/Cargo.toml --all-targets --features viewer --locked -- -D warnings` PASS，失败日志与[最终通过日志](clippy-pass.log)均保留；调整后全target测试再次通过

本轮视觉清理已完成：确认无游戏、录屏或Blender进程使用文件后，root删除已查看的截图、连续PNG、关键帧与视频，以及本轮24468bb临时素材快照。正式模型、贴图、源工程、参数脚本、日志、状态JSON及文字结论保留；数量、字节与归属见 `todo/evidence/TASK-045/integration-r3/cleanup.json`，街树CPU图另见其 `cleanup.json`
