# ESP32-C3 SuperMini 18×24 Perfboard Assembly Guide (Descending Rows R $\to$ A)

This guide details the permanent, soldered build of the Plant Monitor & Automatic Watering system on a standard **0.1" (2.54 mm) pitch plated through-hole perfboard** with **18 Rows descending from R (Top) to A (Bottom)** and **24 Columns (1 to 24)**.

---

## 1. Perfboard vs. Breadboard Fundamentals: Every Hole is Isolated

> [!IMPORTANT]
> **NO INTERNAL TRACES ON PERFBOARD**: Unlike solderless breadboards where rows or columns share internal metal spring clips, **every single through-hole pad on a perfboard is completely electrically isolated**.
> 
> Two components placed in adjacent holes (e.g., Row K and Row J) **DO NOT CONNECT** unless you physically create the connection!

Every electrical connection in this build is explicitly realized using one of four physical methods:

1. **Solder Bridge**: Bridging two or three adjacent copper pads with a generous fillet of solder on the underside (solder side).
2. **Component Lead Bend**: Bending the component's excess wire lead on the underside across to the adjacent hole before snipping and soldering.
3. **Bare Solid Bus Wire**: Laying a length of 22–24 AWG bare tinned copper wire across a row of pads to create a continuous, low-impedance power or ground rail.
4. **Insulated Hookup Wire**: Running 30 AWG Kynar (wire-wrap) or 26 AWG flexible silicone wire between non-adjacent pads on the underside.

---

## 2. Board Grid & Physical Layout (Rows R $\to$ A Descending)

