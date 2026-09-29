# 复杂度审查

初次发现：`game/capture/walk-shop-exterior.json:L40: shrink:` 42 个 InputSpan 默认字段可省略，`#[serde(default)]` 已提供相同值，净减少 96 行

主 Agent 已删除重复默认值；独立逐段补回默认后核对输入等价，新增三项位移条件直接使用已有断言能力，没有增加配置层、工具或依赖。其余已审实现沿用现有盒体批处理、StandardMaterial、导出器与 capture，未找到有价值的进一步删减

Lean already. Ship.
