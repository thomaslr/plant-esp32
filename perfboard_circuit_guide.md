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
* **Bottom Pump Terminal**: **TB3 (DC Pump)** on **Row B**, with wire entries facing **DOWNWARDS** (South off bottom edge).
* **Flyback Diode Separation**: **D1 (1N5819)** is mounted in **Row E (Cols 19 & 21)**, moved up one row from Row D—giving two full rows of clearance from TB3!
* **100µF Bulk Capacitor & C1 Filter Ground**: **C3 (100µF)** is joined directly between **VCC and GND** in **Row L (Cols 3 & 4)** right at the ESP32 power pins. **C1 (100nF)** has its left leg in **L4** (grounded with C3) and right leg in **K5** (Node), avoiding crowding hole K4.

```
       COL:  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20  21  22  23  24
    ROW R: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (TOP EDGE)
    ROW Q: [·] [·] [·] [·] [·] [·] [  TB2 SOIL  ] [·] [·] [·] [·] [    TB1 ULTRASONIC    ] [·] [·] [·]
    ROW P: [·] [·] [====== ANALOG / SENSOR GND BUS ========================================] [·] [·] [·]
    ROW O: [·] [·] [·] [·] [·] [·] [C2] [·] [·] [·] [C2] [·] [·] [·] [·] [·] [R4] [·] [R3] [·] [·] [·] [·] [·]
    ROW N: [·] [·] [==== +5V SENSOR BUS ===] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW M: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (CLEAR)
    ROW L: [·] [·] [C3+][C3-/C1][·][·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (100µF VCC-GND & C1 GND)
    ROW K: [·] [·] [·] [R2GND][C1][Node][·][·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW J: [·] [·] [ 5V GND 3V3  G0  G1  G2  G3  G4 ]  (ESP32-C3 SuperMini Header 1)        [·] [·] [·] [·]
    ROW I: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW H: [·] [·] |      SOCKET CAVITY             |                                      [·] [·] [·] [·]
    ROW G: [·] [·] |      (CLEAR UNDERNEATH)        |                                      [·] [·] [·] [·]
    ROW F: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW E: [·] [·] |                                |                                [D1A] [·] [D1K] [·] [·] [·] (FLYBACK DIODE)
    ROW D: [·] [·] [ G21 G20 G10 G9  G8  G7  G6  G5 ]  (ESP32-C3 SuperMini Header 2)        [·] [·] [·] [·]
    ROW C: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [R5] [·] [·] [R5] [Q1G Q1D Q1S] [·] [·] [·]
    ROW B: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [TB3-] [·] [TB3+] [·] [·] [·]
    ROW A: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [ POWER GND (A18-A20) ] [·] [·] [·] [·] (BOTTOM EDGE)
```

---

## 3. Complete Bill of Materials (BOM) & Coordinate Map

| Designator | Component | Value / Rating | Footprint | Pins / Terminal Coordinates |
| :---: | :--- | :--- | :--- | :--- |
| **U1** | **ESP32-C3 SuperMini** | RISC-V 160MHz, Wi-Fi | 2× 1×8 Female Sockets | Top Header: **Row J, Cols 3–10**<br>Bottom Header: **Row D, Cols 3–10** |
| **TB1** | **Screw Terminal (4-Pin)** | 5.08 mm Pitch (Ultrasonic) | 4P Through-Hole | **Row Q, Cols 15 (5V), 17 (TRIG), 19 (ECHO), 21 (GND)** |
| **TB2** | **Screw Terminal (3-Pin)** | 5.08 mm Pitch (Soil Sensor) | 3P Through-Hole | **Row Q, Cols 7 (Data), 9 (VCC), 11 (GND)** |
| **TB3** | **Screw Terminal (2-Pin)** | 5.08 mm Pitch (DC Pump) | 2P Through-Hole | **Row B, Cols 19 (Drain -), 21 (+5V)** (Dedicated pads!) |
| **Q1** | **N-MOSFET (Logic-Level)** | **FQP30N06L** (60V, 32A) | TO-220 (Vertical) | **Row C, Cols 18 (G), 19 (D), 20 (S)** |
| **D1** | **Schottky Diode** | **1N5819** (40V, 1A) | DO-41 Axial | **Row E, Col 19 (Anode) $\to$ Col 21 (Cathode)** (Moved up to Row E!) |
| **C3** | **Electrolytic Capacitor** | **$100\,\mu\text{F}$** (16V/25V) | Radial Can (2.5mm pitch) | **Row L, Col 3 (+) $\to$ Col 4 (-)** (Joined between VCC & GND!) |
| **C1** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.1" pitch) | **Row L, Col 4 (GND) $\to$ Row K, Col 5 (Node)** (Left leg in L4!) |
| **C2** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.2" pitch) | **Row O, Col 7 (Data) $\to$ Col 11 (GND)** |
| **R1** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 6, Row N (+5V) $\to$ Row K (Node)** |
| **R2** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row K, Col 6 (Node) $\to$ Col 4 (GND)** |
| **R3** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 19 (Echo) $\to$ Col 17 (Node)** |
| **R4** | **Metal Film Resistor** | $2\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 17 (Node) $\to$ Row P (GND)** |
| **R5** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row C, Col 14 $\to$ Col 17** |
| **R6** | **Metal Film Resistor** | $100\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 18, Row C (Gate) $\to$ Row A (GND)** |

