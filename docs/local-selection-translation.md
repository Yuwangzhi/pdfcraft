# 自动划词、截图翻译与公式排版

这是基于 PdfCraft v0.5.0 的独立维护分支，不是官方发行版。

用鼠标选择 PDF 中的文字，松开并停留约 300 毫秒，阅读器内会自动弹出简体中文译文，无需点击按钮。支持复制译文、重新翻译、移动和关闭弹窗；右键菜单、编辑菜单与 Cmd+Shift+T 也可触发。

## 翻译与公式

- 使用 DeepSeek `deepseek-flash`，关闭思考模式。按 2026-10-11 官方接口说明，该名称对应 V4.1 Flash；旧 `deepseek-v4-flash` 请求也已转向这一模型。
- 向 DeepSeek 发送所选文字及其边界附近的截图（边缘留 4 pt）。截图用于核对 PDF 提取时容易错乱的变量、上下标、分式、求和范围和公式编号；不上传 PDF 文件、文件名或路径。跨行选区的矩形可能包含周边文字。
- 单次最多选择 3000 个字符。需要网络和有余额的 API 账户，调用按 DeepSeek 规则计费。
- 译文中的数学式在本机通过 `latex-rust` 与 OFL 许可的 STIX Two Math 字体排版，显示为清晰公式。渲染结果在弹窗存续期间缓存，不会每帧重复排版；长公式可横向滚动。复制译文保留 LaTeX，便于粘贴到论文。
- 暂不支持的公式会显示明确提示，可展开查看表达式。不会把错误渲染伪装成正确公式。模型仍可能误读复杂公式，请与原文核对。
- 划词翻译依赖 PDF 文字层；扫描页或图片内容可直接用下方的截图翻译，无须先运行 OCR。

## 本机配置（macOS）

创建 `~/Library/Application Support/PdfCraft/private/deepseek.json`，内容如下，将示例占位符替换为自己的密钥：

```json
{"api_key": "YOUR_DEEPSEEK_API_KEY"}
```

`private` 目录权限应为 `0700`，配置文件权限为 `0600`。密钥不放入仓库、应用二进制或普通设置文件，不在进程参数、错误提示或日志中显示。当前安装已配置；仓库不包含任何真实密钥。

翻译请求、截图生成和公式排版均在后台执行。请求串行处理，选择变化时保留最后一个待翻译选区；连接超时 8 秒，总超时 45 秒，不自动切换到其他翻译服务。没有磁盘翻译历史，不修改 PDF。

## 构建

```sh
CRAFT_FONTS_DIR=../pdfcraft-build-fonts \
CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
cargo build --release --locked -p pdfcraft -p pdfcraft-cli
```

字体设置与可选的 craft-fonts 构建输入见 [字体修复说明](local-font-fix.md)。这份分支移除了上游不许可 fork 沿用的品牌图片；保留上游来源与许可说明。

## 自动化

`text_translate` 显式要求 `allow_network=true`，可选 `doc`、`page`（从 1 开始）及 `rect=[x0,y0,x1,y1]`（页面左上角为原点，单位 pt）附带选区截图。

```sh
pdfcraft-cli run text_translate 'text=The recurrence is x_{t+1}=a x_t.' allow_network=true
pdfcraft-cli run text_render_math 'text=递推关系：\[x_{t+1}=a x_t\]' --out formula.png
```

`text_render_math` 完全离线，返回按原顺序排列的文字、公式元信息和 PNG。引擎接口为 `translation::to_chinese_with_image` 和 `translation_math::render`。

## 验证

2026-10-11 已在隔离设置的原生测试应用中划选自制英文及数学样例：中文自动出现；公式中的分式、求和上下限、希腊字母、下标和编号准确恢复并实际排版显示。源文档保持未修改。测试没有上传用户研究文档。

回归测试覆盖输入边界、错误响应、公式原文保留、网络显式启用、自动触发去抖及最新选区队列、公式离线渲染与不支持语法。真实 API 测试是显式启用的 ignored test，内容只有自制样例。

后续官方更新可能覆盖这个本地扩展；需要时请从本分支重新构建。

## 截图翻译与关闭弹窗

- 阅读器内按 `⌘⇧X`（默认）或“编辑 → 截图翻译为中文”，在 PDF 页面上拖动框选；松开后自动翻译，无需再点按钮。适用于扫描件、图片内文字和公式；只发送框选区域。
- “偏好设置 → 截图翻译快捷键 → 修改快捷键”，直接按新的组合键。设置自动保存；已有应用快捷键冲突会提示，Esc 取消录入。快捷键仅在阅读器内生效。
- 截图或划词译文都先渲染公式再显示；点击弹窗外任意位置或按 Esc 关闭。关闭后网络请求即使完成，也不会重新弹出同一译文。
- 框选模式按 Esc 退出，完成一次截图后自动回到文字选择工具。
- 自动化：`region_translate` 接受 `doc`、从 1 开始的 `page`、页面点坐标 `rect: [x0,y0,x1,y1]` 和 `allow_network: true`，无须 PDF 文字层。
- 控制接口可用 `set key=translation-shortcut value=Alt+Shift+T` 设置快捷键，`command id=edit.translate_region` 启动框选。
