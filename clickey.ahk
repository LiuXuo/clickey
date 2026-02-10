#NoEnv
#SingleInstance Force
#MaxThreadsPerHotkey 1
SetBatchLines, -1
ListLines, Off
CoordMode, Mouse, Screen

; ========================== 说明 ==========================
; 1) 现为 3 层（合并1+2 为 9x9 双键；第3/4层为 3x3 单键）：
;    行键：wer / sdf / xcv
;    列键：uio / jkl / m,.
;    双键组合定位单元（行键+列键）
;    第3层单键：wer / sdf / xcv
;    第4层单键：uio / jkl / m,.
; 3) 按键：Ctrl+; 左键 / Ctrl+Shift+; 右键 / Ctrl+Shift+Alt+; 中键
; 4) Esc 取消；Backspace 回退上一个按键（无按键时退出）；Space 直接点击当前区域中心点

; ========================== 运行状态 ==========================
global g_active := false ; 是否已进入选择模式
global g_button := ""
global g_step := 0 ; 当前按键步骤(0-based)
global g_stage := 0 ; 0=行键 1=列键 2=单键
global g_screen := {} ; 屏幕坐标(虚拟屏)
global g_region := {} ; 当前可选区域
global g_keys := [] ; 当前层级的按键列表
global g_keyMap := {} ; 当前层级按键->索引
global g_rowKeyMap := {} ; 行键->索引
global g_colKeyMap := {} ; 列键->索引
global g_selectedRowKey := "" ; 已选行键(列键阶段使用)
global g_layers := [] ; 每层定义 {mode, ...}
global g_layerCount := 0
global g_steps := [] ; 步骤表：{layerIndex, mode, stage, stepInLayer, stepsInLayer}
global g_stepHistory := [] ; 按键栈：Backspace 回退上一个按键
; ========================== 外观配置 ==========================
global g_alpha := 120 ; 遮罩透明度(0-255)，越大越不透明
global g_maskColor := "000000" ; 遮罩颜色
global g_lineColor := "FFFFFF" ; 网格线颜色
global g_textColor := "FFFFFF" ; 字体颜色
global g_guiName := "Clickey"
global g_hwnd := 0
global g_guiScale := 1.0 ; DPI缩放系数 = A_ScreenDPI / 96
global g_line := 1 ; 网格线粗细(像素)
global g_rows := 9 ; 行数(由步骤切换)
global g_cols := 9 ; 列数(由步骤切换)
global g_font_size := 12 ; 当前层级字号(运行时切换)

Clickey_Init()
return

^;::Clickey_Start("Left")            ; Ctrl+; 左键
^+;::Clickey_Start("Right")          ; Ctrl+Shift+; 右键
^+!;::Clickey_Start("Middle")        ; Ctrl+Shift+Alt+; 中键
^!d::Clickey_Debug() ; Ctrl+Alt+D 调试信息

Clickey_Start(button) {
    global g_active, g_button, g_step, g_region, g_screen, g_layerCount, g_steps
    global g_selectedRowKey, g_stepHistory
    if (g_active)
        return

    ; 更新屏幕范围与初始区域
    Clickey_UpdateScreen()
    g_active := true
    g_button := button
    g_step := 0
    g_selectedRowKey := ""
    g_region := {x: g_screen.x, y: g_screen.y, w: g_screen.w, h: g_screen.h}

    KeyWait, Ctrl
    KeyWait, Shift
    KeyWait, Alt
    Sleep, 30

    g_stepHistory := [] ; 清空按键历史栈
    totalSteps := g_steps.Length()
    stepIndex := 1
    while (stepIndex <= totalSteps) {
        g_step := stepIndex - 1
        Clickey_SetLayoutForStep(stepIndex)
        Clickey_ShowOverlay()

        key := Clickey_ReadKey()
        if (key = "") {
            Clickey_HideOverlay()
            g_active := false
            return
        }
        if (key = "__BACK__") {
            ; Backspace 回退上一个按键
            if (g_stepHistory.Length() < 1) {
                Clickey_HideOverlay()
                g_active := false
                return
            }
            prev := g_stepHistory.Pop()
            g_region := {x: prev.x, y: prev.y, w: prev.w, h: prev.h}
            g_selectedRowKey := prev.rowKey
            stepIndex := prev.step
            continue
        }
        if (key = "__SPACE__") {
            ; Space 直接点击当前区域中心点，跳过后续层级
            Clickey_HideOverlay()
            Clickey_DoClick()
            g_active := false
            return
        }
        ; 入栈当前状态，便于按键级回退
        stepDef := g_steps[stepIndex]
        g_stepHistory.Push({step: stepIndex, x: g_region.x, y: g_region.y, w: g_region.w, h: g_region.h, rowKey: g_selectedRowKey})
        Clickey_ApplyKey(key)
        if (stepDef.mode = "combo" && stepDef.stage = 0)
            g_selectedRowKey := key
        else
            g_selectedRowKey := ""
        stepIndex += 1
    }

    Clickey_HideOverlay()
    Clickey_DoClick()
    g_active := false
}

