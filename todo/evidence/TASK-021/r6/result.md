# Wiki 浏览器图标兼容

用户反馈 Cloudflare 域名的 favicon 未显示。实际请求中 SVG Logo 返回 200、Content-Type 为 image/svg+xml，根 favicon.ico 返回 404；首页返回带 cf-mitigated: challenge 的 403。现有 SVG 引用没有被目录迁移删除，缺失的是传统图标和 Apple 触屏图标，不能据此断言用户侧的唯一原因

从现有品牌 PNG 派生根 favicon.ico（16、32、48 像素）和 apple-touch-icon.png（180 像素），保留 SVG 主文件与原有构图。Wiki 显式发布两项资源，两种受众共用；构建测试检查资源是否导出及字节是否一致

Cloudflare 防护配置未改动。远端需推送后的自动部署完成，再验证图标资源；真实 Safari 收藏与浏览器图标显示尚未验收

简化审查（ponytail-review）：Lean already. Ship.

PASS：11 项 Wiki 测试、类型检查、文档与 Skills 检查、双站构建；两站 HTML 引用及派生文件字节一致，ICO 三个图层与 Apple 图标尺寸校验通过
