# TASK-014 第1轮：室外胶囊与跟随镜头

基线 `8749a96`；本轮中性代理、真实网格碰撞和镜头进入正式入口的 `--walk-preview`。默认固定镜头入口保持可用，未确定正式主控、动画、室内或任务玩法。输入与构建hash见 [provenance.json](provenance.json)，地图稳定ID与高程未改

## 机器检查

| 检查 | 结果 | 范围与证据 |
| --- | --- | --- |
| Rust默认lib | PASS 41 | `cargo test --manifest-path game/Cargo.toml --locked --lib`，含真实生成网格、上阶、低顶、高墙滑动、60°坡面、恢复、近镜头显隐、输入与capture断言 |
| Rust Viewer全targets | PASS 41+3 | `cargo test --manifest-path game/Cargo.toml --all-targets --features viewer --locked`，日志 `rust-viewer-tests.log` |
| 格式、Clippy与双入口build | PASS | `cargo fmt --manifest-path game/Cargo.toml --check`；`cargo clippy --manifest-path game/Cargo.toml --all-targets --features viewer --locked -- -D warnings`；分别build n-side与map_viewer |
| capture wrapper | PASS 3 | `python3 -B -m unittest discover -s tools/tests -p 'test_capture.py'`；包含walk-preview flag传递 |
| 文档、Skills及双站 | PASS | `bun run check:docs`；`bun run docs:build`，玩家95页、开发158页，开发契约后续修改再次build:dev |
| 上阶／顶墙／离开／双输入 | PASS 650帧、16项 | `walk-final-state.json`；21.67秒、1280×720、30fps，脚本键鼠和真实Gamepad轴资源走实际controller |
| 缓坡／近墙镜头／恢复 | PASS 600帧、13项 | `ramp-final-state.json`；20秒，同样分辨率与固定步长 |
| 原正式入口回归 | PASS 540帧、11项 | `entry-regression-state.json`，保持标题、加载、固定镜头、两次进入清理及取消 |
| 原Viewer自由镜头 | PASS 540帧、9项 | `viewer-regression-state.json`，同一共享capture的前进、转向、释放与恢复 |
| 不可能位移 | 预期FAIL，native与wrapper均退出1 | `negative-state.json` 仅 `impossible_player_motion` 失败，必需素材、120帧落盘及Transform检查通过 |

GPU为 AMD Radeon RX6650XT、RADV/Mesa26.2.3、Vulkan。两条成功步行记录及正式入口均生成视频；本轮通过查看PNG关键帧和连续帧自查，没有把工具生成视频等同实际完整观影。原始输入、日志、连续PNG与MP4保留在 `output/player/2026-09-28/r1/`，本目录保留状态、run记录与质量80 WebP

实际状态：小店北阶末脚点约 `[106.427,29.478,-272.683]`；门面前脚点约 `[88.376,28.056,-255]`，阻挡源为 `buildings[V-04]/derived-facade`。第二路线在门框旁镜头约0.454m，转向墙面收近到约0.104m，真实离开3.2m后恢复3.8m；这些距离来自实际球扫掠，不是脚本写入

## 观察、失败与修复

- 初次CPU检查发现渲染道路是无索引TriangleList，碰撞读取已兼容索引与无索引形式
- 精确skin切向查询可能产生数值命中，记录在 `collision-tests-skin-boundary-fail.log`；运动在查询skin外保留1mm余量，顶墙、沿墙和离墙测试覆盖实际行为
- 初版上阶被圆帽接触法线误判为立面，`player-tests-first-fail.log`保留失败。现区分实际三角面与接触法线，在共享棱边内侧10mm核对真实踏面，同时保留完整上移／前移／落地扫掠与低顶拒绝条件
- `walk-first`机器通过，但389帧门框遮住代理；派生门框、窗框和雨棚已纳入同源BVH。最终完整准备场景为557496三角、3301来源段，基础几何微测105688三角／16.05MB仅代表不含派生立面的子集
- `ramp-first`359帧近镜头被代理遮满，增加距离小于0.8m时隐藏代理并在移远后恢复。该次机器失败是仍贴着门框却要求恢复3.8m；最终路线通过真实D输入先离开门框，再检查恢复，并未降低无遮挡时3.8m的要求
- 最终实际查看 `walk-final` 59、179、239、389、449帧，`ramp-final`209、409、429、539以及连续398–400帧：脚点随阶与坡变化，近墙可见周围空间，离开后代理恢复；门框和墙没有通过调整高度或关闭碰撞绕过；另查看原入口119帧、Viewer239帧，原预览构图保留
- 输入审查发现仅推摇杆不会切换提示，已将摇杆死区外活动与鼠标活动接入同一来源判断，并补实际输入测试

原始查询、初版断言和视觉失败均保留；Windows、实体手柄、人工连续游玩、最终镜头手感、角色动画、室内门洞、导入模型道具碰撞和整段上山尚未验收。运行日志仍有已有的ICU中文分词模型警告，本轮实际文字正常显示，未据此更换Bevy依赖

## 来源与边界

新增查询库Parry固定0.30.2，原有依赖版本未移除或升级，Bevy保持0.19.1，差异见 `dependency-diff.json`。crate checksum、完整Apache-2.0和作者归属见仓库 `third_party/parry/`；Windows包增加对应许可与步行入口，未在本机执行Windows构建或启动

普通运行当前使用Bevy默认64Hz固定步，capture为30Hz；部分局部CPU用60Hz检验数值行为，不能把这些称为同一时间参数。帧序、输入和状态可重复，未主张跨GPU像素确定性

独立正确性审查未发现高影响阻断；按ponytail简化审查结论为 `Lean already. Ship.`。TASK-014继续用于真实路线扩大与体验比较，不把此批技术结果当作作者手感或G1放行

## 下一动作

用同一PreparedScene与PlayerState从home逐步走 `routes[id="hill-short"]` 的130节点，先做64Hz CPU可达性探针，遇阻记录实际位置、支撑和源对象。路线长度1311.468459m、无电梯，不能以已有Viewer视线记录替代人物登山