* **Vertical Axis (Width)**: 18 Rows labeled **R down to A** (Row R is at the **Top Edge**, Row A is at the **Bottom Edge**).
* **Horizontal Axis (Length)**: 24 Columns labeled **1 to 24** (Col 1 on the Left, Col 24 on the Right).
* **Grid Pitch**: Standard 0.100" (2.54 mm) DIP pitch (~65 mm wide × 50 mm tall).
* **ESP32 Sockets**: Dual 1×8 female headers on **Row J** and **Row D** across Columns 3–10 (standard 0.600" / 6-row DIP pitch).
* **USB-C Port**: Extends off the **Left Edge** (Cols 1–2 clearance) for unobstructed cable insertion.
* **Top Sensor Terminals**: **TB2 (Soil)** & **TB1 (Ultrasonic)** on **Row Q**, with wire entries facing **UPWARDS** (North off top edge).
* **Bottom Pump Terminal**: **TB3 (DC Pump)** on **Row B (Cols 13 & 15)**, with wire entries facing **DOWNWARDS** (South off bottom edge).
* **Actuator Driver & Flyback Diode**: **D1 (1N4001)** is mounted in **Row E (Cols 13 & 15)** (right side at E15), directly above **Q1 (Row C, Cols 12–14)** and **TB3 (Row B, Cols 13 & 15)**, leaving Columns 16–24 completely clear!
* **Straight +5V Column 15 Power Feed**: An insulated jumper runs straight down Column 15 from **N15** (+5V bus) to **E15** (D1 Cathode / TB3 +5V), eliminating diagonal cross-board power wiring!
* **Straight GND Column 14 Feed & 4-Pad Solder Bridge**: An insulated jumper runs straight down Column 14 from **P14** (Row P GND bus) to **D14**. On the underside (solder side), a 4-pad vertical solder bridge along Column 14 connects **D14 $\to$ C14 (Q1 Source) $\to$ B14 $\to$ A14 (Row A GND)**, directly grounding Q1 Source and R6 pull-down (at A12) while eliminating diagonal cross-board star ground wiring!
* **Three Symmetrical Vertical Actuator Tracks**: Columns 13, 14, and 15 form three parallel underside tracks: Col 13 (Drain / Diode Anode / Pump -), Col 14 (GND / Q1 Source / Row A), Col 15 (+5V / Diode Cathode / Pump +).
* **Col 3 5-Pad 5V Distribution Track**: On the bottom side, a continuous 5-pad rigid solder track along Column 3 connects **N3 $\to$ M3 $\to$ L3 (C3 +) $\to$ K3 $\to$ J3 (ESP32 5V)**, eliminating top wire W3 and providing an ultra-low-impedance power feed.
* **Col 8 Direct Gate Drive Bridge**: Solder bridged directly on the bottom between **D8 $\to$ C8**, eliminating top wire W11.
* **Col 7 Soil Data Track & C2 Filter**: On the bottom, a 3-pad solder bridge connects **Q7 $\to$ P7 $\to$ O7**, solidly tying TB2 Data to C2 filter left leg. Top wire W6 runs straight down Col 7 from **O7 $\to$ K7** (Soil ADC wire terminal), with bottom bridge **J7 $\to$ K7** to ESP32 GPIO 1.
* **Sensor Ground Bus (P11 to P21) & Col 4 Jump**: Bare bus rail on Row P spanning Cols 11–21, fed from Col 4 GND track via top wires W3a (`M4 → P4`, jumping along Col 4 over Row N +5V) and W3b (`P4 → P11`, arching along Row P over P7 Soil Data). Leaving P5–P10 open allows clean vertical routing down Column 7 with zero shorts!

```
       COL:  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20  21  22  23  24
    ROW R: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (TOP EDGE)
    ROW Q: [·] [·] [·] [·] [·] [·] [  TB2 SOIL  ] [·] [·] [·] [·] [    TB1 ULTRASONIC    ] [·] [·] [·]
    ROW P: [·] [·] [·] [P4] [·] [·] [·] [·] [·] [·] [====== SENSOR GND BUS (P11-P21) ======] [·] [·] [·]
    ROW O: [·] [·] [·] [·] [·] [·] [C2] [·] [·] [·] [C2] [·] [·] [·] [·] [·] [R4] [·] [R3] [·] [·] [·] [·] [·]
    ROW N: [·] [·] [==== +5V SENSOR BUS ===] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW M: [·] [·] [5V] [M4] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (Col 3: 5V Track | Col 4: GND Track)
    ROW L: [·] [·] [C3+][C3-/C1][·][·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (100µF VCC-GND & C1 GND)
    ROW K: [·] [·] [5V][R2G] [·] [K6] [K7] [K8] [K9][K10] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (Row K: K4 GND, K6 Node, K7-K10 Signal Breakouts)
    ROW J: [·] [·] [ 5V GND 3V3  G0  G1  G2  G3  G4 ]  (ESP32-C3 SuperMini Header 1)        [·] [·] [·] [·]
    ROW I: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW H: [·] [·] |      SOCKET CAVITY             |                                      [·] [·] [·] [·]
    ROW G: [·] [·] |      (CLEAR UNDERNEATH)        |                                      [·] [·] [·] [·]
    ROW F: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW E: [·] [·] |                                | [·] [·] [D1A] [·] [D1K] [·] [·] [·] [·] [·] [·] [·] [·] [·] (FLYBACK DIODE)
    ROW D: [·] [·] [ G21 G20 G10 G9  G8  G7  G6  G5 ]  (ESP32-C3 SuperMini Header 2) [GND] [·] [·] [·] [·] [·] [·] [·]
    ROW C: [·] [·] [·] [·] [·] [·] [·] [R5] [·] [·] [R5] [Q1G Q1D Q1S] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW B: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [TB3-] [GND] [TB3+] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW A: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [ POWER GND (A12-A14) ] [·] [·] [·] [·] [·] [·] [·] [·] (BOTTOM EDGE)
```

---

## 3. Complete Bill of Materials (BOM) & Coordinate Map

| Designator | Component | Value / Rating | Footprint | Pins / Terminal Coordinates |
| :---: | :--- | :--- | :--- | :--- |
| **U1** | **ESP32-C3 SuperMini** | RISC-V 160MHz, Wi-Fi | 2× 1×8 Female Sockets | Top Header: **Row J, Cols 3–10**<br>Bottom Header: **Row D, Cols 3–10** |
| **TB1** | **Screw Terminal (4-Pin)** | 5.08 mm Pitch (Ultrasonic) | 4P Through-Hole | **Row Q, Cols 15 (5V), 17 (TRIG), 19 (ECHO), 21 (GND)** |
| **TB2** | **Screw Terminal (3-Pin)** | 5.08 mm Pitch (Soil Sensor) | 3P Through-Hole | **Row Q, Cols 7 (Data), 9 (VCC), 11 (GND)** |
| **TB3** | **Screw Terminal (2-Pin)** | 5.08 mm Pitch (DC Pump) | 2P Through-Hole | **Row B, Cols 13 (Drain -), 15 (+5V)** (Dedicated pads!) |
| **Q1** | **N-MOSFET (Logic-Level)** | **FQP30N06L** (60V, 32A) | TO-220 (Vertical) | **Row C, Cols 12 (G), 13 (D), 14 (S)** |
| **D1** | **Flyback Diode** | **1N4001** (50V, 1A) | DO-41 Axial | **Row E, Col 13 (Anode) $\to$ Col 15 (Cathode)** (Right side at E15!) |
| **C3** | **Electrolytic Capacitor** | **$100\,\mu\text{F}$** (16V/25V) | Radial Can (2.5mm pitch) | **Row L, Col 3 (+) $\to$ Col 4 (-)** (Joined between VCC & GND!) |
| **C1** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.1" pitch) | **Row L, Col 4 (GND) $\to$ Row K, Col 6 (Node)** (Direct to Node at K6!) |
| **C2** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.2" pitch) | **Row O, Col 7 (Data) $\to$ Col 11 (GND)** (Tied to TB2 Data via Col 7 track!) |
| **R1** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 6, Row N (+5V) $\to$ Row K (Node)** |
| **R2** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row K, Col 6 (Node) $\to$ Col 4 (GND)** |
| **R3** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 19 (Echo) $\to$ Col 17 (Node)** |
| **R4** | **Metal Film Resistor** | $2\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 17 (Node) $\to$ Row P (GND)** |
| **R5** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row C, Col 8 $\to$ Col 11** |
| **R6** | **Metal Film Resistor** | $100\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 12, Row C (Gate) $\to$ Row A (GND)** |

---

## 4. Component Coordinate Placement Table (Top View)

| Component | Pin / Terminal | Coordinate | Physical Orientation & Mounting Notes |
| :--- | :--- | :---: | :--- |
| **ESP32 Socket 1** | Pin 1 (5V In)<br>Pin 2 (GND)<br>Pin 3 (3V3 Out)<br>Pin 4 (GPIO 0 / Supply ADC)<br>Pin 5 (GPIO 1 / Soil ADC)<br>Pin 6 (GPIO 2 / Soil VCC)<br>Pin 7 (GPIO 3 / TRIG)<br>Pin 8 (GPIO 4 / ECHO) | **J3**<br>**J4**<br>**J5**<br>**J6**<br>**J7**<br>**J8**<br>**J9**<br>**J10** | 1×8 Female Header in Row J.<br>USB-C points **LEFT** into Cols 1–2.<br>Header spacing is standard 0.600" DIP pitch to Row D.<br>Pin 1 (J3): Solder bridged down Col 3 to C3 (+) and Row N 5V bus.<br>Pin 2 (J4): Part of Col 4 GND track (J4-K4-L4-M4) to C3 (-), R2/C1 GND, and M4 wire terminal.<br>Pin 4 (J6): Solder bridged up to K6 (Supply Divider Node).<br>Pin 5 (J7): Solder bridged up to K7 (Soil DATA breakout).<br>Pin 6 (J8): Solder bridged up to K8 (Soil VCC breakout).<br>Pin 7 (J9): Solder bridged up to K9 (Ultrasonic TRIG breakout).<br>Pin 8 (J10): Solder bridged up to K10 (Ultrasonic ECHO breakout). |
| **ESP32 Socket 2** | Pin 16 (GPIO 21)<br>Pin 15 (GPIO 20)<br>Pin 14 (GPIO 10)<br>Pin 13 (GPIO 9)<br>Pin 12 (GPIO 8 / Blue LED)<br>Pin 11 (GPIO 7 / Pump Gate)<br>Pin 10 (GPIO 6)<br>Pin 9 (GPIO 5) | **D3**<br>**D4**<br>**D5**<br>**D6**<br>**D7**<br>**D8**<br>**D9**<br>**D10** | 1×8 Female Header in Row D.<br>Pin 11 (D8): Solder bridged directly on bottom down Col 8 to C8 (R5 1k Gate resistor). |
| **C3 (100µF Bulk Cap)** | Positive (+)<br>Negative (- stripe) | **L3**<br>**L4** | Radial electrolytic capacitor ($100\,\mu\text{F}$, 16V/25V).<br>**Joined directly between VCC & GND** across ESP32 pins J3 and J4. |
| **TB2 (Soil)** | Pin 1 (AOUT / Data)<br>Pin 2 (Switched VCC)<br>Pin 3 (GND) | **Q7**<br>**Q9**<br>**Q11** | 3-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off Row R).<br>Pin 1 connects via underside Col 7 bridge (Q7-P7-O7) to C2 and top wire from O7 to K7. |
| **TB1 (Ultrasonic)** | Pin 1 (+5V Power)<br>Pin 2 (TRIG Pulse)<br>Pin 3 (ECHO Return)<br>Pin 4 (GND) | **Q15**<br>**Q17**<br>**Q19**<br>**Q21** | 4-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off Row R).<br>Cols 12–14 leave 7.62 mm spacing between TB2 and TB1. |
| **TB3 (Pump)** | Pin 1 (Drain / Pump -)<br>Pin 2 (+5V / Pump +) | **B13**<br>**B15** | 2-Pin 5.08 mm terminal block.<br>Wire entry slots face **DOWNWARDS** (South off Row A).<br>**Dedicated holes**: Not shared with the flyback diode! |
| **D1 (1N4001)** | Anode (+)<br>Cathode (- stripe) | **E13**<br>**E15** | Axial flyback diode mounted in **Row E** (Cols 13 $\to$ 15).<br>Silver cathode stripe faces **RIGHT** (Col 15 / +5V).<br>**Completely separated from TB3** (3 rows away). |
| **Q1 (FQP30N06L)** | Pin 1 (Gate)<br>Pin 2 (Drain)<br>Pin 3 (Source) | **C12**<br>**C13**<br>**C14** | TO-220 package across 3 adjacent holes.<br>Metal heatsink tab faces **NORTH** (inward toward Row D).<br>Plastic labeled front faces **SOUTH** (toward Row B). |
| **R1 (10kΩ)** | Top (+5V)<br>Bottom (Divider Node) | **N6**<br>**K6** | 5V supply divider top leg. Spans vertically from Row N (+5V Bus) to Row K (Node). |
| **R2 (10kΩ)** | Right (Divider Node)<br>Left (GND) | **K6**<br>**K4** | 5V supply divider bottom leg. Spans horizontally along Row K from Col 6 (Node) to Col 4 (GND). |
| **C1 (100nF)** | Right (Divider Node)<br>Left (GND) | **K6**<br>**L4** | ADC ripple filter. Spans diagonally from Col 4 Row L (GND) to Col 6 Row K (Node). Directly grounded via L4 and tied to divider node at K6! |
| **C2 (100nF)** | Lead 1 (Data AOUT)<br>Lead 2 (GND) | **O7**<br>**O11** | Wi-Fi RF bypass capacitor across Soil AOUT and GND via underside tracks Q7-P7-O7 and Q11-P11-O11. |
| **R3 (1kΩ)** | Echo In<br>Divider Node | **O19**<br>**O17** | Echo stepper upper leg. Spans Col 19 to Col 17 in Row O. |
| **R4 (2kΩ)** | Divider Node<br>GND | **O17**<br>**P17** | Echo stepper lower leg. Spans Row O to Row P (Sensor GND Bus). |
| **R5 (1kΩ)** | Input (from GPIO 7)<br>Output (to Gate) | **C8**<br>**C11** | Gate damping resistor. Spans Col 8 to Col 11 in Row C. Bridged C8 $\to$ D8 on bottom. |
| **R6 (100kΩ)** | Gate Node<br>GND | **C12**<br>**A12** | Gate pull-down resistor. Spans Row C (Gate) to Row A (GND). |

