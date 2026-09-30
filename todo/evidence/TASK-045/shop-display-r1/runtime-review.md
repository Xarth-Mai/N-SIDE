# 浅橱窗与主梯首轮实机检查

原标签盒方案已在实际画面中判为需修订，随后改为四立书、两叠书与真实杯口、把手，保留原窗框、横档与下层支撑。新版固定 Viewer 60 帧 / 4 项断言与真实游戏 450 帧 / 9 项断言均 PASS，版本见 [冻结输入](revised-inputs.json)、[固定机位](revised-fixed/run.json)、[真实行走](revised-walk/run.json)

Root 实际查看旧／新固定第 59 帧，以及新版近景第 199、222、330 帧：立书高低和杯形重新可辨，取代首版的统一白标签盒；斜视中保留窗框和商品遮挡顺序，未见书或杯穿出侧框。下层陈列受既有店外植物遮挡，这与路上实际视角一致，不因此宣称整窗内容无遮挡。店招仍偏浅、邻楼重复、大片铺地和空坡仍明显，当前仅为局部结构和辨识改善

本轮同图可见第一段新主梯的上、下横杆及立柱，扶手与道路之间有清楚留空。其余梯段另外按上坡镜头及完整路线记录，不以当前固定画面判全路径验收

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin n-side --bin map_viewer
python3 tools/capture.py --binary game/target/debug/map_viewer --project-root . --script todo/evidence/TASK-045/shop-sign-r1/shop-fixed.json --output output/shop-display-r1/revised-fixed --aa taa-ssao --no-video
python3 tools/capture.py --binary game/target/debug/n-side --project-root . --script game/capture/walk-shop-exterior.json --output output/shop-display-r1/revised-walk --no-video
```

首次构建漏传 `--features viewer`，Cargo 明确拒绝构建该 binary；补充既有 feature 后 35.75s 构建成功，未改 Cargo 配置。实际运行使用冻结材质副本隔离后续并行编辑，完整命令与二进制 hash 以 run.json 为准。本批此时还包含已否定的 reflectance 实验接线，但默认 0.5 与原行为一致；最终交付重编译时移除该接口

这些运行仅为 Linux GPU 与脚本输入自查，未做 Windows／真人手柄／作者审美验收。图像 hash 已登记，临时图待本批统一清理
