# 小店公共区真实往返验收

## 结论与范围

TASK-044 技术实验通过：V-04 源房间与开口生成同一真实场景的地板、内墙、门洞、天花及碰撞，接待与陈列、预约洽谈、主题陈列三房可走入并返回，后场与楼上仍实体封闭。HUD 从脚点、多边形及楼层高度判定室名，站在共用门线上保留稳定名称；室内显示地点／房间两行

本次仅验收可进入灰盒与三种基础材质，不代表 README 图中的完整画质或 G1／G2 放行。用户已明确 README 预览的完成度为实机目标，长期准则写入美术权威页，TASK-045 保持 ready。TASK-043 既有控制／画质功能一并进入本次提交，主观体验和原生显示设备验证仍沿用其 review 范围

## 输入与复现

基础提交及完整文件、二进制 SHA-256 见 inputs.json；每次运行的命令、脚本 hash、断言与关键状态见 runs.json。Bevy 0.19.1、Rust 1.98.1、Bun 1.4.2；Linux / AMD Ryzen 7 5700X / RX 6650 XT / RADV Mesa 26.2.3 / Vulkan。未更改地图主数据、稳定 ID、依赖或正式角色／剧情

1280×720、30Hz 固定步、840帧共28秒，seed=0；当前路线无随机行为，不宣称跨 GPU 像素一致。独立隔离默认画质为 MSAA4、SSAO off、Bloom off、动态阴影 on、EV100 9.7、55° FOV；未读取或修改真实用户设置。最终从本地预览包加载资源，未依赖开发目录的运行资产路径

在仓库根执行，命令可直接用于 fish；输出必须选新目录

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-shop-interior.json --output output/capture/shop-repeat
python3 tools/capture.py --binary game/target/debug/n-side --script todo/evidence/TASK-044/r1/wrong-room.json --output output/capture/shop-wrong-room --no-video
```

第二条 capture 故意要求仍在街上的人物位于主题陈列，应退出1；不得以改写人物位置让它通过

本次包由 `bun tools/package-preview.ts --binary game/target/debug/n-side --viewer game/target/debug/map_viewer --output 'output/packages/N-SIDE Shop Preview 2026-09-29' --strip` 生成，最终录制使用包内二进制，并将 `--project-root` 指向该包。可运行下列本地预览入口后选择进入街区，从出生点走入正前方店门，依次向右沿走廊找到两间公共房，原路返回

```fish
'./output/packages/N-SIDE Shop Preview 2026-09-29/run-walk-preview.sh'
```

## 实际检查

| 检查 | 结果与证据 |
| --- | --- |
| 最初 shop 窄测 | FAIL，残留展示门板挡住真实入口；shop-tests.log 保留失败，修复后真实走路条件未降低 |
| 完整 Rust 检查 | PASS，95库测试 + 3 Viewer测试；rust-tests-final.log。restart_worker 标记 ignored 但由跨进程偏好测试显式调用 |
| Capture 类型化快照窄测 | PASS，12项；capture-tests-final.log，去掉 GraphicsEvidence 转动态JSON再读取字段的中间步骤，保持输出字段 |
| 门线 HUD 边界 | PASS，room-boundary-test.log；共用门线使用包含边界的几何判断 |
| 室内穿插回归 | 先 FAIL 后 PASS；shop-shell-before.log、shop-shell-after.log、shop-shell-after-r2.log，基础最高从29.675降到28m，侵入公共房的装饰顶点54→12→0 |
| 构建与格式 | PASS，build-final.log 及 cargo fmt --check |
| 真实进店录制 | PASS，最终840帧完整落盘，11项检查全部通过；零复位，三间房、暂停、原地继续与返回街道状态均正确 |
| 代表性失败录制 | EXPECTED FAIL，60帧，wrong_room_must_fail返回false，native/wrapper均退出1；原日志与state保留 |
| 本地包 | PASS，106文件创建与哈希检查，最终整段录制从该包运行；package-build.log、package-check.log |
| 资产导出 | PASS，20个环境文件28101642字节，10个场景图形；environment-check.log、scene-assets-check.log |
| TypeScript与打包工具 | PASS，types.log、package-tests.log（4项） |
| 文档与Wiki | PASS，docs-check-final.log，docs-build-final.log（地图54项；player95页、dev159页） |
| 看板与只读校验 | PASS，tasks-check.log；TASK-044完成依据本记录，TASK-045仅准备好未开始 |

## 画面自查

首轮机器断言通过，但画面未通过：frame200 室名出现孤字换行，frame304／505 墙脚出现三角斜面，墙内有外部窗框背面。实际几何检查确认不是漏裁室外地形，而是建筑基础切面高于室内完成面并与墙共面，窗台与层间构件也向房间侵入

修复复用同一源几何：小店基础在真实地面交点保留完成面以下部分，完成面以上由原建筑外壳承担；V-04窗台、窗框、层间带外移6.5cm，线脚端部缩入以免穿进天井拐角。渲染与碰撞一起变化，地形高度、人物路径、机位和材质保持原值；最终脚本只额外声明默认画质断言

最终 `self-audit` 实际查看全部15张关键帧接触表，原分辨率200、304、505，连续帧187–189和501–502。穿插与窗框背面消失，室名两行可读；进门、暂停、继续和返回的画面连续，室内地面、墙顶与门框能区分，门边镜头会按真实碰撞收近。视频已由FFmpeg编码，但本次读取的是关键帧及连续帧，不宣称完整视频观影或隔离盲测

仍是空室灰盒：室内环境光偏均匀、缺少家具陈列和有归属的灯具，窗内纵深、正式角色动画、植被与夜景均未达到概念预览。三种材质复用既有 CC0 混凝土贴图与纯色漆面，不新增贴图、来源授权或付费服务，也未盲目提高全局曝光

## 限制与下一动作

NOT RUN：作者实际操作与镜头舒适度、Windows实机、物理手柄及原生桌面锁鼠／显示效果，本次离屏录制不能替代这些证据；没有新增任务、NPC、潜梦、存档进度或私人住家玩法。日志仍有 Bevy ICU4X 中文分词模型提示，本次实际文字可见，未据此宣称所有中文排版已验收

下一项为 TASK-045 小店—采购街材质与光影对照样板，先固定真实机位和当前基线，再做墙地材质、店窗纵深与灯具。遵照用户“下个里程碑提交停下”，本次提交后不启动该任务

## 审查与清理

ponytail-review 已移除图形证据中间JSON转换，最终结果为 `Lean already. Ship.`；正确性审查另发现并修复门线HUD边界。提交日志仅去除行尾空白与尾部空行，诊断和结果内容保留。AGENTS.md与运行验证页的用户原有修改保持原样，不混入本次提交

画面已实际查看，按仓库约定清理本轮两个成功录制、一个失败录制的原始PNG、关键帧、接触表与视频，详见cleanup.json；保留脚本、日志、state/run JSON、hash与文字结论。独立预览包保留用于实际操作
