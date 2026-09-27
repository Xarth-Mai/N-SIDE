---
id: TASK-025
type: fix
status: active
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

第3轮，步骤6/6 记录与后续修复；校园北缘两段台阶、采购街早餐铺入口和market_turn转角已修复，实际看图补齐学校被前景挡住的下段。全城候选98→85，移除13对且0新增，短登高路线继续为0；全城扫描仍为FAIL。下一批处理音乐街配送路与西侧步行接入，人物碰撞和通行尚未验收

## 结果与证据

问题来源为TASK-023第5轮对全宽路面与踏面的交叉检查，原始复现与[逐项报告](../evidence/TASK-023/r5/road-width-remaining.json)归 `todo/evidence/TASK-023/r5/`，本任务开始后将修复批次和复验记录归 `todo/evidence/TASK-025/`。报告记录源hash及数值检查范围；候选需结合真实几何逐项处理，不等同于全部已定位的视觉缺陷

第1轮[修复与复验](../evidence/TASK-025/r1/review.md)记录冻结源、逐节点变更、剩余98对候选、6项几何测试与CPU Viewer校验。建筑、入口、平台、稳定对象ID和短登高路径保留；画面与人物通行分别记录，不以机器校验代签

[挡墙修复与8机位复验](../evidence/TASK-025/r2/review.md)保留首次视觉FAIL及修后同机位画面，机器退出码与实际观察分开记录

第3轮[校园与采购街源修复及运行复验](../evidence/TASK-025/r3/review.md)记录原98对至85对、逐节点变化、初稿失败、最终源hash与4个实际机位；与第三停步台近坡修改整合后的检查见TASK-023/r6
