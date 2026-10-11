# ESP32-C3 SuperMini 18×24 Perfboard Assembly Guide

This guide details the permanent, soldered build of the Plant Monitor & Automatic Watering system on a standard **0.1" (2.54 mm) pitch plated through-hole perfboard** with **18 Rows (A to R)** and **24 Columns (1 to 24)**.

---

## 1. Board Specifications & Physical Layout

| Parameter | Specification | Physical Dimension |
| :--- | :--- | :--- |
| **Grid Size** | **18 Rows (A–R) × 24 Columns (1–24)** | 432 total plated through-holes |
| **Grid Pitch** | **0.100" (2.54 mm)** standard DIP | Width: ~65 mm (2.56"), Height: ~50 mm (1.97") |
| **Microcontroller Socket** | 2× 1×8 Female Header Sockets (Cols 3–10, Rows G & M) | Standard 0.600" (15.24 mm) DIP row spacing |
| **USB-C Port** | Extends off **Left Edge** (Cols 1–2 clearance) | Allows flush cable connection without case obstruction |
| **Sensor Terminals** | **TB1 & TB2** on **Row B** (Cols 7–11 and 15–21) | Wire entries face **UPWARDS** (off top edge) |
| **Pump Terminals** | **TB3** on **Row Q** (Cols 19 & 21) | Wire entries face **DOWNWARDS** (off bottom edge) |
| **Power Switch** | **Q1 (FQP30N06L)** TO-220 in **Row O** (Cols 18, 19, 20) | Metal heatsink tab faces North (inward), front faces South |

```
       COL:  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20  21  22  23  24
    ROW A: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW B: [·] [·] [·] [·] [·] [·] [  TB2 SOIL  ] [·] [·] [·] [·] [    TB1 ULTRASONIC    ] [·] [·] [·]
    ROW C: [·] [·] [====== ANALOG / SENSOR GND BUS ========================================] [·] [·] [·]
    ROW D: [·] [·] [·] [·] [C1] [R2] [·] [·] [C2] [·] [C2] [·] [·] [·] [·] [·] [R4] [·] [R3] [·] [·] [·] [·] [·]
    ROW E: [·] [·] [==== +5V SENSOR BUS ===] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW F: [·] [·] [·] [·] [C1] [R1] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW G: [·] [·] [ 5V GND 3V3  G0  G1  G2  G3  G4 ]  (ESP32-C3 SuperMini Header 1)        [·] [·] [·] [·]
    ROW H: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW I: [·] [·] |      SOCKET CAVITY             |                                      [·] [·] [·] [·]
    ROW J: [·] [·] |      (CLEAR UNDERNEATH)        |                                      [·] [·] [·] [·]
    ROW K: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW L: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW M: [·] [·] [ G21 G20 G10 G9  G8  G7  G6  G5 ]  (ESP32-C3 SuperMini Header 2)        [·] [·] [·] [·]
    ROW N: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [C3+] [·] [C3-] [·]
    ROW O: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [R5] [·] [·] [R5] [Q1G Q1D Q1S] [·] [·] [·]
    ROW P: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [R6] [D1A] [·] [D1K] [·] [·] [·]
    ROW Q: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [TB3-] [·] [TB3+] [·] [·] [·]
    ROW R: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [=== POWER GND BUS ===] [·]
```

---

## 2. Complete Bill of Materials (BOM)

| Designator | Component | Value / Rating | Package / Footprint | Coordinates (Row & Col) |
| :---: | :--- | :--- | :--- | :--- |
| **U1** | **ESP32-C3 SuperMini** | RISC-V 160MHz, Wi-Fi/BLE | 2× 1×8 Female Header | Header 1: **Row G, Cols 3–10**<br>Header 2: **Row M, Cols 3–10** |
| **TB1** | **Screw Terminal (4-Pin)** | 5.08 mm Pitch (Ultrasonic) | 4P Through-Hole | **Row B, Cols 15, 17, 19, 21** |
| **TB2** | **Screw Terminal (3-Pin)** | 5.08 mm Pitch (Soil Sensor) | 3P Through-Hole | **Row B, Cols 7, 9, 11** |
| **TB3** | **Screw Terminal (2-Pin)** | 5.08 mm Pitch (DC Pump) | 2P Through-Hole | **Row Q, Cols 19, 21** |
| **Q1** | **N-MOSFET (Logic-Level)** | **FQP30N06L** (60V, 32A) | TO-220 (Vertical) | **Row O, Cols 18 (G), 19 (D), 20 (S)** |
| **D1** | **Schottky Diode** | **1N5819** (40V, 1A) | DO-41 Axial | **Row P, Col 19 (Anode) $\to$ Col 21 (Cathode)** |
| **C1** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.1" pitch) | **Cols 5–6, Rows C $\to$ F** |
| **C2** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.2" pitch) | **Row D, Col 9 $\to$ Col 11** |
| **C3** | **Electrolytic Capacitor** | $470\,\mu\text{F}$ (16V or 25V) | Radial Can (5mm pitch) | **Row N, Col 21 (+) $\to$ Col 23 (-)** |
| **R1** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 6, Row E (+5V) $\to$ Row F (Node)** |
| **R2** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 6, Row F (Node) $\to$ Row C (GND)** |
| **R3** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row D, Col 19 (Echo) $\to$ Col 17 (Node)** |
| **R4** | **Metal Film Resistor** | $2\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row D, Col 17 (Node) $\to$ Row C (GND)** |
| **R5** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 14 $\to$ Col 17** |
| **R6** | **Metal Film Resistor** | $100\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 18, Row P (Gate) $\to$ Row R (GND)** |

---

## 3. Component Coordinate Placement Table (Top View)

| Component | Pin / Terminal | Coordinate | Physical Orientation & Notes |
| :--- | :--- | :---: | :--- |
| **ESP32 Socket 1** | Pin 1 (5V In)<br>Pin 2 (GND)<br>Pin 3 (3V3 Out)<br>Pin 4 (GPIO 0 / ADC1_0)<br>Pin 5 (GPIO 1 / ADC1_1)<br>Pin 6 (GPIO 2 / Soil VCC)<br>Pin 7 (GPIO 3 / TRIG)<br>Pin 8 (GPIO 4 / ECHO) | **G3**<br>**G4**<br>**G5**<br>**G6**<br>**G7**<br>**G8**<br>**G9**<br>**G10** | 1×8 Female Header soldered into Row G.<br>USB-C connector points **LEFT** into Columns 1–2.<br>Header spacing is standard 0.600" DIP pitch to Row M. |
| **ESP32 Socket 2** | Pin 16 (GPIO 21)<br>Pin 15 (GPIO 20)<br>Pin 14 (GPIO 10)<br>Pin 13 (GPIO 9)<br>Pin 12 (GPIO 8 / Blue LED)<br>Pin 11 (GPIO 7 / Pump Gate)<br>Pin 10 (GPIO 6)<br>Pin 9 (GPIO 5) | **M3**<br>**M4**<br>**M5**<br>**M6**<br>**M7**<br>**M8**<br>**M9**<br>**M10** | 1×8 Female Header soldered into Row M.<br>Rows H, I, J, K, L remain open underneath the module body for clean underside routing. |
| **TB2 (Soil)** | Pin 1 (Switched VCC)<br>Pin 2 (Analog In / AOUT)<br>Pin 3 (Ground / GND) | **B7**<br>**B9**<br>**B11** | 3-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off top edge).<br>Screw heads face up for easy screwdriver access. |
| **TB1 (Ultrasonic)** | Pin 1 (+5V Power)<br>Pin 2 (TRIG Pulse)<br>Pin 3 (ECHO Return)<br>Pin 4 (Ground / GND) | **B15**<br>**B17**<br>**B19**<br>**B21** | 4-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off top edge).<br>Cols 12–14 leave 7.62 mm spacing between TB2 and TB1. |
| **TB3 (Pump)** | Pin 1 (Drain / Pump -)<br>Pin 2 (+5V / Pump +) | **Q19**<br>**Q21** | 2-Pin 5.08 mm terminal block.<br>Wire entry slots face **DOWNWARDS** (South off bottom edge). |
| **Q1 (FQP30N06L)** | Pin 1 (Gate)<br>Pin 2 (Drain)<br>Pin 3 (Source) | **O18**<br>**O19**<br>**O20** | TO-220 package across 3 adjacent holes.<br>Metal heatsink tab faces **NORTH** (inward).<br>Plastic labeled front faces **SOUTH** (toward Row P). |
| **D1 (1N5819)** | Anode (+)<br>Cathode (- stripe) | **P19**<br>**P21** | Axial flyback diode spanning Cols 19 to 21.<br>Silver cathode stripe faces **RIGHT** (Col 21 / +5V). |
| **C3 (470µF)** | Positive (+)<br>Negative (- stripe) | **N21**<br>**N23** | Radial electrolytic capacitor.<br>Positive lead in Col 21 (+5V pump rail).<br>Negative lead in Col 23 (tied to Row R GND). |
| **R1 (10kΩ)** | Top (+5V)<br>Bottom (Divider Node) | **E6**<br>**F6** | Voltage divider top leg. Spans Row E (+5V) to Row F. |
| **R2 (10kΩ)** | Top (Divider Node)<br>Bottom (GND) | **F6**<br>**C6** | Voltage divider bottom leg. Spans Row F to Row C (GND). |
| **C1 (100nF)** | Top (Divider Node)<br>Bottom (GND) | **F5**<br>**C5** | ADC ripple filter. Solder bridged at Row F to Col 6. |
| **C2 (100nF)** | Lead 1 (AOUT)<br>Lead 2 (GND) | **D9**<br>**D11** | High-frequency RF bypass directly across Soil AOUT and GND. |
| **R3 (1kΩ)** | Echo In<br>Divider Node | **D19**<br>**D17** | Echo divider top leg. Spans Col 19 to Col 17. |
| **R4 (2kΩ)** | Divider Node<br>GND | **D17**<br>**C17** | Echo divider bottom leg. Spans Row D to Row C (GND). |
| **R5 (1kΩ)** | Input (from GPIO 7)<br>Output (to Gate) | **O14**<br>**O17** | Gate damping resistor. Spans Col 14 to Col 17. |
| **R6 (100kΩ)** | Gate Node<br>GND | **P18**<br>**R18** | Gate pull-down resistor. Spans Row P to Row R (GND). |

