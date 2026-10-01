# Blender 接入 R3：实际画面自查

2026-10-01，审查基线 `8effe5ddd7f7a3afc18a24482a05670d0fe5f1a6` 上的本轮未提交改动；使用 nside-reference-analysis 的参考拆解口径，由已了解设计与实现的评审者做 self-audit

本轮实际查看当前 capture 的 53 张 PNG，包括关键帧与指定连续帧；没有观看视频，也没有用已删除的旧轮截图冒充对照。视觉目标对照项目现有 `docs/public/images/shop-street.webp`，不把概念图逐像素当作地图布局约束。机器结果来自主任务实际运行留下的原始记录，视觉观察和作者验收分别记录

## 机器证据边界

| 实际运行 | 帧数 | 包装器汇总检查 | 结果 | 实际范围 |
| --- | ---: | ---: | --- | --- |
| v55-west | 120 | 6 | PASS | Viewer 西立面左右移动 |
| cinema-court | 120 | 6 | PASS | Viewer 前场前后移动 |
| aircon-wall | 120 | 6 | PASS | Viewer 从挂机向下看管线再返回 |
| cinema-facade | 150 | 6 | PASS | Viewer 镜厅立面与前场 |
| shop-stairs | 60 | 4 | PASS | Viewer 高侧向固定检查 |
| shop-stairs-eye | 60 | 4 | PASS | Viewer 原低眼位固定检查 |
| yao-orbit | 240 | 10 | PASS | 正式入口 CHR-001 环绕，含包装器身份检查 |
| yao-motion | 540 | 28 | PASS | 正式入口 CHR-001 四动画与现有输入，含身份检查 |
| cinema-gentle | 9,900 | 7 | PASS | 正式入口胶囊代理 41 节点往返 |

原始命令、输入、逐项断言、二进制与地图版本见 [captures.json](captures.json) 和 [交付汇总](review.md)。本表不把帧数当成人工观看覆盖率，也不将自由相机移动等同于人物碰撞验收；原始 `run.json.visual_review` 的工具提示保持不变

## 实际画面结论

| 对象与已看样本 | 可保留的可见结果 | 当前限制 |
| --- | --- | --- |
| V-55：29／49／79，连续 30–34 | 包装／修补用途字、两处闭门的主次、橱窗样品与门眉能在街侧辨认；横移中未见旧泛型窗重复叠在新窗上，也未见明显跳位 | 玻璃仍像不透明暗色板，上层窗重复，浅墙较空，样品仍是简化方块；本机位以西面为主，不能声称完整南面和四面均已观察，闭门不是室内已可玩 |
| 镜厅前场：29／49，连续 60–64；整体现景 149 | 砖铺地、绿岛、长椅与沿墙通道的相邻关系明确；前后移动中没有看到明显铺地叠面闪烁或植物突然跳位；149 帧能把南西立面与新前场联系起来 | 长椅偏暗细碎、花簇过于块状，种植间距规整，周围仍有大块空草坡和重复灰盒立面；通行数值另由路径与碰撞检查支持，截图不能证明所有绿岛边缘可达 |
| V-W10 空调：29／49／69，连续 45–47 | 挂机栅格、下托架、墙上管线和固定点、低位收水口形成可读的安装关系；俯看时未见明显离墙漂浮 | 这是局部设施接入成立，周围平窗与重复砖面尚未精修；接触距离由几何检查证明，图像观察不等于安装精度测量或空调功能模拟 |
| 小店台阶：高侧位 59、低眼位 59 | 近处目标段的踏面／踢面有反差，低眼位可以读到连续细踏鼻线；未看到整段过亮的发光白条或明显悬空厚盖 | 范围仅三段 12＋26＋32 共 70 级，远处未处理的山路台阶仍呈深灰带；没有当前同机位运行前后图，不能量化改善幅度，也不能将这一结果推广到所有山梯 |
| 曜帽兜：环绕侧面 89、背面 139、连续 137–139；Walk 140–142、Run 190–192 | 正常游玩画幅中能看到帽兜圆收的下缘和浅体积；背视环绕及所看走跑段没有出现明显大块穿出、轮廓突跳或帽兜脱离躯干 | 背部主体仍平，衣身大片灰，布料缝线与受力折形不足；跑步造型仍简化，脸发和配色没有作者品质放行；这不是整角色完成 |
| 曜 Jump：62／65／66／69／72／81 | 能看见起跳、空中姿态变化与回到地面；角色与实际场景同画面出现 | 这些样本主要从正面看，帽兜后部被遮挡，不能凭它们宣布背部在全部跳跃相位无穿插；未穷举骨骼角度 |