Clickey_ReadKey() {
    global g_keyMap
    Loop {
        Input, key, L1, {Esc}{Backspace}{Space}
        if (ErrorLevel = "EndKey:Escape")
            return ""
        if (ErrorLevel = "EndKey:Backspace")
            return "__BACK__"
        if (ErrorLevel = "EndKey:Space")
            return "__SPACE__"
        ; 仅接受当前层级定义的按键
        StringLower, key, key
        if (g_keyMap.HasKey(key))
            return key
        SoundBeep, 900, 40
    }
}

Clickey_ApplyKey(key) {
    global g_region, g_stage, g_rowKeyMap, g_colKeyMap, g_keyMap, g_rows, g_cols
    ; 将按键映射到网格坐标，并收缩可选区域（行键选3x3块，列键选块内3x3）
    if (g_stage = 0) {
        idx := g_rowKeyMap[key]
        blockRow := Ceil(idx / 3)
        blockCol := Mod(idx - 1, 3) + 1
        cellW := g_region.w / 3
        cellH := g_region.h / 3
        g_region.x := g_region.x + (blockCol - 1) * cellW
        g_region.y := g_region.y + (blockRow - 1) * cellH
        g_region.w := cellW
        g_region.h := cellH
    } else if (g_stage = 1) {
        idx := g_colKeyMap[key]
        row := Ceil(idx / 3)
        col := Mod(idx - 1, 3) + 1
        cellW := g_region.w / 3
        cellH := g_region.h / 3
        g_region.x := g_region.x + (col - 1) * cellW
        g_region.y := g_region.y + (row - 1) * cellH
        g_region.w := cellW
        g_region.h := cellH
    } else {
        idx := g_keyMap[key]
        row := Ceil(idx / g_cols)
        col := Mod(idx - 1, g_cols) + 1
        cellW := g_region.w / g_cols
        cellH := g_region.h / g_rows
        g_region.x := g_region.x + (col - 1) * cellW
        g_region.y := g_region.y + (row - 1) * cellH
        g_region.w := cellW
        g_region.h := cellH
    }
}

Clickey_DoClick() {
    global g_region, g_button
    ; 最终点击当前区域中心点
    cx := Round(g_region.x + g_region.w / 2.0)
    cy := Round(g_region.y + g_region.h / 2.0)
    MouseMove, %cx%, %cy%, 0
    Click, %g_button%
}

Clickey_ShowOverlay() {
    global g_guiName, g_hwnd, g_screen, g_region, g_step, g_alpha, g_keys, g_guiScale, g_line, g_layerCount, g_steps
    global g_maskColor, g_lineColor, g_textColor, g_rows, g_cols, g_font_size
    Clickey_HideOverlay()

    Gui, %g_guiName%:New, +AlwaysOnTop -Caption +ToolWindow +LastFound
    g_hwnd := WinExist()
    Gui, %g_guiName%:Color, %g_maskColor%
    Gui, %g_guiName%:Margin, 0, 0
    Gui, %g_guiName%:Font, s%g_font_size% c%g_textColor%, Segoe UI

    stepIndex := g_step + 1
    stepDef := g_steps[stepIndex]
    layerIndex := stepDef.layerIndex
    msg := "层 " layerIndex "/" g_layerCount " 键 " stepDef.stepInLayer "/" stepDef.stepsInLayer " (Esc取消 / Backspace回退 / Space点击)"
    Gui, %g_guiName%:Add, Text, x10 y10 w300 h24 +BackgroundTrans, %msg%

    ; 按DPI缩放系数换算GUI坐标
    scale := (g_guiScale > 0) ? g_guiScale : 1.0
    ox := Round((g_region.x - g_screen.x) / scale)
    oy := Round((g_region.y - g_screen.y) / scale)
    rw := Round(g_region.w / scale)
    rh := Round(g_region.h / scale)
    ox2 := ox + rw - 1
    oy2 := oy + rh - 1
    cellW := rw / g_cols
    cellH := rh / g_rows

    sx := Round(g_screen.x / scale)
    sy := Round(g_screen.y / scale)
    sw := Round(g_screen.w / scale)
    sh := Round(g_screen.h / scale)
    line := g_line
    ; 先画线再画字，避免线条遮挡文字
    Clickey_AddLine(ox, oy, rw, line)
    Clickey_AddLine(ox, oy, line, rh)
    Clickey_AddLine(ox, oy2, rw, line)
    Clickey_AddLine(ox2, oy, line, rh)

    Loop, % (g_cols - 1) {
        lx := Round(ox + cellW * A_Index)
        Clickey_AddLine(lx, oy, line, rh)
    }
    Loop, % (g_rows - 1) {
        ly := Round(oy + cellH * A_Index)
        Clickey_AddLine(ox, ly, rw, line)
    }

    Loop, %g_rows% { ; 逐格绘制字母
        row := A_Index
        Loop, %g_cols% {
            col := A_Index
            idx := (row - 1) * g_cols + col
            label := g_keys[idx]
            StringUpper, label, label
            tx := Round(ox + (col - 1) * cellW)
            ty := Round(oy + (row - 1) * cellH)
            tw := Round(cellW)
            th := Round(cellH)
            Gui, %g_guiName%:Add, Text, x%tx% y%ty% w%tw% h%th% Center +BackgroundTrans +0x200, %label%
        }
    }

    Gui, %g_guiName%:Show, x%sx% y%sy% w%sw% h%sh% NoActivate
    WinSet, Transparent, %g_alpha%, ahk_id %g_hwnd%
    Clickey_DrawDiagonals(ox, oy, ox2, oy2)
}

