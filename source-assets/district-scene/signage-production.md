# 站前与青年商业街标识

本组属于 `AST-004` 的原创经营图形，沿用[地点目录](../../docs/dev/design/catalogs/place-catalog.json)中的正式名称与建筑 ID。源 SVG 保留可编辑文字和独立图形，运行 PNG 由[既有导出器](../../tools/export-district-scene.ts)生成；资产绑定由 `appearance.json` 维护

| 源文件 | 场所与源对象 | 图形构成 | PNG 尺寸 | 米制接入起点 |
| --- | --- | --- | --- | --- |
| [station.svg](station.svg) | Null Station / N站，V-01 | 浅底 N站方章、深底主名称、交通设施的分区秩序与黄色端条 | 1200 × 256 | 7.5 × 1.6 m |
| [byte-beat.svg](byte-beat.svg) | BYTE BEAT，V-35 | 节拍键格、切角边框、分段底线，紫黑与粉色主识别 | 1024 × 256 | 6 × 1.5 m |
| [frame.svg](frame.svg) | FRAME，V-36 | 模型板件式网格、装配角框、刻度，暖白与朱红主识别 | 1024 × 256 | 6 × 1.5 m |
| [playroom.svg](playroom.svg) | PLAYROOM，V-39 | 播放三角、错层标签与成对按键，钴蓝与黄绿色主识别 | 1024 × 256 | 6 × 1.5 m |
| [after9.svg](after9.svg) | AFTER 9，V-79 | 大字与斜切数字区、独立 LIVE HOUSE 类别行，暖黑与橙色主识别 | 1024 × 320 | 8 × 2.5 m |

尺寸为外观接入起点，保留源图宽高比，按真实楼层、窗格、门洞与雨棚间隙调整整块大小。牌面中心为贴附枢轴，厚底盒与支架由建筑构件承担；图片为完全不透明的 sRGB 颜色，按既有招牌材质路径加载。控制远景缩小后的文字闪烁与近景轮廓，字体不通过非等比拉伸填满空位

N站的主要公共入口 `station_entry` 位于 V-01 东立面，地图已有 `front` 位于北面；主识别必须覆盖实际到达方向，不能只把站名放在北面后就假定东入口可读。V-35、V-36、V-39 采用已有北向 `design.front`，AFTER 9 对应北向 `live_entry`；屋顶人物宣传屏与入口经营标识分开承担身份识别

## 来源与导出

图形及排版为 N:SIDE 项目原创，2026-09-30 由 Codex 直接制作 SVG，未使用外部品牌标志、照片、图案或图片生成服务。文字仅包含既定场所名、已使用的日常简称和 AFTER 9 的既有场所类别，没有新增营业时间、票价、活动日期或故事设定

五块标识统一使用 [AST-005 的 Noto Sans SC 2.004](../ui-kit/README.md)，原字体与权利说明继续由该包维护：[版权](../ui-kit/licenses/COPYRIGHT.txt)、[SIL OFL 1.1](../ui-kit/licenses/OFL.txt)。未修改字体文件，SVG 字重使用字体已有的 700—900 范围。导出器通过临时 Fontconfig 加载项目字体，源图无需把文字转成路径，也无需系统安装该字体

```fish
bun tools/export-district-scene.ts
bun tools/export-district-scene.ts --check
```

所有派生文件保存在 `game/assets/environment/signs/`，与源文件同名。`--check` 在临时目录重新导出并逐字节核对，移除临时字体缓存与图片；现有五店的标识和橱窗继续使用各自原来源与配色

## 检查与接入

先核对画布、可编辑名称、字体来源和源—导出一致性，再检查 384 px 宽牌面与明、暗、洋红底对照；此组不需要透明镂空，完整底色参与经营识别。缩小预览检查主名称，LIVE HOUSE 类别行属于近看信息

接入后按对应源入口在真实行走镜头检查尺寸、门窗遮挡、材质受光和主名称辨识，连续转向检查招牌轮廓与小字闪烁；平面预览通过不等于实机验收。本批源图检查与后续场景检查记录在 TASK-049 的证据中