以上连续片段是在逐帧 PNG 中观察轮廓与位置连续性，不代替实时主观手感或完整录屏观看。没有发现需要为本批接入立即撤回的新可见阻断；这只支持保留本批局部成果，不是概念图目标品质已达成，也不是作者审美批准

## 镜厅实际到达与返回

直接核对 `cinema-gentle/run.json` 的 `route_all_nodes_reached`：41／41 节点到达，记录内所有访问 grounded 为 true、resets 为 0；实际 `cinema_entry_landing` 在第 4563 帧到达，前街返程第 4594 帧，回到 `home` 为第 9066 帧，不使用脚本预计时间替代实际结果

实际查看门前连续 4562–4566：胶囊位于闭门前砖铺装上，靠近后开始折返，所看帧未见掉落、明显穿门或视角瞬移。查看沿途 `library_gate` 去程 4476／返程 4650，能辨认建筑边的铺装与沿边道路；这里也暴露周边空地和简化窗格依然明显。回家连续 9064–9068 显示胶囊回到月台杂货外侧，后两帧输入提示切为手柄，与脚本的输入切换相符

这条长路线采用正式控制器的胶囊代理，没有加载曜模型；曜四动画的证据来自两条独立短片。41 节点机器结果支持当前往返主路线，抽查帧只证明所看位置和连续片段，不覆盖镜厅室内、电梯、西侧长椅通道或绿岛横穿，不代表人工首次游玩理解和实体手柄已经验证

## 下一轮最有价值的三项

1. 街面成片衔接：优先处理镜厅与修补店周围孤立路条、空草地及沿街界面，让铺地、服务路、入口和绿化形成可读的街坊空间
2. 表面与陈设层次：减少窗户实心色块观感，为橱窗、檐下、长椅周围和植物补有用途的材质及形体层次；沿用当前许可与来源管理
3. 人物衣着完成度：保持已验证的帽兜和动作链，下一轮集中背部布料结构、比例与脸发辨识，不用技术检查通过替代年轻日漫外观的作者判断

## 已查看帧与校验值

以下路径相对于 `output/blender-integration-r3/`，每项均已实际打开查看后计算 SHA-256；关键帧和连续帧来源保持区分。同内容图可能拥有相同 hash，这是原始结果，未改图或补画

