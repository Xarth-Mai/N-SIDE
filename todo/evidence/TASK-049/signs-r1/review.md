# TASK-049 城市经营标识 r1

日期：2026-09-30；范围：AST-004 新增 5 块经营标识，未修改地图、稳定对象 ID、场景绑定或既有五店配色

## 实际交付

新增 `station`、`byte-beat`、`frame`、`playroom`、`after9` 五个 SVG 与同名 PNG，正文名称依正式地点目录。源图说明见 [signage-production.md](../../../../source-assets/district-scene/signage-production.md)，没有独立复制资产 manifest

`tools/export-district-scene.ts` 沿用既有 ImageMagick 流程与临时 Fontconfig，仅追加五个名称并让它们使用 AST-005 的 Noto Sans SC；原有十个 PNG 的 SHA-256 与本轮制作前一致，见 `existing-before.json`

## 客观检查

| 检查 | 结果与范围 |
| --- | --- |
| `bun tools/export-district-scene.ts` | PASS，导出 15 个原创建筑图形 |
| `bun tools/export-district-scene.ts --check`，连续两次 | PASS，15 个派生结果逐字节一致；额外比较五个新 PNG 的 hash 与 mtime，check 保持只读 |
| `asset_report.py --expect-size … --json` | PASS，五图画布分别为 1200 × 256、三张 1024 × 256、1024 × 320 |
| PNG 解码与 alpha 极值 | PASS，五张可正常解码，RGBA 展开后的 alpha 均为 255；这组为完整不透明牌面 |
| 故意以 `1024x256` 检查 station | 预期 FAIL，退出码 1，见 `wrong-size-failure.json` |
| 已有运行招牌保持 | PASS，本轮前 10 张 PNG 字节不变 |
| SVG XML 与可编辑正文 | PASS，五图 title 对应正式名称，所有正文使用项目字体，无外部图片引用 |
| `bun node_modules/typescript/bin/tsc -p tsconfig.json` | FAIL，当前 `tools/tests/district-map.test.ts:244` 将 `string \| undefined` 传给 `string`；本批未修改该测试，已向集成方报告，见 `typecheck.log` |

实际工具：Bun 1.4.2、ImageMagick 7.1.2-32、Python/Pillow；源与派生 hash、图片报告见 `asset-report.json`，缩放参数见五份 `*-preview.json`

## 实际看图 self-audit

已查看 768 px 和 384 px 宽联系表，以及五张浅、深、洋红背景预览。主名称均完整，N站中文字形正常，未见缺字、裁切或邻接字重叠；在小预览中 FRAME 与 AFTER 9 的大字最突出，其他三牌仍能分清名称。交通分区、节拍键格、模型角框、播放按键和斜切数字区形成不同经营识别

五图完全不透明，对比背景没有透明灰边问题。AFTER 9 的 LIVE HOUSE 行在 384 px 宽缩略图中属于小字，需近距离读取，不作为定位入口的唯一信息。牌面没有方向箭头，N站的标识位置由真实入口决定

NOT RUN：真实场景接入、行走透视、转向闪烁、不同光照和作者视觉验收，交由主 Agent 在统一接入后执行。联系表只能证明平面缩小表现

## 临时产物

完成实际查看后删除 `output/assets/task049-signs-r1/` 下本轮联系表与对比 PNG；保留正式 SVG、运行 PNG、参数、日志和本记录，删除范围与数量见 `cleanup.json`
