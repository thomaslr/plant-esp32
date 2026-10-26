import subprocess
import os
import json
import uuid
import math
from precision_router import PrecisionRouter

def u():
    return str(uuid.uuid4())

def build_board():
    router = PrecisionRouter(width=74.0, height=54.0, res=0.25)
    
    # 1. Add all footprint pads to the router
    # USB-C J1 (X=4.5, Y=34.25, rot=270)
    # Pads are at X=8.545, size 1.45 along X, height pw along Y
    j1_router_pads = [
        (31.0, 1, 0.6),   # A1/B12 GND
        (31.8, 2, 0.6),   # A4/B9 +5V
        (33.0, 24, 0.3),  # A5 CC1
        (33.5, 20, 0.3),  # B7 DN
        (34.0, 19, 0.3),  # A6 DP
        (34.5, 20, 0.3),  # A7 DN
        (35.0, 19, 0.3),  # B6 DP
        (36.0, 25, 0.3),  # B5 CC2
        (36.7, 2, 0.6),   # A9/B4 +5V
        (37.5, 1, 0.6)    # A12/B1 GND
    ]
    for py, nid, pw in j1_router_pads:
        router.add_pad(8.545, py, 1.45, pw, 0, nid, clearance=0.15)
        
    for sx, sy in [(7.63, 29.93), (7.63, 38.57), (3.45, 29.93), (3.45, 38.57)]:
        router.add_pad(sx, sy, 2.1, 1.0, None, 1, clearance=0.25)

    # U2 AMS1117-3.3 (X=8.5, Y=23.0)
    router.add_pad(6.2, 26.1, 1.0, 1.5, 0, 1, clearance=0.20)   # Pin 1 GND
    router.add_pad(8.5, 26.1, 1.0, 1.5, 0, 12, clearance=0.20)  # Pin 2 OUT
    router.add_pad(10.8, 26.1, 1.0, 1.5, 0, 2, clearance=0.20)  # Pin 3 IN
    router.add_pad(8.5, 19.9, 3.3, 1.5, 0, 12, clearance=0.20)  # Tab OUT

    # ESP32-C3-WROOM-02 U1 (X=27.0, Y=35.0, rot=180)
    # West side pins (X=18.25)
    west_pins = [
        (29.0, 13), # Pin 10 IO10 TRIG
        (30.5, 15), # Pin 11 IO20 SDA
        (32.0, 14), # Pin 12 IO21 SCL
        (33.5, 20), # Pin 13 USB_DN
        (35.0, 19), # Pin 14 USB_DP
        (36.5, 6),  # Pin 15 IO3 LDR
        (38.0, 0),  # Pin 16 NC
        (39.5, 4),  # Pin 17 IO1 SOIL
        (41.0, 3)   # Pin 18 IO0 5V ADC
    ]
    for py, nid in west_pins:
        router.add_pad(18.25, py, 1.5, 0.9, 0, nid, clearance=0.15)

    # East side pins (X=35.75)
    east_pins = [
        (41.0, 12), # Pin 1 +3V3
        (39.5, 21), # Pin 2 ESP_EN
        (38.0, 7),  # Pin 3 IO4 VBAT ADC
        (36.5, 5),  # Pin 4 IO5 SOIL VCC
        (35.0, 17), # Pin 5 IO6 ECHO
        (33.5, 9),  # Pin 6 IO7 GATE
        (32.0, 23), # Pin 7 IO8 LED
        (30.5, 22), # Pin 8 IO9 BOOT
        (29.0, 1)   # Pin 9 GND
    ]
    for py, nid in east_pins:
        router.add_pad(35.75, py, 1.5, 0.9, 0, nid, clearance=0.15)
    router.add_pad(26.04, 34.8, 2.9, 2.9, 0, 1, clearance=0.20)

    # SW2 (BOOT) at (47.0, 22.5)
    router.add_pad(44.0, 20.625, 1.0, 0.75, 0, 1, clearance=0.20)
    router.add_pad(50.0, 20.625, 1.0, 0.75, 0, 1, clearance=0.20)
    router.add_pad(44.0, 24.375, 1.0, 0.75, 0, 22, clearance=0.20)
    router.add_pad(50.0, 24.375, 1.0, 0.75, 0, 22, clearance=0.20)

    # SW1 (RST) at (47.0, 30.5)
    router.add_pad(44.0, 28.625, 1.0, 0.75, 0, 21, clearance=0.20)
    router.add_pad(50.0, 28.625, 1.0, 0.75, 0, 21, clearance=0.20)
    router.add_pad(44.0, 32.375, 1.0, 0.75, 0, 1, clearance=0.20)
    router.add_pad(50.0, 32.375, 1.0, 0.75, 0, 1, clearance=0.20)

    # 0805 passives
    def add_0805_h(cx, cy, net1, net2):
        router.add_pad(cx - 0.95, cy, 1.0, 1.25, 0, net1, clearance=0.20)
        router.add_pad(cx + 0.95, cy, 1.0, 1.25, 0, net2, clearance=0.20)

    add_0805_h(15.0, 16.0, 12, 26)  # D3 PWR LED
    add_0805_h(18.5, 16.0, 26, 1)   # R14 1k
    # D4 STAT LED with larger clearance to keep routes away from Pad 2
    router.add_pad(15.0 - 0.95, 19.0, 1.0, 1.25, 0, 12, clearance=0.20)
    router.add_pad(15.0 + 0.95, 19.0, 1.0, 1.25, 0, 27, clearance=0.35)
    add_0805_h(18.5, 19.0, 27, 23)  # R15 1k (STAT_LED_K, GPIO8_LED)
    add_0805_h(12.0, 30.0, 24, 1)   # R12 5.1k (CC1, GND)
    add_0805_h(12.0, 39.0, 25, 1)   # R13 5.1k (CC2, GND)

    add_0805_h(40.0, 24.5, 12, 22)  # R11 10k (BOOT pullup)
    add_0805_h(40.0, 28.5, 12, 21)  # R10 10k (EN pullup)
    add_0805_h(40.0, 31.5, 1, 21)   # C9 1uF (EN cap)

    add_0805_h(2.5, 19.0, 2, 1)     # C10 10uF
    add_0805_h(2.5, 23.0, 2, 1)     # C7 100nF
    add_0805_h(8.5, 14.5, 12, 1)    # C11 10uF
    add_0805_h(8.5, 17.0, 12, 1)    # C8 100nF
    add_0805_h(40.5, 41.0, 12, 1)   # C12 22uF
    add_0805_h(11.0, 41.5, 2, 3)    # R1 100k
    add_0805_h(11.0, 44.5, 1, 3)    # R2 100k
    add_0805_h(15.5, 42.5, 1, 3)    # C1 100nF
    add_0805_h(44.0, 36.0, 7, 18)   # R8 100k
    add_0805_h(44.0, 38.5, 7, 1)    # R9 100k
    add_0805_h(40.5, 38.0, 7, 1)    # C6 100nF
    add_0805_h(41.5, 16.5, 8, 17)   # R3 1k
    add_0805_h(41.5, 19.5, 17, 1)   # R4 2k
    add_0805_h(15.5, 38.0, 4, 1)    # C2 100nF
    add_0805_h(30.0, 23.0, 1, 6)    # R7 10k
    add_0805_h(30.0, 25.5, 1, 6)    # C4 100nF
    add_0805_h(50.0, 34.0, 9, 10)   # R5 1k
    add_0805_h(54.0, 34.0, 10, 1)   # R6 100k

    # Q1 SOT-23 at (58.0, 33.0)
    router.add_pad(58.0 - 0.95, 33.0 + 1.0, 0.8, 0.9, 0, 10, clearance=0.20) # G
    router.add_pad(58.0 + 0.95, 33.0 + 1.0, 0.8, 0.9, 0, 1, clearance=0.20)  # S
    router.add_pad(58.0, 33.0 - 1.0, 0.8, 0.9, 0, 11, clearance=0.20)        # D

    # D1 & D2 SMA
    router.add_pad(54.0 - 2.0, 18.0, 1.5, 1.8, 0, 2, clearance=0.20)   # D1 K (+5V)
    router.add_pad(54.0 + 2.0, 18.0, 1.5, 1.8, 0, 11, clearance=0.20)  # D1 A (DRAIN)
    router.add_pad(63.5 - 2.0, 22.0, 1.5, 1.8, 0, 2, clearance=0.20)   # D2 K (+5V)
    router.add_pad(63.5 + 2.0, 22.0, 1.5, 1.8, 0, 16, clearance=0.20)  # D2 A (EXT_5V)

    # Through-hole connectors (KF301, Sockets, Headers)
    def add_kf301_h(x_start, y, pin_count, net_ids, pitch=5.0):
        for p in range(pin_count):
            px = x_start + p * pitch
            router.add_pad(px, y, 2.5, 2.5, None, net_ids[p], clearance=0.25)

    def add_kf301_v(x, y_start, pin_count, net_ids, pitch=5.0):
        for p in range(pin_count):
            py = y_start + p * pitch
            router.add_pad(x, py, 2.5, 2.5, None, net_ids[p], clearance=0.25)

    add_kf301_h(11.7, 7.5, 3, [4, 5, 1])   # TB2
    add_kf301_h(33.3, 7.5, 4, [2, 13, 8, 1]) # TB1
    add_kf301_h(55.0, 7.5, 2, [2, 11])    # TB3
    add_kf301_v(66.5, 31.0, 3, [16, 1, 18]) # TB4

    # SHT30 & BH1750 sockets
    for p in range(4):
        router.add_pad(48.0 + p*2.54, 44.0, 1.6, 1.6, None, [12, 1, 14, 15][p], clearance=0.25)
    for p in range(5):
        router.add_pad(46.7 + p*2.54, 49.5, 1.6, 1.6, None, [12, 1, 14, 15, 1][p], clearance=0.25)

    # LDR
    router.add_pad(28.73, 19.0, 1.6, 1.6, None, 12, clearance=0.38)
    router.add_pad(31.27, 19.0, 1.6, 1.6, None, 6, clearance=0.25)

    # 2. Register all SMD Ground Vias and their stubs in the router BEFORE routing!
    smd_gnd_vias = [
        # (pad_x, pad_y, via_x, via_y)
        (6.2, 26.1, 5.0, 26.1),        # U2 Pin 1 GND
        (3.45, 19.0, 4.40, 19.0),      # C10 GND
        (3.45, 23.0, 4.40, 23.0),      # C7 GND
        (9.45, 14.5, 10.40, 14.5),     # C11 GND
        (9.45, 17.0, 10.40, 17.0),     # C8 GND
        (19.45, 16.0, 20.40, 16.0),    # R14 (PWR LED) GND
        (12.95, 30.0, 13.90, 30.0),    # R12 (CC1 5.1k) GND
        (12.95, 39.0, 12.95, 40.2),    # R13 (CC2 5.1k) GND
        (10.05, 44.5, 9.10, 44.5),     # R2 (5V ADC) GND
        (14.55, 42.5, 13.60, 42.5),    # C1 (5V ADC cap) GND
        (16.45, 38.0, 16.45, 36.5),    # C2 (Soil sense cap) GND
        (29.05, 23.0, 26.50, 23.0),    # R7 (LDR bottom) GND
        (29.05, 25.5, 26.50, 25.5),    # C4 (LDR cap) GND
        (42.45, 19.5, 43.40, 19.5),    # R4 (Echo bottom) GND
        (41.45, 38.0, 41.45, 36.5),    # C6 (VBAT cap) GND
        (44.95, 38.5, 46.20, 38.5),    # R9 (VBAT bottom) GND
        (41.45, 41.0, 42.40, 41.0),    # C12 (3V3 bulk cap) GND
        (39.05, 31.5, 38.10, 31.5),    # C9 (EN cap) GND
        (54.95, 34.0, 55.90, 34.0),    # R6 (Gate pull-down) GND
        (58.95, 34.0, 59.90, 34.0),    # Q1 Source GND
        (35.75, 29.0, 37.00, 29.0),    # ESP32 Pin 9 GND
        (44.0, 20.625, 42.5, 20.625),  # SW2 (BOOT) GND
        (44.0, 32.375, 42.5, 32.375),  # SW1 (RST) GND
    ]

    all_segs = []
    all_vias = []

    for px, py, vx, vy in smd_gnd_vias:
        all_segs.append((px, py, vx, vy, 0.35, 1, "F.Cu"))
        all_vias.append((vx, vy, 1))
        router.mark_via(vx, vy, 1)
        router.mark_trace([(px, py, 0), (vx, vy, 0)], 1, trace_radius=0.25)

    # ESP32 Center thermal pad 4 ground vias
    for tvx, tvy in [(25.0, 34.0), (27.0, 34.0), (25.0, 35.6), (27.0, 35.6)]:
        all_vias.append((tvx, tvy, 1))
        router.mark_via(tvx, tvy, 1)

    # Pre-mark dedicated local power & signal routes
    # 1. Connect C10 & C7 +5V pads to U2 Pin 3 (going NORTH of U2 at Y=28.0)
    c_pwr_segs = [
        (1.55, 19.0, 1.55, 28.0, 0.50, 2, "F.Cu"),
        (1.55, 28.0, 10.8, 28.0, 0.50, 2, "F.Cu"),
        (10.8, 28.0, 10.8, 26.1, 0.50, 2, "F.Cu"),
    ]
    for seg in c_pwr_segs:
        all_segs.append(seg)
        router.mark_trace([(seg[0], seg[1], 0), (seg[2], seg[3], 0)], 2, trace_radius=0.35)

    # 2. Connect +3V3 from C11 (7.55, 14.5) to LEDs D3/D4 at Y=14.5
    led_3v3_segs = [
        (7.55, 14.5, 7.55, 13.5, 0.40, 12, "F.Cu"),
        (7.55, 13.5, 14.05, 13.5, 0.40, 12, "F.Cu"),
        (14.05, 13.5, 14.05, 16.0, 0.40, 12, "F.Cu"),
        (14.05, 16.0, 14.05, 19.0, 0.40, 12, "F.Cu"),
    ]
    for seg in led_3v3_segs:
        all_segs.append(seg)
        router.mark_trace([(seg[0], seg[1], 0), (seg[2], seg[3], 0)], 12, trace_radius=0.30)

    # C12 to MOD1 +3V3 via escape
    all_segs.append((39.55, 41.0, 39.55, 42.5, 0.40, 12, "F.Cu"))
    all_vias.append((39.55, 42.5, 12))
    router.mark_via(39.55, 42.5, 12)
    router.mark_trace([(39.55, 41.0, 0), (39.55, 42.5, 0)], 12, trace_radius=0.25)

    # ESP_EN Pin 2 to B.Cu escape
    all_segs.append((35.75, 39.5, 37.2, 39.5, 0.30, 21, "F.Cu"))
    all_vias.append((37.2, 39.5, 21))
    router.mark_via(37.2, 39.5, 21)
    router.mark_trace([(35.75, 39.5, 0), (37.2, 39.5, 0)], 21, trace_radius=0.20)

    # C9 Pad 2 to B.Cu escape
    all_segs.append((40.95, 31.5, 41.95, 31.5, 0.30, 21, "F.Cu"))
    all_vias.append((41.95, 31.5, 21))
    router.mark_via(41.95, 31.5, 21)
    router.mark_trace([(40.95, 31.5, 0), (41.95, 31.5, 0)], 21, trace_radius=0.20)

    # 3. USB-C J1 Escapes
    # GND pads: vias placed cleanly
    all_segs.append((8.545, 31.0, 7.3, 31.0, 0.35, 1, "F.Cu"))
    all_vias.append((7.3, 31.0, 1))
    router.mark_via(7.3, 31.0, 1)
    router.mark_trace([(8.545, 31.0, 0), (7.3, 31.0, 0)], 1, trace_radius=0.25)

    all_segs.append((8.545, 37.5, 7.3, 37.5, 0.35, 1, "F.Cu"))
    all_vias.append((7.3, 37.5, 1))
    router.mark_via(7.3, 37.5, 1)
    router.mark_trace([(8.545, 37.5, 0), (7.3, 37.5, 0)], 1, trace_radius=0.25)

    # +5V top & bottom:
    # Bottom: (8.545, 31.8) -> (9.6, 31.8) on F.Cu, drops via to B.Cu -> (9.6, 26.1) -> (10.8, 26.1) U2 Pin 3
    all_segs.append((8.545, 31.8, 9.6, 31.8, 0.35, 2, "F.Cu"))
    all_vias.append((9.6, 31.8, 2))
    all_segs.append((9.6, 31.8, 9.6, 26.1, 0.40, 2, "B.Cu"))
    all_segs.append((9.6, 26.1, 10.8, 26.1, 0.40, 2, "B.Cu"))
    all_vias.append((10.8, 26.1, 2))
    router.mark_via(9.6, 31.8, 2)
    router.mark_via(10.8, 26.1, 2)
    router.mark_trace([(9.6, 31.8, 1), (9.6, 26.1, 1), (10.8, 26.1, 1)], 2, trace_radius=0.30)

    # Top: (8.545, 36.7) -> (9.6, 36.7) on F.Cu, drops via to B.Cu -> (9.6, 41.5) -> (10.05, 41.5) R1 Pad 1
    all_segs.append((8.545, 36.7, 9.6, 36.7, 0.35, 2, "F.Cu"))
    all_vias.append((9.6, 36.7, 2))
    all_segs.append((9.6, 36.7, 9.6, 41.5, 0.40, 2, "B.Cu"))
    all_segs.append((9.6, 41.5, 10.05, 41.5, 0.40, 2, "B.Cu"))
    all_vias.append((10.05, 41.5, 2))
    router.mark_via(9.6, 36.7, 2)
    router.mark_via(10.05, 41.5, 2)
    router.mark_trace([(9.6, 36.7, 1), (9.6, 41.5, 1), (10.05, 41.5, 1)], 2, trace_radius=0.30)

    # Tie bottom +5V to top +5V on B.Cu at X=9.6
    all_segs.append((9.6, 31.8, 9.6, 36.7, 0.40, 2, "B.Cu"))
    router.mark_trace([(9.6, 31.8, 1), (9.6, 36.7, 1)], 2, trace_radius=0.30)

    # CC1 (A5 at 33.0) -> R12 at (12.0, 30.0): Pad 1 is at (11.05, 30.0)
    # Exits pad straight to X=10.2, steps south to Y=30.0 on F.Cu (crossing +5V on B.Cu cleanly)
    all_segs.append((8.545, 33.0, 10.2, 33.0, 0.18, 24, "F.Cu"))
    all_segs.append((10.2, 33.0, 10.2, 30.0, 0.18, 24, "F.Cu"))
    all_segs.append((10.2, 30.0, 11.05, 30.0, 0.18, 24, "F.Cu"))
    router.mark_trace([(8.545, 33.0, 0), (10.2, 33.0, 0), (10.2, 30.0, 0), (11.05, 30.0, 0)], 24, trace_radius=0.18)

    # CC2 (B5 at 36.0) -> R13 at (12.0, 39.0): Pad 1 is at (11.05, 39.0)
    # Exits pad straight to X=10.2, steps north to Y=39.0 on F.Cu (crossing +5V on B.Cu cleanly)
    all_segs.append((8.545, 36.0, 10.2, 36.0, 0.18, 25, "F.Cu"))
    all_segs.append((10.2, 36.0, 10.2, 39.0, 0.18, 25, "F.Cu"))
    all_segs.append((10.2, 39.0, 11.05, 39.0, 0.18, 25, "F.Cu"))
    router.mark_trace([(8.545, 36.0, 0), (10.2, 36.0, 0), (10.2, 39.0, 0), (11.05, 39.0, 0)], 25, trace_radius=0.18)

    # USB Data Escapes:
    # B7 (DN at 33.5): straight horizontal on F.Cu all the way to ESP32 Pin 13 (18.25, 33.5)
    all_segs.append((8.545, 33.5, 18.25, 33.5, 0.18, 20, "F.Cu"))
    router.mark_trace([(8.545, 33.5, 0), (18.25, 33.5, 0)], 20, trace_radius=0.45)
    router.no_via_zones.append((13.4, 33.5, 5.5))  # Keep vias away from DN corridor

    # B6 (DP at 35.0): straight horizontal on F.Cu all the way to ESP32 Pin 14 (18.25, 35.0)
    all_segs.append((8.545, 35.0, 18.25, 35.0, 0.18, 19, "F.Cu"))
    router.mark_trace([(8.545, 35.0, 0), (18.25, 35.0, 0)], 19, trace_radius=0.45)
    router.no_via_zones.append((13.4, 35.0, 5.5))  # Keep vias away from DP corridor

    # A6 (DP at 34.0): straight to (10.5, 34.0) on F.Cu -> via to B.Cu -> (10.5, 35.0) -> via to F.Cu merging into B6
    all_segs.append((8.545, 34.0, 10.5, 34.0, 0.18, 19, "F.Cu"))
    all_vias.append((10.5, 34.0, 19))
    all_segs.append((10.5, 34.0, 10.5, 35.0, 0.18, 19, "B.Cu"))
    all_vias.append((10.5, 35.0, 19))
    router.mark_via(10.5, 34.0, 19)
    router.mark_via(10.5, 35.0, 19)
    router.mark_trace([(8.545, 34.0, 0), (10.5, 34.0, 0)], 19, trace_radius=0.18)
    router.mark_trace([(10.5, 34.0, 1), (10.5, 35.0, 1)], 19, trace_radius=0.18)

    # A7 (DN at 34.5): straight to (12.0, 34.5) on F.Cu -> via to B.Cu -> (12.0, 33.5) -> via to F.Cu merging into B7
    all_segs.append((8.545, 34.5, 12.0, 34.5, 0.18, 20, "F.Cu"))
    all_vias.append((12.0, 34.5, 20))
    all_segs.append((12.0, 34.5, 12.0, 33.5, 0.18, 20, "B.Cu"))
    all_vias.append((12.0, 33.5, 20))
    router.mark_via(12.0, 34.5, 20)
    router.mark_via(12.0, 33.5, 20)
    router.mark_trace([(8.545, 34.5, 0), (12.0, 34.5, 0)], 20, trace_radius=0.18)
    router.mark_trace([(12.0, 34.5, 1), (12.0, 33.5, 1)], 20, trace_radius=0.18)

    # Button paired pins
    all_segs.append((44.0, 20.625, 50.0, 20.625, 0.30, 1, "F.Cu"))
    all_segs.append((44.0, 24.375, 50.0, 24.375, 0.30, 22, "F.Cu"))
    all_segs.append((44.0, 28.625, 50.0, 28.625, 0.30, 21, "F.Cu"))
    all_segs.append((44.0, 32.375, 50.0, 32.375, 0.30, 1, "F.Cu"))
    router.mark_trace([(44.0, 20.625, 0), (50.0, 20.625, 0)], 1, trace_radius=0.25)
    router.mark_trace([(44.0, 24.375, 0), (50.0, 24.375, 0)], 22, trace_radius=0.25)
    router.mark_trace([(44.0, 28.625, 0), (50.0, 28.625, 0)], 21, trace_radius=0.25)
    router.mark_trace([(44.0, 32.375, 0), (50.0, 32.375, 0)], 1, trace_radius=0.25)

    print("Pre-marked dedicated routes registered in router.")

    # 3. Define routing netlist for the remaining connections
    netlist = [
        # LEDs
        (26, (15.95, 16.0), (17.55, 16.0), 0, 0),
        (27, (15.95, 19.0), (17.55, 19.0), 0, 0),
        (23, (19.45, 19.0), (35.75, 32.0), 0, 0),
        # 5V Sense
        (3, (11.95, 41.5), (11.95, 44.5), 0, 0),
        (3, (11.95, 41.5), (16.45, 42.5), 0, 0),
        (3, (16.45, 42.5), (18.25, 41.0), 0, 0),
        # Soil Sense & VCC
        (4, (11.7, 7.5), (14.55, 38.0), None, 0),
        (4, (14.55, 38.0), (18.25, 39.5), 0, 0),
        (5, (16.7, 7.5), (35.75, 36.5), None, 0),
        # LDR
        (6, (31.27, 19.0), (30.95, 23.0), None, 0),
        (6, (30.95, 23.0), (30.95, 25.5), 0, 0),
        (6, (30.95, 25.5), (18.25, 36.5), 0, 0),
        # Ultra Trig & Echo
        (13, (38.3, 7.5), (18.25, 29.0), None, 0),
        (8, (43.3, 7.5), (40.55, 16.5), None, 0),
        (17, (42.45, 16.5), (40.55, 19.5), 0, 0),
        (17, (42.45, 16.5), (35.75, 35.0), 0, 0),
        # Ext 5V & Battery Sense
        (16, (66.5, 31.0), (65.5, 22.0), None, 0),
        (18, (66.5, 41.0), (44.95, 36.0), None, 0),
        (7, (43.05, 36.0), (43.05, 38.5), 0, 0),
        (7, (43.05, 38.5), (39.55, 38.0), 0, 0),
        (7, (39.55, 38.0), (35.75, 38.0), 0, 0),
        # Gate & Pump Drain
        (9, (35.75, 33.5), (49.05, 34.0), 0, 0),
        (10, (50.95, 34.0), (53.05, 34.0), 0, 0),
        (10, (53.05, 34.0), (57.05, 34.0), 0, 0),
        (11, (60.0, 7.5), (58.0, 32.0), None, 0),
        (11, (58.0, 32.0), (56.0, 18.0), 0, 0),
        # ESP_EN & BOOT Buttons
        (22, (35.75, 30.5), (40.95, 24.5), 0, 0),
        (22, (40.95, 24.5), (44.0, 24.375), 0, 0),
        (21, (37.2, 39.5), (41.95, 31.5), 1, 1),
        (21, (40.95, 31.5), (40.95, 28.5), 0, 0),
        (21, (40.95, 28.5), (44.0, 28.625), 0, 0),
        # I2C SCL & SDA
        (14, (18.25, 32.0), (53.08, 44.0), 0, None),
        (14, (53.08, 44.0), (51.78, 49.5), None, None),
        (15, (18.25, 30.5), (55.62, 44.0), 0, None),
        (15, (55.62, 44.0), (54.32, 49.5), None, None),
        # +5V Rail (R1 is directly tied to +5V at J1 top pad on F.Cu)
        (2, (10.8, 26.1), (33.3, 7.5), 0, None),
        (2, (33.3, 7.5), (55.0, 7.5), None, None),
        (2, (55.0, 7.5), (52.0, 18.0), None, 0),
        (2, (52.0, 18.0), (61.5, 22.0), 0, 0),
        # +3V3 Rail
        (12, (8.5, 26.1), (8.5, 19.9), 0, 0),
        (12, (8.5, 19.9), (7.55, 17.0), 0, 0),
        (12, (7.55, 17.0), (7.55, 14.5), 0, 0),
        (12, (8.5, 19.9), (28.73, 19.0), 0, None),
        (12, (28.73, 19.0), (39.05, 24.5), 1, 0),
        (12, (39.05, 24.5), (39.05, 28.5), 0, 0),
        (12, (28.73, 19.0), (35.75, 41.0), 1, 0),
        (12, (35.75, 41.0), (39.55, 41.0), 0, 0),
        (12, (39.55, 42.5), (48.0, 44.0), 1, None),
        (12, (48.0, 44.0), (46.7, 49.5), None, None)
    ]

    print(f"Routing {len(netlist)} nets...")
    all_paths = []
    for nid, p1, p2, sl, el in netlist:
        path = router.route(p1, p2, nid, start_layer=sl, end_layer=el)
        if path:
            all_paths.append((nid, path, p1, p2, sl, el))
        else:
            print(f"FAILED to route Net {nid} from {p1} to {p2}!")
            return False

    print(f"Successfully routed all {len(all_paths)} nets!")

    # 4. Convert all paths to segments with exact pad snapping
    def path_to_segments(path, router, p1, p2, sl, el):
        segs = []
        vias = []
        
        start_coord = router.to_coord(path[0][0], path[0][1])
        start_layer_name = "F.Cu" if path[0][2] == 0 else "B.Cu"
        if (p1[0], p1[1]) != start_coord:
            segs.append((p1[0], p1[1], start_coord[0], start_coord[1], start_layer_name))

        start_pt = path[0]
        prev_pt = path[0]
        for i in range(1, len(path)):
            curr_pt = path[i]
            if curr_pt[2] != prev_pt[2]:
                if start_pt[:2] != prev_pt[:2]:
                    x1, y1 = router.to_coord(start_pt[0], start_pt[1])
                    x2, y2 = router.to_coord(prev_pt[0], prev_pt[1])
                    layer = "F.Cu" if start_pt[2] == 0 else "B.Cu"
                    segs.append((x1, y1, x2, y2, layer))
                vx, vy = router.to_coord(prev_pt[0], prev_pt[1])
                vias.append((vx, vy))
                start_pt = curr_pt
            else:
                dx1 = prev_pt[0] - start_pt[0]
                dy1 = prev_pt[1] - start_pt[1]
                dx2 = curr_pt[0] - prev_pt[0]
                dy2 = curr_pt[1] - prev_pt[1]
                if (dx1 != 0 and dy2 != 0) or (dy1 != 0 and dx2 != 0):
                    x1, y1 = router.to_coord(start_pt[0], start_pt[1])
                    x2, y2 = router.to_coord(prev_pt[0], prev_pt[1])
                    layer = "F.Cu" if start_pt[2] == 0 else "B.Cu"
                    segs.append((x1, y1, x2, y2, layer))
                    start_pt = prev_pt
            prev_pt = curr_pt

        if start_pt[:2] != prev_pt[:2]:
            x1, y1 = router.to_coord(start_pt[0], start_pt[1])
            x2, y2 = router.to_coord(prev_pt[0], prev_pt[1])
            layer = "F.Cu" if start_pt[2] == 0 else "B.Cu"
            segs.append((x1, y1, x2, y2, layer))

        end_coord = router.to_coord(path[-1][0], path[-1][1])
        end_layer_name = "F.Cu" if path[-1][2] == 0 else "B.Cu"
        if (p2[0], p2[1]) != end_coord:
            segs.append((end_coord[0], end_coord[1], p2[0], p2[1], end_layer_name))

        return segs, vias

    for nid, path, p1, p2, sl, el in all_paths:
        segs, vs = path_to_segments(path, router, p1, p2, sl, el)
        tw = 0.30
        if nid in (2, 11, 16): tw = 0.60
        elif nid == 12: tw = 0.50
        for x1, y1, x2, y2, lay in segs:
            all_segs.append((x1, y1, x2, y2, tw, nid, lay))
        for vx, vy in vs:
            all_vias.append((vx, vy, nid))

    print(f"Total traces: {len(all_segs)}, Total vias: {len(all_vias)}")

    # 5. Assemble KiCad PCB content
    lines = []
    lines.append('(kicad_pcb')
    lines.append('\t(version 20240108)')
    lines.append('\t(generator "antigravity")')
    lines.append('\t(generator_version "8.0")')
    lines.append('\t(general (thickness 1.6) (legacy_teardrops no))')
    lines.append('\t(paper "A4")')
    lines.append('\t(layers')
    lines.append('\t\t(0 "F.Cu" signal)')
    lines.append('\t\t(31 "B.Cu" signal)')
    lines.append('\t\t(34 "B.Paste" user)')
    lines.append('\t\t(35 "F.Paste" user)')
    lines.append('\t\t(36 "B.SilkS" user "B.Silkscreen")')
    lines.append('\t\t(37 "F.SilkS" user "F.Silkscreen")')
    lines.append('\t\t(38 "B.Mask" user)')
    lines.append('\t\t(39 "F.Mask" user)')
    lines.append('\t\t(40 "Dwgs.User" user "User.Drawings")')
    lines.append('\t\t(44 "Edge.Cuts" user)')
    lines.append('\t\t(46 "B.CrtYd" user "B.Courtyard")')
    lines.append('\t\t(47 "F.CrtYd" user "F.Courtyard")')
    lines.append('\t\t(48 "B.Fab" user)')
    lines.append('\t\t(49 "F.Fab" user)')
    lines.append('\t)')
    lines.append('\t(setup')
    lines.append('\t\t(aux_axis_origin 0.0 54.0)')
    lines.append('\t\t(pad_to_mask_clearance 0.05)')
    lines.append('\t\t(pcbplotparams')
    lines.append('\t\t\t(layerselection 0x00010fc_ffffffff)')
    lines.append('\t\t\t(plot_on_all_layers_selection 0x0000000_00000000)')
    lines.append('\t\t\t(disableapertmacros no)')
    lines.append('\t\t\t(usegerberextensions yes)')
    lines.append('\t\t\t(usegerberattributes yes)')
    lines.append('\t\t\t(usegerberadvancedattributes yes)')
    lines.append('\t\t\t(creategerberjobfile yes)')
    lines.append('\t\t\t(dashed_line_dash_ratio 12.0)')
    lines.append('\t\t\t(dashed_line_gap_ratio 3.0)')
    lines.append('\t\t\t(svgprecision 4)')
    lines.append('\t\t\t(plotframeref no)')
    lines.append('\t\t\t(viasonmask no)')
    lines.append('\t\t\t(mode 1)')
    lines.append('\t\t\t(useauxorigin yes)')
    lines.append('\t\t\t(hpglpennumber 1)')
    lines.append('\t\t\t(hpglpenspeed 20)')
    lines.append('\t\t\t(hpglpendiameter 15.0)')
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
    lines.append('\t\t\t(subtractmaskfromsilk yes)')
    lines.append('\t\t\t(outputformat 1)')
    lines.append('\t\t\t(mirror no)')
    lines.append('\t\t\t(drillshape 1)')
    lines.append('\t\t\t(scaleselection 1)')
    lines.append('\t\t\t(outputdirectory "gerbers/")')
    lines.append('\t\t)')
    lines.append('\t)')

    net_names = {
        0: "", 1: "GND", 2: "+5V", 3: "GPIO0_5V_ADC", 4: "GPIO1_SOIL",
        5: "GPIO5_SOIL_VCC", 6: "GPIO3_LDR", 7: "GPIO4_VBAT_ADC", 8: "ECHO_5V",
        9: "GPIO7_GATE", 10: "GATE_DRIVE", 11: "PUMP_DRAIN", 12: "+3V3",
        13: "GPIO10_TRIG", 14: "GPIO21_SCL", 15: "GPIO20_SDA", 16: "EXT_5V",
        17: "GPIO6_ECHO", 18: "VBAT_SENSE", 19: "USB_DP", 20: "USB_DN",
        21: "ESP_EN", 22: "GPIO9_BOOT", 23: "GPIO8_LED", 24: "USB_CC1",
        25: "USB_CC2", 26: "PWR_LED_K", 27: "STAT_LED_K"
    }

    for nid in range(28):
        nname = net_names.get(nid, "")
        lines.append(f'\t(net {nid} "{nname}")')

    W, H = 74.0, 54.0
    lines.append(f'\t(gr_rect (start 0 0) (end {W} {H}) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts") (uuid "{u()}"))')

    for i, (hx, hy) in enumerate([(3.5, 3.5), (W-3.5, 3.5), (3.5, H-3.5), (W-3.5, H-3.5)], 1):
        lines.append(f'\t(footprint "MountingHole_3.2mm_M3_Pad"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {hx} {hy})')
        lines.append(f'\t\t(property "Reference" "H{i}" (at 0 0) (layer "F.SilkS") (effects (font (size 0.8 0.8)) (hide yes)))')
        lines.append(f'\t\t(pad "1" thru_hole circle (at 0 0) (size 5.0 5.0) (drill 3.2) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
        lines.append(f'\t)')

    cpl_data = []

    def smd_resistor_0805(ref, val, rx, ry, net1, net2, lcsc_pn="", rot=0.0):
        jlc_x = rx
        jlc_y = 54.0 - ry
        cpl_data.append((ref, val, "0805", round(jlc_x, 3), round(jlc_y, 3), rot, "Top", lcsc_pn))
        lines.append(f'\t(footprint "Resistor_SMD:R_0805_2012Metric"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {rx} {ry} {rot})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -1.6 {rot}) (layer "F.SilkS") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 1.6 {rot}) (layer "F.Fab") (effects (font (size 0.7 0.7) (thickness 0.12))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0 {rot}) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -1.7 -0.95) (end 1.7 -0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.7 -0.95) (end 1.7 0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.7 0.95) (end -1.7 0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -1.7 0.95) (end -1.7 -0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        n1_id, n1_name = net1
        n2_id, n2_name = net2
        lines.append(f'\t\t(pad "1" smd roundrect (at -0.95 0 {rot}) (size 1.0 1.25) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {n1_id} "{n1_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 0.95 0 {rot}) (size 1.0 1.25) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {n2_id} "{n2_name}"))')
        lines.append(f'\t)')

    def smd_capacitor_0805(ref, val, cx, cy, net1, net2, lcsc_pn="", rot=0.0):
        jlc_x = cx
        jlc_y = 54.0 - cy
        cpl_data.append((ref, val, "0805", round(jlc_x, 3), round(jlc_y, 3), rot, "Top", lcsc_pn))
        lines.append(f'\t(footprint "Capacitor_SMD:C_0805_2012Metric"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {cx} {cy} {rot})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -1.6 {rot}) (layer "F.SilkS") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 1.6 {rot}) (layer "F.Fab") (effects (font (size 0.7 0.7) (thickness 0.12))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0 {rot}) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -1.7 -0.95) (end 1.7 -0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.7 -0.95) (end 1.7 0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.7 0.95) (end -1.7 0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -1.7 0.95) (end -1.7 -0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        n1_id, n1_name = net1
        n2_id, n2_name = net2
        lines.append(f'\t\t(pad "1" smd roundrect (at -0.95 0 {rot}) (size 1.0 1.25) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {n1_id} "{n1_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 0.95 0 {rot}) (size 1.0 1.25) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {n2_id} "{n2_name}"))')
        lines.append(f'\t)')

    def smd_led_0805(ref, val, lx, ly, net_a, net_k, lcsc_pn=""):
        jlc_x = lx
        jlc_y = 54.0 - ly
        cpl_data.append((ref, val, "0805", round(jlc_x, 3), round(jlc_y, 3), 0.0, "Top", lcsc_pn))
        lines.append(f'\t(footprint "LED_SMD:LED_0805_2012Metric"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {lx} {ly})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -1.6) (layer "F.SilkS") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 1.6) (layer "F.Fab") (effects (font (size 0.7 0.7) (thickness 0.12))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -1.7 -0.95) (end 1.7 -0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.7 -0.95) (end 1.7 0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.7 0.95) (end -1.7 0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -1.7 0.95) (end -1.7 -0.95) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        na_id, na_name = net_a
        nk_id, nk_name = net_k
        lines.append(f'\t\t(pad "1" smd roundrect (at -0.95 0) (size 1.0 1.25) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {na_id} "{na_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 0.95 0) (size 1.0 1.25) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {nk_id} "{nk_name}"))')
        lines.append(f'\t)')

    def smd_diode_sma(ref, val, dx, dy, net_k, net_a, lcsc_pn=""):
        jlc_x = dx
        jlc_y = 54.0 - dy
        cpl_data.append((ref, val, "SMA_DO-214AC", round(jlc_x, 3), round(jlc_y, 3), 0.0, "Top", lcsc_pn))
        lines.append(f'\t(footprint "Diode_SMD:D_SMA"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {dx} {dy})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -2.4) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 2.4) (layer "F.Fab") (effects (font (size 0.8 0.8) (thickness 0.14))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -3.2 -1.8) (end 3.2 -1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 3.2 -1.8) (end 3.2 1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 3.2 1.8) (end -3.2 1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -3.2 1.8) (end -3.2 -1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        # Silkscreen Cathode Band (thick vertical line on Pad 1 / Cathode side)
        lines.append(f'\t\t(fp_line (start -3.2 -1.5) (end -3.2 1.5) (stroke (width 0.35) (type solid)) (layer "F.SilkS"))')
        lines.append(f'\t\t(fp_line (start -3.2 -1.5) (end -2.0 -1.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))')
        lines.append(f'\t\t(fp_line (start -3.2 1.5) (end -2.0 1.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))')
        lines.append(f'\t\t(fp_line (start 2.0 -1.5) (end 3.2 -1.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))')
        lines.append(f'\t\t(fp_line (start 2.0 1.5) (end 3.2 1.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))')
        nk_id, nk_name = net_k
        na_id, na_name = net_a
        lines.append(f'\t\t(pad "1" smd roundrect (at -2.0 0) (size 1.5 1.8) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {nk_id} "{nk_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 2.0 0) (size 1.5 1.8) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {na_id} "{na_name}"))')
        lines.append(f'\t)')

    def smd_transistor_sot23(ref, val, tx, ty, net_g, net_s, net_d, lcsc_pn=""):
        jlc_x = tx
        jlc_y = 54.0 - ty
        cpl_data.append((ref, val, "SOT-23", round(jlc_x, 3), round(jlc_y, 3), 90.0, "Top", lcsc_pn))
        lines.append(f'\t(footprint "Package_TO_SOT_SMD:SOT-23"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {tx} {ty})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -2.4) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 2.4) (layer "F.Fab") (effects (font (size 0.8 0.8) (thickness 0.14))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -1.8 -1.8) (end 1.8 -1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.8 -1.8) (end 1.8 1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 1.8 1.8) (end -1.8 1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -1.8 1.8) (end -1.8 -1.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        ng_id, ng_name = net_g
        ns_id, ns_name = net_s
        nd_id, nd_name = net_d
        lines.append(f'\t\t(pad "1" smd roundrect (at -0.95 1.0) (size 0.8 0.9) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {ng_id} "{ng_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 0.95 1.0) (size 0.8 0.9) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {ns_id} "{ns_name}"))')
        lines.append(f'\t\t(pad "3" smd roundrect (at 0 -1.0) (size 0.8 0.9) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {nd_id} "{nd_name}"))')
        lines.append(f'\t)')

    def smd_regulator_sot223(ref, val, rx, ry, net_gnd, net_out, net_in, lcsc_pn=""):
        jlc_x = rx
        jlc_y = 54.0 - ry
        cpl_data.append((ref, val, "SOT-223", round(jlc_x, 3), round(jlc_y, 3), 0.0, "Top", lcsc_pn))
        lines.append(f'\t(footprint "Package_TO_SOT_SMD:SOT-223-3_TabPin2"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {rx} {ry})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -4.2) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 4.2) (layer "F.Fab") (effects (font (size 0.8 0.8) (thickness 0.14))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -3.9 -3.8) (end 3.9 -3.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 3.9 -3.8) (end 3.9 3.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 3.9 3.8) (end -3.9 3.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -3.9 3.8) (end -3.9 -3.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        ng_id, ng_name = net_gnd
        no_id, no_name = net_out
        ni_id, ni_name = net_in
        lines.append(f'\t\t(pad "1" smd roundrect (at -2.3 3.1) (size 1.0 1.5) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {ng_id} "{ng_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 0 3.1) (size 1.0 1.5) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {no_id} "{no_name}"))')
        lines.append(f'\t\t(pad "3" smd roundrect (at 2.3 3.1) (size 1.0 1.5) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {ni_id} "{ni_name}"))')
        lines.append(f'\t\t(pad "2" smd roundrect (at 0 -3.1) (size 3.3 1.5) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {no_id} "{no_name}"))')
        lines.append(f'\t)')

    def smd_button_ts1187a(ref, val, bx, by, net_p1, net_p2, lcsc_pn=""):
        jlc_x = bx
        jlc_y = 54.0 - by
        cpl_data.append((ref, val, "TS-1187A", round(jlc_x, 3), round(jlc_y, 3), 0.0, "Top", lcsc_pn))
        btn_label = "RST" if "1" in ref else "BOOT"
        lines.append(f'\t(footprint "Button_Switch_SMD:SW_Push_1P1T_XKB_TS-1187A"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {bx} {by})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at -3.8 0 90) (layer "F.SilkS") (effects (font (size 0.7 0.7) bold (thickness 0.12)) (hide yes)))')
        lines.append(f'\t\t(property "Value" "{btn_label}" (at 3.8 0 90) (layer "F.Fab") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -3.75 -2.4) (end 3.75 -2.4) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 3.75 -2.4) (end 3.75 2.4) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 3.75 2.4) (end -3.75 2.4) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -3.75 2.4) (end -3.75 -2.4) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        n1_id, n1_name = net_p1
        n2_id, n2_name = net_p2
        lines.append(f'\t\t(pad "1" smd rect (at -3.0 -1.875) (size 1.0 0.75) (layers "F.Cu" "F.Mask" "F.Paste") (net {n1_id} "{n1_name}"))')
        lines.append(f'\t\t(pad "1" smd rect (at 3.0 -1.875) (size 1.0 0.75) (layers "F.Cu" "F.Mask" "F.Paste") (net {n1_id} "{n1_name}"))')
        lines.append(f'\t\t(pad "2" smd rect (at -3.0 1.875) (size 1.0 0.75) (layers "F.Cu" "F.Mask" "F.Paste") (net {n2_id} "{n2_name}"))')
        lines.append(f'\t\t(pad "2" smd rect (at 3.0 1.875) (size 1.0 0.75) (layers "F.Cu" "F.Mask" "F.Paste") (net {n2_id} "{n2_name}"))')
        lines.append(f'\t)')

    def usb_type_c_receptacle(ref, val, ux, uy, lcsc_pn=""):
        jlc_x = ux
        jlc_y = 54.0 - uy
        cpl_data.append((ref, val, "TYPE-C-16P", round(jlc_x, 3), round(jlc_y, 3), 270.0, "Top", lcsc_pn))
        lines.append(f'\t(footprint "Connector_USB:USB_C_Receptacle_HRO_TYPE-C-31-M-12"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {ux} {uy} 270)')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -5.6 90) (layer "F.SilkS") (effects (font (size 0.8 0.8))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 5.6 90) (layer "F.Fab") (effects (font (size 0.8 0.8))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0 90) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')
        lines.append(f'\t\t(fp_line (start -4.6 -4.8) (end 4.6 -4.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 4.6 -4.8) (end 4.6 4.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 4.6 4.8) (end -4.6 4.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -4.6 4.8) (end -4.6 -4.8) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        
        # Shield tabs (exact positions matching official HRO TYPE-C-31-M-12 footprint)
        lines.append(f'\t\t(pad "SH" thru_hole oval (at -4.32 -3.13) (size 2.1 1.0) (drill oval 1.3 0.6) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
        lines.append(f'\t\t(pad "SH" thru_hole oval (at 4.32 -3.13) (size 2.1 1.0) (drill oval 1.3 0.6) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
        lines.append(f'\t\t(pad "SH" thru_hole oval (at -4.32 1.05) (size 1.6 1.0) (drill oval 1.1 0.6) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
        lines.append(f'\t\t(pad "SH" thru_hole oval (at 4.32 1.05) (size 1.6 1.0) (drill oval 1.1 0.6) (layers "*.Cu" "*.Mask") (net 1 "GND"))')
        
        # SMT signal pads: in footprint local coords (at X Y), size (1.45 pw)
        smd_pads = [
            ("A1",  -3.25, -4.045, 1.45, 0.6, 1,  "GND"),
            ("B12", -3.25, -4.045, 1.45, 0.6, 1,  "GND"),
            ("A12",  3.25, -4.045, 1.45, 0.6, 1,  "GND"),
            ("B1",   3.25, -4.045, 1.45, 0.6, 1,  "GND"),
            ("A4",  -2.45, -4.045, 1.45, 0.6, 2,  "+5V"),
            ("B9",  -2.45, -4.045, 1.45, 0.6, 2,  "+5V"),
            ("A9",   2.45, -4.045, 1.45, 0.6, 2,  "+5V"),
            ("B4",   2.45, -4.045, 1.45, 0.6, 2,  "+5V"),
            ("A5",  -1.25, -4.045, 1.45, 0.3, 24, "USB_CC1"),
            ("B5",   1.75, -4.045, 1.45, 0.3, 25, "USB_CC2"),
            ("A6",  -0.25, -4.045, 1.45, 0.3, 19, "USB_DP"),
            ("B6",   0.75, -4.045, 1.45, 0.3, 19, "USB_DP"),
            ("A7",   0.25, -4.045, 1.45, 0.3, 20, "USB_DN"),
            ("B7",  -0.75, -4.045, 1.45, 0.3, 20, "USB_DN"),
        ]
        for pnum, px, py, pw, ph, nid, nname in smd_pads:
            lines.append(f'\t\t(pad "{pnum}" smd roundrect (at {px} {py}) (size {pw} {ph}) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {nid} "{nname}"))')
        lines.append(f'\t)')

    def esp32_c3_wroom_02_module(ref, val, mx, my, lcsc_pn=""):
        jlc_x = mx
        jlc_y = 54.0 - my
        cpl_data.append((ref, val, "ESP32-C3-WROOM-02", round(jlc_x, 3), round(jlc_y, 3), 180.0, "Top", lcsc_pn))
        lines.append(f'\t(footprint "RF_Module:ESP32-C3-WROOM-02"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {mx} {my} 180)')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -2.5 180) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 2.5 180) (layer "F.Fab") (effects (font (size 0.8 0.8) (thickness 0.12))))')
        if lcsc_pn:
            lines.append(f'\t\t(property "LCSC" "{lcsc_pn}" (at 0 0) (layer "F.Fab") (effects (font (size 0.5 0.5)) (hide yes)))')

        lines.append(f'\t\t(fp_line (start -9.5 -18.5) (end 9.5 -18.5) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 9.5 -18.5) (end 9.5 7.5) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 9.5 7.5) (end -9.5 7.5) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -9.5 7.5) (end -9.5 -18.5) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_text user "ANTENNA KEEP-OUT" (at 0 -13.0 180) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.7 0.7) bold (thickness 0.12))))')

        # Official ESP32-C3-WROOM-02 pad positions
        # Left side: Pins 1..9 at local X = -8.75
        pins_left = [
            ("1", 12, "+3V3"),
            ("2", 21, "ESP_EN"),
            ("3", 7, "GPIO4_VBAT_ADC"),
            ("4", 5, "GPIO5_SOIL_VCC"),
            ("5", 17, "GPIO6_ECHO"),
            ("6", 9, "GPIO7_GATE"),
            ("7", 23, "GPIO8_LED"),
            ("8", 22, "GPIO9_BOOT"),
            ("9", 1, "GND")
        ]
        for idx, (pnum, nid, nname) in enumerate(pins_left):
            py = -6.0 + idx * 1.5
            lines.append(f'\t\t(pad "{pnum}" smd roundrect (at -8.75 {py}) (size 1.5 0.9) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {nid} "{nname}"))')

        # Right side: Pins 10..18 at local X = +8.75 (counter-clockwise, so Pad 10 is at Y=+6.0, Pad 18 at Y=-6.0)
        pins_right = [
            ("10", 13, "GPIO10_TRIG"),
            ("11", 15, "GPIO20_SDA"),
            ("12", 14, "GPIO21_SCL"),
            ("13", 20, "USB_DN"),
            ("14", 19, "USB_DP"),
            ("15", 6, "GPIO3_LDR"),
            ("16", 0, ""),
            ("17", 4, "GPIO1_SOIL"),
            ("18", 3, "GPIO0_5V_ADC")
        ]
        for idx, (pnum, nid, nname) in enumerate(pins_right):
            py = 6.0 - idx * 1.5
            lines.append(f'\t\t(pad "{pnum}" smd roundrect (at 8.75 {py}) (size 1.5 0.9) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net {nid} "{nname}"))')

        lines.append(f'\t\t(pad "19" smd roundrect (at 0.96 0.2) (size 2.9 2.9) (layers "F.Cu" "F.Mask" "F.Paste") (roundrect_rratio 0.25) (net 1 "GND"))')
        lines.append(f'\t)')

    def tht_terminal_block_kf301_h(ref, val, bx, by, pin_count, nets, labels=None):
        lines.append(f'\t(footprint "TerminalBlock:TerminalBlock_bornier-{pin_count}_P5.00mm"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {bx} {by})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at {(pin_count-1)*2.5} -3.5) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15)) (hide yes)))')
        lines.append(f'\t\t(property "Value" "{val}" (at {(pin_count-1)*2.5} 3.5) (layer "F.Fab") (effects (font (size 0.8 0.8) (thickness 0.14))))')
        bw = pin_count * 5.0 + 0.5
        lines.append(f'\t\t(fp_line (start -2.5 -4.0) (end {bw-2.5} -4.0) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start {bw-2.5} -4.0) (end {bw-2.5} 4.0) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start {bw-2.5} 4.0) (end -2.5 4.0) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -2.5 4.0) (end -2.5 -4.0) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        for p in range(pin_count):
            nid, nname = nets[p]
            ptype = "roundrect" if p == 0 else "circle"
            lines.append(f'\t\t(pad "{p+1}" thru_hole {ptype} (at {p*5.0} 0) (size 2.5 2.5) (drill 1.3) (roundrect_rratio 0.25) (net {nid} "{nname}"))')
            if labels and p < len(labels):
                lines.append(f'\t\t(fp_text user "{labels[p]}" (at {p*5.0} 4.9) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t)')

    def tht_terminal_block_kf301_v(ref, val, bx, by, pin_count, nets, labels=None):
        lines.append(f'\t(footprint "TerminalBlock:TerminalBlock_bornier-{pin_count}_P5.00mm"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {bx} {by})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 3.5 {(pin_count-1)*2.5}) (layer "F.SilkS") (effects (font (size 0.85 0.85) bold (thickness 0.15)) (hide yes)))')
        lines.append(f'\t\t(property "Value" "{val}" (at -3.5 {(pin_count-1)*2.5}) (layer "F.Fab") (effects (font (size 0.8 0.8) (thickness 0.14))))')
        bh = pin_count * 5.0 + 0.5
        lines.append(f'\t\t(fp_line (start -4.0 -2.5) (end 4.0 -2.5) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 4.0 -2.5) (end 4.0 {bh-2.5}) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 4.0 {bh-2.5}) (end -4.0 {bh-2.5}) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -4.0 {bh-2.5}) (end -4.0 -2.5) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        for p in range(pin_count):
            nid, nname = nets[p]
            ptype = "roundrect" if p == 0 else "circle"
            lines.append(f'\t\t(pad "{p+1}" thru_hole {ptype} (at 0 {p*5.0}) (size 2.5 2.5) (drill 1.3) (roundrect_rratio 0.25) (net {nid} "{nname}"))')
            if labels and p < len(labels):
                lines.append(f'\t\t(fp_text user "{labels[p]}" (at -4.9 {p*5.0} 90) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t)')

    def tht_pin_socket(ref, val, sx, sy, pin_count, nets, labels=None, rot=0.0, pin_label_side="below"):
        lines.append(f'\t(footprint "Connector_PinSocket_2.54mm:PinSocket_1x{pin_count:02d}_P2.54mm_Vertical"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {sx} {sy} {rot})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at {(pin_count-1)*1.27} -1.8 {rot}) (layer "F.SilkS") (effects (font (size 0.75 0.75) bold (thickness 0.13)) (hide yes)))')
        lines.append(f'\t\t(property "Value" "{val}" (at {(pin_count-1)*1.27} 1.8 {rot}) (layer "F.Fab") (effects (font (size 0.7 0.7) (thickness 0.12))))')
        bw = pin_count * 2.54 + 0.4
        lines.append(f'\t\t(fp_line (start -1.27 -1.3) (end {bw-1.27} -1.3) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start {bw-1.27} -1.3) (end {bw-1.27} 1.3) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start {bw-1.27} 1.3) (end -1.27 1.3) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -1.27 1.3) (end -1.27 -1.3) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        label_y = 2.4 if pin_label_side == "below" else -2.4
        for p in range(pin_count):
            nid, nname = nets[p]
            ptype = "roundrect" if p == 0 else "circle"
            lines.append(f'\t\t(pad "{p+1}" thru_hole {ptype} (at {p*2.54} 0 {rot}) (size 1.6 1.6) (drill 1.0) (layers "*.Cu" "*.Mask") (roundrect_rratio 0.25) (net {nid} "{nname}"))')
            if labels and p < len(labels):
                lines.append(f'\t\t(fp_text user "{labels[p]}" (at {p*2.54} {label_y} 90) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t)')

    def tht_ldr_sensor(ref, val, lx, ly, net1, net2):
        lines.append(f'\t(footprint "OptoDevice:R_LDR_5.0x4.1mm_P2.54mm_Vertical"')
        lines.append(f'\t\t(layer "F.Cu") (uuid "{u()}") (at {lx} {ly})')
        lines.append(f'\t\t(property "Reference" "{ref}" (at 0 -2.5) (layer "F.SilkS") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
        lines.append(f'\t\t(property "Value" "{val}" (at 0 2.5) (layer "F.Fab") (effects (font (size 0.7 0.7) (thickness 0.12))))')
        lines.append(f'\t\t(fp_line (start -2.5 -2.2) (end 2.5 -2.2) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 2.5 -2.2) (end 2.5 2.2) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start 2.5 2.2) (end -2.5 2.2) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        lines.append(f'\t\t(fp_line (start -2.5 2.2) (end -2.5 -2.2) (stroke (width 0.12) (type solid)) (layer "F.CrtYd"))')
        n1_id, n1_name = net1
        n2_id, n2_name = net2
        lines.append(f'\t\t(pad "1" thru_hole roundrect (at -1.27 0) (size 1.6 1.6) (drill 0.9) (layers "*.Cu" "*.Mask") (roundrect_rratio 0.25) (net {n1_id} "{n1_name}"))')
        lines.append(f'\t\t(pad "2" thru_hole circle (at 1.27 0) (size 1.6 1.6) (drill 0.9) (layers "*.Cu" "*.Mask") (net {n2_id} "{n2_name}"))')
        lines.append(f'\t)')

    # Instantiate Footprints
    usb_type_c_receptacle("J1", "TYPE-C-16P", 4.5, 34.25, lcsc_pn="C165948")
    smd_regulator_sot223("U2", "AMS1117-3.3", 8.5, 23.0, (1, "GND"), (12, "+3V3"), (2, "+5V"), lcsc_pn="C6186")
    esp32_c3_wroom_02_module("U1", "ESP32-C3-WROOM-02", 27.0, 35.0, lcsc_pn="C2934560")

    smd_button_ts1187a("SW1", "TS-1187A", 47.0, 30.5, (21, "ESP_EN"), (1, "GND"), lcsc_pn="C318884")
    smd_button_ts1187a("SW2", "TS-1187A", 47.0, 22.5, (1, "GND"), (22, "GPIO9_BOOT"), lcsc_pn="C318884")

    smd_led_0805("D3", "PWR_GREEN", 15.0, 16.0, (12, "+3V3"), (26, "PWR_LED_K"), lcsc_pn="C2297")
    smd_resistor_0805("R14", "1k", 18.5, 16.0, (26, "PWR_LED_K"), (1, "GND"), lcsc_pn="C17513")

    smd_led_0805("D4", "STAT_BLUE", 15.0, 19.0, (12, "+3V3"), (27, "STAT_LED_K"), lcsc_pn="C2293")
    smd_resistor_0805("R15", "1k", 18.5, 19.0, (27, "STAT_LED_K"), (23, "GPIO8_LED"), lcsc_pn="C17513")

    smd_resistor_0805("R12", "5.1k", 12.0, 30.0, (24, "USB_CC1"), (1, "GND"), lcsc_pn="C27834")
    smd_resistor_0805("R13", "5.1k", 12.0, 39.0, (25, "USB_CC2"), (1, "GND"), lcsc_pn="C27834")

    smd_resistor_0805("R11", "10k", 40.0, 24.5, (12, "+3V3"), (22, "GPIO9_BOOT"), lcsc_pn="C17414")
    smd_resistor_0805("R10", "10k", 40.0, 28.5, (12, "+3V3"), (21, "ESP_EN"), lcsc_pn="C17414")
    smd_capacitor_0805("C9", "1uF", 40.0, 31.5, (1, "GND"), (21, "ESP_EN"), lcsc_pn="C28323")

    smd_capacitor_0805("C10", "10uF", 2.5, 19.0, (2, "+5V"), (1, "GND"), lcsc_pn="C15850")
    smd_capacitor_0805("C7", "100nF", 2.5, 23.0, (2, "+5V"), (1, "GND"), lcsc_pn="C28233")
    smd_capacitor_0805("C11", "10uF", 8.5, 14.5, (12, "+3V3"), (1, "GND"), lcsc_pn="C15850")
    smd_capacitor_0805("C8", "100nF", 8.5, 17.0, (12, "+3V3"), (1, "GND"), lcsc_pn="C28233")
    smd_capacitor_0805("C12", "22uF", 40.5, 41.0, (12, "+3V3"), (1, "GND"), lcsc_pn="C45783")

    smd_resistor_0805("R1", "100k", 11.0, 41.5, (2, "+5V"), (3, "GPIO0_5V_ADC"), lcsc_pn="C149504")
    smd_resistor_0805("R2", "100k", 11.0, 44.5, (1, "GND"), (3, "GPIO0_5V_ADC"), lcsc_pn="C149504")
    smd_capacitor_0805("C1", "100nF", 15.5, 42.5, (1, "GND"), (3, "GPIO0_5V_ADC"), lcsc_pn="C28233")

    smd_resistor_0805("R8", "100k", 44.0, 36.0, (7, "GPIO4_VBAT_ADC"), (18, "VBAT_SENSE"), lcsc_pn="C149504")
    smd_resistor_0805("R9", "100k", 44.0, 38.5, (7, "GPIO4_VBAT_ADC"), (1, "GND"), lcsc_pn="C149504")
    smd_capacitor_0805("C6", "100nF", 40.5, 38.0, (7, "GPIO4_VBAT_ADC"), (1, "GND"), lcsc_pn="C28233")

    smd_resistor_0805("R3", "1k", 41.5, 16.5, (8, "ECHO_5V"), (17, "GPIO6_ECHO"), lcsc_pn="C17513")
    smd_resistor_0805("R4", "2k", 41.5, 19.5, (17, "GPIO6_ECHO"), (1, "GND"), lcsc_pn="C17604")

    smd_capacitor_0805("C2", "100nF", 15.5, 38.0, (4, "GPIO1_SOIL"), (1, "GND"), lcsc_pn="C28233")

    smd_resistor_0805("R7", "10k", 30.0, 23.0, (1, "GND"), (6, "GPIO3_LDR"), lcsc_pn="C17414")
    smd_capacitor_0805("C4", "100nF", 30.0, 25.5, (1, "GND"), (6, "GPIO3_LDR"), lcsc_pn="C28233")

    smd_resistor_0805("R5", "1k", 50.0, 34.0, (9, "GPIO7_GATE"), (10, "GATE_DRIVE"), lcsc_pn="C17513")
    smd_resistor_0805("R6", "100k", 54.0, 34.0, (10, "GATE_DRIVE"), (1, "GND"), lcsc_pn="C149504")

    smd_transistor_sot23("Q1", "AO3400A", 58.0, 33.0, (10, "GATE_DRIVE"), (1, "GND"), (11, "PUMP_DRAIN"), lcsc_pn="C20917")
    smd_diode_sma("D1", "SS34", 54.0, 18.0, (2, "+5V"), (11, "PUMP_DRAIN"), lcsc_pn="C8678")
    smd_diode_sma("D2", "SS34", 63.5, 22.0, (2, "+5V"), (16, "EXT_5V"), lcsc_pn="C8678")

    tht_terminal_block_kf301_h("TB2", "SOIL_SENSOR", 11.7, 7.5, 3, [(4, "GPIO1_SOIL"), (5, "GPIO5_SOIL_VCC"), (1, "GND")], ["DATA", "VCC", "GND"])
    tht_terminal_block_kf301_h("TB1", "ULTRASONIC", 33.3, 7.5, 4, [(2, "+5V"), (13, "GPIO10_TRIG"), (8, "ECHO_5V"), (1, "GND")], ["+5V", "TRIG", "ECHO", "GND"])
    tht_terminal_block_kf301_h("TB3", "PUMP_MOTOR", 55.0, 7.5, 2, [(2, "+5V"), (11, "PUMP_DRAIN")], ["+5V", "PUMP-"])
    tht_terminal_block_kf301_v("TB4", "EXT_POWER_BAT", 66.5, 31.0, 3, [(16, "EXT_5V"), (1, "GND"), (18, "VBAT_SENSE")], ["5V_IN", "GND", "BAT+"])

    tht_pin_socket("MOD1", "SHT30_I2C", 48.0, 44.0, 4, [(12, "+3V3"), (1, "GND"), (14, "GPIO21_SCL"), (15, "GPIO20_SDA")], ["3V3", "GND", "SCL", "SDA"], pin_label_side="above")
    tht_pin_socket("MOD2", "BH1750_I2C", 46.7, 49.5, 5, [(12, "+3V3"), (1, "GND"), (14, "GPIO21_SCL"), (15, "GPIO20_SDA"), (1, "GND")], ["3V3", "GND", "SCL", "SDA", "ADDR"], pin_label_side="below")
    tht_ldr_sensor("R_LDR1", "LDR_5528", 30.0, 19.0, (12, "+3V3"), (6, "GPIO3_LDR"))

    # Silkscreen text
    lines.append(f'\t(gr_text "SOIL SENSOR" (at 16.7 1.8) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
    lines.append(f'\t(gr_text "ULTRASONIC" (at 40.8 1.8) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
    lines.append(f'\t(gr_text "PUMP" (at 57.5 1.8) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
    lines.append(f'\t(gr_text "EXT PWR / BAT" (at 57.8 38.5 90) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.70 0.70) bold (thickness 0.12))))')
    lines.append(f'\t(gr_text "SHT30 (TEMP/HUM)" (at 51.81 39.0) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
    lines.append(f'\t(gr_text "BH1750 (LIGHT)" (at 51.78 46.5) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.75 0.75) bold (thickness 0.13))))')
    lines.append(f'\t(gr_text "ESP32-C3 Onboard Edition • Rev 1.0" (at 28.0 51.5) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.8 0.8) bold (thickness 0.14))))')
    lines.append(f'\t(gr_text "BOOT" (at 52.2 22.5) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')
    lines.append(f'\t(gr_text "RST" (at 52.2 30.5) (layer "F.SilkS") (uuid "{u()}") (effects (font (size 0.85 0.85) bold (thickness 0.15))))')

    # Traces
    for x1, y1, x2, y2, tw, nid, lay in all_segs:
        lines.append(f'\t(segment (start {x1} {y1}) (end {x2} {y2}) (width {tw}) (layer "{lay}") (net {nid}))')

    # Vias
    for vx, vy, nid in all_vias:
        lines.append(f'\t(via (at {vx} {vy}) (size 0.5) (drill 0.3) (layers "F.Cu" "B.Cu") (net {nid}))')

    # Ground Plane on B.Cu
    lines.append(f'\t(zone (net 1) (net_name "GND") (layer "B.Cu") (uuid "{u()}")')
    lines.append(f'\t\t(hatch edge 0.5)')
    lines.append(f'\t\t(connect_pads (clearance 0.25))')
    lines.append(f'\t\t(min_thickness 0.25)')
    lines.append(f'\t\t(fill yes (thermal_gap 0.3) (thermal_bridge_width 0.45))')
    lines.append(f'\t\t(polygon')
    lines.append(f'\t\t\t(pts')
    lines.append(f'\t\t\t\t(xy 0 0)')
    lines.append(f'\t\t\t\t(xy {W} 0)')
    lines.append(f'\t\t\t\t(xy {W} {H})')
    lines.append(f'\t\t\t\t(xy 0 {H})')
    lines.append(f'\t\t\t)')
    lines.append(f'\t\t)')
    lines.append(f'\t)')

    lines.append(')')

    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_pcb = os.path.join(script_dir, "plant-esp32-onboard.kicad_pcb")
    with open(output_pcb, "w") as f:
        f.write("\n".join(lines))
    print(f"Wrote complete board to {output_pcb}")

    # Write CPL
    cpl_path = os.path.join(script_dir, "plant-esp32-onboard_cpl.csv")
    with open(cpl_path, "w") as f:
        f.write("Designator,Val,Package,Mid X,Mid Y,Rotation,Layer\n")
        for ref, val, pkg, x, y, rot, layer, lcsc in cpl_data:
            f.write(f"{ref},{val},{pkg},{x},{y},{rot},{layer}\n")
    print(f"Wrote CPL to {cpl_path}")

    # Write BOM
    bom_path = os.path.join(script_dir, "plant-esp32-onboard_bom.csv")
    with open(bom_path, "w") as f:
        f.write("Comment,Designator,Footprint,LCSC Part #\n")
        from collections import defaultdict
        groups = defaultdict(list)
        for ref, val, pkg, x, y, rot, layer, lcsc in cpl_data:
            if lcsc:
                groups[(val, pkg, lcsc)].append(ref)
        for (val, pkg, lcsc), refs in sorted(groups.items()):
            f.write(f'"{val}","{",".join(refs)}","{pkg}","{lcsc}"\n')
    print(f"Wrote BOM to {bom_path}")

    return True

if __name__ == "__main__":
    build_board()
