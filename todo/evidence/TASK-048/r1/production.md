# 安可海报制作与 CPU 检查

本轮修改仅涉及 AST-007 海报源文件与导出、AST-004 的新材质绑定、真实地图公共屋顶上的两处屏幕，以及对应 Viewer 观察入口。未修改 TASK-046 的叙事内容、人物实际年龄、地图源坐标或玩法

## 交付与实际查看

- 内置 `image_gen` 成功生成一张原创歌手画像，原件已保存 `source-assets/star-posters/anke-portrait.png`，提示词、工具边界与来源由同包 README 和 prompt 管理
- 已实际查看原图：粉色长发、白夹、琥珀眼、短外套和麦克风清楚；左侧保留排版空间。用户后来确定外观年龄为实际年龄减 2，按约 17 岁外观重新自查保留现图，实际 19 岁和真实生成提示词未改；这是制作自查，不是作者外观批准
- 已实际查看最终 `1600 × 900` 排版：「安可 / ENCORE / 下一站晴天」清晰，画像、中文与署名没有重叠；初次 librsvg 未显示外部图片，改为导出时临时内嵌源图后复查，最终图包含真实画像
- V-01 为 `10 × 5.625 m`，V-79 为 `8 × 4.5 m`；单面北向图像使用既有 unlit 标准材质，黑色背箱及金属支架受场景照明，支架落在源屋顶以内

## 执行结果

| 命令或检查 | 结果与范围 |
| --- | --- |
| `bun source-assets/star-posters/export.mjs` | PASS，输出 1600×900 PNG |
| `bun source-assets/star-posters/export.mjs --check` | PASS，与重导出字节一致 |
| 受控在运行 PNG 末尾添加漂移后执行 `--check` | 正确非零失败，诊断 `Stale poster`；随后恢复原字节并再次 PASS，见 export-checks.json |
| `asset_report.py ... --expect-size 1600x900 --json` | PASS，RGBA，alpha 全 255，画像与排版需上述实际看图确认，见 image-report.json |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib star_screens -- --nocapture` | PASS 1/1，源屋顶范围与高度、落地支架、16:9、面朝公共北向、真实资产文件存在 |
| 上一项第一次运行 | 正确发现斜撑根部角点低于屋面，修复为底座上方 0.14m 承托点后通过，见 geometry-test-first.log 与 geometry-test.log |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer` | PASS 3/3，见 view-tests.log |
| `cargo fmt --manifest-path game/Cargo.toml --check`、作用文件 `git diff --check` | PASS |

## 待统一 GPU 复验

`game/capture/poster-station.json` 和 `poster-live.json` 各 300 帧、10 秒，使用真实公共源节点的眼高位置，调用现有 Viewer 控制器接近、侧移和恢复。查看帧中的安可识别、文字尺度、屋面锚固、画面背面与遮挡；机器断言与画面自查分别记录。初始观察相机不是角色出生点，这两段不证明角色可达、碰撞或玩法

本文件编写时尚未执行 GPU 拍摄，主代理将在统一构建后运行。源图片、SVG、导出 PNG 是正式资产；本子任务没有留下临时截图、接触表或视频，导出临时目录已自动删除
