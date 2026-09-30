# V-04 橱窗商品轮廓修订

2026-10-01，本次承接首版浅窗实体陈列，保留既有几何纵深、空底背板、独立视觉批次、原窗位与碰撞。首版工程检查通过，但主线程和资产子项实际看图后均未接受其视觉结果，本次修订围绕书册、杯器和高低节奏收敛

## 实际看图与决策

实际查看 `output/shop-display-r1/baseline/keyframes/frame00059.png`、`after/keyframes/frame00059.png`、`walk-before/keyframes/frame00199.png`、`walk-after/keyframes/frame00199.png`、`walk-before/keyframes/frame00330.png` 和 `walk-after/keyframes/frame00330.png`。首版立体货品在近窗画面形成重复白标签盒，旧图中的杯柄和书册线索丢失；上半窗空、下层又受花箱遮挡，读起来接近门旁货架的重复，故首版视觉 self-audit 为 FAIL

修订移除统一白标签：左上三本不同高度立书、右上一本薄册，下层两本横放叠书；书由纸芯、双封皮和书脊构成，封皮沿用青绿及陶土色。右上加入有真实杯口、杯底和外侧半环把手的浅色杯器，内壁使用原深色 trim；最高立书高 0.74 m，低叠书总高 0.15 m，杯器高 0.30 m。背景 SVG 与 PNG 保持首版空底，不回填假商品

## 几何检查与边界

修订仍只作用于一个实际可见窗格，五个已有材质批次，无新增纹理、材质、GLB 或透明渲染路径。原上层金属横档继续承托货品，下层两半木板继续与侧框、中梃相接；货品留在原墙面之外、窗框前缘以内

初次修订窄测实际发现杯底直径 0.24 m 超出原 0.18 m 搁板深度，记录于 `native-test-merchandise-first.log`；独立审查同时发现完整环形把手的左弧进入杯内。修复将杯底和杯身直径收至 0.17 m，前后各有 5 mm 支承余量，把手改用外侧半环，开口端埋入杯壁厚度内。保留原深度、承托、中梃、横档、碰撞及预算条件

测试从真实 `facades()` 输出逐三角核对窗内范围、墙面／背板／窗框深度、中梃净空与横档无穿插；从实际书脊及搁板顶面确认四本立书和两本叠书的承托。杯口和把手检查复用现有 CollisionWorld 查询：仅在测试内拷贝视觉网格并赋予可查询 source，向下射线须穿过杯口命中杯底、杯缘须更高，把手空隙的微小球扫掠须穿过而上移后须命中环体。正式 source 与碰撞选择不变

新修订 GPU 视觉 self-audit 为 NOT RUN，主线程统一构建后复验；工程检查只能支持真实几何存在、未穿插和预算，不能代替实机上对商品识别与观感的判断。下一次固定镜头与步行路线须检查高立书是否填补上窗、杯口／柄是否可辨、侧向视差是否保持，作者或独立读者体验均未冒称 PASS

## 最终检查

| 检查 | 结果 |
| --- | --- |
| 真实几何窄测 | PASS，`cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib shop_window_display_has_depth_support_and_no_collision_change -- --nocapture`，最终日志 `native-test-merchandise.log` 为 1 passed / 0 failed |
| 实际输出预算 | PASS，1 窗、6 本书、1 杯、26 个盒体加原生杯几何，共 492 三角、5 个新增 mesh/entity 批次，低于 600 三角上限 |
| 碰撞对照 | PASS，保留／移除新视觉批次后的立面 CollisionWorld 均为 499076 三角、877 来源，内存大小及 bounds 相同 |
| 文档与引用 | PASS，`bun run check:docs`，544 Markdown / 129 IDs / 31 Skills，见 `docs-check-merchandise.log` |
| 格式与范围 | PASS，`rustfmt --edition 2024 game/src/world/scene.rs`、`git diff --check`；窗 helper、窄测和 reflectance 单行以外的 scene 字节与接手快照相同 |
| 独立瘦身审查 | PASS，`window_geometry_audit` 独立只读复核，处理杯底越板、把手入腔及底盘共面重复后结论 `Lean already. Ship.` |
| 新修订 GPU 与体验 | NOT RUN，由主线程构建并沿既有固定／步行相机复验 |

完整源码 hash 与保持项见 [merchandise-contract.json](merchandise-contract.json)，首版历史继续保存在 `source-contract.json`。窄测输出中的 20 个 colored book/shelf boxes 为 18 个有色封皮／书脊和 2 块板，另有 6 个纸芯，因此实际盒体总数为 26

## 范围与清理

本次窗口 helper 和其窄测之外，仅按主线程请求接入 `StandardMaterial.reflectance = spec.reflectance`，配合另一子项的材质参数读取；现有默认值为 0.5。窗外场景代码逐字节与本修订接手快照一致，三梯扶手增量完整保留。地图、碰撞实现、光照和正式 appearance 未改，后续重点建筑按主线程安排转入本机 Blender 制作

本子项未运行 GPU，未新增截图或录屏，所查看帧由主线程仍用于比较，清理归主线程。本项 `/tmp` 源码对照快照在所属读取命令完成后已记录 hash 并清理，未传入 Cargo 或 GPU；`fuser` 返回无使用者但有沙箱 socket 提示，因此清理依据也包括明确的文件归属与已完成的一次性读取。源码、正式资产和检查日志保留，未提交