---

## 4. Component Coordinate Placement Table (Top View)

| Component | Pin / Terminal | Coordinate | Physical Orientation & Mounting Notes |
| :--- | :--- | :---: | :--- |
| **ESP32 Socket 1** | Pin 1 (5V In)<br>Pin 2 (GND)<br>Pin 3 (3V3 Out)<br>Pin 4 (GPIO 0 / Supply ADC)<br>Pin 5 (GPIO 1 / Soil ADC)<br>Pin 6 (GPIO 2 / Soil VCC)<br>Pin 7 (GPIO 3 / TRIG)<br>Pin 8 (GPIO 4 / ECHO) | **J3**<br>**J4**<br>**J5**<br>**J6**<br>**J7**<br>**J8**<br>**J9**<br>**J10** | 1×8 Female Header in Row J.<br>USB-C points **LEFT** into Cols 1–2.<br>Header spacing is standard 0.600" DIP pitch to Row D. |
| **ESP32 Socket 2** | Pin 16 (GPIO 21)<br>Pin 15 (GPIO 20)<br>Pin 14 (GPIO 10)<br>Pin 13 (GPIO 9)<br>Pin 12 (GPIO 8 / Blue LED)<br>Pin 11 (GPIO 7 / Pump Gate)<br>Pin 10 (GPIO 6)<br>Pin 9 (GPIO 5) | **D3**<br>**D4**<br>**D5**<br>**D6**<br>**D7**<br>**D8**<br>**D9**<br>**D10** | 1×8 Female Header in Row D.<br>Rows I, H, G, F, E underneath remain clear for underside routing. |
| **C3 (100µF Bulk Cap)** | Positive (+)<br>Negative (- stripe) | **L3**<br>**L4** | Radial electrolytic capacitor ($100\,\mu\text{F}$, 16V/25V).<br>**Joined directly between VCC & GND** across ESP32 pins J3 and J4. |
| **TB2 (Soil)** | Pin 1 (AOUT / Data)<br>Pin 2 (Switched VCC)<br>Pin 3 (GND) | **Q7**<br>**Q9**<br>**Q11** | 3-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off Row R).<br>**Zero wire crossovers**: Pin 1 aligns with Col 7 (GPIO 1), Pin 2 aligns with Col 8/9 (GPIO 2). |
| **TB1 (Ultrasonic)** | Pin 1 (+5V Power)<br>Pin 2 (TRIG Pulse)<br>Pin 3 (ECHO Return)<br>Pin 4 (GND) | **Q15**<br>**Q17**<br>**Q19**<br>**Q21** | 4-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off Row R).<br>Cols 12–14 leave 7.62 mm spacing between TB2 and TB1. |
| **TB3 (Pump)** | Pin 1 (Drain / Pump -)<br>Pin 2 (+5V / Pump +) | **B19**<br>**B21** | 2-Pin 5.08 mm terminal block.<br>Wire entry slots face **DOWNWARDS** (South off Row A).<br>**Dedicated holes**: Not shared with the flyback diode! |
| **D1 (1N5819)** | Anode (+)<br>Cathode (- stripe) | **E19**<br>**E21** | Axial flyback diode mounted in **Row E** (moved up one row!).<br>Silver cathode stripe faces **RIGHT** (Col 21 / +5V).<br>**Completely separated from TB3** (3 rows away). |
| **Q1 (FQP30N06L)** | Pin 1 (Gate)<br>Pin 2 (Drain)<br>Pin 3 (Source) | **C18**<br>**C19**<br>**C20** | TO-220 package across 3 adjacent holes.<br>Metal heatsink tab faces **NORTH** (inward toward Row D).<br>Plastic labeled front faces **SOUTH** (toward Row B). |
| **R1 (10kΩ)** | Top (+5V)<br>Bottom (Divider Node) | **N6**<br>**K6** | 5V supply divider top leg. Spans vertically from Row N (+5V Bus) to Row K (Node). |
| **R2 (10kΩ)** | Right (Divider Node)<br>Left (GND) | **K6**<br>**K4** | 5V supply divider bottom leg. Spans horizontally along Row K from Col 6 (Node) to Col 4 (GND). |
| **C1 (100nF)** | Right (Divider Node)<br>Left (GND) | **K5**<br>**L4** | ADC ripple filter. Spans diagonally from Col 4 Row L (GND) to Col 5 Row K (Node). Left leg moved to L4; eliminates hole crowding at K4! |
| **C2 (100nF)** | Lead 1 (Data AOUT)<br>Lead 2 (GND) | **O7**<br>**O11** | Wi-Fi RF bypass capacitor directly across Soil AOUT and GND. |
| **R3 (1kΩ)** | Echo In<br>Divider Node | **O19**<br>**O17** | Echo stepper upper leg. Spans Col 19 to Col 17 in Row O. |
| **R4 (2kΩ)** | Divider Node<br>GND | **O17**<br>**P17** | Echo stepper lower leg. Spans Row O to Row P (GND). |
| **R5 (1kΩ)** | Input (from GPIO 7)<br>Output (to Gate) | **C14**<br>**C17** | Gate damping resistor. Spans Col 14 to Col 17 in Row C. |
| **R6 (100kΩ)** | Gate Node<br>GND | **C18**<br>**A18** | Gate pull-down resistor. Spans Row C (Gate) to Row A (GND). |

