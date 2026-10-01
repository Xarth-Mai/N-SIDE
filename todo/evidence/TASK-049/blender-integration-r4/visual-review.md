# Blender 接入 R4：画面自查

2026-10-01，基线 `d7222a21786ad79c4c686748bb4fad13aaedd5c1` 上的未提交整合批次；按 nside-reference-analysis 实际看图，属于已知设计与源码的 self-audit，不代替作者审美验收

## 首轮结果与需修问题

`v35-facade` 120 帧／6 检查、`v35-entry` 150 帧／6 检查的原始 run.json 均为 PASS；它们验证 Viewer 加载、输入和截图输出，不代表材质效果通过

实际查看总览关键帧 29／49／79／119、横移连续 30–34，入口关键帧 29／59／99／149、横移连续 40–42。BYTE BEAT 文字、中央闭门、檐下设备与深窗层次可读；所看横移段未见旧窗重叠、招牌跳位或大面互相切换，但两种距离下浅墙都有明显黑椒点，和克制的动漫都市材质目标不符，首轮材质视觉需修订

### 颗粒诊断：已知事实与待验证假设

- 原始 `Plaster001_1K-JPG_Color.jpg` 与 `NormalGL.jpg` 均已打开查看，Color 是低对比浅灰抹灰，不能把画面近黑颗粒直接称作原图颜色特征
- 正式 GLB 的 Plaster 使用 `normalTexture.scale=0.25`；安装的 Bevy 0.19 `bevy_gltf/src/loader/mod.rs` 在读取 normal texture 时仍有 `TODO: handle normal_texture.scale`，实际忽略源法线强度，这是已确认的 DCC／引擎差异
- GLB sampler 为线性放大及三线性缩小过滤；不能据此保证运行时具有完整 mip 链，但没有证据支持把问题归因于 nearest 设置
- 配方表皮背面离旧墙 0.024m、前面 0.12m，泛型北面窗与雨棚已退出；当前画面没有大面交替覆盖的直接证据，尚不能判定为新旧表皮 z-fighting
- 优先以同机位仅关闭 Plaster normal 的一次隔离录制区分法线相关问题；若黑点仍在，再检查阴影及嵌入纹理采样，不先改全局阴影偏移或扩大重建

隔离复验：`v35-normal-off` 150 帧／6 检查 PASS，[normal-isolation.json](normal-isolation.json) 记录仅移除 Plaster normalTexture，几何、其他材质与光照保持；实际查看与入口首轮对应的第 59 帧，浅墙黑椒点消失，窗框投影仍可见。这支持问题位于该法线输入链，不支持以改几何或全局阴影偏移处理；完全关闭法线尚不能证明 `.25` 强度烘焙或 mip 改善的最终效果，后续候选仍需实图复验

## 正式法线修复复验