---

## 4. Underside Point-to-Point Wiring Schedule (Solder Side)

All connections on the bottom of the perfboard are made using either **solder bridges** (for directly adjacent pads), **bare tinned copper bus wire** (for ground/power rails), or **insulated hookup wire** (30 AWG Kynar or 26 AWG silicone wire for signal jumpers).

### A. Power & Ground Rails

| Bus / Rail | From Pad | To Pad | Wire Type | Description & Function |
| :--- | :---: | :---: | :--- | :--- |
| **Analog GND Bus** | **C4** | **C21** | Bare Solid Bus Wire | Straight bare wire soldered across Row C (Cols 4–21). Dedicated ground reference for ADC, voltage dividers, and sensor returns. |
| **Power GND Bus** | **R18** | **R24** | Bare Solid Bus Wire | Straight bare wire soldered across Row R (Cols 18–24). High-current return path for MOSFET Source and Bulk Cap C3. |
| **Star Ground Link** | **G4** (ESP32 GND) | **R18** (Power GND) | 24 AWG Insulated Wire | Heavy ground tie linking ESP32 system ground to the bottom power ground bus without polluting Row C. |
| **Top +5V Bus** | **E3** | **E15** | Bare Solid Bus Wire | Bare bus wire along Row E (Cols 3–15) distributing USB 5V to R1 supply divider and TB1 Ultrasonic 5V. |
| **+5V Power Feed** | **G3** (ESP32 5V) | **N21** (C3 +) | 24 AWG Insulated Wire | High-current feed providing isolated 5V power directly to the pump reservoir cap C3 and TB3 Pin 2. |