---

## 5. Explicit Physical Wiring Schedule (Solder Side Connections)

Because **perfboard holes are isolated**, every net requires a deliberate physical connection method:

### A. Power & Ground Distribution Rails

| Net Name | Start Pad | End Pad | Physical Connection Method | How It Is Made |
| :--- | :---: | :---: | :--- | :--- |
| **Analog GND Bus** | **P4** | **P21** | **Bare Solid Bus Wire** | Lay 22 AWG bare tinned wire along Row P (Cols 4–21). Solder pads P4, P11, P17, P21. |
| **Power GND Bus** | **A18** | **A20** | **Bare Wire / 3-Pad Bridge** | Solder bridge or bare wire along Row A (Cols 18–20). Connects Star GND (A18), R6 (A18), and Q1 Source (A20). Cols 21–24 are clear. |
| **Star Ground Link** | **J4** (ESP32 GND) | **A18** (Power GND) | **24 AWG Insulated Wire** | Heavy insulated wire routed around left margin linking system ground to power ground bus. |
| **Top +5V Bus** | **N3** | **N15** | **Bare Solid Bus Wire** | Lay 22 AWG bare tinned wire along Row N (Cols 3–15). Solder pads N3, N6, N15. |
| **+5V Power Feed** | **J3** (ESP32 5V) | **E21** (TB3 + / D1 Cathode) | **24 AWG Insulated Wire** | Heavy insulated wire feeding USB 5V directly to pump terminal and flyback diode. |
| **C3 (+) 5V Bridge** | **L3** (C3 +) | **J3** (ESP32 5V) | **Lead Bend / Solder Bridge**| Direct vertical solder bridge down Column 3 ties C3 (+) directly to ESP32 5V In (VCC). |
| **C3/C1 (-) GND Bridge**| **L4** (C3 - & C1 GND) | **J4** (ESP32 GND) | **Lead Bend / Solder Bridge**| Direct vertical solder bridge down Column 4 ties C3 (-) and C1 left leg directly to ESP32 GND. |

