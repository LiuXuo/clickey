# Clickey — Agent Guide (Single Source of Truth)

本文件是给“大模型/智能体”看的项目唯一执行指南（除 `README.md` 面向人类以外）。

## 强约束（非常重要）

1. **本仓库只维护两份文档**：`README.md`（人类）与 `AGENT.md`（智能体）。
   - 不要新增 `docs/`、`ROADMAP.md`、`CHECKLIST.md`、看板/验收/任务拆分等散落文档。
   - 需要新增说明时：优先更新本文件相应章节。
2. **架构边界不可打破**：核心逻辑平台无关；原生层最小化；遮罩渲染不抢焦点、不读键盘。
3. **当前阶段“文档先行”**：除非明确指令开始落地代码，否则不做大规模代码实现。

---

## 0. 项目一句话

Clickey 是一个“键盘驱动的分层网格定位”工具：热键激活全屏透明遮罩 → 按键逐层缩小区域 → 自动移动鼠标到区域中心并点击。

---

## 1. 现状与原型真相（不要凭空改交互）

当前行为基准来自 `demo/` 下的版本化 AutoHotkey v1 原型脚本（它们是事实标准）：

- `demo/clickey_v1.0.ahk`：原 `clickey.ahk`，3x3 方案（多层、键位少）
- `demo/clickey_v1.1.ahk`：原 `clickeyy.ahk`，5x5 方案（层数少、定位更细）
- `demo/clickey_v2.x.ahk` / `demo/clickey_v3.x.ahk`：后续迭代
- **当前基准**：`demo/clickey_v3.1.ahk`（后续工作以此为准）

完整版本说明见 `demo/clickey.md`。

### 1.1 通用交互（以 v3.1 为准）

- 激活：`Ctrl+;`（左键） / `Ctrl+Shift+;`（右键） / `Ctrl+Shift+Alt+;`（中键）
- 取消：`Esc`（直接退出，不点击）
- 回退：`Backspace`（撤销最近一次按键，恢复上一次 Region）
- 直达：`Space`（直接点击当前 Region 中心点，跳过后续层级）
- 切换显示器：`Tab`（多屏时）
- 单键层微调：方向键（`Up/Down/Left/Right`，5px 步长，仅单键层）

### 1.2 坐标系与多显示器

- v3.1 初始 Region 取当前显示器（`Monitor` 信息），多屏可用 `Tab` 轮换。
- v1.x 使用 VirtualScreen（`SysGet` 76~79）作为初始 Region（历史参考）。
- 所有裁剪都发生在“屏幕像素坐标”的 Region 上。
- v3.1 的几何按原始像素绘制，字体按 DPI 缩放，减少多屏 DPI 偏移。

> 智能体在重写/抽取算法时：**计算逻辑与渲染逻辑必须拆开**。核心引擎只在像素坐标里做 Region 变换；渲染层自行处理 DPI 与窗口坐标换算。

---

## 2. 选型（锁定，不要在实现过程中随意改）

### 2.1 Desktop Shell

- **Tauri v2**

### 2.2 UI（配置界面 + 遮罩渲染）

- **Svelte + Vite**
- **TypeScript**
- **Tailwind CSS + shadcn-svelte**
- 遮罩渲染：Canvas 或 SVG（优先 Canvas，便于大量网格绘制）

### 2.2.1 设置页（Settings WebView）

除遮罩外，正式版本还会提供一个“设置页面”，它本质上是一个**普通 WebView 窗口**（可获取焦点、可交互），用于编辑配置与管理预设：

- 入口：**应用托盘（system tray）右键菜单**唤出（或切换显示）。
- 职责：配置编辑、预设管理、导入导出、热键冲突提示等。
- 边界：设置页不持有业务状态机；只读写配置并触发“应用配置”。

> 遮罩窗口（Overlay）与设置页（Settings）必须是两套不同的窗口策略：Overlay 必须 click-through/不抢焦点；Settings 必须可交互/可聚焦。

### 2.3 Native Layer（Rust）

- 全局热键：`global-hotkey`（候选）
- 鼠标控制：`enigo`（候选）
- 屏幕信息：`display-info`（候选）
- 托盘：Tauri/system tray 能力（最终以 Tauri v2 实际 API 为准）

> 是否最终采用这些 crate 允许调整，但调整必须在本文件“变更记录”里写明原因与替代方案。

### 2.4 测试与质量（目标，不一定立即落地）

- Core Engine（TS 纯逻辑）：单测覆盖率目标 **100%**
- Rust：`cargo fmt`、`cargo clippy`、`cargo test`
- 前端：Prettier、ESLint

---

## 3. 架构（边界与职责）

目标分层（禁止跨层“偷懒”）：

1. **Core Engine（TypeScript，纯逻辑）**
   - 输入：按键事件（抽象 KeyCode）、当前状态、当前 Region、配置
   - 输出：下一状态、下一 Region、是否完成、最终点击点
   - 特性：无副作用、可序列化、可单测
