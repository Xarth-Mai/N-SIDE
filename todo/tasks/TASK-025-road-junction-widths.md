---
id: TASK-025
type: fix
status: ready
milestone: G1
depends_on: []
specs:
  - docs/dev/design/locations/district-plan.md
  - docs/dev/design/locations/district-space.md
  - source-assets/district-map/README.md
  - docs/dev/validation/runtime.md
---

# 全城既有路口的路幅高程衔接

## 目标与范围

接续TASK-023第5轮发现的既有路口问题，逐批修正共用中心节点、实际路幅却在不同高度重叠的坡路与台阶。按真实道路宽度、转角拼接、台阶踏面和地形检查；保留建筑、入口、稳定对象ID及已确定的山城关系

## 验收条件

逐项保留修前位置与高差，修后复验实际路幅和连续梯段；合法分层通行须具有明确的源定义和净空，不能靠共用节点豁免、放宽误差阈值或渲染混合掩盖不连续。重点路口有实际画面，受影响入口与短登高路线回归；图形检查不代替尚未实现的人物碰撞与通行

## 当前工作与下一步

待执行，冻结源 `49f46aed0e9d3a578304f0a99d7174864e7a9c2dcc9382c7ffcd1c2ca402acf0` 的全城扫描仍有108对候选（105对共节点、3对不共节点），最大约9.05m。按最终路幅报告定位最严重的交叠，先修山脚及日常上街连接，再分批处理其余路口。TASK-024的建筑与素材交付不代表本任务完成；本轮已修的短登高路线由TASK-023记录，不重复计数

## 结果与证据

问题来源为TASK-023第5轮对全宽路面与踏面的交叉检查，原始复现与[逐项报告](../evidence/TASK-023/r5/road-width-remaining.json)归 `todo/evidence/TASK-023/r5/`，本任务开始后将修复批次和复验记录归 `todo/evidence/TASK-025/`。报告记录源hash及数值检查范围；候选需结合真实几何逐项处理，不等同于全部已定位的视觉缺陷
