# Clickey

用键盘把鼠标“投送”到屏幕任意位置：按下热键 → 看到全屏网格遮罩 → 连续按几下键把区域逐层缩小 → 自动移动并点击目标点。

> 当前仓库阶段：**Tauri + Rust + Svelte 已独立演化**。`demo/` 下的 AHK 脚本仅作历史归档，不再作为后续迭代基准。

---

## 这是什么 / 解决什么问题

Clickey 的目标是提供一种高效、精确、可肌肉记忆的鼠标定位方式：

- **不离开键盘**：用固定键位序列完成“移动 + 点击”。
- **精确可控**：通过“分层网格裁剪”逐步缩小选择区域。
- **跨平台一致**：最终用 Web 技术统一 UI，Rust 只负责系统能力。

典型使用场景：

- IDE/浏览器里频繁点小按钮、标签页、面板折叠箭头
- 多显示器 + 高 DPI 下，鼠标移动成本很高
- 录屏演示/讲解时，需要“可预测”的鼠标轨迹与落点

---

## AHK 历史原型（归档）

仓库里保留了一套版本化的 AutoHotkey（AHK v1）脚本，集中在 `demo/`，用于历史回溯与行为对照：

- `demo/clickey_v1.0.ahk`：原 `clickey.ahk`，3x3 方案
- `demo/clickey_v1.1.ahk`：原 `clickeyy.ahk`，5x5 方案
- 后续版本：`demo/clickey_v2.x.ahk` / `demo/clickey_v3.x.ahk`
- **冻结版本**：`demo/clickey_v3.1.ahk`（此后不再继续迭代）

完整迭代说明见 `demo/clickey.md`；这些脚本仅作为历史参考，不再代表当前项目的实现真相。

---

## 快速开始（当前项目）

1. 安装依赖：Node.js、Rust、Tauri v2 所需系统依赖。
2. 安装前端依赖：`npm install`
3. 启动开发环境：`npm run tauri:dev`
4. 在 Settings 中调整配置，使用激活热键进入 Overlay 执行定位与点击。

如果你只想回看历史原型，可单独运行 `demo/clickey_v3.1.ahk`（需 AutoHotkey v1.1），但它不会再跟随主项目更新。

---

## Settings UI（Tauri 原型）

当前 Settings 窗口已提供表单式配置编辑，可读取/保存配置，重启保留。

可用能力：

- Layer 编辑：增删 / 排序 / mode 切换（single/combo）/ rows/cols/keys 修改 / auto-fit
- 热键编辑：activation + controls
- 鼠标行为：平滑移动、按压时长、落点随机、曲线/抖动、远距离提速与步进策略
- Overlay 样式：alpha/line width/font size + color picker
- 配置导入/导出：override JSON（仅包含与默认配置不同的字段）
- i18n：`zh-CN` / `en-US`，切换后设置页即时生效
- 托盘联动：
  - 左键单击托盘图标直接打开设置页
  - 右键菜单包含 `设置 / 暂停或启动 / 退出`
  - 右键菜单文本会跟随 i18n 语言实时变化

按钮：

- `Apply`：应用并保存配置
- `Reset to default`：恢复默认配置
- `Import / Export`：导入/导出 override JSON

> 当前配置以“默认配置 + 覆盖项”的方式保存：AppConfig 目录下的 `settings.override.json` 只记录与默认值不同的字段。

---

## 快捷键（以当前默认配置为准）

不同配置可在 Settings 调整，以下为当前默认值：

| 操作                                 | 快捷键 / 按键                                   |
| ------------------------------------ | ----------------------------------------------- |
| 激活（默认左键模式）                 | `Ctrl + ;`                                      |
| 切换点击动作（左键 -> 右键 -> 中键） | `hotkeys.controls.switchAction`（默认 `Enter`） |
| 取消并退出                           | `Esc`                                           |
| 回退一步（撤销上一按键）             | `Backspace`                                     |
| 直接点击当前区域中心（跳过后续层级） | `Space`                                         |
| 切换显示器（多屏）                   | `hotkeys.controls.nextMonitor`（默认 `Tab`）    |
| 单键层微调                           | `Up / Down / Left / Right`（5px）               |