---

## 5. Anti-Short Dual-Layer Wiring Architecture

> [!IMPORTANT]
> **PHYSICAL LAYER SEPARATION PREVENTS ALL SHORT CIRCUITS**:
> In a soldered perfboard build, running crossing jumper wires across the bottom (solder) side risks severe shorts against bare bus rails (Row P GND, Row N 5V) or melting insulation during soldering.
>
> To guarantee **100% short-free operation**, all connections in this build are strictly partitioned into two physical layers separated by the **1.6 mm FR4 fiberglass substrate**:
> 1. **BOTTOM LAYER (Solder Side)**: ONLY rigid bare bus rails and direct adjacent solder bridges. **Zero crossing wires!**
> 2. **TOP LAYER (Component Side)**: ALL 10 long-distance and rail-crossing jumper wires, run as neatly dressed insulated hookup wires elevated above the board surface.

### A. Top Layer Wiring Schedule (Component Side — 10 Insulated Wires)

| Wire ID | Net Name | From Pad | To Pad | Wire Gauge / Color | Physical Routing & Anti-Short Rationale |
| :---: | :--- | :---: | :---: | :--- | :--- |
| **W1** | **GND Power Feed** | **P14** | **D14** | 24 AWG Insulated (Blue) | Runs straight down Column 14 through the ESP32 socket cavity. **Top-side routing prevents any contact with the Row N +5V bus on the bottom!** |
| **W2** | **+5V Pump Feed** | **N15** | **E15** | 24 AWG Insulated (Red) | Runs straight down Column 15 from Row N 5V bus to D1 Cathode. Isolated high-current feed. |
| **W3a** | **Col 4 GND Jump** | **M4** | **P4** | 24 AWG Insulated (Blue) | Runs straight along Column 4 jumping safely over Row N +5V bus to Row P. |
| **W3b** | **Row P GND Feed** | **P4** | **P11** | 24 AWG Insulated (Blue) | Runs along Row P arching safely over P7 (Soil Data) to connect into Sensor GND bus at P11. Zero short hazard! |
| **W4** | **Ultrasonic +5V** | **N15** | **Q15** | 26 AWG Insulated (Red) | Runs from Row N +5V bus up to TB1 Pin 1. **Arches safely over Row P GND on the top side—zero risk of shorting 5V to GND!** |
| **W5** | **Echo Input Link** | **Q19** | **O19** | 26 AWG Insulated (Green) | Connects TB1 Pin 3 (Echo) to R3 (1kΩ) input. **Arches safely over Row P GND on the top side—zero risk of grounding the Echo line!** |
| **W6** | **Soil Moisture DATA** | **O7** | **K7** | 26 AWG Insulated (Green) | Runs straight down Column 7 from C2/TB2 Data track (O7) to K7 (Soil ADC terminal, bridged to J7 on bottom). Zero crossovers! |
| **W7** | **Soil Switched VCC** | **Q9** | **K8** | 26 AWG Insulated (Purple)| Parallel run down Column 9/8 from TB2 Pin 2 to K8 (bridged to ESP32 GPIO 2). Zero crossovers! |
| **W8** | **Ultrasonic TRIG** | **K9** | **Q17** | 26 AWG Cyan | Digital trigger pulse from K9 (bridged to ESP32 GPIO 3) to TB1 Pin 2. Safely crosses over Row N and P on top. |
| **W9** | **Ultrasonic ECHO 3.3V**| **O17** | **K10** | 26 AWG Insulated (Green)| Stepped Echo pulse from R3/R4 node to K10 (bridged to ESP32 GPIO 4). Safely crosses over Row N on top. |

