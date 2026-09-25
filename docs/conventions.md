# 项目约定

## 文件归属

| 位置 | 内容 |
| --- | --- |
| `docs/` | 长期项目知识、完整设定集与 Wiki 页面 |
| `todo/` | 当前开发任务与交付记录 |
| `.agents/skills/` | Agent 工作流程 |
| `source-assets/` | 项目级共享资产与 Blender、Krita、音频工程等唯一源文件 |
| `docs/public/images/` | Wiki 使用的导出图片 |
| `tools/` | 检查、转换与测试工具 |
| `game/` | 游戏源码、运行资产、工程配置与依赖锁文件 |
| `AGENTS.md` | Agent 总入口 |
| `README.md` | GitHub 仓库与开发者入口 |
| `docs/index.md` | Wiki 读者入口 |
| `package.json`、`bun.lock` | Wiki 命令、依赖与锁定结果 |

`docs/` 是项目设计与设定的 single source of truth。每个主题在所属文件中维护完整正文，相关内容通过相对链接与稳定 ID 关联

人物与地点档案记录设定和制作需求；玩法文件记录公共交互与状态；具体任务记录事件流程、对白和后果。当前未决问题、下一步工作与验证结果记录在 `todo/`，确认后更新所属主题正文

## Markdown 与发布

正文使用中文，技术名词、标识符与命令使用英文。文本采用 UTF-8、LF，普通文件名采用 ASCII `kebab-case`。正文以项目自身的设定与设计原则直接表述，将创作理念融入相应主题。开发任务记录当前目标与实际结果；素材来源与许可信息按实际情况保留

Wiki 分类入口使用 `index.md`。正文采用标准 Markdown，页面链接使用相对 `.md` 路径。Wiki 专用图片使用以质量参数 `80` 编码的 WebP，保存在 `docs/public/images/`。项目级共享资产遵循[资产管理](production/assets.md)，Wiki 通过 `/project-assets/` 引用。模板占位符使用行内代码或代码块

Wiki 构建流程见[Wiki](production/wiki.md)

## 对象与 ID

| 对象 | ID |
| --- | --- |
| 游戏任务、敌人、角色、地点 | `QST-001`、`ENM-001`、`CHR-001`、`LOC-001` |
| 开发任务 | `TASK-001` |
| 资产 | `AST-001` |
| 委托内的 Beat、信息、选择、场景、台词 | `QST-001-B001`、`QST-001-I001`、`QST-001-C001`、`QST-001-S001`、`QST-001-L001` |

对象改名沿用原 ID。短档案使用一个 Markdown；包含对白、流程数据或资产清单时使用对象目录，以 `README.md` 为入口。模板中的 `000` 与 `{{...}}` 为占位

对象的元数据示例：

```yaml
---
id: ENM-001
status: draft
depends_on: []
---
```

`draft` 为草案，`approved` 为已定设计，`superseded` 为已替代设计。系统文档可使用 `DOC-*` ID。`depends_on` 引用前置文档或对象 ID

## 数据与记录

正式台词保存在任务的 `dialogue.csv`，数值保存在游戏数据，资产关系保存在 `asset-manifest.json`。内容档案描述意图与规则，交付记录保存在对应开发任务