---

## 它是怎么工作的（分层网格裁剪）

### 基本概念

- **Region（区域）**：当前可选范围（v3.1 初始为当前显示器；历史版本为虚拟屏幕）。
- **Grid（网格）**：把 Region 切成 `rows x cols` 个格子。
- **Key（按键）**：每个格子对应一个键；按下键就选中对应格子，Region 收缩到该格子。
- 重复上述过程，直到达到你希望的精度，然后点击 Region 中心点。

### 历史基线：v3.1（`demo/clickey_v3.1.ahk`，已冻结）

结构为 2 层：第 1 层为双键（固定顺序先列后行），第 2 层为单键 3x5。

列键（15）：

`q a z w s`  
`x e d c r`  
`f v t g b`

行键（15）：

`y h n u j`  
`m i k , o`  
`l . p ; /`

单键层 3x5：

`q w e r t`  
`a s d f g`  
`z x c v b`

流程：

1. combo 第 1 步：按“列键”选列（15 列）
2. combo 第 2 步：按“行键”选行（15 行）
3. single：单键 3x5 精细裁剪；可用方向键 5px 微调；`hotkeys.controls.nextMonitor`（默认 `Tab`）切换显示器

### v1.0（历史原型，原 `clickey.ahk` → `demo/clickey_v1.0.ahk`）

键位分两组：

- 行键（9 个）：`w e r / s d f / x c v`
- 列键（9 个）：`u i o / j k l / m , .`

流程（按键次数更多，但每一步键位更少、更好记）：

1. combo 第 1 步：按“行键”选一个 3x3 大块
2. combo 第 2 步：按“列键”在大块内再选 3x3
3. single：再用行键做一次 3x3 精细裁剪
4. single：再用列键做一次 3x3 精细裁剪

额外调试（仅 v1.0 提供）：

- `Ctrl + Alt + D`：显示屏幕/DPI/虚拟屏范围信息（便于排查多显示器与缩放问题）

### v1.1（历史原型，原 `clickeyy.ahk` → `demo/clickey_v1.1.ahk`）

键位为 5x5：

`q w e r t`  
`y u i o p`  
`a s d f g`  
`h j k l ;`  
`z x c v b`

流程（按键更少，但键位密度更高）：

1. combo 第 1 步：5x5 选块
2. combo 第 2 步：块内 5x5 再选一次
3. single：再 5x5 精细裁剪一次，然后点击

---

## 与 AHK 原型的关系（以及接下来的产品形态）

AHK 原型已完成历史使命。当前项目的唯一演进主线是 Tauri + Rust + Svelte 代码本身，后续行为与算法不再以 AHK 脚本为基准。

AHK 脚本保留价值仅限于：

- 回溯历史交互决策
- 对照某些旧行为与默认键位
- 排查“是否是历史遗留差异”这类问题

正式版本计划演进为：

- **Core Engine（TypeScript）**：纯逻辑、可单测的状态机与区域裁剪算法
- **Native Layer（Rust / Tauri）**：全局热键、鼠标控制、屏幕与 DPI 信息
- **UI（Svelte）**：两类 WebView
  - **Overlay（遮罩渲染）**：全屏透明、click-through、不抢焦点（Canvas/SVG）
  - **Settings（设置页）**：普通可交互窗口，用于 layers/热键/overlay 配置管理与 override 导入导出

---

## 路线图（高层）

更细的执行细节与任务拆分统一收敛在 `AGENTS.md`，这里只保留“对人类友好”的里程碑视图：

1. **M0：原型验证**（已具备）
   - AHK 原型归档完成（版本化；冻结于 v3.1）
   - 交互闭环：热键 → 遮罩 → 定位 → 点击
