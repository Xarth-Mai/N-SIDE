# 玲 Jump 真实录制画面自查

2026-10-01，self-audit，仅评审 `output/jump-runtime-r1/ling-walk/` 的真实输出；已读实现与状态，不属于盲评，不代替作者审美或真实操作验收

## 结果与方法

PASS · 在实际查看的 30 张 1280×720 原始帧中，未见膝脚断裂、网格炸裂、落地后持续悬空或整幅画面突变；动作表现仍有下述品质限制，不据此放行正式角色动作

通过 `view_image` 的 original 模式逐张查看 59–81 的完整首跳帧序列、390–393 的连续背面下降帧，以及 398–400 的背面落地帧；其中 59、62、65、69、72、78、81 从 keyframes 读取，其余从 frames 读取，实际文件与哈希见末表

`video.mp4` 存在且 run 标记编码 PASS；当前没有可读取视频内容的工具，本评审没有播放视频，只使用原始帧序列观察，不把编码成功写成观看完成

## 实际观察

- 首跳 59 帧站立，60–66 帧鞋底与地面阴影逐步分离，67–72 帧可见双膝上抬和手臂略外展，73–78 帧回伸；头、躯干、腿、袜和鞋保持完整连接，未见关节反折、脱节或异常拉长
- 76–81 的连续落地段中，鞋逐步回到原地面，78→79 从略屈腿回到 Idle 站姿时有小幅姿态切换，未见脚点横向瞬移或落地后继续悬浮；地面阴影重新靠近鞋底，画面中的台阶、路缘和 UI 保持连贯
- 390–393 背面下降段显示相同的屈膝和伸腿过程，双鞋仍跟随小腿；398–400 回到地面并站稳，未见背面视角下新增的断裂、脚部甩出或落地错位
- 观察到的“上提感”属于当前姿态表现限制：60–66 的腿部弯曲和躯干压缩较弱，直立身体先升高，到顶点附近才明显收腿；78→79 和 398→399 接地后直接回到直立 Idle，缓冲下蹲较弱，当前帧证据支持继续打磨预备、空中姿态与落地衔接
- 镜头随高度变化令角色大致保持屏幕中心，地面与楼梯相对上下移动；连续帧内变化一致，未见摄像机突然换位、角色消失或全屏闪白，不能仅因鞋在空气中就认定为接地故障

## 与机器状态交叉核对

`run.json` 与 `state.json` 均为 PASS，实际进程返回 0，540/540 帧、27/27 项检查通过；角色为 CHR-002、ready=true，已查看区间没有 character error；录制为 30 fps，脚本 SHA-256 为 `6a15efbc24c5a10a1d67b873a6fd50b188951a7678834571bfd06bc5e954f6f2`，运行二进制 SHA-256 为 `5ba0c688249b6cf1d30bc27e6f0f9e6978062a337719ac1e719f870c7ab63a36`

| 帧 | 脚点 Y / m | grounded | 实际动作 | 观察用途 |
| --- | --- | --- | --- | --- |
| 59 | 28.045998 | true | Idle | 起跳前地面基线 |
| 60 | 28.225998 | false | Jump | 起跳与动画实际开始 |
| 68–69 | 28.945999 | false | Jump | 最高点比基线高约 0.900m |
| 78 | 28.045998 | false | Jump | 下降末段已到原脚高 |
| 79–81 | 28.045998 | true | Idle | 实际支撑恢复并播放地面动作 |
| 390→393 | 28.925999→28.745998 | false | Jump | 背面连续下降 |
| 398 | 28.045998 | false | Jump | 第三跳下降末段 |
| 399–400 | 28.045998 | true | Idle | 背面接地并返回地面动作 |

首跳所看区间的 X/Z 恒为 100/-255，jumps 从 0 到 1 且 resets=0；第三跳所看区间 jumps=3、resets=3 均保持不变，未发生该段新增复位；状态与画面中的原地跳跃、落地恢复相符

## 限制与保留

完整 18 秒视频播放、未列出的帧、侧面近景、所有动作时刻的微小穿插、物理输入手感、音效与作者／玩家验收均为 NOT RUN；原生帧逐张查看能暴露形变和位置突变，但不等同于按正常速度观看时的节奏判断，当前动作自然度仍 needs_revision

本评审只新增本文件，不修改 Rust、录制或共享文档；没有生成新的截图、接触表或视频，root 的原始录制保持原样，等待主执行者统一完成归档与视觉产物清理

## 实际读取文件 SHA-256

以下路径相对 `output/jump-runtime-r1/ling-walk/`，哈希对应本次实际查看或读取的文件