### B. Bottom Layer Wiring Schedule (Solder Side — 18 Rigid Buses & Solder Bridges)

| Track ID | Net Name | Pads Involved | Physical Method | Detailed Assembly Instruction |
| :---: | :--- | :---: | :--- | :--- |
| **B1** | **Sensor GND Bus** | **P11 $\to$ P21** | **Bare Bus Wire** | Lay 22 AWG bare tinned wire along Row P (Cols 11–21). Solder at P11, P14, P17, P21. Leaves P5–P10 open. |
| **B2** | **Top +5V Bus** | **N3 $\to$ N15** | **Bare Bus Wire** | Lay 22 AWG bare tinned wire along Row N (Cols 3–15). Solder at N3, N6, N15. |
| **B3** | **Power GND Bus** | **A12 $\to$ A14** | **Bare Bus / Solder Bridge** | Solder bridge or bare wire along Row A (Cols 12–14). Connects R6 pull-down (A12) to Col 14 GND track (A14). |
| **B4** | **Supply Sense Node** | **K6 $\to$ J6** | **2-Pad Solder Bridge** | Direct solder bridge between Row K and Row J on Col 6 ties R1/R2/C1 node to ESP32 GPIO 0. |
| **B5** | **5V Distribution Track**| **N3 $\to$ M3 $\to$ L3 $\to$ K3 $\to$ J3** | **5-Pad Solder Track** | Straight vertical solder track down Column 3 ties Row N 5V bus directly to C3 (+) (L3) and ESP32 5V In (J3). |
| **B6** | **Col 4 GND Track** | **M4 $\to$ L4 $\to$ K4 $\to$ J4** | **4-Pad Solder Track** | Straight vertical solder track down Column 4 ties M4 top wire terminal directly to C3 (-), R2 GND, and ESP32 GND (J4). |
| **B7** | **Soil ADC Bridge** | **J7 $\to$ K7** | **2-Pad Solder Bridge** | Direct vertical solder bridge between Row J and Row K on Col 7 ties ESP32 GPIO 1 (J7) to K7 top wire terminal. |
| **B8** | **Soil VCC Bridge** | **J8 $\to$ K8** | **2-Pad Solder Bridge** | Direct vertical solder bridge between Row J and Row K on Col 8 ties ESP32 GPIO 2 (J8) to K8 top wire terminal. |
| **B9** | **Ultrasonic TRIG Bridge** | **J9 $\to$ K9** | **2-Pad Solder Bridge** | Direct vertical solder bridge between Row J and Row K on Col 9 ties ESP32 GPIO 3 (J9) to K9 top wire terminal. |
| **B10** | **Ultrasonic ECHO Bridge** | **J10 $\to$ K10** | **2-Pad Solder Bridge** | Direct vertical solder bridge between Row J and Row K on Col 10 ties ESP32 GPIO 4 (J10) to K10 top wire terminal. |
| **B11** | **Pump Drain Track** | **E13 $\to$ D13 $\to$ C13 $\to$ B13** | **4-Pad Solder Bridge** | Straight vertical solder bridge down Column 13 links D1 Anode (E13), Q1 Drain (C13), and TB3 Pin 1 (B13). |
| **B12** | **Pump GND Track** | **D14 $\to$ C14 $\to$ B14 $\to$ A14** | **4-Pad Solder Bridge** | Straight vertical solder bridge down Column 14 links Driver GND wire (D14), Q1 Source (C14), and Row A GND (A14). |
| **B13** | **Pump +5V Track** | **E15 $\to$ D15 $\to$ C15 $\to$ B15** | **4-Pad Solder Bridge** | Straight vertical solder bridge down Column 15 links 5V Feed wire (E15), D1 Cathode, and TB3 Pin 2 (B15). |
| **B14** | **Gate Resistor Bridge**| **C11 $\to$ C12** | **2-Pad Solder Bridge** | Solder bridge along Row C connects output of R5 (C11) to Q1 Gate pin (C12) and R6 top lead. |
| **B15** | **Gate Drive Bridge** | **D8 $\to$ C8** | **2-Pad Solder Bridge** | Direct vertical solder bridge down Column 8 links ESP32 GPIO 7 (D8) directly to R5 input (C8). |
| **B16** | **Soil Data Track** | **Q7 $\to$ P7 $\to$ O7** | **3-Pad Solder Bridge** | Straight vertical solder bridge down Column 7 links TB2 Pin 1 Data (Q7), through-hole pad P7, and C2 filter left leg (O7). |
| **B17** | **Soil GND Tie** | **Q11 $\to$ P11 $\to$ O11** | **3-Pad Solder Bridge** | Solder bridge down Column 11 ties TB2 Pin 3 (GND) to Row P GND (P11) and C2 right leg (O11). |
| **B18** | **Ultrasonic GND Tie** | **Q21 $\to$ P21** | **2-Pad Solder Bridge** | Solder bridge down Column 21 ties TB1 Pin 4 (GND) directly into Row P GND Bus (P21). |

