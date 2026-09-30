# 小店招牌局部反照率对照准备

状态：局部反照率和非金属镜面强度对照均已执行，两候选未采用，正式材质与光照保持

## 实际结果

同一 Viewer 二进制、脚本、分辨率与 TAA+SSAO 下，基线、0.60 候选、纯黑反照率诊断各完成 60 帧及 4 项断言；同一游戏二进制下基线与 0.60 候选各完成 450 帧及 9 项断言。完整输入、状态、日志与观察帧 hash 见 [实际对照](runtime-comparison.json)，`inputs.json` 保留准备时版本，不能代替这些实际运行记录

Root 已看固定第 59 帧和行走第 330 帧：0.60 让底色变暗，但字色也偏灰，未解决整体发浅问题，因此未写入正式配置。固定画面同位置的招牌底色分别为 RGB(153,203,198)、(116,157,152)、(82,82,80)，旁边雨棚抽样点始终 RGB(140,200,179)。纯黑仍有灰底说明该像素存在非漫反射亮度贡献，仅凭这一测量不能独立认定具体 shader 或色彩空间错误

后续将 Bevy 非金属 reflectance 从默认 0.5 改为 0.2，颜色仍为原始 `[1,1,1,1]`。同一新版二进制与橱窗/扶手几何下，两组各 60 帧与 4 项状态断言 PASS，基线在[新版固定机位](../shop-display-r1/revised-fixed/run.json)，候选在[镜面试验](reflectance-fixed/run.json)。同像素招牌底色从 RGB(153,203,198) 变为 (146,200,195)，邻接雨棚保持不变；Root 实际查看第 59 帧，变化轻微，仍未得到目标深绿招牌，因此本候选也不采用

未采用的 reflectance 配置字段与运行接线已从正式代码移除，避免留下无使用方的接口。可复现 [实验补丁](reflectance-experiment.patch)保留 schema、边界窄测和单行 StandardMaterial 接线，`git apply --check` 实际通过；机器验证了 0、0.2、0.5、1 与默认值、越界、溢出、错误类型，但这些代码不属于本轮正式交付。需要复现实验时先在对应基线应用补丁、编译，再用隔离配置执行本页命令，不对生产配置直接加已移除字段

下一步随建筑、局部材质和有归属的明暗层次继续检查，当前没有色彩空间加载错误的证据，不以两次负实验宣称画质问题已解决

以下保留最初准备方式与复现命令，运行图仍待本批实际查看完成后统一清理

已实际查看源招牌、前一轮固定 Viewer 第 59 帧、行走第 330 帧及 README 小店参考。源招牌底色 RGB(25,75,72) 在当前实机显浅青；代码中颜色纹理明确使用 sRGB、法线使用线性，StandardMaterial 上传时将 base_color 转为线性，目前没有色彩空间 API 错误的证据。照明、曝光、反射与 TonyMcMapface 共同影响最终显示，本实验仅检查局部反照率是否有助于保持招牌识别

临时项目根为 `output/shop-sign-r1/candidate-project`，仅复制 `district-scene/appearance.json` 与 `daylight.json` 作为本次配置，其他源目录、运行资产与相关源图链接到主库。唯一语义变化是 `/materials/sign_shop/color` 从 `[1,1,1,1]` 变成 `[0.60,0.60,0.60,1]`；daylight 副本与正式配置逐字节一致，正式 appearance 与 daylight 未改动。0.60 sRGB 约对应 0.319 的线性乘数，属于待验证的印刷面反照率小样

准备检查已实际确认仅有该字段语义变化、daylight hash 相等及正式配置 hash 未变，详见 [输入登记](inputs.json)。共享链接仍指向实时主库，正式基线与候选运行前需再次核对地图、二进制与环境资产 hash；如本轮橱窗代码或资产已更新，应以同一新版二进制重新取得 A/B，不能用旧画面宣称单变量结果

固定机位沿用 [shop-fixed.json](shop-fixed.json)，1280×720、30 FPS、60 帧、seed 0、TAA+SSAO；近景沿用 [walk-exterior.json](walk-exterior.json)，真实输入走近橱窗并返回公共街道，使用脚本中的画质状态与断言。两份脚本逐字节复制现有对照输入，参数未重新设计

以下命令从仓库根执行，输出目录必须尚不存在；基线使用正式项目根，候选使用隔离项目根，不需要重新编译候选材质配置

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --project-root . --script todo/evidence/TASK-045/shop-sign-r1/shop-fixed.json --output output/shop-sign-r1/fixed-baseline --aa taa-ssao --no-video
python3 tools/capture.py --binary game/target/debug/map_viewer --project-root output/shop-sign-r1/candidate-project --script todo/evidence/TASK-045/shop-sign-r1/shop-fixed.json --output output/shop-sign-r1/fixed-candidate --aa taa-ssao --no-video
python3 tools/capture.py --binary game/target/debug/n-side --project-root . --script todo/evidence/TASK-045/shop-sign-r1/walk-exterior.json --output output/shop-sign-r1/walk-baseline --no-video
python3 tools/capture.py --binary game/target/debug/n-side --project-root output/shop-sign-r1/candidate-project --script todo/evidence/TASK-045/shop-sign-r1/walk-exterior.json --output output/shop-sign-r1/walk-candidate --no-video
```

画面自查问题：深绿底与奶油色字是否更易区分，远处店名是否清楚，近处文字是否变脏灰，牌面是否仍保留真实受光关系；相邻墙面、雨棚、道路和所有全局光照保持不变。若文字变脏或改善不足则撤回本候选，不将参数推广到全城。状态断言与视觉自查分开记录，不代替作者审美验收

本准备阶段没有创建截图、视频或临时探测程序；配置副本和链接用于可复现局部实验，不是新增正式资产源