### B. High-Current Actuator Connections (Direct Solder Bridges)

| Connection | Pads | Method | Notes |
| :--- | :---: | :--- | :--- |
| **Q1 Drain $\to$ Flyback Diode $\to$ TB3 Drain** | **O19 $\to$ P19 $\to$ Q19** | 3-Pad Solder Bridge | Direct solder bridge linking MOSFET Drain (O19), D1 Anode (P19), and TB3 Pin 1 Pump - (Q19). **Zero wire required!** |
| **C3 (+) $\to$ Flyback Diode $\to$ TB3 (+5V)** | **N21 $\to$ P21 $\to$ Q21** | Solder Bridge / Bus Trace | Links C3 positive lead, D1 Cathode, and TB3 Pin 2 (+5V) into the heavy +5V feed. |
| **Q1 Source $\to$ Power GND Bus** | **O20 $\to$ R20** | Component Lead Bend | Bend Q1 Source lead down Column 20 from Row O to Row R and solder directly to Power GND bus. |
| **C3 (-) $\to$ Power GND Bus** | **N23 $\to$ R23** | Component Lead Bend | Bend C3 negative lead down Column 23 from Row N to Row R and solder directly to Power GND bus. |
| **R5 Right $\to$ Q1 Gate Node** | **O17 $\to$ O18** | 2-Pad Solder Bridge | Connects output of R5 (1kΩ) directly to Q1 Gate pin. |
| **Q1 Gate $\to$ R6 Top Node** | **O18 $\to$ P18** | 2-Pad Solder Bridge | Ties Q1 Gate to top lead of R6 (100kΩ pull-down). |