---

## 6. Noise Immunity & Ground Architecture

This layout guarantees **complete galvanic isolation** between noisy motor surges and sensitive analog inputs:

```
 [ USB-C 5V Input ]
       │
       ├─────────────────────────────────┐
       ▼                                 ▼
 [ Row N +5V Bus (Clean) ]         [ High-Current Feed (Pump) ]
   • ESP32 SuperMini (J3)            • D1 Cathode (E15)
   • 100µF Bulk Reservoir (L3/L4)    • TB3 Pin 2 (Pump +5V, B15)
   • 10k/10k Supply Divider (R1)         │
   • TB1 Ultrasonic 5V (Q15)             │
       │                                 │
       ▼                                 ▼
 [ Row P Sensor GND Bus (P11-P21) ][ Row A Power GND (A12-A14) ]
   • Fed via Wires W3a/W3b           • Q1 MOSFET Source (C14)
   • Soil Sensor GND & C2 (P11/O11)  • R6 Pull-Down Resistor (A12)
   • Echo Divider R4 (P17)               │
   • Ultrasonic GND (Q21)                │
       ▲                                 │
       │                                 │
       └──── [ Col 14 Feed: P14→D14→A14 ]┘
```

1. **Local Microcontroller Decoupling**: The $100\,\mu\text{F}$ capacitor is joined directly across **J3 (5V/VCC)** and **J4 (GND)**, providing low-impedance energy reservoir right at the ESP32 module.
2. **Dedicated Actuator Pads**: Flyback diode D1 in Row E absorbs inductive turn-off energy right across Column 13 (Drain) and Column 15 (+5V), with zero mechanical strain or crammed terminal pins.

