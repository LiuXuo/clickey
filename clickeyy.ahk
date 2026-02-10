#NoEnv
#SingleInstance Force
#MaxThreadsPerHotkey 1
SetBatchLines, -1
ListLines, Off
CoordMode, Mouse, Screen

; ========================== 说明 ==========================
; 1) 改造为 5x5 结构：
;    双键层（合并）：25x25 区域，由 5x5 个大块组成，每块内部又是 5x5
;    单键层：5x5 精细定位
; 2) 按键：
;    q w e r t
;    y u i o p
;    a s d f g
;    h j k l ;
;    z x c v b
; 3) 交互：Ctrl+; 左键 / Ctrl+Shift+; 右键 / Ctrl+Shift+Alt+; 中键
; 4) 功能：Esc 取消；Backspace 回退；Space 直接点击中心
; ========================== 运行状态 ==========================
global g_active := false 
global g_button := ""
global g_step := 0 
global g_stage := 0 ; 0=行键块 1=列键格 2=单键层
global g_screen := {} 
global g_region := {} 
global g_keys := [] 
global g_keyMap := {} 
global g_selectedRowKey := "" 
global g_layers := [] 
global g_layerCount := 0
global g_steps := [] 
global g_stepHistory := [] 

; ========================== 外观配置 (完全还原原始样式) ==========================
global g_alpha := 120 
global g_maskColor := "000000" 
global g_lineColor := "FFFFFF" 
global g_textColor := "FFFFFF" 
global g_guiName := "Clickey5x5"
global g_hwnd := 0
global g_guiScale := 1.0 
global g_line := 1 
global g_rows := 25 ; 第一阶段 25x25
global g_cols := 25 
global g_font_size := 8 

Clickey_Init()
return

^;::Clickey_Start("Left")            
^+;::Clickey_Start("Right")          
^+!;::Clickey_Start("Middle")        

Clickey_Start(button) {
    global g_active, g_button, g_step, g_region, g_screen, g_layerCount, g_steps
    global g_selectedRowKey, g_stepHistory
    if (g_active)
        return

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

    g_stepHistory := [] 
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
            Clickey_HideOverlay()
            Clickey_DoClick()
            g_active := false
            return
        }
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
        StringLower, key, key
        if (g_keyMap.HasKey(key))
            return key
        SoundBeep, 900, 40
    }
}

Clickey_ApplyKey(key) {
    global g_region, g_stage, g_keyMap, g_rows, g_cols
    idx := g_keyMap[key]
    row := Ceil(idx / 5)
    col := Mod(idx - 1, 5) + 1

    cellW := g_region.w / 5
    cellH := g_region.h / 5
    g_region.x := g_region.x + (col - 1) * cellW
    g_region.y := g_region.y + (row - 1) * cellH
    g_region.w := cellW
    g_region.h := cellH
}