### C. Signal & Sensor Jumpers (Insulated Hookup Wire)

| Signal Net | From Pad | To Pad | Route Path | Description |
| :--- | :---: | :---: | :--- | :--- |
| **5V Supply Sense** | **F6** | **G6** | Solder Bridge (F6 $\to$ G6) | Directly links R1/R2 divider node to ESP32 **GPIO 0** (Pin 4). |
| **C1 Filter Bridge** | **F5** | **F6** | Solder Bridge (F5 $\to$ F6) | Parallels C1 filter cap directly across R2 to suppress ADC sampling ripple. |
| **Soil Switched VCC** | **B7** (TB2 Pin 1) | **G8** (ESP32 GPIO 2) | Direct Insulated Jumper | Delivers duty-cycled 3.3V power to soil sensor only during sampling. |
| **Soil Analog Sense** | **C9** (via B9) | **G7** (ESP32 GPIO 1) | Direct Insulated Jumper | Carries filtered analog moisture voltage from TB2 Pin 2 to ESP32 ADC1. |
| **Soil Filter Bridge** | **B9 $\to$ D9** & **B11 $\to$ D11** | Short Solder Bridges | Links C2 (100nF) directly across Soil AOUT and GND pins. |
| **Ultrasonic TRIG** | **B17** (TB2 Pin 2) | **G9** (ESP32 GPIO 3) | Direct Insulated Jumper | Delivers 10µs trigger pulse to HC-SR04. |
| **Echo Input Link** | **B19** (TB1 Pin 3) | **D19** (R3 In) | Short Solder Bridge (B19 $\to$ D19)| Feeds raw 5V Echo pulse into top leg of R3 (1kΩ). |
| **Echo 3.3V Stepped** | **D17** (Divider Node)| **G10** (ESP32 GPIO 4)| Direct Insulated Jumper | Transmits stepped 3.33V Echo pulse safely into ESP32 GPIO 4. |
| **Pump Gate Drive** | **M8** (ESP32 GPIO 7)| **O14** (R5 Left) | Under-Socket Insulated Jumper | Connects GPIO 7 output to R5 gate damping resistor. |

---

## 5. Noise Immunity & Ground Architecture

This perfboard layout implements strict **galvanic and star-ground partitioning**:

```
 [ USB-C 5V Input ]
       │
       ├─────────────────────────────────┐
       ▼                                 ▼
 [ Top Rail +5V (Clean) ]          [ High-Current Feed (Pump) ]
   • ESP32 SuperMini (G3)            • C3 Bulk Capacitor (470µF)
   • 10k/10k Supply Divider (R1)     • TB3 Pin 2 (Pump +5V)
   • TB1 Ultrasonic 5V (B15)         • D1 Flyback Diode (Cathode)
       │                                 │
       ▼                                 ▼
 [ Row C Analog GND Bus ]          [ Row R Power GND Bus ]
   • ESP32 GND (G4)                  • Q1 MOSFET Source (O20)
   • Supply Filter C1 & R2           • C3 Negative Lead (N23)
   • Soil Sensor GND & C2            • R6 Pull-Down Resistor (P18)
   • Echo Divider R4 (C17)               │
   • Ultrasonic GND (B21)                │
       ▲                                 │
       │                                 │
       └──────── [ Star Ground Tie ] ────┘
```

1. **Pump Inrush Isolation**: Submersible DC water pumps create $0.8\text{A}\text{--}1.2\text{A}$ inductive startup spikes. Because C3 ($470\,\mu\text{F}$) and Q1 Source return exclusively through the **Row R Power GND Bus**, motor ground currents never pass through the Row C sensor bus.
2. **ADC Precision**: ADC1 channels (`GPIO 0` for supply monitoring and `GPIO 1` for soil moisture) reference the ultra-quiet **Row C Analog Ground Bus**, eliminating ADC jitter and false dry/wet readings.
3. **RF Suppression**: The $100\,\text{nF}$ ceramic capacitor (C2) placed directly across TB2 Pins 2 & 3 absorbs 2.4 GHz RF bursts transmitted by the ESP32 Wi-Fi antenna.