2. **Overlay Renderer（Svelte）**
   - 只根据 Runtime State 渲染：网格线、标签、高亮、层级提示
   - 不抢焦点、不读键盘、不修改状态
3. **Settings UI（Svelte，普通 WebView）**
   - 提供配置编辑器（表单/预设/导入导出）
   - 只产出“配置变更事件”，不直接调用鼠标/热键等系统能力
4. **Native Layer（Rust）**
   - 只做系统能力：监听全局热键、读屏幕/DPI/多显示器、执行鼠标移动与点击
   - 不实现网格裁剪/状态机业务逻辑
5. **Desktop Shell（Tauri）**
   - 负责窗口与通信（Overlay/Settings）、托盘菜单、连接 UI/Core/Native
   - 负责把配置下发给 Native（注册热键）与 Core（运行时行为）

---

## 4. 核心概念与术语（统一用词，避免文档与实现分叉）

- **Region**：当前可选区域（像素坐标）
  - `{ x, y, width, height }`
- **Grid**：把 Region 切成 `rows x cols`
- **Preset**：一个可命名的交互方案（包含一组 `layers`），由 Settings 维护与切换
- **Layer**：一层交互（可能包含多步）
- **Step**：一次按键输入（会把 Region 收缩一次）
- **Mode**
  - `single`：一次按键对应一个 `rows x cols` 网格裁剪
  - `combo`：两段式（先选一维，再选另一维），本质仍是两次裁剪
- **Stage（仅 combo）**
  - Stage 0：选择“块”（第一步）
  - Stage 1：选择“块内格”（第二步）

---

## 5. 算法规格（可直接转为 Core Engine 代码）

### 5.1 单步裁剪（通用）

给定：

- `currentRegion`
- `rows, cols`
- `keyIndex`（1-based，按键在 keys 列表里的位置）

计算：

- `row = ceil(keyIndex / cols)`
- `col = ((keyIndex - 1) % cols) + 1`
- `cellWidth = currentRegion.width / cols`
- `cellHeight = currentRegion.height / rows`
- 返回新 Region：
  - `x = currentRegion.x + (col - 1) * cellWidth`
  - `y = currentRegion.y + (row - 1) * cellHeight`
  - `width = cellWidth`
  - `height = cellHeight`

点击点（最终输出）：

- `centerX = round(x + width / 2)`
- `centerY = round(y + height / 2)`

### 5.2 v3.1 基准的“分层步骤表”（当前事实标准）

键位（固定顺序先列后行）：

列键（15）：

- `q a z w s`
- `x e d c r`
- `f v t g b`

行键（15）：

- `y h n u j`
- `m i k , o`
- `l . p ; /`

单键层 3x5：

- `q w e r t`
- `a s d f g`
- `z x c v b`

步骤（共 3 次按键）：

1. combo stage 0：用“列键”选列（裁剪一次）
2. combo stage 1：用“行键”选行（裁剪一次）
3. single：用单键层 3x5（裁剪一次）

补充：

- 仅单键层支持方向键 5px 微调。
- 多显示器可用 `Tab` 切换当前屏幕。

### 5.3 v1.0（原 `clickey.ahk` → `demo/clickey_v1.0.ahk`）的“分层步骤表”（历史）

键位：

- 行键（9）：`w e r / s d f / x c v`
- 列键（9）：`u i o / j k l / m , .`

步骤（共 4 次按键）：

1. combo stage 0：用“行键”选 3x3 块（裁剪一次）
2. combo stage 1：用“列键”选块内 3x3（裁剪一次）
3. single：用行键 3x3（裁剪一次）
4. single：用列键 3x3（裁剪一次）

> 注意：v1.0 在 combo stage 0 时，输入允许的是“行键集合”；combo stage 1 时允许的是“列键集合”。

### 5.4 v1.1（原 `clickeyy.ahk` → `demo/clickey_v1.1.ahk`）的“分层步骤表”（历史）

键位 5x5：
`q w e r t / y u i o p / a s d f g / h j k l ; / z x c v b`

步骤（共 3 次按键）：

1. combo stage 0：5x5（裁剪一次）
2. combo stage 1：5x5（裁剪一次）
3. single：5x5（裁剪一次）

---

## 6. 配置模型（建议的最小 JSON 形态）

> 这是“将来要实现”的目标配置结构，用于约束讨论与任务拆分；当前不要求落地代码。

