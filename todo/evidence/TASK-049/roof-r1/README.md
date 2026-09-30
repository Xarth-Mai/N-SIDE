# V-A08 Blender 深檐样板 r1

2026-10-01，本轮根据真实 V-A08 主数据制作独立屋檐，保留地图、建筑主体、楼层、入口、正常游戏系统；接入由根 Agent 处理，本资产制作子任务没有修改 scene、map、appearance、任务卡或提交

## 输入与产物

主数据 `buildings[V-A08]` 为18×13m、底高30.021515m、楼高10.4m、三个入口，屋顶中心 `[79,277.5,40.421515]`。原四边压顶截面0.3×0.16m；本次新檐外挑0.45m、下降0.35m，连续折面包含低槽和回升外唇

产物为 [可编辑 Blender 主文件](../../../../source-assets/buildings/V-A08/roof-eaves.blend)、[建模／导出脚本](../../../../source-assets/buildings/V-A08/build.py)、[源与GLB检查脚本](../../../../source-assets/buildings/V-A08/check.py)、[资产登记](../../../../source-assets/buildings/V-A08/asset-manifest.json)和 [GLB](../../../../game/assets/environment/buildings/v-a08-roof-eaves.glb)。使用本机 Blender 4.5.14 LTS，无外部模型素材、服务调用或新增费用

## 实际执行

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --factory-startup --python source-assets/buildings/V-A08/build.py -- --rebuild --render-output output/roof-r1
output/tools/blender-4.5.14-linux-x64/blender -b --factory-startup --python-exit-code 1 --python source-assets/buildings/V-A08/check.py -- --output todo/evidence/TASK-049/roof-r1/geometry.json
output/tools/blender-4.5.14-linux-x64/blender -b --factory-startup --python-exit-code 1 --python source-assets/buildings/V-A08/build.py
```

初次构建、24 sample CPU Cycles 三个960×720预览与检查均成功。首次构建命令未设 `--python-exit-code`，因此结果没有只凭进程0判定，而是核对实际文件、导出哈希及独立检查；后续命令均携带错误退出码参数

| 检查 | 结果与证据 |
| --- | --- |
| 可编辑源 | PASS：仅一个静态网格，28顶点／28四边面，米制、identity变换、闭合manifold、正有向体积，无预览物体或未应用modifier |
| 实际GLB | PASS：Scene0，一个根、三个材质primitive、56三角，无skin／animation／texture／extension |
| 几何与法线 | PASS：实际accessor有限、索引有效、三角非退化、单位法线与winding一致、导出轴和尺寸对应目标包络 |
| 米制UV | PASS：逐三角面积与逐边长度均核对；UV／3D边長比0.9999907162…1.0000151642，避免只用面积掩盖单轴拉伸 |
| 材质 | PASS：roof／metal／trim 的sRGB颜色先转线性，与主文件的颜色／roughness／metallic逐项对比；本轮appearance快照哈希写入源文件及报告 |
| 主文件复用 | PASS：默认不重建，仅从现有.blend导出的GLB与首次导出逐字节相同，SHA-256 `f52eb7559f4eb7e65f274f6b03dfaed9efbe0472e9b67a4348da9e4734c9b952` |
| 数值报告 | [geometry.json](geometry.json)，构建日志 [blender-build.log](blender-build.log)，检查日志 [blender-check.log](blender-check.log)，主文件再导出日志 [master-export.log](master-export.log) |
| 游戏接入 | NOT RUN：本子任务没有运行Bevy或修改场景；接入端另外记录真实画面、加载与街行恢复 |

## 实际看图：Blender 自查

已实际查看 `overall.png`、`gutter-corner.png`、`street-under-eave.png`。整体可见连续檐口，近景能区分下坡屋面、浅凹金属槽与外唇；四角斜接闭合，街面仰视显示檐下厚度与投影，没有可见断角和反面。当前颜色保持既有roof／trim／metal，未以临时亮色伪装接入效果

三个画面的楼体是按18×13×10.4m放置的临时方盒，不含真实V-A08窗饰、雨水管、邻楼与游戏光照；它只能证明新模型轮廓和硬边，不能证明实际游戏接缝、材质或作者审美通过。没有新增整楼完成状态

## 接入边界与独立复核

只替换 `V-A08 && !court` 外轮廓的四个 `Roof coping` add_box，不删除其他trim、窗饰、楼层腰线和雨水管。新GLB不带碰撞；移除旧压顶会使结构碰撞减少48个三角，这与上一轮阶梯扶手“不增碰撞”是不同修改，接入方需按真实街行路线复验

东侧既有横雨槽与新檐下方有约7cm竖向接合，当前未制作完整水力排水连接；北邻V-A09实体距离新包络0.55m，上层窗檐距新檐底0.21m。根Agent已收到这些数值与接入边界，不能记录零穿插或碰撞计数不变

只读复核指出初版检查仅验证UV面积，无法排除U放大／V缩小。本轮补齐逐边长度检查并重新执行，数值通过。修复后复核结论为 `Lean already. Ship.`，无剩余必须修复项。复核未参与制作、未运行Blender或查看预览，不将其写成隔离视觉评审

## 清理

三幅预览已完成实际查看与哈希登记，见 [previews.json](previews.json)；收尾删除本轮 `output/roof-r1/*.png` 并移除空目录，保留源工程、正式GLB、脚本、日志和数值报告。复现可使用上述命令，不保留连续帧或临时可执行文件
