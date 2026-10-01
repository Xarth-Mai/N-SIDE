# V-A08 窗洞与地形只读复核

结论：PASS，现有新立面的27组窗没有在被检位置埋入地形。东面首层北窗在低眼位画面中的下角遮挡来自离墙约4.8m的前景草坡，无需因此抬窗、删窗或抬整栋建筑；这是窗洞接地核对，不是地形和建筑整体视觉已获作者认可

## 实际观察

实际打开 `output/blender-integration-r2/va08-east-host/keyframes/frame00059.png` 与 `output/blender-integration-r2/va08-southeast/keyframes/frame00059.png` 原图。东侧人眼机位中，坡面从画面右侧遮住首层北窗下角；东南抬高机位中，窗台与地面的间距完整可见，窗洞没有沿坡切成斜口。这两幅图是当前真实 Viewer 捕获，未通过离线 Blender 渲染替代

两机位的世界状态眼位转换回地图坐标分别为 `[104.0199966,277.875,31.7465153]` 和 `[98,263,41.7042007]`。本次仅读取 Root 产物，没有改动、重新捕获或删除这些图片；其最终清理由集成负责人统一完成

## 机器证据

直接链接本机已编译 `n_side` 库，调用真实 `Map::load`、`Ground::new` 和 `geometry::generate`，没有运行 Cargo、GPU 或 Blender。使用源工程生成脚本中的27组窗坐标，逐窗取左边、1/4、中央、3/4和右边，并在玻璃外面0.045m、窗框外面0.225m和窗台外缘0.30m三个深度查询地形，共405组位置。窗高下沿和上沿同一水平坐标，因此最严条件是下沿高于地面；窗玻璃实际下沿还比开口下沿高0.06m

- 全部405组 `opening_low - Ground` 均为正，最低0.90m，首层7窗均满足
- 353组同时取得 `/terrain` 实际导出网格的垂直插值；52组落在道路等地形裁剪范围，没有terrain三角，不把缺三角当作地形低于窗的独立证明，仍保留对应 Ground 高程
- 东面首层北窗 `VA08_E_Window_25` 的玻璃面与窗框／窗台处15点均有实际terrain三角；其范围为 north280.05…283.05m，地形均30.021515m，开口下沿30.921515m、上沿32.771515m
- 对低眼位至该窗左／中／右三列上、下沿作各999点视线采样，右下角射线在地图 `[92.853474,281.492325]` 高31.169840m时，前景地形高31.932315m，遮挡余量0.762475m；这处地形距玻璃平面4.808474m，说明遮挡发生在窗前
- 同一低眼位至右上角的最大地形超高为−0.528860m，至左／中下沿也为负；东南抬高机位的六条视线均未被地形截断，与两幅实机图的可见范围一致

视线采样只裁决该窗与地形的遮挡关系，不评估树木、其他建筑、阴影、材质和整体构图，也不承诺未采样区间绝对无遮挡。窗口离地结果基于真实 Ground 和实际terrain网格，不仅依据画面推测

原始数值见 [405点采样](ground-review-samples.csv)、[视线采样](ground-review-rays.csv)，最小复现源码见 [Rust probe](ground-review-probe.rs)

```fish
rustc --edition=2024 -O todo/evidence/TASK-049/va08-facade-r1/ground-review-probe.rs --extern n_side=game/target/debug/deps/libn_side-bd339b1f3f0d39e4.rlib -L dependency=game/target/debug/deps -o /tmp/nside-va08-ground-review
/tmp/nside-va08-ground-review > /tmp/nside-va08-ground-review.csv 2> /tmp/nside-va08-ground-rays.csv
```

实际编译和执行 exit0，现有rlib标识只针对本次构建，重编后应选择新构建对应库。探测程序约1.93GB，完成后已清理临时二进制及/tmp中的本次源与CSV副本；复现源码和日志保留在本证据目录

## 输入版本

| 文件 | SHA-256 |
| --- | --- |
| `source-assets/district-map/district.json` | `aecbdb41adc3c4fac4cf8fb36884d80708b9b776086c01a8f00e4ef6210f8779` |
| `source-assets/buildings/V-A08/facade-build.py` | `b130d2f7e2f8e417c0e5eb7b46e990b10dd5696460aa263da6ba32dc8aa2454f` |
| `source-assets/buildings/V-A08/facade.blend` | `a31ee285b7bd832019db0421f6ecd0d8c9b26dae4a702dcf910e5a17d5361ee1` |
| `game/assets/environment/buildings/v-a08-facade.glb` | `aa2b9186b64d07a23ada84fa83f66fe0042d1dc5c8969d0b715638835e254808` |
| `game/target/debug/deps/libn_side-bd339b1f3f0d39e4.rlib` | `929e007d81e1b54813f37e98221ec38eaed8d3e3b7fc30577a104f82b5fa70fe` |
| `output/blender-integration-r2/va08-east-host/keyframes/frame00059.png` | `fbf2785dcabdb3a2ddf7ef6754f3e4bfee6c4bd10d1b8b138ca4cbcccf986463` |
| `output/blender-integration-r2/va08-southeast/keyframes/frame00059.png` | `7cd42c282e563d1911c48d549d75ca88d937698cf07037857d6b1fa16625b8b3` |