### B. High-Current Actuator Connections (Separated D1 & TB3 Bridges)

| Connection | Pads Involved | Physical Method | Detailed Assembly Instruction |
| :--- | :---: | :--- | :--- |
| **D1 Anode $\to$ Q1 Drain $\to$ TB3 Drain** | **E19 $\to$ D19 $\to$ C19 $\to$ B19** | **4-Pad Solder Bridge** | Straight vertical solder bridge along Column 19 links D1 Anode (E19), Q1 Drain (C19), and TB3 Pin 1 (B19) via pad D19. **Zero wire needed!** |
| **D1 Cathode $\to$ TB3 (+5V)** | **E21 $\to$ D21 $\to$ C21 $\to$ B21** | **4-Pad Bridge / Lead Bend** | Straight vertical solder bridge or D1 lead bend along Column 21 connects D1 Cathode (E21) to TB3 Pin 2 (B21) and the +5V feed wire from J3. |
| **Q1 Source $\to$ Power GND** | **C20 $\to$ A20** | **Lead Bend** | Bend Q1 Source lead down Column 20 from Row C to Row A; solder directly onto Row A GND bus. |
| **R5 Right $\to$ Q1 Gate** | **C17 $\to$ C18** | **2-Pad Solder Bridge** | Solder bridge connects output of R5 (C17) directly to Q1 Gate pin (C18). |
| **Q1 Gate $\to$ R6 Top** | **C18** | **Shared Hole / Pad** | R6 top lead and Q1 Gate share pad C18 (or bridge C18 to D18). |

### C. Signal & Sensor Jumpers (Zero Crossovers!)

| Signal Net | From Pad | To Pad | Method | Exact Function |
| :--- | :---: | :---: | :--- | :--- |
| **5V Supply Sense** | **K6** (Divider Node) | **J6** (ESP32 GPIO 0) | **1-Pad Solder Bridge** | Direct solder bridge between Row K and Row J on Col 6 feeds 2.5V into ADC. |
| **C1 Filter Bridge** | **K5** (C1 Node) | **K6** (Divider Node) | **1-Pad Solder Bridge** | Solder bridge on Row K links C1 right leg directly in parallel with R2 at divider node. |
| **R2 GND Bridge** | **K4** (R2 GND) | **J4** (ESP32 GND) | **1-Pad Solder Bridge** | Direct solder bridge between Row K and Row J on Col 4 ties R2 ground to ESP32 ground. |
| **Soil Analog Sense** | **Q7** (TB2 Pin 1 Data) | **J7** (ESP32 GPIO 1) | **Straight Insulated Wire** | Runs straight down Column 7 with **zero wire crossovers** into ADC1_CH1. |
| **Soil Switched VCC** | **Q9** (TB2 Pin 2 VCC) | **J8** (ESP32 GPIO 2) | **Parallel Insulated Wire** | Runs parallel down Column 9/8 with **zero wire crossovers** from GPIO 2. |
| **Soil GND & RF Filter**| **Q11 $\to$ P11 $\to$ O11** | — | **Solder Bridge** | Links TB2 GND to Row P GND and C2 bypass capacitor. |
| **Ultrasonic TRIG** | **Q17** (TB1 Pin 2) | **J9** (ESP32 GPIO 3) | **Insulated Jumper Wire** | Carries 10µs digital trigger pulse to HC-SR04. |
| **Echo Input Link** | **Q19** (TB1 Pin 3) | **O19** (R3 In) | **Short Solder Bridge** | Feeds raw 5V Echo pulse into top leg of R3 (1kΩ). |
| **Echo 3.3V Stepped** | **O17** (Divider Node)| **J10** (ESP32 GPIO 4)| **Insulated Jumper Wire** | Safely connects stepped 3.33V Echo pulse to ESP32 GPIO 4. |
| **Pump Gate Drive** | **D8** (ESP32 GPIO 7)| **C14** (R5 Left) | **Insulated Jumper Wire** | Connects GPIO 7 digital output to R5 gate damping resistor. |

---

## 6. Noise Immunity & Ground Architecture