正式源将 Plaster／Concrete 的 `.25` 法线强度烘入无损 PNG，GLB 使用 scale=1，运行资产 SHA-256 为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`；[保持检查](normal-scale-r1/runtime-preservation.json) 记录几何、UV、法线、材质其余参数及两张原 Color 图字节不变

`v35-entry-final` 150 帧／6 检查 PASS。实际查看同机位第 59 帧与横移连续 40–42：首轮突出的黑椒点消失，浅墙仍有轻微灰泥颗粒，窗框和檐下投影保留，所看连续段没有出现大块斑点跳动或立面互相覆盖。此入口视点的修复可保留；这是强度修复，不是关闭法线后直接交付

`v35-facade-final` 120 帧／6 检查 PASS 后，补看总览第 49 帧及横移连续 31–33：首轮远景浅墙的黑椒点也明显消退，窗框／腰线／雨棚的明暗与几何层次仍在，未见可辨的大块颗粒跳动或表皮覆盖切换。近景与该总览机位均支持本次修复保留

嵌入 PNG 仍为 1 mip，远距缩小过滤没有因本次烘焙获得 mip 链；所看两个固定距离不代表任意距离下纹理稳定。仍有设备造型过简、暗窗缺少真实玻璃层次、外场过空等资产品质工作，不能据此宣布 BYTE BEAT 或整段城市达到参考成品效果

## 曜上衣 r11 实机自查

`yao-orbit` 240 帧／10 检查、`yao-motion` 540 帧／28 检查均 PASS，包含真实 CHR-001 身份与四动画路径；检查数来自包装器汇总，模拟输入不等于实体手柄测试

实际查看环绕侧面 89、背面 139 及连续 137–139；Walk 141；Run 连续 190–192；Jump 正面 69、背侧 392。衣身与袖筒的宽松外轮廓在正常游玩画幅中可读，帽兜、袖口与手腕保持相邻，所看连续段没有新增大块破洞、袖筒拉散或突然跳位；Jump 背侧样本也能看见衣身与帽兜的整体关系

本次只支持 r11 局部衣形继续保留。袖筒仍偏圆滑膨起，背部大片灰面、布料裁片和受力折形不足；裤形、脸发与配色没有因此完成。没有当前运行 r10 同机位 A/B，不能把源模型位移量等同于同比视觉改善，更不能由采样图保证完整动作无自相交

## 已实际查看的代表帧

路径相对于 `output/blender-integration-r4/`；下表保留已看关键样本的 SHA-256，连续帧范围见上文。视觉媒体在主任务完成记录后统一清理，原路径不承诺清理后仍可打开

| 文件 | SHA-256 |
| --- | --- |
| `v35-facade-final/keyframes/frame00049.png` | `7b0a25f6ce07934cb033b4c86255e9f5dcf470d14823b8df6bef68c06518f19f` |
| `v35-facade-final/frames/frame00033.png` | `aa00f097059f524cacf945fa4948b92a12f30e59d3aebe7c9212593de2ab5edb` |
| `v35-entry-final/keyframes/frame00059.png` | `e837bcfc169d4c4f0a4d710dc6d7089df9f92888d42f79ae4d709dd0be23a8a8` |
| `v35-entry-final/frames/frame00041.png` | `4af1c91657d6ab35283aad5faec6fd1861ae6f782432eb0b6fae397c82548740` |
| `yao-orbit/keyframes/frame00139.png` | `d452e5ddd48418bbc8a38bf29bcf03425b1d1498bd73c8be0b57d4d8056a3000` |
| `yao-motion/frames/frame00191.png` | `a9d39e98364dfdf00e86e61c00bf8fee5319074c591023b756396463f7d76d9e` |
| `yao-motion/keyframes/frame00069.png` | `1e00757c41fc0759f77d470243feb34ab5a1c946ceb139b0ad3abbbf299a2108` |
| `yao-motion/keyframes/frame00392.png` | `e656fb928fe2a767f774106993fa5ac5c2271c2185218f74f7572b4baea0f580` |
| `v35-normal-off/keyframes/frame00059.png` | `ac403dc7846311c746c9a4bb08e208480d6612e7b0124f1449681931187f68b3` |
| `v35-facade/keyframes/frame00029.png` | `bdafaa2f194e85698525afadad7bf6c599ebde159db295f57e7d003ad04dbbb0` |
| `v35-facade/keyframes/frame00049.png` | `c48a8593517f88c06b0e32e250cfabfcf001a4e8a59ea5740cd390dac4015c6a` |
| `v35-facade/frames/frame00032.png` | `9fac3db1206bc5edc72ab84c2321c8a01dd087b728e240d9bd696f47ff82312a` |
| `v35-entry/keyframes/frame00029.png` | `6b24f9cc7df420eac7b77cd988e84d44f0fc882fbbf1088fd4eece236d3b881d` |
| `v35-entry/keyframes/frame00059.png` | `d795565be8b235313d09c3cac0977775d112381e58ac65f75f7acb3d940a24f2` |
| `v35-entry/frames/frame00041.png` | `e0905679eeba33b3974f0fffb9a42d7e010ebb95dfbee691955c561a27c228ef` |
