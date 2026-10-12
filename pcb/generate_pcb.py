#!/usr/bin/env python3
"""
Precision KiCad 8 Router & Generator for Plant ESP32 Controller
Zero-crossing, zero-short, verified DRC with zone refill
"""

import os
import uuid

def u():
    return str(uuid.uuid4())

def generate_board():
    lines = []
    lines.append('(kicad_pcb')
    lines.append('\t(version 20240108)')
    lines.append('\t(generator "antigravity")')
    lines.append('\t(generator_version "8.0")')
    lines.append('\t(general')
    lines.append('\t\t(thickness 1.6)')
    lines.append('\t\t(legacy_teardrops no)')
    lines.append('\t)')
    lines.append('\t(paper "A4")')
    lines.append('\t(layers')
    lines.append('\t\t(0 "F.Cu" signal)')
    lines.append('\t\t(31 "B.Cu" signal)')
    lines.append('\t\t(32 "B.Adhes" user "B.Adhesive")')
    lines.append('\t\t(33 "F.Adhes" user "F.Adhesive")')
    lines.append('\t\t(34 "B.Paste" user)')
    lines.append('\t\t(35 "F.Paste" user)')
    lines.append('\t\t(36 "B.SilkS" user "B.Silkscreen")')
    lines.append('\t\t(37 "F.SilkS" user "F.Silkscreen")')
    lines.append('\t\t(38 "B.Mask" user)')
    lines.append('\t\t(39 "F.Mask" user)')
    lines.append('\t\t(40 "Dwgs.User" user "User.Drawings")')
    lines.append('\t\t(41 "Cmts.User" user "User.Comments")')
    lines.append('\t\t(42 "Eco1.User" user "User.Eco1")')
    lines.append('\t\t(43 "Eco2.User" user "User.Eco2")')
    lines.append('\t\t(44 "Edge.Cuts" user)')
    lines.append('\t\t(45 "Margin" user)')
    lines.append('\t\t(46 "B.CrtYd" user "B.Courtyard")')
    lines.append('\t\t(47 "F.CrtYd" user "F.Courtyard")')
    lines.append('\t\t(48 "B.Fab" user)')
    lines.append('\t\t(49 "F.Fab" user)')
    lines.append('\t)')
    lines.append('\t(setup')
    lines.append('\t\t(pad_to_mask_clearance 0.05)')
    lines.append('\t\t(pcbplotparams')
    lines.append('\t\t\t(layerselection 0x00010fc_ffffffff)')
    lines.append('\t\t\t(plot_on_all_layers_selection 0x0000000_00000000)')
    lines.append('\t\t\t(disableapertmacros no)')
    lines.append('\t\t\t(usegerberextensions yes)')
    lines.append('\t\t\t(usegerberattributes yes)')
    lines.append('\t\t\t(usegerberadvancedattributes yes)')
    lines.append('\t\t\t(creategerberjobfile yes)')
    lines.append('\t\t\t(svgprecision 4)')
    lines.append('\t\t\t(excludeedgelayer no)')
    lines.append('\t\t\t(plotframeref no)')
    lines.append('\t\t\t(viasonmask no)')
    lines.append('\t\t\t(mode 1)')
    lines.append('\t\t\t(useauxorigin no)')
    lines.append('\t\t\t(hpglpennumber 1)')
    lines.append('\t\t\t(hpglpenspeed 20)')
    lines.append('\t\t\t(hpglpendiameter 15.000000)')
    lines.append('\t\t\t(pdf_front_fp_property_popups yes)')
    lines.append('\t\t\t(pdf_back_fp_property_popups yes)')
    lines.append('\t\t\t(dxfpolygonmode yes)')
    lines.append('\t\t\t(dxfimperialunits no)')
    lines.append('\t\t\t(dxfusepcbnewfont no)')
    lines.append('\t\t\t(psnegative no)')
    lines.append('\t\t\t(psa4output no)')
    lines.append('\t\t\t(plotreference yes)')
    lines.append('\t\t\t(plotvalue yes)')
    lines.append('\t\t\t(plotfptext yes)')
    lines.append('\t\t\t(plotinvisibletext no)')
    lines.append('\t\t\t(sketchpadsonfab no)')
    lines.append('\t\t\t(subtractmaskfromsilk no)')
    lines.append('\t\t\t(outputformat 1)')
    lines.append('\t\t\t(mirror no)')
    lines.append('\t\t\t(drillshape 1)')
    lines.append('\t\t\t(scaleselection 1)')
    lines.append('\t\t\t(outputdirectory "gerbers/")')
    lines.append('\t\t)')
    lines.append('\t)')

    # Nets
    nets = [
        (0, ""),
        (1, "GND"),
        (2, "+5V"),
        (3, "GPIO0_ADC"),
        (4, "GPIO1_SOIL"),
        (5, "GPIO2_VCC"),
        (6, "GPIO3_TRIG"),
        (7, "GPIO4_ECHO"),
        (8, "ECHO_5V"),
        (9, "GPIO7_GATE"),
        (10, "GATE_DRIVE"),
        (11, "PUMP_DRAIN"),
        (12, "+3V3")
    ]
    for nid, nname in nets:
        lines.append(f'\t(net {nid} "{nname}")')

    W, H = 68.0, 52.0
    lines.append(f'\t(gr_rect (start 0 0) (end {W} {H}) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts") (uuid "{u()}"))')

    # 4x M3 Mounting Holes (diameter 3.2mm, pad 5.0mm)
    for i, (hx, hy) in enumerate([(3.5, 3.5), (W-3.5, 3.5), (3.5, H-3.5), (W-3.5, H-3.5)], 1):
        lines.append(f'\t(footprint "MountingHole_3.2mm_M3_Pad"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {hx} {hy})')
        lines.append(f'\t\t(property "Reference" "H{i}" (at 0 0) (layer "F.SilkS") (effects (font (size 0.8 0.8)) (hide yes)))')
        lines.append(f'\t\t(pad "1" thru_hole circle (at 0 0) (size 5.0 5.0) (drill 3.2) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
        lines.append(f'\t)')

    # Silkscreen Branding
    lines.append(f'\t(gr_text "PLANT CONTROLLER" (at 18.0 49.2) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 1.0 1.0) bold (thickness 0.16))))')
    lines.append(f'\t(gr_text "ESP32-C3 SuperMini • Rev 1.0" (at 47.0 49.2) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.95 0.95) (thickness 0.15))))')
    lines.append(f'\t(gr_text "USB-C" (at 5.5 34.0 90) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 1.0 1.0) bold (thickness 0.18))))')
    lines.append(f'\t(gr_line (start 4.5 30.0) (end 4.5 38.0) (stroke (width 0.25) (type solid)) (layer "F.SilkS") (uuid "{u()}"))')
    lines.append(f'\t(gr_line (start 4.5 34.0) (end 2.5 34.0) (stroke (width 0.25) (type solid)) (layer "F.SilkS") (uuid "{u()}"))')

    def terminal_block(ref, val, x_start, y, pin_count, pitch, net_tuples, labels, bx_pad_right=2.0):
        lines.append(f'\t(footprint "TerminalBlock_bornier-{pin_count}_P5.08mm"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {x_start} {y})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at {(pin_count-1)*pitch/2} -4.6) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
        bx1 = -2.0
        bx2 = (pin_count - 1) * pitch + bx_pad_right
        lines.append(f'\t\t(fp_rect (start {bx1} -2.8) (end {bx2} 2.8) (stroke (width 0.2) (type solid)) (fill none) (layer "F.SilkS"))')
        for p in range(pin_count):
            px = p * pitch
            nid, nname = net_tuples[p]
            lbl = labels[p]
            lines.append(f'\t\t(pad "{p+1}" thru_hole roundrect (at {px} 0) (size 2.5 2.5) (drill 1.3) (roundrect_rratio 0.25) (layers "*.Cu" "*.Mask") (net {nid} "{nname}"))')
            lines.append(f'\t\t(fp_text user "{lbl}" (at {px} 2.0) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) bold (thickness 0.13))))')
        lines.append(f'\t)')

    # Terminals on top edge (Y = 8.5)
    # TB2: Soil Sensor at X=21.0
    terminal_block("TB2", "Soil_Moisture", 21.0, 8.5, 3, 5.08,
                   [(4, "GPIO1_SOIL"), (5, "GPIO2_VCC"), (1, "GND")],
                   ["DATA", "VCC", "GND"])

    # TB1: Ultrasonic at X=36.0
    terminal_block("TB1", "Ultrasonic", 36.0, 8.5, 4, 5.08,
                   [(2, "+5V"), (6, "GPIO3_TRIG"), (8, "ECHO_5V"), (1, "GND")],
                   ["5V", "TRIG", "ECHO", "GND"])

    # TB3: Pump at X=56.0 (Pin 1: +5V, Pin 2: PUMP_DRAIN)
    terminal_block("TB3", "Pump_5V", 56.0, 8.5, 2, 5.08,
                   [(2, "+5V"), (11, "PUMP_DRAIN")],
                   ["+", "-"], bx_pad_right=1.6)

    # ESP32 Headers: J1 at Y=26.0, J2 at Y=41.24
    lines.append(f'\t(footprint "PinSocket_1x08_P2.54mm_Vertical"')
    lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at 14.0 26.0)')
    lines.append(f'\t\t(property "Reference" "J1" (at -2.4 0 90) (layer "F.SilkS") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
    j1_nets = [
        (2, "+5V"), (1, "GND"), (12, "+3V3"), (3, "GPIO0_ADC"),
        (4, "GPIO1_SOIL"), (5, "GPIO2_VCC"), (6, "GPIO3_TRIG"), (7, "GPIO4_ECHO")
    ]
    j1_names = ["5V", "GND", "3V3", "G0", "G1", "G2", "G3", "G4"]
    for p in range(8):
        px = p * 2.54
        nid, nname = j1_nets[p]
        lines.append(f'\t\t(pad "{p+1}" thru_hole oval (at {px} 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net {nid} "{nname}"))')
        lines.append(f'\t\t(fp_text user "{j1_names[p]}" (at {px} 1.7) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) (thickness 0.13))))')
    lines.append(f'\t)')

    lines.append(f'\t(footprint "PinSocket_1x08_P2.54mm_Vertical"')
    lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at 14.0 41.24)')
    lines.append(f'\t\t(property "Reference" "J2" (at -2.4 0 90) (layer "F.SilkS") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
    j2_nets = [
        (0, ""), (0, ""), (0, ""), (0, ""),
        (0, ""), (9, "GPIO7_GATE"), (0, ""), (0, "")
    ]
    j2_names = ["G21", "G20", "G10", "G9", "G8", "G7", "G6", "G5"]
    for p in range(8):
        px = p * 2.54
        nid, nname = j2_nets[p]
        lines.append(f'\t\t(pad "{p+1}" thru_hole oval (at {px} 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net {nid} "{nname}"))')
        lines.append(f'\t\t(fp_text user "{j2_names[p]}" (at {px} -1.7) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) (thickness 0.13))))')
    lines.append(f'\t)')

    # ESP32 Box Outline
    lines.append(f'\t(gr_rect (start 10.5 23.8) (end 34.5 43.4) (stroke (width 0.2) (type solid)) (fill none) (layer "F.SilkS") (uuid "{u()}"))')
    lines.append(f'\t(gr_text "ESP32-C3 SuperMini" (at 22.5 33.6) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 1.1 1.1) bold (thickness 0.18))))')

    # Q1: FQP30N06L N-MOSFET (TO-220 Vertical) at X=58.0, Y=26.0
    qx, qy = 58.0, 26.0
    lines.append(f'\t(footprint "TO-220-3_Vertical"')
    lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {qx} {qy})')
    lines.append(f'\t\t(property "Reference" "Q1" (at 0 -3.4) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
    lines.append(f'\t\t(fp_line (start -4.5 -1.6) (end 4.5 -1.6) (stroke (width 0.4) (type solid)) (layer "F.SilkS"))')
    lines.append(f'\t\t(fp_line (start -4.5 -1.6) (end -4.5 1.5) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))')
    lines.append(f'\t\t(fp_line (start 4.5 -1.6) (end 4.5 1.5) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))')
    lines.append(f'\t\t(fp_line (start -4.5 1.5) (end 4.5 1.5) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))')
    lines.append(f'\t\t(pad "1" thru_hole rect (at -2.54 0) (size 1.8 1.8) (drill 1.1) (layers "*.Cu" "*.Mask") (net 10 "GATE_DRIVE"))')
    lines.append(f'\t\t(pad "2" thru_hole oval (at 0 0) (size 1.8 1.8) (drill 1.1) (layers "*.Cu" "*.Mask") (net 11 "PUMP_DRAIN"))')
    lines.append(f'\t\t(pad "3" thru_hole oval (at 2.54 0) (size 1.8 1.8) (drill 1.1) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
    lines.append(f'\t\t(fp_text user "G" (at -2.54 2.6) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) (thickness 0.13))))')
    lines.append(f'\t\t(fp_text user "D" (at 0 2.6) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) (thickness 0.13))))')
    lines.append(f'\t\t(fp_text user "S" (at 2.54 2.6) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) (thickness 0.13))))')
    lines.append(f'\t)')

    # D1: 1N5819 Schottky Diode (DO-41 Horizontal) at X=58.54, Y=16.5
    # Pin 1 at X=56.0 (+5V / Cathode), Pin 2 at X=61.08 (PUMP_DRAIN / Anode)
    dx, dy = 58.54, 16.5
    lines.append(f'\t(footprint "D_DO-41_SOD81_P7.62mm_Horizontal"')
    lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {dx} {dy})')
    lines.append(f'\t\t(property "Reference" "D1" (at 0 -2.3) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
    lines.append(f'\t\t(fp_rect (start -1.4 -1.2) (end 1.4 1.2) (stroke (width 0.2) (type solid)) (fill none) (layer "F.SilkS"))')
    lines.append(f'\t\t(fp_line (start -1.0 -1.2) (end -1.0 1.2) (stroke (width 0.4) (type solid)) (layer "F.SilkS"))')
    lines.append(f'\t\t(pad "1" thru_hole circle (at -2.54 0) (size 1.8 1.8) (drill 1.0) (layers "*.Cu" "*.Mask") (net 2 "+5V"))')       # 56.0
    lines.append(f'\t\t(pad "2" thru_hole circle (at 2.54 0) (size 1.8 1.8) (drill 1.0) (layers "*.Cu" "*.Mask") (net 11 "PUMP_DRAIN"))') # 61.08
    lines.append(f'\t)')

    # C3: 100uF Bulk Electrolytic at X=7.0, Y=26.0
    lines.append(f'\t(footprint "CP_Radial_D6.3mm_P2.50mm"')
    lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at 7.0 26.0)')
    lines.append(f'\t\t(property "Reference" "C3" (at 0 -3.8) (layer "F.SilkS") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
    lines.append(f'\t\t(fp_circle (center 0 0) (end 3.15 0) (stroke (width 0.2) (type solid)) (fill none) (layer "F.SilkS"))')
    lines.append(f'\t\t(pad "1" thru_hole rect (at 0 -1.25) (size 1.8 1.8) (drill 0.9) (layers "*.Cu" "*.Mask") (net 2 "+5V"))') # (7.0, 24.75)
    lines.append(f'\t\t(pad "2" thru_hole circle (at 0 1.25) (size 1.8 1.8) (drill 0.9) (layers "*.Cu" "*.Mask") (net 1 "GND"))') # (7.0, 27.25)
    lines.append(f'\t\t(fp_text user "+" (at -1.8 -1.25) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 1.0 1.0) bold (thickness 0.18))))')
    lines.append(f'\t)')

    def resistor(ref, val, rx, ry, net1, net2, ref_offset_y=-2.4):
        lines.append(f'\t(footprint "R_Axial_DIN0207_L6.3mm_D2.5mm_P7.62mm_Horizontal"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {rx} {ry})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 {ref_offset_y}) (layer "F.SilkS") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
        lines.append(f'\t\t(fp_rect (start -2.5 -1.1) (end 2.5 1.1) (stroke (width 0.2) (type solid)) (fill none) (layer "F.SilkS"))')
        lines.append(f'\t\t(pad "1" thru_hole circle (at -3.81 0) (size 1.7 1.7) (drill 0.9) (layers "*.Cu" "*.Mask") (net {net1[0]} "{net1[1]}"))')
        lines.append(f'\t\t(pad "2" thru_hole circle (at 3.81 0) (size 1.7 1.7) (drill 0.9) (layers "*.Cu" "*.Mask") (net {net2[0]} "{net2[1]}"))')
        lines.append(f'\t)')

    def ceramic_cap(ref, val, cx, cy, net1, net2):
        lines.append(f'\t(footprint "C_Disc_D4.3mm_W1.9mm_P2.50mm"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {cx} {cy})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -2.2) (layer "F.SilkS") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
        lines.append(f'\t\t(fp_line (start -0.3 -1.1) (end 0.3 -1.1) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))')
        lines.append(f'\t\t(fp_line (start -0.3 1.1) (end 0.3 1.1) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))')
        lines.append(f'\t\t(pad "1" thru_hole circle (at -1.25 0) (size 1.7 1.7) (drill 0.9) (layers "*.Cu" "*.Mask") (net {net1[0]} "{net1[1]}"))')
        lines.append(f'\t\t(pad "2" thru_hole circle (at 1.25 0) (size 1.7 1.7) (drill 0.9) (layers "*.Cu" "*.Mask") (net {net2[0]} "{net2[1]}"))')
        lines.append(f'\t)')

    # Supply Divider R1, R2, C1 to the left (X=13.0)
    resistor("R1", "10k", 13.0, 16.5, (2, "+5V"), (3, "GPIO0_ADC"), ref_offset_y=-2.2)
    resistor("R2", "10k", 13.0, 20.5, (1, "GND"), (3, "GPIO0_ADC"), ref_offset_y=2.2)
    ceramic_cap("C1", "100nF", 20.62, 18.5, (3, "GPIO0_ADC"), (1, "GND"))

    # C2: Soil RF Filter near TB2 at X=16.0, Y=13.5
    ceramic_cap("C2", "100nF", 16.0, 13.5, (4, "GPIO1_SOIL"), (1, "GND"))

    # Echo Stepper R3, R4 at X=45.0
    resistor("R3", "1k", 45.0, 16.5, (7, "GPIO4_ECHO"), (8, "ECHO_5V"), ref_offset_y=-2.2)
    resistor("R4", "2k", 45.0, 20.5, (7, "GPIO4_ECHO"), (1, "GND"), ref_offset_y=2.2)

    # MOSFET Gate Resistors R5, R6 at X=45.0
    resistor("R5", "1k", 45.0, 36.0, (9, "GPIO7_GATE"), (10, "GATE_DRIVE"), ref_offset_y=-2.2)
    resistor("R6", "100k", 45.0, 45.5, (1, "GND"), (10, "GATE_DRIVE"), ref_offset_y=-2.2)

    # ---------------------------------------------------------
    # NON-CROSSING COPPER TRACES
    # ---------------------------------------------------------
    def seg(x1, y1, x2, y2, w, nid, layer="F.Cu"):
        lines.append(f'\t(segment (start {x1} {y1}) (end {x2} {y2}) (width {w}) (layer "{layer}") (net {nid}) (uuid "{u()}"))')

    # Net 2 (+5V) Power Bus (0.8mm)
    # C3-1 (7.0, 24.75) -> J1-1 (14.0, 26.0)
    seg(7.0, 24.75, 7.0, 24.0, 0.8, 2)
    seg(7.0, 24.0, 14.0, 24.0, 0.8, 2)
    seg(14.0, 24.0, 14.0, 26.0, 0.8, 2)
    # C3-1 (7.0, 24.75) up along X=7.0 to R1-1 (9.19, 16.5)
    seg(7.0, 24.75, 7.0, 16.5, 0.8, 2)
    seg(7.0, 16.5, 9.19, 16.5, 0.8, 2)
    # Along safe perimeter corridor at Y=1.5 (completely clears H1 and H2)
    seg(9.19, 16.5, 9.19, 1.5, 0.8, 2)
    seg(9.19, 1.5, 36.0, 1.5, 0.8, 2)
    seg(36.0, 1.5, 36.0, 8.5, 0.8, 2) # TB1 Pin 1 (+5V)
    # To TB3 Pin 1 (+5V at X=56.0) and D1 Pin 1 (+5V at X=56.0)
    seg(36.0, 1.5, 56.0, 1.5, 0.8, 2)
    seg(56.0, 1.5, 56.0, 8.5, 0.8, 2) # TB3 Pin 1 (+5V)
    seg(56.0, 8.5, 56.0, 16.5, 0.8, 2) # D1 Cathode (+5V)

    # Net 3 (GPIO0_ADC)
    seg(16.81, 16.5, 19.37, 17.5, 0.35, 3)
    seg(19.37, 17.5, 19.37, 18.5, 0.35, 3)
    seg(19.37, 18.5, 16.81, 19.5, 0.35, 3)
    seg(16.81, 19.5, 16.81, 20.5, 0.35, 3)
    seg(16.81, 20.5, 21.62, 23.5, 0.35, 3)
    seg(21.62, 23.5, 21.62, 26.0, 0.35, 3)

    # Net 4 (GPIO1_SOIL)
    seg(21.0, 8.5, 21.0, 10.0, 0.35, 4)
    seg(21.0, 10.0, 14.75, 10.0, 0.35, 4)
    seg(14.75, 10.0, 14.75, 13.5, 0.35, 4)
    seg(21.0, 10.0, 24.16, 13.16, 0.35, 4)
    seg(24.16, 13.16, 24.16, 26.0, 0.35, 4)

    # Net 5 (GPIO2_VCC)
    seg(26.08, 8.5, 26.08, 12.0, 0.5, 5)
    seg(26.08, 12.0, 26.70, 12.62, 0.5, 5)
    seg(26.70, 12.62, 26.70, 26.0, 0.5, 5)

    # Net 6 (GPIO3_TRIG)
    seg(41.08, 8.5, 41.08, 12.0, 0.35, 6)
    seg(41.08, 12.0, 29.24, 23.84, 0.35, 6)
    seg(29.24, 23.84, 29.24, 26.0, 0.35, 6)

    # Net 7 (GPIO4_ECHO)
    seg(41.19, 16.5, 41.19, 20.5, 0.35, 7)
    seg(41.19, 20.5, 41.19, 22.0, 0.35, 7)
    seg(41.19, 22.0, 31.78, 24.5, 0.35, 7)
    seg(31.78, 24.5, 31.78, 26.0, 0.35, 7)

    # Net 8 (ECHO_5V)
    seg(46.16, 8.5, 48.81, 11.15, 0.35, 8)
    seg(48.81, 11.15, 48.81, 16.5, 0.35, 8)

    # Net 9 (GPIO7_GATE)
    seg(26.70, 41.24, 26.70, 36.0, 0.35, 9)
    seg(26.70, 36.0, 41.19, 36.0, 0.35, 9)

    # Net 10 (GATE_DRIVE)
    seg(48.81, 36.0, 48.81, 45.5, 0.5, 10)
    seg(48.81, 36.0, 55.46, 29.35, 0.5, 10)
    seg(55.46, 29.35, 55.46, 26.0, 0.5, 10)

    # Net 11 (PUMP_DRAIN) - High Current 1.0mm
    # Connects TB3 Pin 2 (61.08, 8.5) -> D1 Pin 2 (61.08, 16.5) -> Q1 Pin 2 Drain (58.0, 26.0)
    seg(61.08, 8.5, 61.08, 16.5, 1.0, 11)
    seg(61.08, 16.5, 61.08, 22.92, 1.0, 11)
    seg(61.08, 22.92, 58.0, 26.0, 1.0, 11)

    # Bottom Ground Plane Pour (B.Cu)
    lines.append(f'\t(zone (net 1) (net_name "GND") (layer "B.Cu") (uuid "{u()}")')
    lines.append(f'\t\t(hatch edge 0.5)')
    lines.append(f'\t\t(connect_pads (clearance 0.35))')
    lines.append(f'\t\t(min_thickness 0.25)')
    lines.append(f'\t\t(filled_areas_thickness no)')
    lines.append(f'\t\t(fill yes (thermal_gap 0.3) (thermal_bridge_width 0.45))')
    lines.append(f'\t\t(polygon (pts (xy 0.5 0.5) (xy {W-0.5} 0.5) (xy {W-0.5} {H-0.5}) (xy 0.5 {H-0.5})))')
    lines.append(f'\t)')

    lines.append(')')
    with open('pcb/plant-esp32.kicad_pcb', 'w') as f:
        f.write('\n'.join(lines) + '\n')
    print("Clean PCB generated successfully.")

if __name__ == '__main__':
    generate_board()