This layout guarantees **complete galvanic isolation** between noisy motor surges and sensitive analog inputs:

```
 [ USB-C 5V Input ]
       │
       ├─────────────────────────────────┐
       ▼                                 ▼
 [ Row N +5V Bus (Clean) ]         [ High-Current Feed (Pump) ]
   • ESP32 SuperMini (J3)            • D1 Cathode (E21)
   • 100µF Bulk Reservoir (L3/L4)    • TB3 Pin 2 (Pump +5V, B21)
   • 10k/10k Supply Divider (R1)         │
   • TB1 Ultrasonic 5V (Q15)             │
       │                                 │
       ▼                                 ▼
 [ Row P Analog GND Bus ]          [ Row A Power GND (A18-A20) ]
   • ESP32 GND (J4)                  • Q1 MOSFET Source (C20)
   • Supply Filter C1 & R2 (L4/K4)   • R6 Pull-Down Resistor (A18)
   • Soil Sensor GND & C2                │
   • Echo Divider R4 (P17)               │
   • Ultrasonic GND (Q21)                │
       ▲                                 │
       │                                 │
       └──────── [ Star Ground Tie ] ────┘
```

1. **Local Microcontroller Decoupling**: The $100\,\mu\text{F}$ capacitor is joined directly across **J3 (5V/VCC)** and **J4 (GND)**, providing low-impedance energy reservoir right at the ESP32 module.
2. **Dedicated Actuator Pads**: Flyback diode D1 in Row E absorbs inductive turn-off energy right across Column 19 (Drain) and Column 21 (+5V), with zero mechanical strain or crammed terminal pins.

---

## 7. Step-by-Step Soldering Sequence

1. **Female Header Sockets**:
   * Insert two 1×8 female headers into **Row J (Cols 3–10)** and **Row D (Cols 3–10)**.
   * Tack-solder corner pins, verify perpendicular alignment, then finish all 16 pins.
2. **Ground & Power Bus Rails**:
   * Lay bare tinned wire along **Row P (Cols 4–21)** for Analog GND. Solder at P4, P11, P17, P21.
   * Solder bridge or lay tinned wire along **Row A (Cols 18–20)** for Power GND. Solder at A18, A19, A20. (Cols 21–24 remain completely open).
   * Lay bare tinned wire along **Row N (Cols 3–15)** for +5V Bus. Solder at N3, N6, N15.
   * Solder the **Star Ground Link** (24 AWG insulated) from **J4** to **A18**.
   * Solder the **+5V Power Feed** (24 AWG insulated) from **J3** to **E21**.
3. **Passive Resistors & Capacitors (Row K and Row L)**:
   * Insert **C3 ($100\,\mu\text{F}$)** into **Row L (Cols 3 $\to$ 4)** with positive lead in L3 and negative stripe in L4.
   * On solder side, bridge **L3 $\to$ K3 $\to$ J3** (to 5V) and **L4 $\to$ K4 $\to$ J4** (to GND).
   * Insert **R1 (10kΩ)** vertically into **N6 $\to$ K6**.
   * Insert **R2 (10kΩ)** horizontally along Row K into **K6 $\to$ K4**.
   * Insert **C1 (100nF)** diagonally with left lead in **L4** (GND) and right lead in **K5** (Node).
   * On underside (solder side):
     * Create solder bridge across **K5 $\to$ K6 $\to$ J6** (connecting divider node and C1 directly to ESP32 GPIO 0).
     * Solder bridge **L4 $\to$ K4 $\to$ J4** ties C3 (-), C1 left lead, and R2 ground directly to ESP32 GND.
   * Insert **R3 (1kΩ)** into O19 $\to$ O17; insert **R4 (2kΩ)** into O17 $\to$ P17. Solder and tie P17 to Row P GND.
   * Insert **C2 (100nF)** into O7 $\to$ O11.
   * Insert **R5 (1kΩ)** into C14 $\to$ C17; insert **R6 (100kΩ)** into C18 $\to$ A18.