| 查看文件 | SHA-256 |
| --- | --- |
| `v55-west/keyframes/frame00029.png` | `90a3ee560924af3270ccc1e6085e8f096348a65a2d8b88a4572842e024a23d40` |
| `v55-west/keyframes/frame00049.png` | `a3b719c281dbe8ce0e181f26bc55bf6b59436410a150793eee79ca37a56cb51e` |
| `v55-west/keyframes/frame00079.png` | `90a3ee560924af3270ccc1e6085e8f096348a65a2d8b88a4572842e024a23d40` |
| `v55-west/frames/frame00030.png` | `6d593ee6816e2776767fb4122dccd1ed08c65119b7b2a3703e0b874d1fca6ad8` |
| `v55-west/frames/frame00032.png` | `fc3aacce2058a03501c970a9e29ca0cd800abad7101358669a3cbd5cf205081a` |
| `v55-west/frames/frame00034.png` | `286a092f35a68e3c53cd51923a69274ea340a0f65fe1e6b25ea5327a8f2b5315` |
| `v55-west/frames/frame00031.png` | `380f139e691db5fdec50f99fed0e5a70832c840083a90e16f6a5663d4e4bfb56` |
| `v55-west/frames/frame00033.png` | `8695b0a3cd9861e6e26eeb28f0b6acfaf88f1ab078ad28c464bc637230e5c419` |
| `cinema-court/keyframes/frame00029.png` | `6b5f9a5aaa82b975027b1a850473a1b7ec922d32e9251a4457d51d5fe77505f1` |
| `cinema-court/keyframes/frame00049.png` | `857b12de8cd97cec2db812131484e9c5a509c174c1671fac69ace0364911e3d1` |
| `cinema-court/frames/frame00060.png` | `512461e8ec09d41f183b4ec2a1459b040c55d5ea1b4a5b27ad8596ff69ae1636` |
| `cinema-court/frames/frame00061.png` | `0fb7ac152a6411fd90e5b721d2cc4c9c7f49be906d5266e4950a9c8ca5ce37f6` |
| `cinema-court/frames/frame00062.png` | `44cafbb304781298ee378515d84ce1e111cb84fb1134034b2a8e353e575cb43c` |
| `cinema-court/frames/frame00063.png` | `d0033978956657e3abf911c05893c83ca45281fc1511d66c1b8e5e82bef4568d` |
| `cinema-court/frames/frame00064.png` | `8dad74748923946c125e5edab27998f7b34e6ac27e5996b27613d49106a108c3` |
| `aircon-wall/keyframes/frame00029.png` | `4fe96fb2fd0c72148c083006ce405d4ccfd6d64c448c0a5fd01c1a16bf0c7b1a` |
| `aircon-wall/keyframes/frame00049.png` | `222383fccbfbc86f6429c89e31faff34ae67345f9a2acd1ad6bd45afdf58ee3b` |
| `aircon-wall/keyframes/frame00069.png` | `222383fccbfbc86f6429c89e31faff34ae67345f9a2acd1ad6bd45afdf58ee3b` |
| `aircon-wall/frames/frame00045.png` | `9d75be31d59765fed71d130bb3c40b7a884784cbf66bf2ce1e64cd35d563f13a` |
| `aircon-wall/frames/frame00046.png` | `fd2f023305e6cd3bf52b7970746191cdb8c9143f013a30c1e38f0b13f9a058c0` |
| `aircon-wall/frames/frame00047.png` | `0e289cc1a44eac75c18a3d8011cfe864207aa9fc9700a90a46bb2a38a37be4d5` |
| `cinema-facade/keyframes/frame00149.png` | `d18ffebe7cb3fec53e66633bb99e06bdddaa54da244f3f867e3182c9dc1811d5` |
| `shop-stairs/keyframes/frame00059.png` | `919f0a111f3032d71487ceaee9bda408b3f90d109e4020b60c638c6a2944ef5e` |
| `shop-stairs-eye/keyframes/frame00059.png` | `bd0dc1caca0508c98c2f192b312e11f93fa3ace141ca708d02ba5c181a3f68d3` |
| `yao-orbit/keyframes/frame00089.png` | `b73d4e36b5643d280ceed01b3c3f9118f5bbf637d774b10cdc5e9dde55c44f8f` |
| `yao-orbit/keyframes/frame00139.png` | `569352236c72d48d0ceb53298ba99414679e64d4000515bc22f685e64a2880ed` |
| `yao-motion/keyframes/frame00062.png` | `59e9eddfaa104bf114ee92521902f360c78bf6872bdec98f1982d6a6a305827a` |
| `yao-motion/keyframes/frame00065.png` | `f6319ce2c2b95a4f9df75e4b5c118d2318710bc1729029d587d90ff762065cde` |
| `yao-motion/keyframes/frame00066.png` | `06b74dfa66dc67084c4dac6b437e8590e529134c7112c48a23bdb5be8e3ae2a5` |
| `yao-motion/keyframes/frame00069.png` | `1c23d91f264eaf812eac10c77673260a703da958b104b3e573a48e317b4ebc36` |
| `yao-motion/keyframes/frame00072.png` | `b99f4f57de34c593a8ecf5bc00a0fc15554668c622afe10ce877010139b161e3` |
| `yao-motion/keyframes/frame00081.png` | `2e93a8ec7a5f1c1733d69a497a5f3edd2bd9870ce55d4f44e0700f60e7db6bf6` |
| `yao-orbit/frames/frame00137.png` | `a7897ed256a6c25586bf599f0572768e580b6c15ff39bc31247c0e0d2d92267c` |
| `yao-orbit/frames/frame00138.png` | `c49b645dc752b0305da2f3b0bd7b786dd99dc44bec96d08c8ca437ed0dcdbffe` |
| `yao-orbit/frames/frame00139.png` | `569352236c72d48d0ceb53298ba99414679e64d4000515bc22f685e64a2880ed` |
| `yao-motion/frames/frame00140.png` | `4c2ac3dfb1bd39980cfe6dc97cf55b5de4fd29f4ac6d582e2021bf2e190695a0` |
| `yao-motion/frames/frame00141.png` | `861de19f486b2d618f710f6bb5adc13117119d23f77f2c0dafd66fca5033eaf5` |
| `yao-motion/frames/frame00142.png` | `ac34590ec84b57121eac8b17994caf6cf9cd93dc4ff3f0d05dc12f6842866a88` |
| `yao-motion/frames/frame00190.png` | `443b81b7cef8c198d5435c5ce4e726199334e01f96c9947f074f72c8353c7090` |
| `yao-motion/frames/frame00191.png` | `dbce23c4e987c7e1a44f1d2df9b7c5b0f238198cbce474c0d5ef84df2f491af4` |
| `yao-motion/frames/frame00192.png` | `ee1f50a3d744d2284fa2f227de03cabdc7ced5e70d3af55b967731e797d8d4cb` |
| `cinema-gentle/frames/frame04562.png` | `5dff1a8008946c58822b801acfa09c397ab8582b4fa1cf97b3b773d0b8e5d6e5` |
| `cinema-gentle/frames/frame04563.png` | `20661b95c30e056c3c33b40140ef6202bf218467ae814583e33acd0c294b8baf` |
| `cinema-gentle/frames/frame04564.png` | `0beb648c988fd3acb52d10fc6bc564453c9106bfb4d4647c33e9a3c2067a1a0d` |
| `cinema-gentle/frames/frame04565.png` | `6ada843f042e127b3c011944647d2d92ff7d9307a14204bc2e89b29d15cabb60` |
| `cinema-gentle/frames/frame04566.png` | `565e7fcd8544d2e27831e454ed7b013e72b870e4b99f1043edff849e685fd578` |
| `cinema-gentle/frames/frame09064.png` | `48ea52bee67be30a54bdfad6d4517c927e32bb5729d71dc666cbcb6555167e6a` |
| `cinema-gentle/frames/frame09065.png` | `f4d8c60e2edb0687e73cd8e6d6fa3dd4ef8cee042d14435bd05ed1c80107fc3e` |
| `cinema-gentle/frames/frame09066.png` | `2c4bd66ea1102b07cd5358845ba352822ec9be179b0a92fdc5837bca1df55e4c` |
| `cinema-gentle/frames/frame09067.png` | `a8641905ff72bc203c71e8c76c194e6d6623ccdf9db57f01c41d490bea5a391e` |
| `cinema-gentle/frames/frame09068.png` | `a8641905ff72bc203c71e8c76c194e6d6623ccdf9db57f01c41d490bea5a391e` |
| `cinema-gentle/frames/frame04476.png` | `d0388cda4efc2e158c39f3afa998374d606be8f12ccb11f538a92a8e1a569952` |
| `cinema-gentle/frames/frame04650.png` | `8a012784a09dcebe60cb676f40e6a5306275cb68a60ab3c1d321961f1883ca86` |

这些视觉产物按项目约定由主任务在完成记录后统一清理，保留命令、参数、日志、状态和本文 hash。上述路径用于追溯当时所看产物，不承诺清理后仍能打开；实际清理状态以主任务收尾记录为准
