# N:SIDE Skill 前向使用探针

本轮只读分析，不制作两张变体、不修改仓库、不重新运行游戏；方案来自实际资产、代码与画面，未读取 `todo/` 本轮审查记录或现有 `visual-review.md` 的结论

## 两张招牌的最小生产路线

沿用 `AST-004`、`source-assets/district-scene/shop.svg` 与现有导出器即可，不需要新模型服务或全局资产表

1. 以现有 SVG 和已核对一致的 `shop.png` 为母版，保留深青 `#194b48`、暖黄 `#f8d869`、细边框、几何标识与 N:SIDE 名称；现有橱窗中的杯子、文具和低饱和陈列继续作为邻接风格参照
2. 两张候选各只改变一个重点：A 放大店名并简化下行信息，检验路过时的识别；B 保留店名字级、放大左侧图形，检验斜向来路识别；均沿用 1024×128 SVG、DejaVu Sans 与当前固定白天条件，先将候选和预览放 `output/assets/`，不覆盖正式源文件
3. Brief 写入已有 `source-assets/district-scene/README.md`，记录母版路径、两项变化、用途、尺寸、展示镜头、来源与导出参数；确认保留的变体源进入同资产包并沿用 AST-004 的关系记录，不复制上游 manifest 模板建立另一账本
4. 已有导出器只枚举五个店名及各自 display，新增变体文件不会自动导出；若两张都保留为运行资产，最小扩展现有文件名列表并在现有 `appearance.json` 的 `sign_shop.color_texture` 中逐个切换做对照，仍复用 `world::scene`，不加运行时变体系统
5. 执行 `bun tools/export-district-scene.mjs` 与 `--check`，检查两图尺寸、alpha、边缘、原尺寸和半尺寸预览，再用真实白天场景检查；Wiki 发布图另导出质量 80 的 WebP，运行纹理仍为 PNG

有一项现状需要先核对：`shop.svg` 是 8:1，`world::scene` 给 V-04 的招牌全幅 UV 是 9×0.9 m，即 10:1，水平相对拉伸 25%；这是由源码与源地图算出的比例事实，当前低分辨率录像不足以判定体验影响。先在正面近景比较，若影响字形，再在既有招牌网格处修正比例并检查相邻店面，不通过反向挤压 SVG 隐藏问题

现有材质路径为 `appearance.json` → `sign_shop` → `environment/signs/shop.png`，颜色纹理按 sRGB 加载、linear 采样、anisotropy 8；当前牌面为 StandardMaterial，保持同材质和光照完成 A/B，避免同时改变照明与图形造成无法归因

## 录像观察与后续 brief

指定 `output/capture/viewer-tour-final/video.mp4` 存在，无需回退；现场 ffprobe 得到 H.264、640×360、30 fps、540 帧、18 秒。父任务随后告知最终捕获已更新为 `viewer-tour-verified`；已补查其 run/state/report 与视频存在、状态 PASS 且脚本与 final 相同，但以下实际看图范围仍为 final，没有把 verified 冒记为已看图

机器证据：读取本次已有 `run.json`、`state.json` 与 `script.json`，原运行状态 PASS，9 项检查均通过，540 张连续 PNG 仍存在；包括约 12 m 横移、约 0.30 rad 转向、Esc 后停止及恢复控制。Cargo.lock 当前哈希与记录相同，但本轮未重跑，不能据此声称当前所有源码与录制构建一致

视觉观察标记为 `self-audit`：实际打开 `shop.png`、`shop-display.png`、已有 `video-review/contact-sheet.png`，另打开原始连续帧 `frames/frame00147.png` 至 `frame00150.png`（4.90–5.00 秒）；未播放完整 MP4，也未声称覆盖全部时段

