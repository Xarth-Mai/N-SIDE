# TASK-045 地表去重复，第五轮

基线 `b948cab`，本轮目标是减弱真实山坡上连续黄绿条带，保留已成立的草岩坡度过渡、近处米制纹理、法线与真实阴影

## 根因对照

使用已有 `game/capture/forest-overview.json`、同一二进制与冻结的地图、appearance、daylight 和运行资产；`prepare-diagnostics.py` 将这些输入复制至忽略的 `output/assets/terrain-r5-diagnostics/shared`，各候选只替换自身 shader，输入 SHA256 见 `diagnostic-inputs.json`

| 候选 | 运行结果 | 实际画面判断 |
| --- | --- | --- |
| baseline | 90 帧、6 项检查 PASS | 整山宽黄绿平行条带明显 |
| no-normal | 90 帧、6 项检查 PASS | frame29 的宽条带基本不变，法线不是主要来源 |
| solid-color | 90 帧、6 项检查 PASS | frame29 宽条带消失，保留真实坡面明暗和细法线，证明主要变量是色图 |
| rotated-pair | 90 帧、6 项检查 PASS，视觉拒绝 | 同尺度固定 37° 双采样把条带换成均匀斜交叉细格，仍具有高度周期性 |
| local-patches | 90 帧、6 项检查 PASS，视觉拒绝 | 6m 覆盖与正交旋转仍形成横竖拼块，宽条纹能量没有充分打散 |
| local-angles | 90 帧、6 项检查 PASS，已采用 | 2m 覆盖、连续固定角度明显消除上述条带和棋盘，原草岩边界保留 |

root 与本轮实现者分别实际看过以上 frame29，独立审查还看过29/44/89；这里是视觉自查，不是作者或隔离陌生玩家验收

运行日志、状态和参数位于 `output/capture/task045-ground-r5-{baseline,no-normal,solid-color,rotated-pair,local-patches,local-angles}`，这些图片由 root 统一收集、清理，本子任务未删除 root capture

## 源图与 mip

直接检查已登记的 Ground037 JPEG 与实际运行 DDS，没有新素材或服务依赖；`inspect-texture.py` 解析 DDS 的 BGRA8 全部11层 mip，mip0 与已解码 JPEG 像素一致，未发现 DDS 内容损坏

色图线性亮度在64×64层的行均值标准差约0.00841、列均值标准差约0.02056，到4×4仍约0.00470/0.01384；源图较强的列向明暗在缩小后仍存在，细法线的同类变化明显较弱，完整数据和 hash 见 `texture-analysis.json`

这些 CPU 数值描述源图，实际渲染因果由上面的同机位 GPU 对照补证；本轮没有以模糊或降低 anisotropy 隐藏重复

## 已采用的局部采样

`prepare-local-patches.py` 从冻结基线生成初步候选，`prepare-local-angles.py` 只调整覆盖距离和旋转角度；`local-angles.patch/json` 保存最终 shader 差异和 hash，已写入 `game/assets/shaders/terrain-slope.wgsl`，Rust、地表 UV、地图几何和采样尺度保持不变

每个2m三角格使用三个共享格点，以整数格坐标决定固定的连续方向和相位，原地表纹理仍按2.1m重复尺度采样；共享格点在同一米制 UV chart 的相邻覆盖三角中使用同一变换，平滑归一权重在边界连续，纹理颜色自身不增加随机色噪声；原地形主轴投影 chart 切换仍是既有边界，本轮没有将它改成 triplanar

颜色和 NormalGL 使用同一变换，切线法线经逆旋转回到原切线坐标再参与坡度混合；forward 与 normal prepass 使用相同计算，SSAO forward 保留已有 prepass normal，阴影与 motion vector 路径不改

新增局部采样函数在源码中对地表颜色与法线分别使用三个采样；是否消除原 `pbr_input_from_standard_material` 中被覆盖的采样由编译器决定，本轮尚未测 GPU 增量成本，使用旋转后的原始梯度和既有 mip bias，避免离散相位变化被错误解释为极大 UV 导数；不增加纹理绑定、外部依赖或新渲染框架

`check-local-patches.py local-angles --formal` 已验证正式 shader 的1215组正负网格、水平/竖直与对角边界，以及68组独立有限差分的法线旋转；最大跨边界差值8.01e-6，梯度误差8.30e-9，结果见 `local-angles-check.json`，这些检查不代表画面或 SSAO 已通过

实现者实际查看最终远景29/44及连续40–43帧，条带明显减弱、覆盖位置稳定；root 与独立审查对远景获得一致观察

最终集成近景为 `output/capture/task049-understory-after-r1` 和 `output/capture/task049-understory-detail-r1`，两者均150帧、6项检查 PASS，实际运行日志明确 `aa=TaaSsao`；分别使用最新真实 Viewer 与正式 shader，后者新增观察机位，因此不把两者当成同机位 A/B

实现者实际看两组29/59和 detail60–64连续帧：地表细颗粒和泥土/苔色仍可辨、宽条纹与规则格未回归，树影正常，未见采样接缝或镜头转动引起的覆盖漂移；对应版本与二进制 hash 见 `closeup-review.json`

本轮接受地表去重复改进，山体结构、覆盖丰富度与整体画质仍有独立制作空间，未标记为完整自然山景；GPU增量成本、Windows GPU 与作者审美验收仍未执行

## 可复现入口

```fish
python3 todo/evidence/TASK-045/terrain-surface-r5/prepare-diagnostics.py
python3 todo/evidence/TASK-045/terrain-surface-r5/inspect-texture.py
python3 todo/evidence/TASK-045/terrain-surface-r5/prepare-rotated-pair.py
python3 todo/evidence/TASK-045/terrain-surface-r5/check-rotation.py
python3 todo/evidence/TASK-045/terrain-surface-r5/prepare-local-patches.py
python3 todo/evidence/TASK-045/terrain-surface-r5/check-local-patches.py
python3 todo/evidence/TASK-045/terrain-surface-r5/prepare-local-angles.py
python3 todo/evidence/TASK-045/terrain-surface-r5/check-local-patches.py local-angles --formal
python3 tools/capture.py --script game/capture/forest-overview.json --output output/capture/task045-ground-r5-local-angles --binary game/target/debug/map_viewer --project-root output/assets/terrain-r5-diagnostics/local-angles
```

准备脚本要求输出目录不存在，避免覆盖前一次证据；通过只读 `git archive` 获取固定基线 `b948cabe717c1c6081055267a4b0c063b7de855d` 的资产和数据，因此正式 shader 更新后仍能重建旧候选；二进制版本和 hash 仍须按运行记录核对

## 收尾

已按 root 确认的 GPU 使用边界清理本子任务 `output/assets/terrain-r5-diagnostics`，70个普通文件共91,420,085B和30个符号链接已移除，源资产、正式 DDS、正式 shader、复现脚本、patch、hash及文字保留，详见 `cleanup.json`；root capture 图片仍交 root 统一清理

正式源码只修改 `game/assets/shaders/terrain-slope.wgsl`，无 Rust、appearance、manifest、几何或稳定ID改动；复杂度复查结论为 `Lean already. Ship.`，此结论只覆盖实现复杂度