---

## 7. Step-by-Step Soldering Sequence (Dual-Layer Assembly)

### Phase 1: Headers & Bottom-Side Copper Soldering (Buses & Solder Bridges)

1. **Female Header Sockets**:
   * Insert two 1×8 female headers into **Row J (Cols 3–10)** and **Row D (Cols 3–10)** from the top.
   * Tack-solder corner pins on the bottom side, verify perpendicular alignment, then finish all 16 pins.
2. **Bottom-Side Ground & Power Bus Rails**:
   * Lay 22 AWG bare tinned wire along **Row P (Cols 11–21)** on the solder side for Sensor GND. Solder at P11, P14, P17, P21. (Leave P5–P10 open!)
   * Solder bridge or lay bare wire along **Row A (Cols 12–14)** on the solder side for Power GND. Solder at A12, A13, A14.
   * Lay 22 AWG bare tinned wire along **Row N (Cols 3–15)** on the solder side for +5V Bus. Solder at N3, N6, N15.
3. **Top Components Insertion & Underside Solder Bridges**:
   * Insert **C3 ($100\,\mu\text{F}$)** into Row L (Cols 3 $\to$ 4); on solder side create the Col 3 track **N3 $\to$ M3 $\to$ L3 $\to$ K3 $\to$ J3** (ties 5V bus to C3 (+) and ESP32 5V In) and Col 4 GND track **M4 $\to$ L4 $\to$ K4 $\to$ J4** (ties M4 wire pad to C3 (-), R2 GND, and ESP32 GND).
   * Insert **R1 (10kΩ)** into N6 $\to$ K6; insert **R2 (10kΩ)** into K6 $\to$ K4; insert **C1 (100nF)** into L4 $\to$ K6.
   * On solder side, bridge **K6 $\to$ J6** (Divider Node to GPIO 0), **J7 $\to$ K7** (GPIO 1 to Soil Data), **J8 $\to$ K8** (GPIO 2 to Soil VCC), **J9 $\to$ K9** (GPIO 3 to TRIG), and **J10 $\to$ K10** (GPIO 4 to ECHO). Row K now completely breaks out Row J signal pins!
   * Insert **R3 (1kΩ)** into O19 $\to$ O17; insert **R4 (2kΩ)** into O17 $\to$ P17 (solder P17 into Row P GND).
   * Insert **C2 (100nF)** into O7 $\to$ O11; bridge **Q11 $\to$ P11 $\to$ O11** (GND) and **Q7 $\to$ P7 $\to$ O7** (Data) on solder side.
   * Insert **R5 (1kΩ)** into C8 $\to$ C11; insert **R6 (100kΩ)** into C12 $\to$ A12. Solder bridge **D8 $\to$ C8** (GPIO 7 to R5).
   * Insert **TB2 (Soil)** into Row Q (Cols 7, 9, 11); insert **TB1 (Ultrasonic)** into Row Q (Cols 15, 17, 19, 21); bridge **Q21 $\to$ P21** on solder side.
   * Insert **TB3 (Pump)** into Row B (Cols 13, 15); insert **D1 (1N4001)** into Row E (Cols 13 $\to$ 15).
   * Insert **Q1 (FQP30N06L)** into Row C (Cols 12, 13, 14).
   * On solder side, create the 3 vertical actuator tracks:
     * Col 13: **E13 $\to$ D13 $\to$ C13 $\to$ B13** (Pump Drain Track).
     * Col 14: **D14 $\to$ C14 $\to$ B14 $\to$ A14** (Pump GND Track).
     * Col 15: **E15 $\to$ D15 $\to$ C15 $\to$ B15** (Pump +5V Track).
     * Row C: Solder bridge **C11 $\to$ C12** (R5 to Q1 Gate).

