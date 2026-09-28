# Linux独立室外预览包验收

PASS：从本次构建的游戏与Viewer二进制生成97文件的本地Linux包，在仓库外含空格目录解压，经真实启动脚本完成交互、设置写入和第二进程恢复。本次验收为本机可搬移预览，不是完整Demo、跨发行版兼容或公开发布放行

## 输入与产物

游戏内容来自 `72eaf124e3f990858fb4244289613c1179cae164`，打包选中的运行源无未提交改动；工具最终输入见 [inputs.json](inputs.json)，修订hash为 `15c67733e373c262bf8f95d35668bd2eb8585b9833afb5b4faf2e7d1500c811d`。[真实构建日志](game-build.log)记录 `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins`；使用现有dev优化配置，包副本移除调试信息，没有重编成release或修改开发二进制

压缩包位于 `output/packages/n-side-linux-preview-2026-09-28.tar.gz`，125634463字节，SHA-256为 `4fc3fd8ee5ddbf002f62ec11ba9ddc05731e344806f483ffec4533b850b27530`；同目录提供 `.sha256` 文件，解压约414MiB。大文件、完整PNG序列和视频保留在忽略的output中，本目录保留可追溯日志、摘要与WebP80

[包清单](package-manifest.json)记录输入二进制hash、97个包内文件的hash/大小/权限、源提交和未提交范围；清单证明一致性而非签名或独立编译证明。项目源码、Cargo锁文件、编译使用的tokens/capture样例、MPL全文、既有环境和字体素材声明、Parry声明均保留原路径或明确许可目录

## 实际独立运行

[搬移记录](relocation.json)保留命令、绝对目录、退出码、宿主与产物hash。先从tar解压到 `/tmp/n-side-relocated-nb9sgazg/N-SIDE Linux Preview`，在 `/tmp` 调用游戏/步行帮助和Viewer `--validate`，均退出0；Viewer实际读取该目录下地图、外观、光照和绑定，核对1638节点、904道路、231建筑、82表面、72树木及3302网格

随后将包内 `game/src`、`game/Cargo.toml`、`game/Cargo.lock` 暂时移出运行根目录，连续执行下列三个真实GPU进程。外层Python记录器仍在仓库内运行；子进程工作目录、资源根、启动脚本、二进制与脚本均来自解压包，不调用Rust或Bun编译游戏。实际命令在各run记录中，run的binary_sha256指向启动脚本，原生二进制hash另见搬移记录和包清单

| 路径 | 实际结果 |
| --- | --- |
| observation | 600帧／20秒，13项PASS；实际靠近、观察、空白点击、鼠标返回、关闭持键隔离、移动、模拟手柄重开、暂停恢复、键盘返回 |
| settings-write | 600帧／20秒，17项PASS；真实菜单将文字设为125%、镜头设为65%，保存到包外独立目录并继续室外操作 |
| settings-restore | 240帧／8秒，7项PASS；新进程在标题读取125%／65%，进入同一世界后真实镜头1秒约转动1.04rad，未重置人物 |

三条均由 `run-walk-preview.sh` 启动，原生与wrapper退出0，完整PNG及H.264视频生成通过；运行时资产25项依赖就绪，失败与降级均为0，263个模型实例正常加载。录制为固定30Hz、seed0的输入复现，墙钟录制耗时不作为游戏性能测试，未宣称跨GPU像素一致

设置目录最终确有 `{version:1, large_text:true, slow_camera:true}`。测试后恢复包内源码与Cargo文件，再从外部工作目录执行只读完整性检查，97文件PASS，确认运行没有往包内写入用户设置或临时截图

## 失败边界与工具检查

- PASS：独立包步行脚本接收未知参数，返回1且保留具体参数诊断，见 [invalid-option.log](invalid-option.log)
- PASS：暂移解压包地图后，实际Viewer校验返回1并指出缺少的 `district.json`；恢复地图后包完整性通过，见 [missing-map.log](missing-map.log) 与 [restored-check.log](restored-check.log)
- PASS：打包器4项测试／37断言，覆盖路径空格、argv、非零透传、缺失/篡改/多余文件、链接、权限、损坏清单、缺少许可、只读检查、不覆盖已有输出和strip仅修改副本，见 [package-tests.log](package-tests.log)
- PASS：capture的3项窄测覆盖原生参数/工作目录、丢帧、坏PNG、失败状态和真实进程超时，见 [capture-tests.log](capture-tests.log)；初次误用模块导入路径导致ImportError，按仓库unittest discover入口重跑通过，初始日志保留
- PASS：TypeScript/Vue类型检查、Rust格式检查、文档/任务检查和最终差异空白检查；双站实际构建及指南产物检查见 [TASK-040](../../TASK-040/r1/review.md)，后续没有修改Wiki页面
- PASS：独立审查未发现正确性问题，Ponytail复查为 `Lean already. Ship.`；GPU证据仍由实际运行与图像核对承担

## 画面自查与限制

实际查看observation259—261连续帧、settings-write99、settings-restore49及179。观察面板文字、悬停返回轮廓完整，点击后面板消失且人物/镜头保持；两个不同进程的设置页都显示125%／65%与「已保存」，室外HUD使用大字号。大字号地点名换成两行但内容未裁切，街景仍是现有灰盒品质。图像为self-audit，未完整播放视频，不代签作者体验

运行日志仍包含已有ICU4X中文分词模型提醒；本轮检查的中文文本可读，未因此把提醒当成渲染失败或宣称已修复文本引擎

实际宿主为CachyOS Linux 7.2.6、glibc2.44、AMD RX6650XT／Mesa26.2.3 Vulkan；[动态库解析](dependencies.log)记录Wayland、libm、udev、ALSA、libgcc、libc和libffi等实际依赖。没有卸载宿主开发工具，也未在无开发工具的全新操作系统测试；本轮证明子进程不读取包内源码/构建文件并可从仓库外运行

NOT RUN：Windows、其他Linux发行版、物理手柄、桌面窗口手工交互、完整上山GPU重录、作者/陌生玩家验收。全路线最终地形CPU回归沿用 [TASK-038](../../TASK-038/r1/review.md)，未把20秒小店交互当成全城验证。公开分发所需全部链接crate许可与兼容性审计留在正式发布门槛，本包只保留并验证当前已有项目/素材/Parry声明

## 日常复验入口

在仓库根执行，命令可直接用于fish；使用新的输出目录，解压根与设置目录可自行调整

```fish
bun test tools/tests/package-preview.test.ts
python3 -B -m unittest discover -s tools/tests -p test_capture.py
bun tools/package-preview.ts --binary game/target/debug/n-side --viewer game/target/debug/map_viewer --output 'output/packages/N-SIDE New Preview' --strip
bun tools/package-preview.ts --check 'output/packages/N-SIDE New Preview'
python3 tools/capture.py --binary '/tmp/N-SIDE Linux Preview/run-walk-preview.sh' --project-root '/tmp/N-SIDE Linux Preview' --script '/tmp/N-SIDE Linux Preview/game/capture/walk-observation-pointer.json' --output output/capture/package-new
```

长期命令和边界已回写 `game/README.md` 与 `tools/README.md`。本轮五项技术交付至此结束，G1整体验收、山体整体品质、室内及调查/潜梦仍按各自任务继续，不自动放行或启动下一项
