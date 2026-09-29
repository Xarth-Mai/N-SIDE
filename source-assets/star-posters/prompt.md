# 安可海报 r2 生成记录

工具：当前 Codex 内置 `image_gen`，2026-09-30，`transparent_background: false`，修改模式，未调用独立付费 API

作者要求整体日漫感，指定仓库镜厅预览图左上竖版角色海报作为画法基准。两张输入图均已实际打开：第一张仅参考其中角色海报的画法，第二张保留安可的身份特征与构图，不保留被否定的柔光画法

| 输入 | 用途 | SHA-256 |
| --- | --- | --- |
| `source-assets/area-previews/cinema-music-street.png` | 画法参考，左上角色竖海报 | `0997197f95ee0cd2d63836c6aafdfc09b42e25e4078ec8a59cb95cfbc3b7eb68` |
| r1 原图，现为 `rejected/anke-portrait-r1.png` | 身份和构图参考、重绘目标 | `4e999ad07a35cfa81bf9c7ac9fa97e11bdd2e0df5bb6f03903a4315703cf70a2` |

生成结果文件：`exec-96de1adc-725c-4798-a458-37cfc62b61b3.png`，选定原件复制为本目录 `anke-portrait.png`，实际输出 `1672 × 941`。模型版本、种子、收费、生成服务任务 ID 未返回；文件名不是服务任务 ID。生成图像不含文字，文字由 `anke-sunny.svg` 管理

以下为实际提交提示词

```text
Use case: style-transfer. Redraw image 2 from scratch into clean Japanese 2D anime promotional art. Image 1 is STYLE REFERENCE ONLY: look specifically at the big vertical anime character poster at the far upper LEFT of the cinema facade, above the sidewalk poster cases. That poster has visible dark contour lines, a simplified angular anime face, grouped graphic hair shapes, broad flat color areas and sparse crisp cel-shadow shapes. Match that hand-drawn anime poster approach, NOT the realistic lighting, buildings or city scene around it. Do not copy the anonymous dark-haired character or any text in image 1. Image 2 is the EDIT TARGET and the composition/identity reference only; its soft painterly semi-realistic rendering was rejected. Completely remove its airbrushed skin gradients, soft bloom rim lighting, many fine hair strands, detailed glossy eyes, photographic bokeh and elaborate clothing texture. Retain this original fictional singer Anke: pink waist-length hair in large clean grouped locks, a simple white fabric hair clip on one side, amber anime eyes, cheerful lively smile, microphone in one hand, normal modest youthful urban performance clothes, ivory cropped jacket with charcoal crewneck top and subtle coral accents. She is actually 19; the user wants a youthful late-teen visual design around 17, nonsexualized. Visible dark ink contours on face, hair and clothing; small simple nose and mouth, one flat skin base plus one clean shadow tone, solid hair base plus one shadow tone with a few graphic highlights. Eyes should be recognizable 2D anime eyes with simple iris color, not gem-like detailed shading. Hair silhouette graphic and readable, very few loose strands. Jacket folds as a handful of clear outlined shapes. It should look like finished anime key animation / illustrated movie poster art, not a painting, not 3D, not a beauty portrait, not a cel-shading filter over image 2. Preserve a wide 16:9 composition with the character mainly on the right two thirds and plenty of completely usable dark graphic negative space on the left third for later typography. Replace background bokeh with sparse flat coral/pink and charcoal abstract graphic shapes and minimal music-poster accents. No letters, text, numbers, logos, watermarks, date, tickets, buildings or billboard mockup. Keep full head and hand within frame. Output only flat final poster artwork.
```
