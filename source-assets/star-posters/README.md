# 城市明星海报

| 项目 | 内容 |
| --- | --- |
| 资产 ID | `AST-007` |
| 所属角色 | [安可 `CHR-036`](../../docs/player/characters/neighbors/anke.md) |
| 主文件 | [anke-portrait.png](anke-portrait.png) 原创画像、[anke-sunny.svg](anke-sunny.svg) 可编辑文字与图形排版 |
| 作者与来源 | Codex 当前内置 `image_gen` 生成原创画像；N:SIDE 原创 SVG 排版与屏幕几何 |
| 来源记录 | [prompt.md](prompt.md)，r2 于 2026-09-30 按镜厅预览图的角色海报画法重绘；生成工具未返回模型版本、种子与实际成本，均记为未知 |
| 许可状态 | 项目生成资产，按所用 OpenAI 服务适用条款管理；未另行引用第三方人物画或商标，不登记为 CC0；字形沿用 [AST-005](../ui-kit/README.md) 的 Noto Sans SC 与原 OFL 许可 |
| 运行文件 | `game/assets/environment/posters/anke-sunny.png` |
| 状态 | `exported`，r2 源图和最终排版已自查，尚无作者接受；r1 画风已被作者否定，历史记录保留；真实 GPU 场景检查与结论归 [TASK-048](../../todo/tasks/TASK-048-star-city-screens.md) |

## 视觉与内容

本海报将安可作为城市内已有知名度的歌手呈现，粉色长发、白色布质发夹、琥珀眼、短外套、麦克风来自正式人物设定，采用年轻、明亮的都市二次元形象。服装配色、镜头与排版为本海报制作选择，未锁定角色最终模型或三视图

实际年龄保持 19 岁，外观按作者规定的实际年龄减 2、约 17 岁组织。r2 参考[镜厅预览图](../area-previews/cinema-music-street.png)左上竖版角色海报的清楚轮廓、概括五官、成组发束、平涂主色和少量硬边阴影；参考对象是海报中的二维角色画法，不把外围写实街景光照用于人物画像，也不复制预览中的匿名角色

r1 的柔光绘画、细碎发丝和半写实渐变已被作者明确否定，原件保存在 [rejected/anke-portrait-r1.png](rejected/anke-portrait-r1.png)，原始生成提示词保存在 [rejected/prompt-r1.md](rejected/prompt-r1.md)。r2 保留既定身份、排版用途与城市安装位置，重新绘制画像；新图的制作自查不等于作者接受，r1 的场景运行证据也不替代 r2 画面复验

主标题为「安可」，辅助罗马字 `ENCORE` 是海报图形字，不新增法定姓名；曲名采用已经存在的成名曲《下一站晴天》。不为尚未完成的新歌、演出、售票或商业合作编造发布日期、票价和成功状态

画像为右侧人物、左侧深色留白，文字用暖白和淡黄粗体构成远距离识别；人物面部与中文主标题是屏幕缩小后的优先信息。完整提示词保存一次，源图保留工具实际生成的原始字节

## 导出与纹理

当前 r2 画像原始尺寸 `1672 × 941`、sRGB、PNG，SHA-256 为 `82f06776ef641489aea19913c82d5d5c6eec258fe05c499c078fdedc16484689`。SVG 是 `1600 × 900` 画布，通过相对路径引用原图，保持可编辑字形。导出器为 librsvg 将图像临时内嵌，解决默认阻止外部图片的问题；临时 SVG、Fontconfig 与字体缓存使用完即清理

```fish
bun source-assets/star-posters/export.mjs
bun source-assets/star-posters/export.mjs --check
python3 -B .agents/skills/create-game-assets/scripts/asset_report.py game/assets/environment/posters/anke-sunny.png --expect-size 1600x900 --json
```

导出依赖已有 Bun、ImageMagick 和项目 Noto Sans SC 源字体，不安装新系统字体。`--check` 在临时目录重算并比较运行文件字节，文件漂移会非零退出；图像尺寸检查不能代替图像内容和实际显示检查

## 场景安装

`world::scene::star_screens` 只在两个现有公共建筑的北侧屋顶安装一面海报屏，屏幕中心平面位于北边界向内 `1 m`，底缘距源屋顶 `0.9 m`。黑色背箱与金属双支架使用现有材质，脚座落于屋顶，没有在门口、步行道路或住宅窗前放置支柱。屋顶和建筑稳定 ID 继续来自 `district.json`

| 源对象 | 展示尺寸 | 屏面中心，地图 `[x, north, elevation]` | 公共观察位置 |
| --- | --- | --- | --- |
| `V-01` Null Station | `10 × 5.625 m` | `[-30, 25, 18.7125]` | `square` 站前广场，向西南上看 |
| `V-79` AFTER 9 | `8 × 4.5 m` | `[734, 38, 23.15]` | `live_meeting` 北侧会合庭，向南上看 |

屏面法线朝地图北向 `[0, 1]`，图形使用单位 UV、背面剔除，源图保持 `16:9`。`poster_anke` 通过现有 `unlit` 标准材质呈现屏幕图形；框体与支架受光。本实现没有对周围街道投射屏幕光，白天可读性由实际曝光、观察距离和 GPU 画面核实

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-station.json --output output/capture/poster-station
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-live.json --output output/capture/poster-live
```

两份脚本各 `10 s`、`1280 × 720`、固定 `30 Hz`，从真实公共地图节点初始化观察镜头，用现有 Viewer 的 W/A/D 控制器作小幅接近与侧移，最后解除输入。它们验证真实场景与画面，不将自由相机操作计作角色行走或玩法通过
