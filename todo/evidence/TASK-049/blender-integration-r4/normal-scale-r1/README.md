# V-35 灰泥法线强度归因候选

2026-10-01，实际查看 `output/blender-integration-r4/v35-facade/keyframes/frame00049.png`：细黑点集中于新正面白灰泥，原侧墙没有同样密集的点状外观；本记录是已知实现背景下的 self-audit，不是作者品质放行

本页保留修源前的归因与候选阶段记录，下方 NOT RUN 和“正式源未修改”均指当时该子任务范围；后续已完成[正式源修与保持检查](source-repair.md)，两张衍生法线已进入主文件与运行 GLB。入口和总览最终短片及实际连续帧自查均支持保留本次修复，见[R4 画面复验](../visual-review.md)，作者外观验收仍待完成

## 已核对的原因候选

- [V-35 源构建](../../../../../source-assets/buildings/V-35/build.py) 设置 Normal Map Strength 为 `0.25`，实际 GLB 的 Plaster 与 Concrete 都导出 `normalTexture.scale=0.25`；本机 Bevy 0.19.1 的 `bevy_gltf/src/loader/mod.rs:1283` 只读取法线图 handle，注释明确保留 `TODO: handle normal_texture.scale`，实际扰动比源制作意图更强
- 嵌入 JPEG 通过同版本 `Image::from_buffer` / `from_dynamic` 创建，默认只有一层 mip；glTF 的 `minFilter=9987` 只选择过滤方式，不生成 mip 链。GLB 的默认 anisotropy 为 1，`scene.rs` 的 8 只作用于 appearance 外部纹理；已有 `plaster-normal.dds` 和 `plaster-color.dds` 各有 11 层 mip
- V-35 与 V-55 的四张嵌入 JPEG 逐字节相同；灰泥法线、绕序、UV 米制比例与切线生成路径没有发现 V-35 独有错误。Plaster Color 的全图 RGB 范围为 `189–238 / 185–236 / 182–232`，已实际查看的源颜色图没有黑点

以上支持法线强度丢失和远距过滤不足这两个候选，尚不能证明它们分别造成多少画面差异。Root 的首个隔离 capture 只移除 Plaster normalTexture，保持几何、阴影、曝光、分辨率与机位；结果由该实际运行记录，不在本文件预判

## 可复现的强度候选

[bake_normal.py](bake_normal.py) 使用已安装 Pillow 12.3.0，按 glTF 法线语义解码 RGB 为 `[-1,1]`，仅将 `x/y` 乘 `0.25`、保持 `z`，再归一化并量化为无损 RGB PNG；不做 sRGB 转换。接入烘焙图后，Blender Normal Map Strength 与 glTF `normalTexture.scale` 均应为 `1`，避免其他正确实现强度的读取器重复缩放

原 CC0 图保持不变，来源、作者和许可继续沿用 [AST-003 清单](../../../../../source-assets/environment-kit/asset-manifest.json)。候选源图 SHA-256 为 `ea293c335a0b9243f0a825573b81304b8f99e60b2c067e493d0792adf96dfc72`；输出与工具版本见 [bake.json](bake.json)

```sh
python3 todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/bake_normal.py --self-test
python3 todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/bake_normal.py source-assets/environment-kit/materials/Plaster001_1K-JPG_NormalGL.jpg output/v35-normal-scale-r1/Plaster001-NormalGL-scale025.png
```

输出路径必须不存在；本轮候选已生成在上述位置，并已实际打开检查，只有法线扰动减弱，没有重绘纹理。PNG 仍只有一级 mip，因此后续将它替换到隔离 GLB 只检验强度，不能把结果称作 mip 问题已解决

## 候选阶段结果与边界

- PASS：自检覆盖平面、倾斜、镜像、零强度和单位强度的可计算结果；原图前后 hash 相同，输出为 `1024×1024 RGB`，没有新增依赖
- NOT RUN：烘焙候选的 GLB 接入、真实 capture 及与原法线的同机位连续画面对比；正式源、正式运行 GLB 和 `visual.rs` 均未由本子任务修改
- R5 环境反射 patch 暂停，先处理本轮新增画面问题；本子任务未生成截图、视频或临时可执行文件，候选 PNG 按 root 委托保留，待接入判断后统一清理或归入正式资产