```json
{
  "app": {
    "tray": {
      "enabled": true
    },
    "settingsWindow": {
      "openFromTray": true
    }
  },
  "hotkeys": {
    "activation": {
      "leftClick": "Ctrl+;",
      "rightClick": "Ctrl+Shift+;",
      "middleClick": "Ctrl+Shift+Alt+;"
    },
    "controls": {
      "cancel": "Esc",
      "undo": "Backspace",
      "directClick": "Space"
    }
  },
  "activePresetId": "3x3-default",
  "presets": [
    {
      "id": "3x3-default",
      "name": "3x3 (default)",
      "layers": [
        {
          "mode": "combo",
          "stage0": {
            "rows": 3,
            "cols": 3,
            "keys": ["w", "e", "r", "s", "d", "f", "x", "c", "v"]
          },
          "stage1": {
            "rows": 3,
            "cols": 3,
            "keys": ["u", "i", "o", "j", "k", "l", "m", ",", "."]
          }
        },
        {
          "mode": "single",
          "rows": 3,
          "cols": 3,
          "keys": ["w", "e", "r", "s", "d", "f", "x", "c", "v"]
        },
        {
          "mode": "single",
          "rows": 3,
          "cols": 3,
          "keys": ["u", "i", "o", "j", "k", "l", "m", ",", "."]
        }
      ]
    },
    {
      "id": "5x5-default",
      "name": "5x5 (default)",
      "layers": [
        {
          "mode": "combo",
          "stage0": {
            "rows": 5,
            "cols": 5,
            "keys": [
              "q",
              "w",
              "e",
              "r",
              "t",
              "y",
              "u",
              "i",
              "o",
              "p",
              "a",
              "s",
              "d",
              "f",
              "g",
              "h",
              "j",
              "k",
              "l",
              ";",
              "z",
              "x",
              "c",
              "v",
              "b"
            ]
          },
          "stage1": {
            "rows": 5,
            "cols": 5,
            "keys": [
              "q",
              "w",
              "e",
              "r",
              "t",
              "y",
              "u",
              "i",
              "o",
              "p",
              "a",
              "s",
              "d",
              "f",
              "g",
              "h",
              "j",
              "k",
              "l",
              ";",
              "z",
              "x",
              "c",
              "v",
              "b"
            ]
          }
        },
        {
          "mode": "single",
          "rows": 5,
          "cols": 5,
          "keys": [
            "q",
            "w",
            "e",
            "r",
            "t",
            "y",
            "u",
            "i",
            "o",
            "p",
            "a",
            "s",
            "d",
            "f",
            "g",
            "h",
            "j",
            "k",
            "l",
            ";",
            "z",
            "x",
            "c",
            "v",
            "b"
          ]
        }
      ]
    }
  ],
  "overlay": {
    "alpha": 120,
    "maskColor": "#000000",
    "lineColor": "#ffffff",
    "textColor": "#ffffff",
    "lineWidthPx": 1,
    "font": {
      "family": "Segoe UI",
      "sizePx": 12
    }
  }
}
```

关键原则：

- **配置是单一事实来源**（UI 只是配置编辑器）。
- Core Engine 只依赖配置与输入事件，不读取 OS。
- **“设置页/托盘”也是配置入口的一部分**：Settings 只负责编辑配置与触发应用；不直接参与 Overlay 的事件循环。
- **预设是定制化的单位**：Settings 管理 `presets[]` 与 `activePresetId`；Overlay 只消费“当前运行时配置”。

---

## 7. 任务拆分（精简版）

我们不维护 100+ 细颗粒任务清单；只保留“可执行、可审阅、可跟踪”的最小任务面板。

### 7.1 Kanban（在本文件更新）

状态定义：

- `Backlog`：未开始
- `In Progress`：智能体执行中
- `Review`：等待人工审阅
- `Done`：审阅通过

### 7.2 当前任务面板

> 更新规则：每次完成一个任务，把它从一个栏目移动到下一个栏目，并在“变更记录”里写 1~3 行。

#### Backlog

- E0.1 明确 MVP 边界（3x3/5x5 默认、是否支持自定义、Windows 优先级）
- E0.2 产出 Core Engine 规格（从原型抽象成可测试接口与状态机定义）
- E2 Rust Native PoC（热键/鼠标/屏幕信息在 Windows 可用）

#### In Progress

- （无）

#### Review

- （无）

#### Done

- E1 初始化 Tauri + Svelte + TS 工程骨架（空壳可跑）
- E3 遮罩窗口 PoC（透明/置顶/click-through/不抢焦点）
- AHK 行为原型可运行（版本化；基准 v3.1）
- 仓库文档收敛为 `README.md` + `AGENT.md`

---

## 8. 审阅清单（精简版）

每次交付（哪怕只是文档）都按以下顺序自检：

1. **一致性**：README 与 AGENT 的说法不矛盾；术语统一；交互不偏离 AHK 原型。
2. **可执行**：新人能按 README 跑起 AHK 演示；智能体能按 AGENT 找到下一步。
3. **边界清晰**：Core/Native/UI 的职责没有混写。
4. **可回滚**：改动集中、容易 review；不要一次把仓库“重构式大改”。

---

## 9. 变更记录（只记关键决策）

> 目的：让后续Agent知道“为什么这么做”，避免反复在同一问题上打转。

- 2026-02-10：将文档收敛为两份：`README.md`（人类）与 `AGENT.md`（智能体）。
- 2026-02-10: Initialized Tauri v2 + Svelte/TS skeleton; tray menu, overlay PoC, core engine unit test.
- 2026-02-17：AHK 原型版本化（v1.0/v1.1 对应原脚本），基准更新为 `demo/clickey_v3.1.ahk`，文档同步。