| 观察位置 | 直接看到的内容 | 可实施要求与复验 |
| --- | --- | --- |
| 开场至约 5 秒，小店外 | 青绿色雨棚、招牌和窗格形成首层色带，暖灰建筑大面与道路层次明确；招牌正文在 640×360 的这一距离下难读 | 先复用 `map_viewer.rs` 已有 `eye-shop`，道路节点上方 1.7 m、55° FOV，固定光照比较原版与 A/B；招牌以店名和入口可识别为主，副文案仅需近处阅读 |
| 原始 147–150 帧 | 镜头转动结束前后，建筑边缘位置逐帧小幅移动，未见这四帧出现明显位置跳变；边缘细节有像素级变化 | 保留真实 FreeCamera 输入，补一段正面靠近、斜向转头、释放停稳的同路线录制；像素变化不能仅凭这四帧归因为 AA 故障，需要固定光照和原生分辨率连续帧对照 |
| 约 8–10 秒代表帧 | 画面位置基本保持，小店退到画面左侧，宽路面与空地占据主体 | 这段对应输入释放检查，保留其控制测试用途；招牌验收单独复用同 capture 的 `eye-shop` 脚本参数与短停留，不为构图改地图或伪造 camera Transform |
| 17.97 秒代表帧 | 镜头升高，可见屋顶和街区层次，招牌识别进一步减弱 | 高视点继续用于空间概览；人眼高度招牌验收使用现有 eye-shop/eye-corner，不把自由飞行录像当作人物行走或碰撞证据 |

后续只需扩展现有 capture 输入脚本的镜头、分辨率、移动/停留区间与关键帧，用 `python3 tools/capture.py --script <本轮脚本> --output output/capture/<新目录>` 生成证据；原片作基线，新输出保留脚本、状态、日志、关键帧、连续帧与视频，机器断言和看图结论分别写入同一任务记录。验收先看招牌/入口在正面和转角是否可认、边框与文字是否变形、雨棚是否遮挡、停止后是否稳定；再由作者实际操作评价观看是否舒服，Codex self-audit 不计作陌生玩家反馈

## 参考中的扩张建议如何处理

- 新模型服务：本需求已有 SVG 路线；不把上游 Gemini/Grok/Qwen/Tripo/Kimodo 推荐当作已配置能力或调用授权。将来确需生图才读当前环境 `imagegen` Skill，外部付费/授权新增按项目要求确认；本轮未调用任何生成服务
- 全局资产表：沿用 AST-004 资产包 README、对象已有 manifest 与 AST-003 环境包，各自补字段，不创建第二套总表
- 单平面关卡：现有地图包含道路高程、平台、台阶、建筑轮廓，继续使用 `source-assets/district-map/district.json` 与 `[x,y,h] → [x,h,-y]`；参考构图不授权压平地图或复制参考布局
- 另一套录制器：已有 `tools/capture.py`、`map_viewer/capture.rs` 通过输入资源驱动真实 FreeCamera 与 WorldScenePlugin，继续复用；上游 `bevy_capture` 示例不是选型要求，离屏结果也不替代桌面输入、手感和碰撞验收

## 实际使用与未运行项

实际读取 Skills：`nside`、`create-game-assets`（含 art-direction / provenance / raster-pipeline 三份参考）、`nside-reference-analysis`、`bevy-cameras`、`ponytail:ponytail`

另读根 AGENTS、Wiki 首页、项目约定、美术、资产管理、运行验证、game README、Cargo.toml/lock、资产包 README/母版/appearance、导出器、世界资产/场景/视觉和 Viewer/capture 相关源码；未读取本轮 todo 审查记录，未读取现有 visual-review.md

实际运行：文件枚举与 `rg`/`cat`/`sed`/`jq` 只读检查；`ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height,avg_frame_rate,nb_frames:format=duration -of json output/capture/viewer-tour-final/video.mp4`；`bun tools/export-district-scene.mjs --check`（PASS，临时目录复算 10 张现有导出后清理，未写入仓库）；两次 `asset_report.py` 检查 shop 的 1024×128 和 shop-display 的 640×448（PASS）；Python 读取状态、计数图片、核对 lock 哈希、计算牌面比例；`view_image` 打开上述 7 张既有图像

shop PNG 的 RGBA 通道存在，但 alpha 范围为 6–255，完全透明像素为 0；shop-display 的 alpha 全为 255。通道存在不能当作抠图通过，本方案本来就是实底招牌，不要求无背景贴纸

NOT RUN：新变体生产、任何付费/图片生成、修改源图或运行资产、重新录制/GPU 渲染、编译/游戏测试、Wiki 构建、完整视频播放、verified 画面观察、人物碰撞/动画/玩法、作者手感和陌生玩家验收；本轮仅完成 brief，不更新项目已验收数量。这是局部使用探针与 self-audit，不是盲测，也不能推导全部 Skills 行为通过；比例观察只验证该工作流能发现可检查事项，不替代美术方向判断