### Phase 2: Top-Side Insulated Hookup Wires (Zero Risk of Shorts)

4. **Power & Bus Feed Wires (24 AWG Insulated)**:
   * **W1 (GND Feed)**: Run 24 AWG blue insulated wire down Column 14 on the **TOP side** from **P14** to **D14**.
   * **W2 (+5V Pump Feed)**: Run 24 AWG red insulated wire down Column 15 on the **TOP side** from **N15** to **E15**.
   * **W3a (Col 4 GND Jump)**: Run 24 AWG blue insulated wire on the **TOP side** from **M4** to **P4** (straight jump along Col 4 over Row N +5V).
   * **W3b (Row P GND Feed)**: Run 24 AWG blue insulated wire on the **TOP side** from **P4** to **P11** (arches safely along Row P over P7 Soil Data into Sensor GND bus).
5. **Sensor Signal Wires (26 AWG Insulated)**:
   * **W4 (Ultrasonic 5V)**: Run 26 AWG red insulated wire on the **TOP side** from **N15** up to **Q15** (arches safely over Row P GND!).
   * **W5 (Echo In Link)**: Run 26 AWG green insulated wire on the **TOP side** from **Q19** down to **O19** (arches safely over Row P GND!).
   * **W6 (Soil DATA)**: Run 26 AWG green insulated wire straight down Column 7 on the **TOP side** from **O7** to **K7**.
   * **W7 (Soil VCC)**: Run 26 AWG purple insulated wire parallel down Column 9/8 on the **TOP side** from **Q9** to **K8**.
   * **W8 (Ultrasonic TRIG)**: Run 26 AWG cyan insulated wire on the **TOP side** from **K9** to **Q17**.
   * **W9 (Echo 3.3V)**: Run 26 AWG green insulated wire on the **TOP side** from **O17** to **K10**.
   * Strip 2mm of insulation off each wire end, insert into through-hole pads from the top side, flip the board and solder the pins on the solder side. Snip flush.

