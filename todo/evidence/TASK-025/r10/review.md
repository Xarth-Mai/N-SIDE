# 第10轮：平台接入与楼梯立板

基线`8a5003a`，最终地图SHA-256为`b5c85885cca75c521f6d1289e0b35c0d7632b85c99c93b0265cb64cc85ed7d7d`。路口与地面平台专项按技术范围完成，人物碰撞、车辆、最终美术和G1仍未验收；输入与二进制对应见[provenance](provenance.json)和各批运行清单

## 修改与范围

地面道路彼此保持[0候选](all-road-width.json)。平台检查补入运行时路面2.5cm抬升后，旧源实际[22处](formal-before-final-checker.json)，本轮[0处](formal-after.json)；原20处报告漏掉邻里后勤平台的两处0.36875m高差，保留校准原因，未放宽0.5m²／0.35m阈值

小店后场、装卸位、邻里服务院、站西庭院、滨水广场和峰顶入口先同高接入，再在边界外爬升。新增工具检查凹形平台、水平踏面与真实路面偏移；静态检查不使用建筑遮罩豁免冲突，架空及归属建筑的平台另验结构净空

10个新增节点、3条新增道路；原有3节点调整（洗衣后巷2处、小店台阶中段平台入口1处），建筑、楼门、地块、树、架空节点、42手工地形点均原样。5处平台轮廓按使用范围调整，具体前后形状、面积和消费者见[逐项源检查](final-source-check.json)：小店基底700→624.11m²、服务边台315→231m²、住宅后步道574→410m²；中段休息台100m²和转弯休息台160m²保留面积，只移到同高且不占梯段的一侧

额外修正小店补货原支路擦过V-F31楼角的0.278m²占地重叠，增加转折并同步3个道路/路径消费者。第一试排引出0.354m路口高差，调整该段坡度后归零；受修改道路完整路幅对所有建筑占地0交叠

长短登高各增加0.281m（约5229.08m／1311.47m），由峰顶平台前新接入节点形成，原路线节点坐标保留。影院缓行长度保持961.69m；唯一最高点仍为摘星台450m。烘焙样点8348→8334，原手工点不动

## 实际检查

平台回归在原源真实FAIL，见[red](red.log)；最终`bun run check:map`包含地图、地形、站前庭院和全路幅共52项PASS。检查器审查发现漏加2.5cm运行偏移，补入单个边界测试，旧源重算22／最终0

| 命令（fish可直接执行） | 结果 |
| --- | --- |
| `bun run check:map` | 52 PASS／0 FAIL |
| `bun tools/check-road-width.ts all` | 0道路候选 |
| `bun tools/check-road-width.ts all --surfaces` | 0平台候选 |
| `bun tools/terrain-shape.ts --check` | 8334样点PASS |
| `bun run check:types` | PASS |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib --bin map_viewer` | 22库＋4Viewer测试PASS |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --locked --all-targets -- -D warnings` | PASS |
| `cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer` | PASS |
| `game/target/debug/map_viewer --project-root . --validate` | PASS |
| `bun run docs:build` | player95页／dev157页PASS |

6修前、13源修后、6立板修后共25次GPU固定机位退出0；机器运行通过与实际看图结果分别记录。原连续录制和立板修后复录都经真实FreeCamera输入，各540帧／18秒／8断言PASS，资源、Transform、转向、释放后持有W均检查。重跑使用新目录：

```sh
python3 tools/capture.py --binary game/target/debug/map_viewer --script todo/evidence/TASK-025/r10/capture-final/script.json --output output/capture/road-platforms
```

原始PNG、连续帧与视频在`output/roads/2026-09-27/r10/`，归档WebP质量80；日志仅清理行尾空白

## 视觉失败、定位与复验

实际查看小店、住宅、邻里、滨水、站前、峰顶目标画面；source修复后原平台截断/楔面消失，但峰顶东梯出现绿灰三角斑，因此原after记录为视觉FAIL，不能拿0候选替代实机结论

[真实mesh射线](mesh/mesh-ray.json)定位到楼梯立板背面。升/降两种方向的首、中、末立板都朝上坡，下面看时被剔除；修复交换立板端点，扩展原真实generate测试，[原FAIL](mesh/riser-red.log)→[修后PASS](mesh/riser-green.log)。没有关闭背面剔除或改材质遮掩

初版临时探针忽略terrain mesh的索引，错误高度数字已经撤回；修正后按真实indices读取，[实际重叠](mesh/mesh-overlap.json)未发现地形高于踏面，不采用该错误诊断。生成网格绕序与实际看图共同裁决原因

最终[东梯](final/inspect-road-platform-summit-east.webp)、[住宅后梯](final/inspect-road-platform-homes-rear.webp)、码头三阶和小店中段梯复看，立板完整，绿灰透空斑消失。邻近非道路切坡与狭窄绿地仍是灰盒大面；不把它们判为精修完成。[第三停步台](final/eye-ascent-3.webp)保留城区/河岸视线，单峰全景保持

连续录制实际查看修前90/91/149/359和最终对应帧，转向中台阶、平台稳定；已编码视频，未整段播放，视觉结论限于实际查看帧。这是读取设计与代码后的self-audit；Windows、人物/车辆实际通行、物理手柄、作者手感与最终美术NOT RUN

本轮复用已有几何生成、polygon工具、任务与capture；检查器和立板差异经复杂度复查：`Lean already. Ship.`