Clickey_Debug() {
    ; 调试：查看当前屏幕/缩放信息
    SysGet, monCount, MonitorCount
    SysGet, vx, 76
    SysGet, vy, 77
    SysGet, vw, 78
    SysGet, vh, 79
    msg := "A_ScreenWidth=" A_ScreenWidth "`nA_ScreenHeight=" A_ScreenHeight
    . "`nA_ScreenDPI=" A_ScreenDPI
    . "`nGuiScale=" ((A_ScreenDPI > 0) ? (A_ScreenDPI / 96.0) : 1.0)
    . "`nMonCount=" monCount
    . "`nVirtualScreen x=" vx " y=" vy " w=" vw " h=" vh
    MsgBox, 64, Clickey Debug, %msg%
}

Clickey_HideOverlay() {
    ; 销毁遮罩
    global g_guiName, g_hwnd
    Gui, %g_guiName%:Destroy
    g_hwnd := 0
}

Clickey_AddLine(x, y, w, h) {
    ; 用 Progress 画线，支持自定义颜色
    global g_guiName, g_lineColor
    Gui, %g_guiName%:Add, Progress, % "x" x " y" y " w" w " h" h " c" g_lineColor " Background" g_lineColor, 100
}

Clickey_DrawDiagonals(x1, y1, x2, y2) {
    global g_hwnd, g_lineColor, g_line, g_guiScale
    if (!g_hwnd)
        return
    scale := (g_guiScale > 0) ? g_guiScale : 1.0
    if (scale != 1.0) {
        x1 := Round(x1 * scale)
        y1 := Round(y1 * scale)
        x2 := Round(x2 * scale)
        y2 := Round(y2 * scale)
    }
    hdc := DllCall("GetDC", "ptr", g_hwnd, "ptr")
    if (!hdc)
        return
    color := Clickey_ColorToBGR(g_lineColor)
    penWidth := (scale != 1.0) ? Round(g_line * scale) : g_line
    hPen := DllCall("gdi32.dll\CreatePen", "int", 0, "int", penWidth, "uint", color, "ptr")
    oldPen := DllCall("gdi32.dll\SelectObject", "ptr", hdc, "ptr", hPen, "ptr")

    DllCall("gdi32.dll\MoveToEx", "ptr", hdc, "int", x1, "int", y1, "ptr", 0)
    DllCall("gdi32.dll\LineTo", "ptr", hdc, "int", x2, "int", y2)
    DllCall("gdi32.dll\MoveToEx", "ptr", hdc, "int", x2, "int", y1, "ptr", 0)
    DllCall("gdi32.dll\LineTo", "ptr", hdc, "int", x1, "int", y2)

    DllCall("gdi32.dll\SelectObject", "ptr", hdc, "ptr", oldPen)
    DllCall("gdi32.dll\DeleteObject", "ptr", hPen)
    DllCall("ReleaseDC", "ptr", g_hwnd, "ptr", hdc)
}

Clickey_ColorToBGR(hex) {
    if (SubStr(hex, 1, 2) = "0x")
        hex := SubStr(hex, 3)
    if (StrLen(hex) != 6)
        return hex
    r := SubStr(hex, 1, 2)
    g := SubStr(hex, 3, 2)
    b := SubStr(hex, 5, 2)
    return "0x" . b . g . r
}

