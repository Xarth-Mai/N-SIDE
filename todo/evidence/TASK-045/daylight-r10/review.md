# 白天光照最小 A/B

2026-09-30，本轮只在 `output/daylight-r10/` 创建两套隔离配置与地图快照，复用已有 Viewer 二进制和正式资产，只读检查 `daylight.json`、`appearance.json`、真实 `world::visual` / `world::scene` 路径及 r9 证据，未修改正式材质、地图、人物或渲染代码，未编译

实际查看 README 的小店街概念图：目标有可辨认的深青雨棚、暖墙与深门窗、檐下和侧墙阴影；当前 Viewer 的同类色块较浅且店面构件较少。本轮仅检验曝光和环境补光是否造成部分发灰，不声称复现《绝区零》或概念图品质

| 参数 | A 正式基线 | B 隔离候选 |
| --- | --- | --- |
| `exposure_ev100` | 9.7 | 10.2 |
| `environment_intensity` | 1100 | 650 |

曝光降低半档、六面环境补光降低约 41%；太阳照度 18000、方向、材质 sRGB 色值、纹理、天空亮度、ambient 80、TonyMcMapface 及其他设置均相同。材质在真实加载路径以 sRGB 转为 `StandardMaterial.base_color` 并乘颜色纹理，阴影来自真实太阳与几何，不使用图片后期调色

固定 Viewer `shop`、1280×720、30 FPS、seed 0、60 帧、TAA+SSAO，无移动和转向事件，两组分别通过 4 个原生断言：资源就绪、60 帧完整、Transform 有限、固定镜头。逐帧相机位置、旋转和资源计数完全一致，位置 `[110,33,-238]`，资源为 5440 entities / 3383 meshes / 49 images / 52 materials，参数与输入 hash 见 [inputs.json](inputs.json)、[A 配置](daylight-A.json)、[B 配置](daylight-B.json)、[固定脚本](shop-fixed.json)

实际查看两组第 59 帧：B 的檐下、侧墙和远处背阴楼面更沉，前墙与窗框层次更清楚；亮墙稍暖，雨棚稍深。招牌仍呈浅青、草坡仍偏黄浅，窗户和陈列的材质区分及城市密度没有改变。子代理自查初判为小幅改善候选，整体保持 `needs_revision`；仅此单镜头，未测室内、人物、夜景或整条真实步行路线

Root 随后实际查看两张原图，明确决定本轮不采用 B、保留 A 正式基线：虽然采样亮暗比增强，主要感受仍是整幅变暗，缺少店面材质自身的层次，淡青招牌与大面积统一墙色问题仍在。本轮记为未采用的负实验；下一步针对局部材料和可见构件，不继续全局降低曝光，这不是作者的最终美术验收

同位置截图样本中，受光墙／檐下墙的显示亮度比由 2.77 增至 4.67，雨棚 HSV 饱和度由 0.306 增至 0.332；这些是色调映射后的截图采样，不能解释为物理照度或独立美术验收，矩形范围与数值见 [comparison.json](comparison.json)

两组运行分别见 [A 运行](A/run.json)、[A 状态](A/state.json)、[B 运行](B/run.json)、[B 状态](B/state.json)，实际命令为：

```sh
python3 tools/capture.py --script todo/evidence/TASK-045/daylight-r10/shop-fixed.json --output output/daylight-r10/A/capture --project-root output/daylight-r10/A/root --binary game/target/debug/map_viewer --aa taa-ssao --no-video
python3 tools/capture.py --script todo/evidence/TASK-045/daylight-r10/shop-fixed.json --output output/daylight-r10/B/capture --project-root output/daylight-r10/B/root --binary game/target/debug/map_viewer --aa taa-ssao --no-video
```

运行需要本机 GPU，capture 目录必须为新目录；隔离 root 中保留本轮地图和 JSON 快照，`game/assets` 链接到正式资产，只读使用。两次均在已授权的原生 GPU 环境执行，日志中的既有手柄映射警告不影响此次无输入固定相机检查

两组使用同一已构建 Viewer，二进制 hash 相同；检查期间其他并行工作修改了 `scene.rs`，本轮未编辑或重编译，因此画面不用于证明该并行代码变更。正式 daylight / appearance / district map 的前后 hash 均相同，源代码检查与二进制实际运行的边界在 inputs.json 单独记录

两组原图已由子代理与 root 实际查看，root 确认可清理后，临时视觉产物全部删除，具体数量、hash 与占用见 [cleanup.json](cleanup.json)；本轮不生成视频，保留输入参数、脚本、隔离地图快照、运行日志与状态证据