Clickey_DoClick() {
    global g_region, g_button
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
    msg := "层 " layerIndex "/" g_layerCount " (Esc取消 / Backspace回退 / Space点击)"
    Gui, %g_guiName%:Add, Text, x10 y10 w400 h24 +BackgroundTrans, %msg%

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

    ; 绘制网格线
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

    ; 绘制标签
    Loop, %g_rows% {
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

Clickey_HideOverlay() {
    global g_guiName
    Gui, %g_guiName%:Destroy
}

Clickey_AddLine(x, y, w, h) {
    global g_guiName, g_lineColor
    Gui, %g_guiName%:Add, Progress, % "x" x " y" y " w" w " h" h " c" g_lineColor " Background" g_lineColor, 100
}

Clickey_DrawDiagonals(x1, y1, x2, y2) {
    global g_hwnd, g_lineColor, g_line, g_guiScale
    if (!g_hwnd)
        return
    scale := (g_guiScale > 0) ? g_guiScale : 1.0
    if (scale != 1.0) {
        x1 := Round(x1 * scale), y1 := Round(y1 * scale), x2 := Round(x2 * scale), y2 := Round(y2 * scale)
    }
    hdc := DllCall("GetDC", "ptr", g_hwnd, "ptr")
    color := Clickey_ColorToBGR(g_lineColor)
    hPen := DllCall("gdi32.dll\CreatePen", "int", 0, "int", g_line, "uint", color, "ptr")
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
    return "0x" . SubStr(hex, 5, 2) . SubStr(hex, 3, 2) . SubStr(hex, 1, 2)
}

Clickey_SetLayoutForStep(stepIndex) {
    global g_rows, g_cols, g_keys, g_keyMap, g_font_size, g_stage
    global g_layers, g_selectedRowKey, g_steps
    stepDef := g_steps[stepIndex]
    layer := g_layers[stepDef.layerIndex]
    g_font_size := layer.font

    if (stepDef.mode = "combo") {
        if (stepDef.stage = 0) {
            g_stage := 0
            g_rows := 25, g_cols := 25
            g_keys := Clickey_BuildComboLabels(layer.rawKeys, layer.rawKeys)
            g_keyMap := layer.map
        } else {
            g_stage := 1
            g_rows := 5, g_cols := 5
            g_keys := Clickey_BuildRowLabels(g_selectedRowKey, layer.rawKeys)
            g_keyMap := layer.map
        }
    } else {
        g_stage := 2
        g_rows := 5, g_cols := 5
        g_keys := layer.rawKeys
        g_keyMap := layer.map
    }
}

Clickey_Init() {
    global g_layers, g_layerCount, g_guiScale, g_font_size

    ; 你的 5x5 按键定义
    keys := ["q","w","e","r","t"
    ,"y","u","i","o","p"
    ,"a","s","d","f","g"
    ,"h","j","k","l",";"
    ,"z","x","c","v","b"]

    g_guiScale := (A_ScreenDPI > 0) ? (A_ScreenDPI / 96.0) : 1.0

    g_layers := []
    ; 双键层：合并 1+2
    layer1 := {mode: "combo", rawKeys: keys, font: Round(7 * g_guiScale)}
    layer1.map := Clickey_BuildKeyMap(keys)
    g_layers.Push(layer1)

    ; 第三层：单键精细定位
    layer2 := {mode: "single", rawKeys: keys, font: Round(3 * g_guiScale)}
    layer2.map := Clickey_BuildKeyMap(keys)
    g_layers.Push(layer2)

    g_layerCount := g_layers.Length()
    Clickey_BuildSteps()
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
            g_steps.Push({layerIndex: idx, mode: "combo", stage: 0})
            g_steps.Push({layerIndex: idx, mode: "combo", stage: 1})
        } else {
            g_steps.Push({layerIndex: idx, mode: "single", stage: 2})
        }
    }
}

; 生成 25x25 标签矩阵：qq, qw, qe...
Clickey_BuildComboLabels(rowKeys, colKeys) {
    labels := []
    Loop, 5 {
        rGrp := A_Index - 1
        Loop, 5 {
            cGrp := A_Index - 1
            Loop, 5 {
                rIdx := A_Index - 1
                rk := rowKeys[rGrp * 5 + rIdx + 1]
                Loop, 5 {
                    cIdx := A_Index - 1
                    ck := colKeys[cGrp * 5 + cIdx + 1]
                    labels.Push(rk . ck)
                }
            }
        }
    }
    return labels
}

Clickey_BuildRowLabels(prefix, keys) {
    labels := []
    for _, k in keys
        labels.Push(prefix . k)
    return labels
}

Clickey_UpdateScreen() {
    global g_screen
    SysGet, monCount, MonitorCount
    if (monCount <= 1) {
        g_screen := {x: 0, y: 0, w: A_ScreenWidth, h: A_ScreenHeight}
    } else {
        SysGet, vx, 76
        SysGet, vy, 77
        SysGet, vw, 78
        SysGet, vh, 79
        g_screen := {x: vx, y: vy, w: vw, h: vh}
    }
}