---

## 8. Pre-Power Cold Testing Checklist (Multimeter DMM)

**DO NOT INSERT THE ESP32-C3 MODULE YET.** Verify these tests with a DMM in resistance mode:

| Test ID | Multimeter Probes | Expected Value | What It Verifies |
| :---: | :--- | :---: | :--- |
| **T1** | Socket J3 (5V) to Socket J4 (GND) | **$> 10\,\text{k}\Omega$** (Rising) | **No power short!** C3 ($100\,\mu\text{F}$) charging curve confirms clean 5V rail. |
| **T2** | Socket J4 (GND) to Socket J5 (3V3) | **Open circuit ($\infty$)** | No 3.3V rail shorts. |
| **T3** | Socket J4 (GND) to TB1 Pin 4, TB2 Pin 3 | **$< 0.5\,\Omega$** | Sensor ground continuity through Col 4 track, wires W3a/W3b, and Row P bus. |
| **T4** | Socket J4 (GND) to Q1 Source (C14) | **$< 0.5\,\Omega$** | Power ground continuity through Row P bus and Col 14 GND track. |
| **T5** | Socket J3 (5V) to TB1 Pin 1, TB3 Pin 2 | **$< 0.5\,\Omega$** | 5V power distribution continuity via Col 3 track and W2, W4. |
| **T6** | Socket J6 (GPIO 0) to Socket J4 (GND) | **$\approx 10.0\,\text{k}\Omega$** | Exact R2 resistance to ground (via K6-J6 bridge and Col 4 GND track). |
| **T7** | Socket J6 (GPIO 0) to Socket J3 (5V) | **$\approx 10.0\,\text{k}\Omega$** | Exact R1 resistance to 5V rail (via N6). |
| **T8** | Socket D8 (GPIO 7) to Socket J4 (GND) | **$\approx 101\,\text{k}\Omega$** | Series R5 ($1\,\text{k}\Omega$) + R6 ($100\,\text{k}\Omega$) pull-down path via D8-C8 bridge. |
| **T9** | TB1 Pin 3 (Echo) to Socket J10 (GPIO 4)| **$\approx 1.0\,\text{k}\Omega$** | Upper Echo resistor R3. |
| **T10** | Socket J10 (GPIO 4) to Socket J4 (GND)| **$\approx 2.0\,\text{k}\Omega$** | Lower Echo resistor R4 to ground. |
| **T11** | Pad E13 (Anode) to Pad E15 (Cathode) | **Diode Drop ($\sim 0.6\text{V} - 0.7\text{V}$)** | Diode D1 polarity check (red probe on E13, black on E15). |
| **T12** | TB2 Pin 1 (Soil Data at Q7) to Socket J7 (GPIO 1) | **$< 0.5\,\Omega$** | Soil data continuity through Col 7 track (Q7-P7-O7), Wire W6 (O7-K7), and bridge J7-K7. |
| **T13** | TB2 Pin 2 (Soil VCC at Q9) to Socket J8 (GPIO 2) | **$< 0.5\,\Omega$** | Switched soil power continuity via Wire W7 (Q9-K8) and bridge J8-K8. |
| **T14** | TB1 Pin 2 (TRIG at Q17) to Socket J9 (GPIO 3) | **$< 0.5\,\Omega$** | Ultrasonic trigger continuity via Wire W8 (K9-Q17) and bridge J9-K9. |
