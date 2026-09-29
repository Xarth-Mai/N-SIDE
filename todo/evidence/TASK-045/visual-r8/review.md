# 第8轮：地表重复、林下层、住宅窗饰与双角色预览

基线 `b948cabe717c1c6081055267a4b0c063b7de855d`，目标继续为完整游戏，当前先制作地图、画面和人物资产。原工作区 `AGENTS.md` 与 `docs/dev/validation/runtime.md` 的用户改动保留，不纳入本批提交；地图主数据、稳定对象ID、故事及日照参数未改

## 实际交付

- 地表：去法线与纯色诊断确认宽黄绿条纹主要来自Ground037颜色重复。拒绝固定37°双层和90°局部格两个候选，最终使用2m覆盖格的连续角度／相位与三个共享邻点混合；原2.1m采样尺度、草岩坡度过渡、真实阴影和normal prepass保留。实现与1215组边界、68组独立法线核对见[地表r5](../terrain-surface-r5/README.md)
- 林下：原38松树及旧庭院不变，三组加入12灌木、27草、6石，共45件／73,548三角；实际派生植被共259件，上限280。先放灌木、再石和草，保留6m来路开口、入口与望城视线，实际根顶点最大埋深为0.250／0.063／0.085m。检查与不足见[林下层](../../TASK-049/understory-r1/review.md)
- 住宅：十栋样板加入20组部分卷帘和60组上部木百叶，净增420盒／5,040三角，复用原材质合批；入口、玻璃、邻楼及原外壳边界继续通过。见[住宅窗饰](../../TASK-049/residential-r2/README.md)
- 曜：一份标准材质增加线性粗糙度／金属度图与轻微缝线法线，原底色PNG、几何、UV、权重、节点和三个动作值保持；引擎不再覆盖导入粗糙度与反射率。对照、15项语义核验及坏色彩空间负例见[材质r1](../../TASK-047/material-r1/review.md)
- 玲：独立长发、发夹、脸形、短外套／短裤候选，72可编辑分件，运行单网格／单材质、20,886顶点／39,676三角、32骨及Idle／Walk／Run；实际全高1.636m。同比缩放的骨架和位移曲线对应自然步速3.027／5.298m/s，在真实控制器速度下补偿播放倍率。制作、穿插修正和实际120FPS足轨见[玲模型r1](../../TASK-047/ling-model-r1/review.md)

原 `--character-preview` 仍选曜，`--character-preview=CHR-002` 选玲，默认入口保持原代理。两者使用同一加载、场景、输入、碰撞、动画与清理路径；它是制作检查入口，不新增运行中角色切换玩法。两人物均为 `needs_revision` 灰阶候选，配色、体型和作者审美未获批准

## 真实检查与失败处理

环境为Linux、RX6650XT／RADV Vulkan、Bevy0.19.1、Blender4.5.14LTS；具体每次命令、参数、固定时间步、种子、二进制和脚本hash均保存于下列每个run.json。输出目录中的完整state.json保留，版本化state-summary记录检查结果、样本数、首个ready状态与完整状态hash