---

## 6. Step-by-Step Soldering Sequence

To ensure the cleanest build and easiest physical access, solder components in this exact order:

### Step 1: ESP32 Female Header Sockets
1. Insert two 1×8 female pin headers into **Row G (Cols 3–10)** and **Row M (Cols 3–10)**.
2. Tape the headers in place or invert onto a flat surface.
3. Tack-solder pin 1 and pin 8 of each header, verify perpendicular alignment, then solder all remaining pins.

### Step 2: Ground & Power Bus Rails
1. Lay a straight length of bare tinned copper wire along **Row C (Cols 4–21)**. Solder at every pin that requires ground (C4, C5, C6, C11, C17, C21).
2. Lay a straight length of bare copper wire along **Row R (Cols 18–24)**. Solder across the pads to create the Power GND bus.
3. Lay a bare copper wire along **Row E (Cols 3–15)** for the Top +5V bus.
4. Solder the **Star Ground Link wire** (24 AWG insulated) from **G4 (ESP32 GND)** around to **R18 (Power GND)**.
5. Solder the **+5V Power Feed wire** (24 AWG insulated) from **G3 (ESP32 5V)** over to **N21 (C3 +)**.

### Step 3: Passive Resistors & Capacitors
1. **Supply Divider**: Insert **R1 (10kΩ)** into E6 $\to$ F6; insert **R2 (10kΩ)** into F6 $\to$ C6; insert **C1 (100nF)** into F5 $\to$ C5. Solder pads and create bridge across F5 $\to$ F6 $\to$ G6.
2. **Echo Stepper**: Insert **R3 (1kΩ)** into D19 $\to$ D17; insert **R4 (2kΩ)** into D17 $\to$ C17. Solder and bridge C17 to Row C GND.
3. **Soil RF Filter**: Insert **C2 (100nF)** into D9 $\to$ D11.
4. **Gate Passives**: Insert **R5 (1kΩ)** into O14 $\to$ O17; insert **R6 (100kΩ)** into P18 $\to$ R18.

### Step 4: Screw Terminal Blocks
1. Insert **TB2 (3-Pin Soil)** into **Row B, Cols 7, 9, 11** with wire entries facing North (UPWARDS).
2. Insert **TB1 (4-Pin Ultrasonic)** into **Row B, Cols 15, 17, 19, 21** with wire entries facing North (UPWARDS).
3. Insert **TB3 (2-Pin Pump)** into **Row Q, Cols 19, 21** with wire entries facing South (DOWNWARDS).
4. Solder all terminal block pins securely (use ample solder to fill the plated barrel for mechanical strength).

### Step 5: Power Switching (MOSFET, Diode, Reservoir Cap)
1. Insert **Q1 (FQP30N06L)** into **Row O, Cols 18 (G), 19 (D), 20 (S)**. Ensure the metal tab faces North and the plastic label faces South.
2. Insert **D1 (1N5819)** into **Row P, Cols 19 (A) $\to$ 21 (K)**. Verify cathode silver band points toward Col 21.
3. Insert **C3 (470µF)** into **Row N, Cols 21 (+) $\to$ 23 (-)**. Verify stripe faces Col 23.
4. On the solder side:
   * Solder bridge **O19 $\to$ P19 $\to$ Q19** (MOSFET Drain to Diode Anode to TB3 Pin 1).
   * Solder bridge **N21 $\to$ P21 $\to$ Q21** (C3+ to Diode Cathode to TB3 Pin 2).
   * Solder bridge **O17 $\to$ O18** (R5 to Gate) and **O18 $\to$ P18** (Gate to R6).
   * Solder Q1 Source (O20) and C3- (N23) down to Row R GND bus.

### Step 6: Insulated Signal Jumpers
1. Jumper from **M8 (GPIO 7)** to **O14 (R5 Left)**.
2. Jumper from **B7 (TB2 Pin 1 / Soil VCC)** to **G8 (GPIO 2)**.
3. Jumper from **B9 (TB2 Pin 2 / Soil AOUT)** to **G7 (GPIO 1)**.
4. Jumper from **B17 (TB1 Pin 2 / TRIG)** to **G9 (GPIO 3)**.
5. Jumper from **D17 (Echo 3.3V Node)** to **G10 (GPIO 4)**.
6. Connect TB1 Pin 1 (B15) to Row E +5V bus.
7. Connect TB1 Pin 4 (B21) and TB2 Pin 3 (B11) to Row C GND bus.