Clickey_SetLayoutForStep(stepIndex) {
    global g_rows, g_cols, g_keys, g_keyMap, g_font_size, g_stage
    global g_layers, g_layerCount, g_selectedRowKey, g_rowKeyMap, g_colKeyMap, g_steps
    totalSteps := g_steps.Length()
    if (stepIndex < 1 || stepIndex > totalSteps)
        return

    stepDef := g_steps[stepIndex]
    layer := g_layers[stepDef.layerIndex]
    g_font_size := layer.font

    if (stepDef.mode = "combo") {
        g_rowKeyMap := layer.rowMap
        g_colKeyMap := layer.colMap
        if (stepDef.stage = 0) {
            g_stage := 0
            g_rows := 9
            g_cols := 9
            g_keys := Clickey_BuildComboLabels(layer.rowKeys, layer.colKeys)
            g_keyMap := g_rowKeyMap
        } else {
            g_stage := 1
            g_rows := 3
            g_cols := 3
            g_keys := Clickey_BuildRowLabels(g_selectedRowKey, layer.colKeys)
            g_keyMap := g_colKeyMap
        }
    } else {
        g_stage := 2
        g_rows := 3
        g_cols := 3
        g_keys := layer.keys
        g_keyMap := layer.map
    }
}

Clickey_Init() {
    global g_keys, g_keyMap, g_guiScale, g_line, g_rows, g_cols
    global g_layers, g_layerCount, g_font_size

    ; 行键/列键（各9键，行优先）
    keys1 := ["w","e","r"
    ,"s","d","f"
    ,"x","c","v"]
    keys2 := ["u","i","o"
    ,"j","k","l"
    ,"m",",","."]

    ; DPI缩放系数
    g_guiScale := (A_ScreenDPI > 0) ? (A_ScreenDPI / 96.0) : 1.0
    if (g_line <= 0)
        g_line := (g_guiScale >= 1.5) ? 2 : 1

    ; 字号按DPI缩放（9x9 更小一些）
    font_combo := Round(12 * g_guiScale)
    font_main := Round(8 * g_guiScale)
    font_small := Round(4 * g_guiScale)

    g_layers := []
    layer1 := {mode: "combo", rowKeys: keys1, colKeys: keys2, font: font_combo}
    layer1.rowMap := Clickey_BuildKeyMap(layer1.rowKeys)
    layer1.colMap := Clickey_BuildKeyMap(layer1.colKeys)
    g_layers.Push(layer1)
    layer2 := {mode: "single", keys: keys1, font: font_main}
    layer2.map := Clickey_BuildKeyMap(layer2.keys)
    g_layers.Push(layer2)
    layer3 := {mode: "single", keys: keys2, font: font_small}
    layer3.map := Clickey_BuildKeyMap(layer3.keys)
    g_layers.Push(layer3)
    g_layerCount := g_layers.Length()
    Clickey_BuildSteps()

    g_font_size := g_layers[1].font
    Clickey_SetLayoutForStep(1)
    Clickey_UpdateScreen()
}

Clickey_BuildKeyMap(keys) {
    map := {}
    for idx, k in keys
        map[k] := idx
    return map
}

Clickey_BuildSteps() {
    global g_layers, g_steps
    g_steps := []
    for idx, layer in g_layers {
        if (layer.mode = "combo") {
            g_steps.Push({layerIndex: idx, mode: "combo", stage: 0, stepInLayer: 1, stepsInLayer: 2})
            g_steps.Push({layerIndex: idx, mode: "combo", stage: 1, stepInLayer: 2, stepsInLayer: 2})
        } else {
            g_steps.Push({layerIndex: idx, mode: "single", stage: 2, stepInLayer: 1, stepsInLayer: 1})
        }
    }
}

Clickey_BuildComboLabels(rowKeys, colKeys) {
    labels := []
    Loop, 3 {
        rowGroup := A_Index - 1
        Loop, 3 {
            colGroup := A_Index - 1
            Loop, 3 {
                rowPos := A_Index - 1
                rk := rowKeys[rowGroup * 3 + rowPos + 1]
                Loop, 3 {
                    colPos := A_Index - 1
                    ck := colKeys[colGroup * 3 + colPos + 1]
                    labels.Push(rk . ck)
                }
            }
        }
    }
    return labels
}

Clickey_BuildRowLabels(prefix, colKeys) {
    labels := []
    for _, ck in colKeys
        labels.Push(prefix . ck)
    return labels
}

Clickey_UpdateScreen() {
    global g_screen
    ; 单显示器：直接用 A_ScreenWidth/Height
    SysGet, monCount, MonitorCount
    if (monCount <= 1) {
        g_screen := {x: 0, y: 0, w: A_ScreenWidth, h: A_ScreenHeight}
        return
    }

    SysGet, vx, 76
    SysGet, vy, 77
    SysGet, vw, 78
    SysGet, vh, 79

    if (vw = "")
        g_screen := {x: 0, y: 0, w: A_ScreenWidth, h: A_ScreenHeight}
    else
        g_screen := {x: vx, y: vy, w: vw, h: vh}
}