2. **M1：核心引擎抽象**
   - 把“区域裁剪 + 状态机 + undo”抽成可测试的 TS Core
3. **M2：桌面壳与原生能力**
   - Tauri 项目初始化
   - Rust：热键/鼠标/屏幕信息（Windows 优先）
4. **M3：遮罩与配置 UI**
   - 透明遮罩窗口
   - 配置持久化（override JSON）与 layers 编辑
   - 托盘菜单 + 设置页（WebView）联动
5. **M4：跨平台与发布**
   - macOS 权限与适配
   - Linux（X11 优先，Wayland 功能受限）

---

## 设置页与托盘（当前实现）

除“遮罩定位 + 点击”本体外，当前版本已提供 **托盘（system tray）入口**，用于“轻量控制”和“打开设置”：

- **托盘左键**：直接打开设置页。
- **托盘右键菜单**：`设置 / 暂停(或启动) / 退出`。
- **菜单 i18n 联动**：菜单文本跟随当前语言（`zh-CN` / `en-US`）即时更新。
- **设置窗口兜底**：即使设置窗口被关闭销毁，托盘“设置”仍会自动重建并打开。

可定制项（方向）：

- **交互热键**：单一激活热键（`hotkeys.activation.trigger`）与控制键（`cancel/undo/directClick/switchAction/nextMonitor`）。
- **分层与网格（`layers`）**：基于单一默认配置直接编辑层；每层可定义行列数（`rows x cols`）与模式（`single` / `combo`）。
- **按键映射**：每个（子）步骤都有自己的 `keys` 列表，决定 `keyIndex → row/col` 的映射。
- **鼠标策略（`mouse`）**：平滑移动、落点随机、曲率/抖动、时长与步进随机、远距离提速、自适应步长与步数上限。
- **遮罩外观**：透明度、线条颜色/粗细、文字颜色/字号等。

字段名约定（与 `AGENTS.md` 的配置模型一致）：

- 语言：`app.locale`
- 托盘显隐：`app.tray.enabled`
- 激活热键：`hotkeys.activation.trigger`
- 控制键：`hotkeys.controls.cancel` / `hotkeys.controls.undo` / `hotkeys.controls.directClick` / `hotkeys.controls.switchAction` / `hotkeys.controls.nextMonitor`
- 鼠标策略：`mouse.*`（`smoothMove` / `moveDurationMs` / `moveStepMs` / `pressDurationMs` / `landingRadiusPx` / `durationRandomness` / `stepRandomness` / `distanceBoostPx` / `durationDistanceBoost` / `stepDistanceBoost` / `curveAlongRatio` / `curveSpreadRatio` / `jitterRatio` / `adaptiveStrideBasePx` / `adaptiveStrideDistanceRatio` / `adaptiveStrideMaxPx` / `extraStepsMax` / `maxSteps` / `maxStepSleepMs`）
- 分层列表：`layers[]`
- 遮罩外观：`overlay.*`
- 覆盖配置文件：`settings.override.json`（仅记录与默认配置差异，支持导入/导出）

核心原则：

- **配置是单一事实来源**：设置页只编辑配置；不把业务状态“塞在 UI 里”。
- **操作解耦**：热键监听、区域裁剪（Core Engine）、遮罩渲染（Overlay）、鼠标执行（Native）彼此独立，才能支持高程度的定制化与后续扩展。

## 贡献（当前阶段）

目前更需要的是“把行为规格写清楚、把边界约束定死”，而不是堆功能：

- 交互细节（撤销、取消、直达点击）是否需要扩展
- 默认网格方案选择（3x3 vs 5x5，是否支持自定义）
- 目标平台优先级（Windows 优先 / 多平台并行）

如果你要让大模型接力开发/维护，请先阅读 `AGENTS.md`。

---

## License

MIT（见 `LICENSE`）。
