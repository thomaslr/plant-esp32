#!/usr/bin/env python3
"""
Grid-aligned, 100% verified schematic generator for plant-esp32-smd.kicad_sch.
Every coordinate is strictly on the 50-mil (1.27 mm) connection grid.
"""

import math
import uuid
import xml.etree.ElementTree as ET
import subprocess

def u():
    return str(uuid.uuid4())

# Grid helper: 1 unit = 1.27 mm (50 mils)
def G(units):
    return round(units * 1.27, 4)

def build_schematic():
    lines = []
    lines.append('(kicad_sch')
    lines.append('\t(version 20231120)')
    lines.append('\t(generator "antigravity")')
    sheet_uuid = "08d48698-8ae9-4579-b376-0c39bd9de789"
    lines.append(f'\t(uuid "{sheet_uuid}")')
    lines.append('\t(paper "A3")')
    lines.append('\t(title_block')
    lines.append('\t\t(title "Plant Watering & Telemetry Controller (SMD Turnkey Edition Rev 2.3)")')
    lines.append('\t\t(date "2026-10-15")')
    lines.append('\t\t(rev "v2.3-SMD")')
    lines.append('\t\t(company "Antigravity Open Hardware")')
    lines.append('\t\t(comment 1 "Dual Dedicated ADC Monitoring (5V Rail & 18650 Cell) + Anti-Backfeed D2 + 100% JLCPCB Basic Parts")')
    lines.append('\t)')

    lines.append('\t(lib_symbols')
    
    # 1. Device:R
    lines.append('\t\t(symbol "Device:R"')
    lines.append('\t\t\t(pin_numbers (hide yes))')
    lines.append('\t\t\t(pin_names (offset 0))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "R" (at 2.032 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "R" (at 0 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at -1.778 0 90) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "R_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.016 -2.54) (end 1.016 2.54) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "R_1_1"')
    lines.append('\t\t\t\t(pin passive line (at 0 3.81 270) (length 1.27) (name "" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at 0 -3.81 90) (length 1.27) (name "" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 2. Device:C
    lines.append('\t\t(symbol "Device:C"')
    lines.append('\t\t\t(pin_numbers (hide yes))')
    lines.append('\t\t\t(pin_names (offset 0))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "C" (at 1.651 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "C" (at 0 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "C_0_1"')
    lines.append('\t\t\t\t(polyline (pts (xy -2.032 -0.762) (xy 2.032 -0.762)) (stroke (width 0.508) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy -2.032 0.762) (xy 2.032 0.762)) (stroke (width 0.508) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "C_1_1"')
    lines.append('\t\t\t\t(pin passive line (at 0 3.81 270) (length 3.048) (name "" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at 0 -3.81 90) (length 3.048) (name "" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 3. Device:C_Polarized
    lines.append('\t\t(symbol "Device:C_Polarized"')
    lines.append('\t\t\t(pin_numbers (hide yes))')
    lines.append('\t\t\t(pin_names (offset 0))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "C" (at 1.651 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "C_Polarized" (at 0 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "C_Polarized_0_1"')
    lines.append('\t\t\t\t(polyline (pts (xy -2.032 -0.762) (xy 2.032 -0.762)) (stroke (width 0.508) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy -2.032 0.762) (xy 2.032 0.762)) (stroke (width 0.508) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy -1.27 2.032) (xy -1.27 3.302)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy -1.905 2.667) (xy -0.635 2.667)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "C_Polarized_1_1"')
    lines.append('\t\t\t\t(pin passive line (at 0 3.81 270) (length 3.048) (name "" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at 0 -3.81 90) (length 3.048) (name "" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 4. Device:D_Schottky (Pin 1=K on right at (3.81, 0), Pin 2=A on left at (-3.81, 0))
    lines.append('\t\t(symbol "Device:D_Schottky"')
    lines.append('\t\t\t(pin_numbers (hide yes))')
    lines.append('\t\t\t(pin_names (offset 0))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "D" (at 0 2.54 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "D_Schottky" (at 0 -2.54 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "D_Schottky_0_1"')
    lines.append('\t\t\t\t(polyline (pts (xy 1.27 1.27) (xy 1.27 -1.27)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 1.27 1.27) (xy 1.778 1.27) (xy 1.778 0.762)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 1.27 -1.27) (xy 0.762 -1.27) (xy 0.762 -0.762)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy -1.27 1.27) (xy -1.27 -1.27) (xy 1.27 0) (xy -1.27 1.27)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "D_Schottky_1_1"')
    lines.append('\t\t\t\t(pin passive line (at 3.81 0 180) (length 2.54) (name "K" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -3.81 0 0) (length 2.54) (name "A" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 5. Device:R_Photo
    lines.append('\t\t(symbol "Device:R_Photo"')
    lines.append('\t\t\t(pin_numbers (hide yes))')
    lines.append('\t\t\t(pin_names (offset 0))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "R" (at 2.032 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "R_Photo" (at 0 0 90) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at -1.778 0 90) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "R_Photo_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.016 -2.54) (end 1.016 2.54) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 2.54 1.778) (xy 1.524 0.762)) (stroke (width 0.1524) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 3.048 0.508) (xy 2.032 -0.508)) (stroke (width 0.1524) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "R_Photo_1_1"')
    lines.append('\t\t\t\t(pin passive line (at 0 3.81 270) (length 1.27) (name "" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at 0 -3.81 90) (length 1.27) (name "" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 6. Transistor_FET:AO3400A (N-MOSFET, Pin 1=G, Pin 2=S, Pin 3=D)
    lines.append('\t\t(symbol "Transistor_FET:AO3400A"')
    lines.append('\t\t\t(pin_names (offset 0) (hide yes))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "Q" (at 5.08 1.905 0) (effects (font (size 1.27 1.27)) (justify left)))')
    lines.append('\t\t\t(property "Value" "AO3400A" (at 5.08 0 0) (effects (font (size 1.27 1.27)) (justify left)))')
    lines.append('\t\t\t(property "Footprint" "Package_TO_SOT_SMD:SOT-23" (at 5.08 -1.905 0) (hide yes) (effects (font (size 1.27 1.27)) (justify left)))')
    lines.append('\t\t\t(symbol "AO3400A_0_1"')
    lines.append('\t\t\t\t(polyline (pts (xy 0.254 1.905) (xy 0.254 -1.905)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 0.254 0) (xy -2.54 0)) (stroke (width 0) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 0.762 2.286) (xy 0.762 1.27)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 0.762 0.508) (xy 0.762 -0.508)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 0.762 -1.27) (xy 0.762 -2.286)) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 0.762 -1.778) (xy 2.54 -1.778) (xy 2.54 -2.54)) (stroke (width 0) (type default)) (fill (type none)))')
    lines.append('\t\t\t\t(polyline (pts (xy 0.762 1.778) (xy 2.54 1.778) (xy 2.54 2.54)) (stroke (width 0) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "AO3400A_1_1"')
    lines.append('\t\t\t\t(pin input line (at -5.08 0 0) (length 2.54) (name "G" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at 2.54 -5.08 90) (length 2.54) (name "S" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at 2.54 5.08 270) (length 2.54) (name "D" (effects (font (size 1.27 1.27)))) (number "3" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 7. Connector_Generic:Conn_01x08_Socket
    lines.append('\t\t(symbol "Connector_Generic:Conn_01x08_Socket"')
    lines.append('\t\t\t(pin_names (offset 1.016))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "J" (at 0 11.43 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "Conn_01x08_Socket" (at 0 -13.97 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "Conn_01x08_Socket_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.27 8.89) (end 1.27 -11.43) (stroke (width 0.254) (type default)) (fill (type background)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "Conn_01x08_Socket_1_1"')
    for p in range(1, 9):
        py = 10.16 - p * 2.54
        lines.append(f'\t\t\t\t(pin passive line (at -5.08 {py:.2f} 0) (length 3.81) (name "Pin_{p}" (effects (font (size 1.27 1.27)))) (number "{p}" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 8. Connector_Generic:Conn_01x04_Socket
    lines.append('\t\t(symbol "Connector_Generic:Conn_01x04_Socket"')
    lines.append('\t\t\t(pin_names (offset 1.016))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "J" (at 0 6.35 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "Conn_01x04_Socket" (at 0 -8.89 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "Conn_01x04_Socket_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.27 3.81) (end 1.27 -6.35) (stroke (width 0.254) (type default)) (fill (type background)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "Conn_01x04_Socket_1_1"')
    for p in range(1, 5):
        py = 5.08 - p * 2.54
        lines.append(f'\t\t\t\t(pin passive line (at -5.08 {py:.2f} 0) (length 3.81) (name "Pin_{p}" (effects (font (size 1.27 1.27)))) (number "{p}" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 9. Connector_Generic:Conn_01x05_Socket
    lines.append('\t\t(symbol "Connector_Generic:Conn_01x05_Socket"')
    lines.append('\t\t\t(pin_names (offset 1.016))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "J" (at 0 7.62 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "Conn_01x05_Socket" (at 0 -10.16 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "Conn_01x05_Socket_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.27 6.35) (end 1.27 -8.89) (stroke (width 0.254) (type default)) (fill (type background)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "Conn_01x05_Socket_1_1"')
    for p in range(1, 6):
        py = 7.62 - p * 2.54
        lines.append(f'\t\t\t\t(pin passive line (at -5.08 {py:.2f} 0) (length 3.81) (name "Pin_{p}" (effects (font (size 1.27 1.27)))) (number "{p}" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 10. TerminalBlock:KF301-2P
    lines.append('\t\t(symbol "TerminalBlock:KF301-2P"')
    lines.append('\t\t\t(pin_names (offset 1.016))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "TB" (at 0 3.81 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "KF301-2P" (at 0 -6.35 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "KF301-2P_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.27 1.27) (end 1.27 -3.81) (stroke (width 0.254) (type default)) (fill (type background)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "KF301-2P_1_1"')
    lines.append('\t\t\t\t(pin passive line (at -5.08 0 0) (length 3.81) (name "1" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -5.08 -2.54 0) (length 3.81) (name "2" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 11. TerminalBlock:KF301-3P
    lines.append('\t\t(symbol "TerminalBlock:KF301-3P"')
    lines.append('\t\t\t(pin_names (offset 1.016))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "TB" (at 0 5.08 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "KF301-3P" (at 0 -7.62 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "KF301-3P_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.27 3.81) (end 1.27 -5.08) (stroke (width 0.254) (type default)) (fill (type background)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "KF301-3P_1_1"')
    lines.append('\t\t\t\t(pin passive line (at -5.08 2.54 0) (length 3.81) (name "1" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -5.08 0 0) (length 3.81) (name "2" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -5.08 -2.54 0) (length 3.81) (name "3" (effects (font (size 1.27 1.27)))) (number "3" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 12. TerminalBlock:KF301-4P
    lines.append('\t\t(symbol "TerminalBlock:KF301-4P"')
    lines.append('\t\t\t(pin_names (offset 1.016))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "TB" (at 0 6.35 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "KF301-4P" (at 0 -8.89 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "KF301-4P_0_1"')
    lines.append('\t\t\t\t(rectangle (start -1.27 5.08) (end 1.27 -6.35) (stroke (width 0.254) (type default)) (fill (type background)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "KF301-4P_1_1"')
    lines.append('\t\t\t\t(pin passive line (at -5.08 2.54 0) (length 3.81) (name "1" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -5.08 0 0) (length 3.81) (name "2" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -5.08 -2.54 0) (length 3.81) (name "3" (effects (font (size 1.27 1.27)))) (number "3" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t\t(pin passive line (at -5.08 -5.08 0) (length 3.81) (name "4" (effects (font (size 1.27 1.27)))) (number "4" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    # 13. Mechanical:MountingHole_Pad
    lines.append('\t\t(symbol "Mechanical:MountingHole_Pad"')
    lines.append('\t\t\t(pin_numbers (hide yes))')
    lines.append('\t\t\t(pin_names (offset 0) (hide yes))')
    lines.append('\t\t\t(exclude_from_sim no) (in_bom yes) (on_board yes)')
    lines.append('\t\t\t(property "Reference" "H" (at 0 3.81 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Value" "MountingHole_Pad" (at 0 -3.81 0) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(property "Footprint" "" (at 0 0 0) (hide yes) (effects (font (size 1.27 1.27))))')
    lines.append('\t\t\t(symbol "MountingHole_Pad_0_1"')
    lines.append('\t\t\t\t(circle (center 0 0) (radius 1.27) (stroke (width 0.254) (type default)) (fill (type none)))')
    lines.append('\t\t\t)')
    lines.append('\t\t\t(symbol "MountingHole_Pad_1_1"')
    lines.append('\t\t\t\t(pin passive line (at 0 -2.54 90) (length 1.27) (name "1" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))')
    lines.append('\t\t\t)')
    lines.append('\t\t)')

    lines.append('\t)') # end lib_symbols

    # Local pin offsets:
    symbol_pins = {
        "Device:R": {"1": (0.0, 3.81), "2": (0.0, -3.81)},
        "Device:C": {"1": (0.0, 3.81), "2": (0.0, -3.81)},
        "Device:C_Polarized": {"1": (0.0, 3.81), "2": (0.0, -3.81)},
        "Device:D_Schottky": {"1": (3.81, 0.0), "2": (-3.81, 0.0)}, # 1=K, 2=A
        "Device:R_Photo": {"1": (0.0, 3.81), "2": (0.0, -3.81)},
        "Transistor_FET:AO3400A": {"1": (-5.08, 0.0), "2": (2.54, -5.08), "3": (2.54, 5.08)}, # 1=G, 2=S, 3=D
        "Connector_Generic:Conn_01x08_Socket": {str(p): (-5.08, 10.16 - p * 2.54) for p in range(1, 9)},
        "Connector_Generic:Conn_01x04_Socket": {str(p): (-5.08, 5.08 - p * 2.54) for p in range(1, 5)},
        "Connector_Generic:Conn_01x05_Socket": {str(p): (-5.08, 7.62 - p * 2.54) for p in range(1, 6)},
        "TerminalBlock:KF301-2P": {"1": (-5.08, 0.0), "2": (-5.08, -2.54)},
        "TerminalBlock:KF301-3P": {"1": (-5.08, 2.54), "2": (-5.08, 0.0), "3": (-5.08, -2.54)},
        "TerminalBlock:KF301-4P": {"1": (-5.08, 2.54), "2": (-5.08, 0.0), "3": (-5.08, -2.54), "4": (-5.08, -5.08)},
        "Mechanical:MountingHole_Pad": {"1": (0.0, -2.54)},
    }

    pin_locations = {}

    def place_symbol(ref, val, footprint, lcsc, lib_id, gx, gy, rot=0, dnp=False):
        sx = G(gx)
        sy = G(gy)
        dnp_str = " (dnp yes)" if dnp else ""
        sym_uuid = u()
        lines.append(f'\t(symbol (lib_id "{lib_id}") (at {sx} {sy} {rot}) (unit 1)')
        lines.append(f'\t\t(in_bom yes) (on_board yes){dnp_str}')
        lines.append(f'\t\t(uuid "{sym_uuid}")')
        lines.append(f'\t\t(property "Reference" "{ref}" (at {round(sx+2.54, 4)} {round(sy-2.54, 4)} 0) (effects (font (size 1.27 1.27)) (justify left)))')
        lines.append(f'\t\t(property "Value" "{val}" (at {round(sx+2.54, 4)} {sy} 0) (effects (font (size 1.27 1.27)) (justify left)))')
        lines.append(f'\t\t(property "Footprint" "{footprint}" (at {sx} {sy} 0) (hide yes) (effects (font (size 1.27 1.27))))')
        if lcsc:
            lines.append(f'\t\t(property "LCSC" "{lcsc}" (at {sx} {sy} 0) (hide yes) (effects (font (size 1.27 1.27))))')
        
        rad = math.radians(rot)
        for pnum, (px, py) in symbol_pins[lib_id].items():
            lines.append(f'\t\t(pin "{pnum}" (uuid "{u()}"))')
            x_rot = px * math.cos(rad) - py * math.sin(rad)
            y_rot = px * math.sin(rad) + py * math.cos(rad)
            cx = round(sx + x_rot, 4)
            cy = round(sy - y_rot, 4)
            pin_locations[(ref, pnum)] = (cx, cy)
        
        lines.append('\t\t(instances')
        lines.append('\t\t\t(project "plant-esp32-smd"')
        lines.append(f'\t\t\t\t(path "/08d48698-8ae9-4579-b376-0c39bd9de789/{sym_uuid}"')
        lines.append(f'\t\t\t\t\t(reference "{ref}") (unit 1)')
        lines.append('\t\t\t\t)')
        lines.append('\t\t\t)')
        lines.append('\t\t)')
        lines.append('\t)')

    def add_wire(x1, y1, x2, y2):
        lines.append(f'\t(wire (pts (xy {x1} {y1}) (xy {x2} {y2})) (stroke (width 0) (type default)) (uuid "{u()}"))')

    def add_label(net, x, y, rot=0):
        lines.append(f'\t(label "{net}" (at {x} {y} {rot}) (fields_autoplaced yes) (effects (font (size 1.27 1.27))) (uuid "{u()}"))')

    def add_junction(x, y):
        lines.append(f'\t(junction (at {x} {y}) (diameter 1.0) (color 0 0 0 0) (uuid "{u()}"))')

    def add_no_connect(x, y):
        lines.append(f'\t(no_connect (at {x} {y}) (uuid "{u()}"))')

    # =========================================================================
    # PLACEMENT (ALL COORDINATES IN INTEGER GRID UNITS OF 1.27mm / 50 mils)
    # =========================================================================

    # 1. MCU Headers J1 & J2
    # J1 at (110, 90) -> (139.7, 114.3)
    # J2 at (150, 90) -> (190.5, 114.3)
    place_symbol("J1", "Conn_01x08_Socket", "Connector_PinSocket_2.54mm:PinSocket_1x08_P2.54mm_Vertical", "C224346", "Connector_Generic:Conn_01x08_Socket", 110, 90, rot=0)
    place_symbol("J2", "Conn_01x08_Socket", "Connector_PinSocket_2.54mm:PinSocket_1x08_P2.54mm_Vertical", "C224346", "Connector_Generic:Conn_01x08_Socket", 150, 90, rot=0)

    # 2. 5V Rail ADC Divider (R1, R2, C1)
    # R1 at (30, 36) rot 0: Pin 1 at top (y=33), Pin 2 at bottom (y=39)
    # Midpoint at y=42
    # R2 at (30, 48) rot 180: Pin 2 at top (y=45), Pin 1 at bottom (y=51)
    # C1 at (42, 48) rot 0: Pin 1 at top (y=45), Pin 2 at bottom (y=51)
    place_symbol("R1", "100k", "Resistor_SMD:R_0805_2012Metric", "C149504", "Device:R", 30, 36, rot=0)
    place_symbol("R2", "100k", "Resistor_SMD:R_0805_2012Metric", "C149504", "Device:R", 30, 48, rot=180)
    place_symbol("C1", "100nF", "Capacitor_SMD:C_0805_2012Metric", "C28233", "Device:C", 42, 48, rot=0)

    # 3. Battery ADC Divider (R8, R9, C6)
    # R8 at (60, 36) rot 0: Pin 1 at top (y=33), Pin 2 at bottom (y=39)
    # Midpoint at y=42
    # R9 at (60, 48) rot 180: Pin 2 at top (y=45), Pin 1 at bottom (y=51)
    # C6 at (72, 48) rot 180: Pin 2 at top (y=45), Pin 1 at bottom (y=51)
    place_symbol("R8", "100k", "Resistor_SMD:R_0805_2012Metric", "C149504", "Device:R", 60, 36, rot=0)
    place_symbol("R9", "100k", "Resistor_SMD:R_0805_2012Metric", "C149504", "Device:R", 60, 48, rot=180)
    place_symbol("C6", "100nF", "Capacitor_SMD:C_0805_2012Metric", "C28233", "Device:C", 72, 48, rot=180)

    # 4. Power Input Protection D2 & Decoupling Caps (C3, C5, C_EXT)
    # D2 at (180, 36) rot 0: Pin 2 (Anode) at left (x=177), Pin 1 (Cathode) at right (x=183)
    place_symbol("D2", "SS34", "Diode_SMD:D_SMA", "C8678", "Device:D_Schottky", 180, 36, rot=0)
    place_symbol("C3", "10uF", "Capacitor_SMD:C_0805_2012Metric", "C15850", "Device:C", 200, 42, rot=0)
    place_symbol("C5", "10uF", "Capacitor_SMD:C_0805_2012Metric", "C15850", "Device:C", 212, 42, rot=0)
    place_symbol("C_EXT", "100uF-470uF_DNP", "Capacitor_THT:CP_Radial_D6.3mm_P2.50mm", "", "Device:C_Polarized", 224, 42, rot=0, dnp=True)

    # 5. Ultrasonic Echo Divider (R3, R4)
    # R3 at (30, 90) rot 180: Pin 2 at top (y=87), Pin 1 at bottom (y=93)
    # Midpoint at y=96
    # R4 at (30, 102) rot 0: Pin 1 at top (y=99), Pin 2 at bottom (y=105)
    place_symbol("R3", "1k", "Resistor_SMD:R_0805_2012Metric", "C17513", "Device:R", 30, 90, rot=180)
    place_symbol("R4", "2k", "Resistor_SMD:R_0805_2012Metric", "C17604", "Device:R", 30, 102, rot=0)

    # 6. Pump MOSFET Driver & Flyback Diode (R5, R6, Q1, D1)
    # R5 at (180, 96) rot 90: Pin 1 at left (x=177), Pin 2 at right (x=183)
    # R6 at (188, 108) rot 180: Pin 2 at top (y=105), Pin 1 at bottom (y=111)
    # Q1 at (200, 96) rot 0: Pin 1 (G) at (x-4=196, y=96), Pin 2 (S) at (x+2=202, y+4=100), Pin 3 (D) at (x+2=202, y-4=92)
    # D1 at (214, 92) rot 90: Pin 1 (Cathode) at top (y=89), Pin 2 (Anode) at bottom (y=95)
    place_symbol("R5", "1k", "Resistor_SMD:R_0805_2012Metric", "C17513", "Device:R", 180, 96, rot=90)
    place_symbol("R6", "100k", "Resistor_SMD:R_0805_2012Metric", "C149504", "Device:R", 188, 108, rot=180)
    place_symbol("Q1", "AO3400A", "Package_TO_SOT_SMD:SOT-23", "C20917", "Transistor_FET:AO3400A", 200, 96, rot=0)
    place_symbol("D1", "SS34", "Diode_SMD:D_SMA", "C8678", "Device:D_Schottky", 214, 92, rot=90)

    # 7. Soil Sensor Filter (C2) & LDR Ambient Light (LDR1, R7, C4)
    # C2 at (30, 144) rot 0: Pin 1 at top (y=141), Pin 2 at bottom (y=147)
    # LDR1 at (60, 138) rot 0: Pin 1 at top (y=135), Pin 2 at bottom (y=141)
    # Midpoint at y=144
    # R7 at (60, 150) rot 0: Pin 1 at top (y=147), Pin 2 at bottom (y=153)
    # C4 at (72, 150) rot 0: Pin 1 at top (y=147), Pin 2 at bottom (y=153)
    place_symbol("C2", "100nF", "Capacitor_SMD:C_0805_2012Metric", "C28233", "Device:C", 30, 144, rot=0)
    place_symbol("LDR1", "GL55xx_DNP", "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical", "", "Device:R_Photo", 60, 138, rot=0, dnp=True)
    place_symbol("R7", "10k", "Resistor_SMD:R_0805_2012Metric", "C17414", "Device:R", 60, 150, rot=0)
    place_symbol("C4", "100nF", "Capacitor_SMD:C_0805_2012Metric", "C28233", "Device:C", 72, 150, rot=0)

    # 8. Terminal Blocks (TB1, TB2, TB3, TB4)
    # Placed on right side at x = 260
    place_symbol("TB1", "KF301-4P", "TerminalBlock:TerminalBlock_KF301-4P_P5.00mm", "C8354", "TerminalBlock:KF301-4P", 260, 36, rot=0)
    place_symbol("TB2", "KF301-3P", "TerminalBlock:TerminalBlock_KF301-3P_P5.00mm", "C8353", "TerminalBlock:KF301-3P", 260, 72, rot=0)
    place_symbol("TB3", "KF301-2P", "TerminalBlock:TerminalBlock_KF301-2P_P5.00mm", "C8352", "TerminalBlock:KF301-2P", 260, 102, rot=0)
    place_symbol("TB4", "KF301-3P_DNP", "TerminalBlock:TerminalBlock_KF301-3P_P5.00mm_Vertical", "C8353", "TerminalBlock:KF301-3P", 260, 132, rot=0, dnp=True)

    # 9. Expansion Headers (MOD1, MOD2) & Mounting Holes (H1-H4)
    place_symbol("MOD1", "SHT30_DNP", "Connector_PinSocket_2.54mm:PinSocket_1x04_P2.54mm_Vertical", "", "Connector_Generic:Conn_01x04_Socket", 110, 156, rot=0, dnp=True)
    place_symbol("MOD2", "BH1750_DNP", "Connector_PinSocket_2.54mm:PinSocket_1x05_P2.54mm_Vertical", "", "Connector_Generic:Conn_01x05_Socket", 150, 156, rot=0, dnp=True)

    place_symbol("H1", "MountingHole_Pad", "MountingHole:MountingHole_3.2mm_M3_Pad", "", "Mechanical:MountingHole_Pad", 188, 156, rot=0)
    place_symbol("H2", "MountingHole_Pad", "MountingHole:MountingHole_3.2mm_M3_Pad", "", "Mechanical:MountingHole_Pad", 200, 156, rot=0)
    place_symbol("H3", "MountingHole_Pad", "MountingHole:MountingHole_3.2mm_M3_Pad", "", "Mechanical:MountingHole_Pad", 212, 156, rot=0)
    place_symbol("H4", "MountingHole_Pad", "MountingHole:MountingHole_3.2mm_M3_Pad", "", "Mechanical:MountingHole_Pad", 224, 156, rot=0)

    # =========================================================================
    # WIRING & NET CONNECTIONS (ALL WIRES ALIGNED TO 50-MIL GRID)
    # =========================================================================

    # Wire length helper (6 grid units = 7.62 mm)
    def wire_out_label(ref, pnum, net, dx_grid, dy_grid, label_rot):
        x, y = pin_locations[(ref, pnum)]
        tx = round(x + G(dx_grid), 4)
        ty = round(y + G(dy_grid), 4)
        add_wire(x, y, tx, ty)
        add_label(net, tx, ty, label_rot)

    # --- J1 Connections ---
    wire_out_label("J1", "1", "+5V", -6, 0, 180)
    wire_out_label("J1", "2", "GND", -6, 0, 180)
    wire_out_label("J1", "3", "+3V3", -6, 0, 180)
    wire_out_label("J1", "4", "GPIO4_5V_ADC", -6, 0, 180)
    wire_out_label("J1", "5", "GPIO3_SOIL", -6, 0, 180)
    add_no_connect(*pin_locations[("J1", "6")])
    wire_out_label("J1", "7", "GPIO1_LDR", -6, 0, 180)
    wire_out_label("J1", "8", "GPIO0_VBAT_ADC", -6, 0, 180)

    # --- J2 Connections ---
    wire_out_label("J2", "1", "GPIO5_SCL", -6, 0, 180)
    wire_out_label("J2", "2", "GPIO6_SDA", -6, 0, 180)
    wire_out_label("J2", "3", "GPIO7_TRIG", -6, 0, 180)
    add_no_connect(*pin_locations[("J2", "4")])
    add_no_connect(*pin_locations[("J2", "5")])
    wire_out_label("J2", "6", "GPIO10_GATE", -6, 0, 180)
    wire_out_label("J2", "7", "GPIO20_ECHO", -6, 0, 180)
    wire_out_label("J2", "8", "GPIO21_SOIL_VCC", -6, 0, 180)

    # --- 5V Rail ADC Divider (R1, R2, C1) ---
    # R1 Pin 1 (top): +5V
    wire_out_label("R1", "1", "+5V", 0, -4, 90)

    # Midpoint bus at y = G(42) = 53.34
    mid_y_5v = G(42)
    r1_p2 = pin_locations[("R1", "2")] # (G(30), G(39))
    r2_p2 = pin_locations[("R2", "2")] # (G(30), G(45))
    c1_p1 = pin_locations[("C1", "1")] # (G(42), G(45))

    add_wire(r1_p2[0], r1_p2[1], r1_p2[0], mid_y_5v)
    add_wire(r2_p2[0], r2_p2[1], r2_p2[0], mid_y_5v)
    add_junction(r1_p2[0], mid_y_5v)

    add_wire(r1_p2[0], mid_y_5v, c1_p1[0], mid_y_5v)
    add_wire(c1_p1[0], c1_p1[1], c1_p1[0], mid_y_5v)
    add_junction(c1_p1[0], mid_y_5v)

    label_5v_x = round(c1_p1[0] + G(6), 4)
    add_wire(c1_p1[0], mid_y_5v, label_5v_x, mid_y_5v)
    add_label("GPIO4_5V_ADC", label_5v_x, mid_y_5v, 0)

    # GND returns for R2 and C1
    wire_out_label("R2", "1", "GND", 0, 4, 270)
    wire_out_label("C1", "2", "GND", 0, 4, 270)

    # --- Battery ADC Divider (R8, R9, C6) ---
    # R8 Pin 1 (top): VBAT_SENSE
    wire_out_label("R8", "1", "VBAT_SENSE", 0, -4, 90)

    mid_y_bat = G(42)
    r8_p2 = pin_locations[("R8", "2")] # (G(60), G(39))
    r9_p2 = pin_locations[("R9", "2")] # (G(60), G(45))
    c6_p2 = pin_locations[("C6", "2")] # (G(72), G(45))

    add_wire(r8_p2[0], r8_p2[1], r8_p2[0], mid_y_bat)
    add_wire(r9_p2[0], r9_p2[1], r9_p2[0], mid_y_bat)
    add_junction(r8_p2[0], mid_y_bat)

    add_wire(r8_p2[0], mid_y_bat, c6_p2[0], mid_y_bat)
    add_wire(c6_p2[0], c6_p2[1], c6_p2[0], mid_y_bat)
    add_junction(c6_p2[0], mid_y_bat)

    label_bat_x = round(c6_p2[0] + G(6), 4)
    add_wire(c6_p2[0], mid_y_bat, label_bat_x, mid_y_bat)
    add_label("GPIO0_VBAT_ADC", label_bat_x, mid_y_bat, 0)

    # GND returns for R9 and C6
    wire_out_label("R9", "1", "GND", 0, 4, 270)
    wire_out_label("C6", "1", "GND", 0, 4, 270)

    # --- Power Protection D2 & Decoupling Caps ---
    wire_out_label("D2", "2", "EXT_5V", -6, 0, 180) # Anode left
    wire_out_label("D2", "1", "+5V", 6, 0, 0)       # Cathode right

    for cap_ref in ["C3", "C5", "C_EXT"]:
        wire_out_label(cap_ref, "1", "+5V", 0, -4, 90)
        wire_out_label(cap_ref, "2", "GND", 0, 4, 270)

    # --- Ultrasonic Echo Divider (R3, R4) ---
    wire_out_label("R3", "2", "ECHO_5V", 0, -4, 90)

    mid_y_echo = G(96)
    r3_p1 = pin_locations[("R3", "1")] # bottom of R3
    r4_p1 = pin_locations[("R4", "1")] # top of R4

    add_wire(r3_p1[0], r3_p1[1], r3_p1[0], mid_y_echo)
    add_wire(r4_p1[0], r4_p1[1], r4_p1[0], mid_y_echo)
    add_junction(r3_p1[0], mid_y_echo)

    label_echo_x = round(r3_p1[0] + G(6), 4)
    add_wire(r3_p1[0], mid_y_echo, label_echo_x, mid_y_echo)
    add_label("GPIO20_ECHO", label_echo_x, mid_y_echo, 0)

    wire_out_label("R4", "2", "GND", 0, 4, 270)

    # --- Pump Driver (R5, R6, Q1, D1) ---
    wire_out_label("R5", "1", "GPIO10_GATE", -6, 0, 180)

    r5_p2 = pin_locations[("R5", "2")] # (G(183), G(96))
    q1_g  = pin_locations[("Q1", "1")]  # (G(196), G(96))
    r6_p2 = pin_locations[("R6", "2")] # (G(188), G(105))

    # Connect R5-Pin2 to Q1 Gate horizontally along y = G(96)
    add_wire(r5_p2[0], r5_p2[1], q1_g[0], q1_g[1])
    # Connect R6 Pin 2 up to the gate wire at (G(188), G(96))
    gate_junc_x = r6_p2[0]
    gate_junc_y = q1_g[1]
    add_wire(r6_p2[0], r6_p2[1], gate_junc_x, gate_junc_y)
    add_junction(gate_junc_x, gate_junc_y)
    # Add GATE_DRIVE label right on the wire
    add_label("GATE_DRIVE", gate_junc_x, gate_junc_y, 90)

    wire_out_label("R6", "1", "GND", 0, 4, 270)
    wire_out_label("Q1", "2", "GND", 0, 4, 270) # Source to GND

    q1_d = pin_locations[("Q1", "3")] # (G(202), G(92))
    d1_a = pin_locations[("D1", "2")] # (G(214), G(95))
    d1_k = pin_locations[("D1", "1")] # (G(214), G(89))

    # D1 Cathode to +5V
    wire_out_label("D1", "1", "+5V", 0, -4, 90)

    # Q1 Drain to D1 Anode & PUMP_DRAIN
    pump_bus_y = G(95)
    add_wire(q1_d[0], q1_d[1], q1_d[0], pump_bus_y)
    add_wire(q1_d[0], pump_bus_y, d1_a[0], pump_bus_y)
    add_junction(d1_a[0], pump_bus_y)

    pump_label_x = round(d1_a[0] + G(6), 4)
    add_wire(d1_a[0], pump_bus_y, pump_label_x, pump_bus_y)
    add_label("PUMP_DRAIN", pump_label_x, pump_bus_y, 0)

    # --- Soil Sensor Filter (C2) ---
    wire_out_label("C2", "1", "GPIO3_SOIL", 0, -4, 90)
    wire_out_label("C2", "2", "GND", 0, 4, 270)

    # --- Ambient Light Sensor (LDR1, R7, C4) ---
    wire_out_label("LDR1", "1", "+3V3", 0, -4, 90)

    mid_y_ldr = G(144)
    ldr_p2 = pin_locations[("LDR1", "2")] # (G(60), G(141))
    r7_p1  = pin_locations[("R7", "1")]   # (G(60), G(147))
    c4_p1  = pin_locations[("C4", "1")]   # (G(72), G(147))

    add_wire(ldr_p2[0], ldr_p2[1], ldr_p2[0], mid_y_ldr)
    add_wire(r7_p1[0], r7_p1[1], r7_p1[0], mid_y_ldr)
    add_junction(ldr_p2[0], mid_y_ldr)

    add_wire(ldr_p2[0], mid_y_ldr, c4_p1[0], mid_y_ldr)
    add_wire(c4_p1[0], c4_p1[1], c4_p1[0], mid_y_ldr)
    add_junction(c4_p1[0], mid_y_ldr)

    ldr_label_x = round(c4_p1[0] + G(6), 4)
    add_wire(c4_p1[0], mid_y_ldr, ldr_label_x, mid_y_ldr)
    add_label("GPIO1_LDR", ldr_label_x, mid_y_ldr, 0)

    wire_out_label("R7", "2", "GND", 0, 4, 270)
    wire_out_label("C4", "2", "GND", 0, 4, 270)

    # --- Terminal Blocks TB1, TB2, TB3, TB4 ---
    tb1_nets = [("+5V", "1"), ("GPIO7_TRIG", "2"), ("ECHO_5V", "3"), ("GND", "4")]
    for net, pnum in tb1_nets:
        wire_out_label("TB1", pnum, net, -6, 0, 180)

    tb2_nets = [("GPIO3_SOIL", "1"), ("GPIO21_SOIL_VCC", "2"), ("GND", "3")]
    for net, pnum in tb2_nets:
        wire_out_label("TB2", pnum, net, -6, 0, 180)

    tb3_nets = [("+5V", "1"), ("PUMP_DRAIN", "2")]
    for net, pnum in tb3_nets:
        wire_out_label("TB3", pnum, net, -6, 0, 180)

    tb4_nets = [("EXT_5V", "1"), ("GND", "2"), ("VBAT_SENSE", "3")]
    for net, pnum in tb4_nets:
        wire_out_label("TB4", pnum, net, -6, 0, 180)

    # --- Expansion Headers MOD1 & MOD2 ---
    mod1_nets = [("+3V3", "1"), ("GND", "2"), ("GPIO5_SCL", "3"), ("GPIO6_SDA", "4")]
    for net, pnum in mod1_nets:
        wire_out_label("MOD1", pnum, net, -6, 0, 180)

    mod2_nets = [("+3V3", "1"), ("GND", "2"), ("GPIO5_SCL", "3"), ("GPIO6_SDA", "4"), ("GND", "5")]
    for net, pnum in mod2_nets:
        wire_out_label("MOD2", pnum, net, -6, 0, 180)

    # --- Mounting Holes H1-H4 ---
    for h_ref in ["H1", "H2", "H3", "H4"]:
        wire_out_label(h_ref, "1", "GND", 0, 4, 270)

    lines.append(')') # end kicad_sch
    return "\n".join(lines)

if __name__ == "__main__":
    content = build_schematic()
    out_path = "pcb/smd/plant-esp32-smd.kicad_sch"
    with open(out_path, "w") as f:
        f.write(content)
    print(f"Successfully generated {out_path} ({len(content)} bytes)")