4. **Screw Terminal Blocks & Separated Flyback Diode**:
   * Insert **TB2 (Soil)** into **Row Q (Cols 7, 9, 11)** facing UPWARDS.
   * Insert **TB1 (Ultrasonic)** into **Row Q (Cols 15, 17, 19, 21)** facing UPWARDS.
   * Insert **TB3 (Pump)** into **Row B (Cols 19, 21)** facing DOWNWARDS.
   * Insert **D1 (1N5819)** into **Row E (Cols 19 $\to$ 21)** with silver cathode stripe facing Col 21. **(Mounted on Row E—completely separated from TB3!)**
   * Solder all terminal block and diode pins securely.
5. **Actuator Switching (MOSFET & Underside Solder Bridges)**:
   * Insert **Q1 (FQP30N06L)** into **Row C (Cols 18, 19, 20)** with metal tab facing North.
   * On solder side:
     * Create 4-pad solder bridge along Col 19: **E19 $\to$ D19 $\to$ C19 $\to$ B19** (D1 Anode to Q1 Drain to TB3 Pin 1 via D19).
     * Create 4-pad solder bridge / lead bend along Col 21: **E21 $\to$ D21 $\to$ C21 $\to$ B21** (D1 Cathode to TB3 Pin 2 +5V).
     * Solder bridge **C17 $\to$ C18** (R5 to Gate).
     * Bend Q1 Source (C20) down to A20 on Row A Power GND bus.
6. **Insulated Signal Jumpers**:
   * Jumper from **D8 (GPIO 7)** to **C14 (R5 Left)**.
   * Straight jumper from **Q7 (TB2 Pin 1 Data)** to **J7 (GPIO 1 ADC)**.
   * Parallel jumper from **Q9 (TB2 Pin 2 VCC)** to **J8 (GPIO 2 Power)**.
   * Jumper from **Q17 (TB1 TRIG)** to **J9 (GPIO 3)**.
   * Jumper from **O17 (Echo 3.3V)** to **J10 (GPIO 4)**.

---

## 8. Pre-Power Cold Testing Checklist (Multimeter DMM)

**DO NOT INSERT THE ESP32-C3 MODULE YET.** Verify these tests with a DMM in resistance mode:

| Test ID | Multimeter Probes | Expected Value | What It Verifies |
| :---: | :--- | :---: | :--- |
| **T1** | Socket J3 (5V) to Socket J4 (GND) | **$> 10\,\text{k}\Omega$** (Rising) | **No power short!** C3 ($100\,\mu\text{F}$) charging curve confirms clean 5V rail. |
| **T2** | Socket J4 (GND) to Socket J5 (3V3) | **Open circuit ($\infty$)** | No 3.3V rail shorts. |
| **T3** | Socket J4 (GND) to TB1 Pin 4, TB2 Pin 3 | **$< 0.5\,\Omega$** | Sensor ground continuity through Row P bus. |
| **T4** | Socket J4 (GND) to Q1 Source (C20) | **$< 0.5\,\Omega$** | Power ground continuity through Star Link. |
| **T5** | Socket J3 (5V) to TB1 Pin 1, TB3 Pin 2 | **$< 0.5\,\Omega$** | 5V power distribution continuity. |
| **T6** | Socket J6 (GPIO 0) to Socket J4 (GND) | **$\approx 10.0\,\text{k}\Omega$** | Exact R2 resistance to ground (via K4-J4 bridge). |
| **T7** | Socket J6 (GPIO 0) to Socket J3 (5V) | **$\approx 10.0\,\text{k}\Omega$** | Exact R1 resistance to 5V rail (via N6). |
| **T8** | Socket D8 (GPIO 7) to Socket J4 (GND) | **$\approx 101\,\text{k}\Omega$** | Series R5 ($1\,\text{k}\Omega$) + R6 ($100\,\text{k}\Omega$) pull-down path. |
| **T9** | TB1 Pin 3 (Echo) to Socket J10 (GPIO 4)| **$\approx 1.0\,\text{k}\Omega$** | Upper Echo resistor R3. |
| **T10** | Socket J10 (GPIO 4) to Socket J4 (GND)| **$\approx 2.0\,\text{k}\Omega$** | Lower Echo resistor R4 to ground. |
| **T11** | Pad E19 (Anode) to Pad E21 (Cathode) | **Diode Drop ($\sim 0.35\text{V}$)** | Diode D1 polarity check (red probe on E19, black on E21). |