---

## 7. Pre-Power Cold Testing Checklist (Multimeter DMM)

**DO NOT INSERT THE ESP32-C3 MODULE YET.** Perform these cold tests with a digital multimeter in continuity / resistance mode:

| Test ID | Multimeter Probes | Expected Result | What It Verifies |
| :---: | :--- | :---: | :--- |
| **T1** | Socket G3 (5V) to Socket G4 (GND) | **$> 10\,\text{k}\Omega$** (Rising) | **No power short!** C3 charging curve confirms clean 5V rail. |
| **T2** | Socket G4 (GND) to Socket G5 (3V3) | **Open circuit ($\infty$)** | No 3.3V rail shorts. |
| **T3** | Socket G4 (GND) to TB1 Pin 4, TB2 Pin 3 | **$< 0.5\,\Omega$** | Sensor ground continuity through Row C. |
| **T4** | Socket G4 (GND) to Q1 Source (O20) | **$< 0.5\,\Omega$** | Power ground continuity through Star Link. |
| **T5** | Socket G3 (5V) to TB1 Pin 1, TB3 Pin 2 | **$< 0.5\,\Omega$** | 5V power distribution continuity. |
| **T6** | Socket G6 (GPIO 0) to Socket G4 (GND) | **$\approx 10.0\,\text{k}\Omega$** | Exact R2 resistance to ground. |
| **T7** | Socket G6 (GPIO 0) to Socket G3 (5V) | **$\approx 10.0\,\text{k}\Omega$** | Exact R1 resistance to 5V rail. |
| **T8** | Socket M8 (GPIO 7) to Socket G4 (GND) | **$\approx 101\,\text{k}\Omega$** | Series R5 ($1\,\text{k}\Omega$) + R6 ($100\,\text{k}\Omega$) pull-down path. |
| **T9** | TB1 Pin 3 (Echo) to Socket G10 (GPIO 4) | **$\approx 1.0\,\text{k}\Omega$** | Upper Echo resistor R3. |
| **T10** | Socket G10 (GPIO 4) to Socket G4 (GND) | **$\approx 2.0\,\text{k}\Omega$** | Lower Echo resistor R4 to ground. |
| **T11** | TB3 Pin 1 (Drain) to TB3 Pin 2 (+5V) | **Diode Drop ($\sim 0.35\text{V}$)** | Diode D1 polarity check (red probe on TB3 Pin 1, black on Pin 2). |

---

## 8. Final Commissioning & Smoke Test

1. **Insert ESP32 Module**: Plug the ESP32-C3 SuperMini into the female header sockets (USB-C facing left).
2. **Apply USB-C Power**: Connect a 5V USB-C cable to the SuperMini.
   * Observe onboard LED.
   * Verify no component becomes warm to the touch.
3. **Verify Rail Voltages**:
   * Measure between G3 (5V) and G4 (GND): should read **$4.90\text{V}\text{--}5.25\text{V}$**.
   * Measure between G5 (3V3) and G4 (GND): should read **$3.28\text{V}\text{--}3.32\text{V}$**.
   * Measure between G6 (GPIO 0) and G4 (GND): should read exactly **$\text{USB Voltage} / 2 \approx 2.50\text{V}$**.
4. **Connect Sensors**:
   * Wire Capacitive Soil Moisture Sensor into **TB2** (`VCC` $\to$ Pin 1, `AOUT` $\to$ Pin 2, `GND` $\to$ Pin 3).
   * Wire HC-SR04 Ultrasonic Sensor into **TB1** (`VCC` $\to$ Pin 1, `TRIG` $\to$ Pin 2, `ECHO` $\to$ Pin 3, `GND` $\to$ Pin 4).
   * Wire 5V Submersible Pump into **TB3** (`- / Black` $\to$ Pin 1, `+ / Red` $\to$ Pin 2).
5. **Software Functional Test**: Open the web dashboard and verify:
   * Supply voltage reads $\approx 5.0\,\text{V}$.
   * Soil moisture reads accurately when dry vs in water.
   * Water reservoir level updates in real-time.
   * Manual pump activation fires cleanly with zero Wi-Fi disconnects or ESP32 resets.