| 检查 | 实际结果 |
| --- | --- |
| `cargo fmt --manifest-path game/Cargo.toml -- --check` | PASS |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins`，新增检视机位后再构建map_viewer | PASS |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib -- --nocapture` | 132 PASS，1 ignored辅助入口由另一测试启动两次隔离进程；含完整短登高及返回 |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer` | 3 PASS |
| `cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings` | PASS |
| `python3 -B -m unittest discover -s tools/tests -p test_capture.py` | 4 PASS，覆盖请求角色与实际ID不符、无ready样本、正确ID，以及缺帧／状态失败／GPU错误／超时 |
| `python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py` | 1 PASS |
| `bun run check:types` | PASS |
| `bun run check:docs` | 458 Markdown／102 IDs、31 Skills／25 recorded imports PASS |
| `bun run docs:build` | 56 map tests、player和dev实际构建及受众边界检查 PASS |
| `bun run tasks:sync`、`bun run tasks:check` | PASS，三项任务继续active，没有增加验收数 |

首轮scene窄测为21 PASS／1 FAIL，真实发现rest2组没有灌木 `[0,9,2]`。诊断表明草石先占了可用间隙；改为先灌木后石草后完整库通过，没有放宽根支持、入口或视线阈值。首次失败日志 `scene-tests.log` 与复验 `lib-tests-retry.log` 均保留

独立评审发现新角色只凭ready／clip检查会让“请求玲却加载曜”也通过。现在wrapper要求至少一个真实ready样本且全部ready ID吻合请求，额外结果标记 `source: capture wrapper`；原native检查不冒充新增身份检查。实际 `n-side --character-preview=CHR-999` 返回1并提示合法ID，日志为 `invalid-character.log`

## 实际录制

本批16次录制均成功，共3,090帧；每次完整PNG序列和MP4生成通过。机器结果不包含自动美术评分，[summary.json](summary.json)分开native检查与wrapper身份检查；`collect.py` 收集运行记录和视觉文件hash

| 录制名（位于 `output/capture/`） | 结果与用途 |
| --- | --- |
| `task045-ground-r5-baseline`、`no-normal`、`solid-color`、`rotated-pair`、`local-patches`、`local-angles`（同前缀） | 各90帧／6项，通过的是录制与状态；前两个改进候选因画面重复被拒绝，仅最后方案采用 |
| `task049-understory-before-r1`、`task049-understory-after-r1` | 各150帧／6项，后一组TAA／SSAO；原机位正对开口，未覆盖新增灌丛，不能作为林下效果证明 |
| `task049-understory-detail-r1` | 新检视机位150帧／6项，TAA／SSAO，实际看见灌木／草／石和树根 |
| `task049-residential-before-r2`、`task049-residential-after-r2` | 各150帧／6项，同V-A13机位和横移 |
| `task047-material-before-r1`、`task047-material-after-r1`、`task047-ling-orbit-r1` | 各240帧／9项；后两组另有1项wrapper身份检查 |
| `task047-material-motion-r1`、`task047-ling-motion-r1` | 各540帧／26项native＋1项wrapper身份检查；真实Idle、Walk、两种疾跑、暂停和恢复 |

## 实际看图与剩余工作

根Agent及独立审查分别查看了远景29／44／89及最终40–43连续帧，住宅59／89及60–64横移帧，林下29／59及60–64横移帧，人物CPU静态和完整走跑接触表，以及真实环绕关键帧、80–82帧和两角色Run190–194连续帧。采用抽帧与连续帧审查，没有把生成MP4本身声称为看完视频或真人试玩

地表宽条纹、菱格和局部正交格明显消除，近处细粒苔土仍可辨，TAA／SSAO下未见新增采样缝或地表游动；大山依然存在宽平面、人工台阶轮廓、空坡和单一黄绿色，整体山景未达到预览目标

住宅卷帘与百叶在横移中可见厚度与遮挡关系，门和主要视窗仍清楚；窗窗同型、玻璃平整、灰墙和用途细节不足仍明显。新林下两灌丛形成局部中层，但总体稀疏，草簇粗硬、石头过亮而且呈三角楔，需要继续制作草石与林缘；机位只露出少量路边，未覆盖整个短路关系

曜CPU材质候选A的冠顶高光过强已拒绝，B收敛后进入真实游戏；实机距离下材质增益较轻，没有把它称为正式人物贴图完成。玲已能凭长发、发夹和服装轮廓区别于曜，真实动作与阴影存在；环绕80–82可见的曜衣襟黑白碎边在同条件基线也存在，列为后续几何／边缘修订；两者脸部仍通用、颈肩比例／指姿／发束规律性需要调整，尚无正式色板、完整动作、跳跃动作、动作混合、坡地脚底适应或头发二级运动

本批Windows实机、真人键鼠／手柄手感、作者审美放行以及同条件GPU增量性能均为NOT RUN。任务继续active，技术资产与运行通过不计作G2或完整游戏完成

## 继续与复现

下一批优先制作草石材质及形体、林缘疏密和道路挡墙衔接，同时深化人物脸／颈肩／发束与真实绘制贴图；继续按当前完整游戏路线图，不扩展新剧情、室内或战斗

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/understory.json --aa taa-ssao --output output/capture/understory-next
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/residential-next
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-character.json --output output/capture/yao-next
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-002 --script game/capture/walk-character.json --output output/capture/ling-next
```

输出使用新目录；帧和视频实际查看并记录结论后，按项目规则清理。本轮最终清理数量、保留内容与剩余占用见 `cleanup.json`；正式模型、贴图、参考图和可复现代码保留

最终独立代码与复杂度复核已处理角色身份门禁缺口，结论为 `Lean already. Ship.`；实际画面审查与上述技术结论分开，作者审美仍未放行

清理已完成：本批3,239份录制PNG／MP4及两张额外已查看CPU图合计3,501,399,960B已删除，capture目录保留6,563,848B日志／状态／参数，余下0张本轮capture图或视频。各制作子任务另记录自己的临时源副本与DCC图清理；没有删除正式资产、参考图或Blender工具