| 文件 | SHA-256 |
| --- | --- |
| `run.json` | `21989ad5e37d1870bc8d309b9ef063aaf6f4d9111f15dee44ba18c171badee62` |
| `state.json` | `63856a4bcbd23ac10ed65e461fb9e3a88ca09fd0ba4cf2e950e293f4bcf57691` |
| `script.json` | `6a15efbc24c5a10a1d67b873a6fd50b188951a7678834571bfd06bc5e954f6f2` |
| `keyframes/frame00059.png` | `6e2f66888277427728f35c14b2fea1cc338288a13b7c872ab45d9a84592ef2d2` |
| `keyframes/frame00062.png` | `fbd68cd52076a2b8580e127105aa903b89323141f58336ca0f4dac156e16ee4e` |
| `keyframes/frame00065.png` | `03941970a07d15f3dd5baea2b51961b36700e903585a48dedcaa346602485324` |
| `keyframes/frame00069.png` | `4e807896fddfd91ab46d4e5e16fd725c43d9d02cfe32c99882ab6d23a0198939` |
| `keyframes/frame00072.png` | `b5a9dd945ecda701f08080925603a6f6d69b65f1841995903adecaa95b0a69ed` |
| `keyframes/frame00078.png` | `22d5ea537791617cb7c3f88f739ca2c06d73e82e5768a8becae902b7125849d6` |
| `keyframes/frame00081.png` | `66abf71f4410ba0939b78c865f10e8665a4ec58e2243fb5c05841a34c8366513` |
| `frames/frame00060.png` | `f8b9ea0c712ef0d343246aafd9c3568fe6214ac8caecdce7a7186075418491bd` |
| `frames/frame00061.png` | `bacec9072ed191c35b2b6f6abb0feef64ed767ab4f9d649eb1f1c0ef6cc7a5a1` |
| `frames/frame00063.png` | `af6f2b8b8f35ce3d3e6d349f6db0542c41ec2d7f8a1000b0297bdd15380f8e1f` |
| `frames/frame00064.png` | `3697f8073e6ca0c5d2ccb10daa2d915da7a2de9f7414b880c7d10795726c17a7` |
| `frames/frame00066.png` | `96fcd224602c31c93cb2cb4181e1cc01210382b58033eca6465e990e67cfeb96` |
| `frames/frame00067.png` | `30ccc0e5e88188018b1df65b0861928763eeda225feb595f264bc78a77ba90fc` |
| `frames/frame00068.png` | `297163f4ada07858b9b9e4b3d4e0c265f6655dcab374633d7d5d6223b949bc2f` |
| `frames/frame00070.png` | `8e073afd7dff4224275f5c574dd960a93260227cc05994b951d012dffad2216b` |
| `frames/frame00071.png` | `039149502dfc2d35a7f05ee01f53539fd4fa86b5b47e7ce554f154a53136a8f6` |
| `frames/frame00073.png` | `1880a0e2f879e40a1b8c446256ef0582524a142aae44856c06a944f262a0e53c` |
| `frames/frame00074.png` | `7f3fe5be757d0f097da81414be235e9311d7dae562a20e4520a669539647eddd` |
| `frames/frame00075.png` | `72323fe59ef6fffabcdc0dda3100aa9b69321a01e64932ced7e8c1b5cdf185b7` |
| `frames/frame00076.png` | `6af0a837caafc8a5aee0c5eba5ba0d2725ef4baba1004f85c503fa6490f35b0d` |
| `frames/frame00077.png` | `63d6c5cd535107678680f1caf82721c8684f9159a0f3eda083c0d7cbc0f46fdf` |
| `frames/frame00079.png` | `5e2b7d55615363827905707a453a08ef35bf0278d7d96994b978152117a3b084` |
| `frames/frame00080.png` | `cc6fd50ee763d80716cede2b9abbff9460d9ab2764f4721777215078a4959a5c` |
| `frames/frame00390.png` | `55218d0137c5ec363a78cf268f3c14d11eecb414c6ef7ef2ffe3387193f4e980` |
| `frames/frame00391.png` | `b68279740940dbf116ac8e4595f8e561e9110779e3c15e185205588a4a044b76` |
| `frames/frame00392.png` | `bfe80cd8a089c4749b709fe986bdff678bdec442e2fcb1341514b530eee967f9` |
| `frames/frame00393.png` | `5360ace74c21a49b8842198285587b92a8eb36bbe6d5b757d716978865fc75d3` |
| `frames/frame00398.png` | `6579a4a36c44285002a966cddb69183726b3bb2666d147ff3e4e87a4b5101a11` |
| `frames/frame00399.png` | `41f5a3174cc801aec2dca329bbb1e5ea0fffaca14c903e6c1b04110f45eaec48` |
| `frames/frame00400.png` | `c92be9eb180a80097771eba8ff6a812d6fdb8eb5250ab7528ffbd64fdac57124` |
