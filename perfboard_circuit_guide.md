# ESP32-C3 SuperMini 18×24 Perfboard Assembly Guide (Descending Rows R $\to$ A)

This guide details the permanent, soldered build of the Plant Monitor & Automatic Watering system on a standard **0.1" (2.54 mm) pitch plated through-hole perfboard** with **18 Rows descending from R (Top) to A (Bottom)** and **24 Columns (1 to 24)**.

---

## 1. Perfboard vs. Breadboard Fundamentals: Every Hole is Isolated

> [!IMPORTANT]
> **NO INTERNAL TRACES ON PERFBOARD**: Unlike solderless breadboards where rows or columns share internal metal spring clips, **every single through-hole pad on a perfboard is completely electrically isolated**.
> 
> Two components placed in adjacent holes (e.g., Row L and Row M) **DO NOT CONNECT** unless you physically create the connection!

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
* **ESP32 Sockets**: Dual 1×8 female headers on **Row L** and **Row F** across Columns 3–10 (standard 0.600" / 6-row DIP pitch).
* **USB-C Port**: Extends off the **Left Edge** (Cols 1–2 clearance) for unobstructed cable insertion.
* **Top Sensor Terminals**: **TB2 (Soil)** & **TB1 (Ultrasonic)** on **Row Q**, with wire entries facing **UPWARDS** (North off top edge).
* **Bottom Pump Terminal**: **TB3 (DC Pump)** on **Row B**, with wire entries facing **DOWNWARDS** (South off bottom edge).

```
       COL:  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20  21  22  23  24
    ROW R: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] (TOP EDGE)
    ROW Q: [·] [·] [·] [·] [·] [·] [  TB2 SOIL  ] [·] [·] [·] [·] [    TB1 ULTRASONIC    ] [·] [·] [·]
    ROW P: [·] [·] [====== ANALOG / SENSOR GND BUS ========================================] [·] [·] [·]
    ROW O: [·] [·] [·] [·] [·] [·] [C2] [·] [·] [·] [C2] [·] [·] [·] [·] [·] [R4] [·] [R3] [·] [·] [·] [·] [·]
    ROW N: [·] [·] [==== +5V SENSOR BUS ===] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW M: [·] [·] [·] [GND][C1][Node][·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW L: [·] [·] [ 5V GND 3V3  G0  G1  G2  G3  G4 ]  (ESP32-C3 SuperMini Header 1)        [·] [·] [·] [·]
    ROW K: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW J: [·] [·] |      SOCKET CAVITY             |                                      [·] [·] [·] [·]
    ROW I: [·] [·] |      (CLEAR UNDERNEATH)        |                                      [·] [·] [·] [·]
    ROW H: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW G: [·] [·] |                                |                                      [·] [·] [·] [·]
    ROW F: [·] [·] [ G21 G20 G10 G9  G8  G7  G6  G5 ]  (ESP32-C3 SuperMini Header 2)        [·] [·] [·] [·]
    ROW E: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·]
    ROW D: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [C3+] [·] [C3-] [·]
    ROW C: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [R5] [·] [·] [R5] [Q1G Q1D Q1S] [·] [·] [·]
    ROW B: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [TB3-] [·] [TB3+] [·] [·] [·]
    ROW A: [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [·] [=== POWER GND BUS ===] [·] (BOTTOM EDGE)
```

> [!TIP]
> **Capacitor C3 Selection ($470\,\mu\text{F}$ vs. $100\,\mu\text{F}$)**:
> While a $470\,\mu\text{F}$ capacitor provides maximum headroom for large submersible motors, a standard **$100\,\mu\text{F}$ (16V–25V) electrolytic capacitor** is completely adequate for standard 5V USB fountain pumps ($< 2.5\,\text{W}$). It must remain physically placed in **Row D (Cols 21 & 23)** right next to TB3 and Q1 so motor start surges are arrested locally before reaching the ESP32.

---

## 3. Complete Bill of Materials (BOM) & Coordinate Map

| Designator | Component | Value / Rating | Footprint | Pins / Terminal Coordinates |
| :---: | :--- | :--- | :--- | :--- |
| **U1** | **ESP32-C3 SuperMini** | RISC-V 160MHz, Wi-Fi | 2× 1×8 Female Sockets | Top Header: **Row L, Cols 3–10**<br>Bottom Header: **Row F, Cols 3–10** |
| **TB1** | **Screw Terminal (4-Pin)** | 5.08 mm Pitch (Ultrasonic) | 4P Through-Hole | **Row Q, Cols 15 (5V), 17 (TRIG), 19 (ECHO), 21 (GND)** |
| **TB2** | **Screw Terminal (3-Pin)** | 5.08 mm Pitch (Soil Sensor) | 3P Through-Hole | **Row Q, Cols 7 (Data), 9 (VCC), 11 (GND)** |
| **TB3** | **Screw Terminal (2-Pin)** | 5.08 mm Pitch (DC Pump) | 2P Through-Hole | **Row B, Cols 19 (Drain -), 21 (+5V)** |
| **Q1** | **N-MOSFET (Logic-Level)** | **FQP30N06L** (60V, 32A) | TO-220 (Vertical) | **Row C, Cols 18 (G), 19 (D), 20 (S)** |
| **D1** | **Schottky Diode** | **1N5819** (40V, 1A) | DO-41 Axial | **Row B, Col 19 (Anode) $\to$ Col 21 (Cathode)** |
| **C1** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.1" pitch) | **Row M, Col 5 (Node) $\to$ Col 4 (GND)** |
| **C2** | **Ceramic Capacitor** | $100\,\text{nF}$ (50V, "104") | Radial Disc (0.2" pitch) | **Row O, Col 7 (Data) $\to$ Col 11 (GND)** |
| **C3** | **Electrolytic Capacitor** | $470\,\mu\text{F}$ or $100\,\mu\text{F}$ (16V/25V) | Radial Can (5mm pitch) | **Row D, Col 21 (+) $\to$ Col 23 (-)** |
| **R1** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 6, Row N (+5V) $\to$ Row M (Node)** |
| **R2** | **Metal Film Resistor** | $10\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row M, Col 6 (Node) $\to$ Col 4 (GND)** |
| **R3** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 19 (Echo) $\to$ Col 17 (Node)** |
| **R4** | **Metal Film Resistor** | $2\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row O, Col 17 (Node) $\to$ Row P (GND)** |
| **R5** | **Metal Film Resistor** | $1\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Row C, Col 14 $\to$ Col 17** |
| **R6** | **Metal Film Resistor** | $100\,\text{k}\Omega$ ($0.25\,\text{W}, 1\%$) | Axial | **Col 18, Row C (Gate) $\to$ Row A (GND)** |

---

## 4. Component Coordinate Placement Table (Top View)

| Component | Pin / Terminal | Coordinate | Physical Orientation & Mounting Notes |
| :--- | :--- | :---: | :--- |
| **ESP32 Socket 1** | Pin 1 (5V In)<br>Pin 2 (GND)<br>Pin 3 (3V3 Out)<br>Pin 4 (GPIO 0 / Supply ADC)<br>Pin 5 (GPIO 1 / Soil ADC)<br>Pin 6 (GPIO 2 / Soil VCC)<br>Pin 7 (GPIO 3 / TRIG)<br>Pin 8 (GPIO 4 / ECHO) | **L3**<br>**L4**<br>**L5**<br>**L6**<br>**L7**<br>**L8**<br>**L9**<br>**L10** | 1×8 Female Header in Row L.<br>USB-C points **LEFT** into Cols 1–2.<br>Header spacing is standard 0.600" DIP pitch to Row F. |
| **ESP32 Socket 2** | Pin 16 (GPIO 21)<br>Pin 15 (GPIO 20)<br>Pin 14 (GPIO 10)<br>Pin 13 (GPIO 9)<br>Pin 12 (GPIO 8 / Blue LED)<br>Pin 11 (GPIO 7 / Pump Gate)<br>Pin 10 (GPIO 6)<br>Pin 9 (GPIO 5) | **F3**<br>**F4**<br>**F5**<br>**F6**<br>**F7**<br>**F8**<br>**F9**<br>**F10** | 1×8 Female Header in Row F.<br>Rows K, J, I, H, G underneath remain clear for underside routing. |
| **TB2 (Soil)** | Pin 1 (AOUT / Data)<br>Pin 2 (Switched VCC)<br>Pin 3 (GND) | **Q7**<br>**Q9**<br>**Q11** | 3-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off Row R).<br>**Zero wire crossovers**: Pin 1 aligns with Col 7 (GPIO 1), Pin 2 aligns with Col 8/9 (GPIO 2). |
| **TB1 (Ultrasonic)** | Pin 1 (+5V Power)<br>Pin 2 (TRIG Pulse)<br>Pin 3 (ECHO Return)<br>Pin 4 (GND) | **Q15**<br>**Q17**<br>**Q19**<br>**Q21** | 4-Pin 5.08 mm terminal block.<br>Wire entry slots face **UPWARDS** (North off Row R).<br>Cols 12–14 leave 7.62 mm spacing between TB2 and TB1. |
| **TB3 (Pump)** | Pin 1 (Drain / Pump -)<br>Pin 2 (+5V / Pump +) | **B19**<br>**B21** | 2-Pin 5.08 mm terminal block.<br>Wire entry slots face **DOWNWARDS** (South off Row A). |
| **Q1 (FQP30N06L)** | Pin 1 (Gate)<br>Pin 2 (Drain)<br>Pin 3 (Source) | **C18**<br>**C19**<br>**C20** | TO-220 package across 3 adjacent holes.<br>Metal heatsink tab faces **NORTH** (inward toward Row D).<br>Plastic labeled front faces **SOUTH** (toward Row B). |
| **D1 (1N5819)** | Anode (+)<br>Cathode (- stripe) | **B19**<br>**B21** | Axial flyback diode spanning Cols 19 to 21 directly across TB3.<br>Silver cathode stripe faces **RIGHT** (Col 21 / +5V). |
| **C3 (470µF / 100µF)** | Positive (+)<br>Negative (- stripe) | **D21**<br>**D23** | Radial electrolytic capacitor.<br>Positive lead in Col 21 (+5V pump rail).<br>Negative lead in Col 23 (tied to Row A Power GND). |
| **R1 (10kΩ)** | Top (+5V)<br>Bottom (Divider Node) | **N6**<br>**M6** | 5V supply divider top leg. Spans vertically from Row N (+5V Bus) to Row M (Node). |
| **R2 (10kΩ)** | Right (Divider Node)<br>Left (GND) | **M6**<br>**M4** | 5V supply divider bottom leg. Spans horizontally along Row M from Col 6 (Node) to Col 4 (GND). |
| **C1 (100nF)** | Right (Divider Node)<br>Left (GND) | **M5**<br>**M4** | ADC ripple filter. Spans horizontally along Row M from Col 5 to Col 4. Solder bridged: M5 $\to$ M6 (Node) and M4 $\to$ L4 (GND). **Zero bus crossing!** |
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
| **Power GND Bus** | **A18** | **A24** | **Bare Solid Bus Wire** | Lay 22 AWG bare tinned wire along Row A (Cols 18–24). Solder pads A18, A20, A23, A24. |
| **Star Ground Link** | **L4** (ESP32 GND) | **A18** (Power GND) | **24 AWG Insulated Wire** | Heavy insulated wire routed around left margin linking system ground to power ground bus. |
| **Top +5V Bus** | **N3** | **N15** | **Bare Solid Bus Wire** | Lay 22 AWG bare tinned wire along Row N (Cols 3–15). Solder pads N3, N6, N15. |
| **+5V Power Feed** | **L3** (ESP32 5V) | **D21** (C3 +) | **24 AWG Insulated Wire** | Heavy insulated wire feeding USB 5V directly to reservoir capacitor C3 and TB3 Pin 2. |

### B. High-Current Actuator Connections (Direct Solder Bridges)

| Connection | Pads Involved | Physical Method | Detailed Assembly Instruction |
| :--- | :---: | :--- | :--- |
| **Q1 Drain $\to$ TB3 Drain $\to$ Diode Anode** | **C19 $\to$ B19** | **2-Pad Solder Bridge** | Solder bridge directly connects Q1 Drain (C19) to TB3 Pin 1 / D1 Anode (B19). **Zero wire needed!** |
| **C3 (+) $\to$ TB3 (+5V) $\to$ Diode Cathode** | **D21 $\to$ B21** | **Bridge / Lead Trace** | Run C3 (+) lead down Column 21 from Row D to Row B to tie into TB3 Pin 2 and D1 Cathode. |
| **Q1 Source $\to$ Power GND** | **C20 $\to$ A20** | **Lead Bend** | Bend Q1 Source lead down Column 20 from Row C to Row A; solder directly onto Row A GND bus. |
| **C3 (-) $\to$ Power GND** | **D23 $\to$ A23** | **Lead Bend** | Bend C3 negative lead down Column 23 from Row D to Row A; solder directly onto Row A GND bus. |
| **R5 Right $\to$ Q1 Gate** | **C17 $\to$ C18** | **2-Pad Solder Bridge** | Solder bridge connects output of R5 (C17) directly to Q1 Gate pin (C18). |
| **Q1 Gate $\to$ R6 Top** | **C18** | **Shared Hole / Pad** | R6 top lead and Q1 Gate share pad C18 (or bridge C18 to D18). |

### C. Signal & Sensor Jumpers (Zero Crossovers!)

| Signal Net | From Pad | To Pad | Method | Exact Function |
| :--- | :---: | :---: | :--- | :--- |
| **5V Supply Sense** | **M6** (Divider Node) | **L6** (ESP32 GPIO 0) | **1-Pad Solder Bridge** | Direct solder bridge between Row M and Row L on Col 6 feeds 2.5V into ADC. |
| **C1 Filter Bridge** | **M5** (C1 Node) | **M6** (Divider Node) | **1-Pad Solder Bridge** | Solder bridge on Row M links C1 filter cap directly in parallel with R2. |
| **R2/C1 GND Bridge** | **M4** (R2/C1 GND) | **L4** (ESP32 GND) | **1-Pad Solder Bridge** | Direct solder bridge between Row M and Row L on Col 4 ties divider ground to ESP32 ground without crossing Row N. |
| **Soil Analog Sense** | **Q7** (TB2 Pin 1 Data) | **L7** (ESP32 GPIO 1) | **Straight Insulated Wire** | Runs straight down Column 7 with **zero wire crossovers** into ADC1_CH1. |
| **Soil Switched VCC** | **Q9** (TB2 Pin 2 VCC) | **L8** (ESP32 GPIO 2) | **Parallel Insulated Wire** | Runs parallel down Column 9/8 with **zero wire crossovers** from GPIO 2. |
| **Soil GND & RF Filter**| **Q11 $\to$ P11 $\to$ O11** | — | **Solder Bridge** | Links TB2 GND to Row P GND and C2 bypass capacitor. |
| **Ultrasonic TRIG** | **Q17** (TB1 Pin 2) | **L9** (ESP32 GPIO 3) | **Insulated Jumper Wire** | Carries 10µs digital trigger pulse to HC-SR04. |
| **Echo Input Link** | **Q19** (TB1 Pin 3) | **O19** (R3 In) | **Short Solder Bridge** | Feeds raw 5V Echo pulse into top leg of R3 (1kΩ). |
| **Echo 3.3V Stepped** | **O17** (Divider Node)| **L10** (ESP32 GPIO 4)| **Insulated Jumper Wire** | Safely connects stepped 3.33V Echo pulse to ESP32 GPIO 4. |
| **Pump Gate Drive** | **F8** (ESP32 GPIO 7)| **C14** (R5 Left) | **Insulated Jumper Wire** | Connects GPIO 7 digital output to R5 gate damping resistor. |

---

## 6. Noise Immunity & Ground Architecture

This layout guarantees **complete galvanic isolation** between noisy motor surges and sensitive analog inputs:

```
 [ USB-C 5V Input ]
       │
       ├─────────────────────────────────┐
       ▼                                 ▼
 [ Row N +5V Bus (Clean) ]         [ High-Current Feed (Pump) ]
   • ESP32 SuperMini (L3)            • C3 Bulk Capacitor (470µF / 100µF)
   • 10k/10k Supply Divider (R1)     • TB3 Pin 2 (Pump +5V)
   • TB1 Ultrasonic 5V (Q15)         • D1 Flyback Diode (Cathode)
       │                                 │
       ▼                                 ▼
 [ Row P Analog GND Bus ]          [ Row A Power GND Bus ]
   • ESP32 GND (L4)                  • Q1 MOSFET Source (C20)
   • Supply Filter C1 & R2 (via M4)  • C3 Negative Lead (D23)
   • Soil Sensor GND & C2            • R6 Pull-Down Resistor (A18)
   • Echo Divider R4 (P17)               │
   • Ultrasonic GND (Q21)                │
       ▲                                 │
       │                                 │
       └──────── [ Star Ground Tie ] ────┘
```

1. **Pump Inrush Separation**: The motor startup spike is handled exclusively by **Row A Power GND** and the bulk reservoir cap (C3). This current never flows through the Row P sensor ground bus.
2. **ADC Stability**: ADC1 channels (`GPIO 0` on pin L6 and `GPIO 1` on pin L7) reference clean ground without ground bounce, ensuring stable, jitter-free sensor readings.

---

## 7. Step-by-Step Soldering Sequence

1. **Female Header Sockets**:
   * Insert two 1×8 female headers into **Row L (Cols 3–10)** and **Row F (Cols 3–10)**.
   * Tack-solder corner pins, verify perpendicular alignment, then finish all 16 pins.
2. **Ground & Power Bus Rails**:
   * Lay bare tinned wire along **Row P (Cols 4–21)** for Analog GND. Solder at P4, P11, P17, P21.
   * Lay bare tinned wire along **Row A (Cols 18–24)** for Power GND. Solder at A18, A20, A23, A24.
   * Lay bare tinned wire along **Row N (Cols 3–15)** for +5V Bus. Solder at N3, N6, N15.
   * Solder the **Star Ground Link** (24 AWG insulated) from **L4** to **A18**.
   * Solder the **+5V Power Feed** (24 AWG insulated) from **L3** to **D21**.
3. **Passive Resistors & Capacitors (Zero-Crossover Layout)**:
   * Insert **R1 (10kΩ)** vertically into **N6 $\to$ M6**.
   * Insert **R2 (10kΩ)** horizontally along Row M into **M6 $\to$ M4**.
   * Insert **C1 (100nF)** horizontally along Row M into **M5 $\to$ M4**.
   * On underside (solder side):
     * Create solder bridge across **M5 $\to$ M6 $\to$ L6** (connecting divider node and C1 directly to ESP32 GPIO 0).
     * Create solder bridge across **M4 $\to$ L4** (connecting R2 and C1 ground directly to ESP32 GND).
   * Insert **R3 (1kΩ)** into O19 $\to$ O17; insert **R4 (2kΩ)** into O17 $\to$ P17. Solder and tie P17 to Row P GND.
   * Insert **C2 (100nF)** into O7 $\to$ O11.
   * Insert **R5 (1kΩ)** into C14 $\to$ C17; insert **R6 (100kΩ)** into C18 $\to$ A18.
4. **Screw Terminal Blocks**:
   * Insert **TB2 (Soil)** into **Row Q (Cols 7, 9, 11)** facing UPWARDS.
   * Insert **TB1 (Ultrasonic)** into **Row Q (Cols 15, 17, 19, 21)** facing UPWARDS.
   * Insert **TB3 (Pump)** into **Row B (Cols 19, 21)** facing DOWNWARDS.
   * Solder all terminal block pins securely.
5. **Actuator Switching (MOSFET, Diode, Reservoir Cap)**:
   * Insert **Q1 (FQP30N06L)** into **Row C (Cols 18, 19, 20)** with metal tab facing North.
   * Insert **D1 (1N5819)** into **Row B (Cols 19 $\to$ 21)**. Cathode stripe faces Col 21.
   * Insert **C3 ($470\,\mu\text{F}$ or $100\,\mu\text{F}$)** into **Row D (Cols 21 $\to$ 23)**. Negative stripe faces Col 23.
   * On solder side:
     * Solder bridge **C19 $\to$ B19** (Q1 Drain to TB3 Pin 1 / D1 Anode).
     * Connect C3 (+) at D21 to TB3 (+) at B21.
     * Solder bridge **C17 $\to$ C18** (R5 to Gate).
     * Bend Q1 Source (C20) down to A20; bend C3 (-) (D23) down to A23.
6. **Insulated Signal Jumpers**:
   * Jumper from **F8 (GPIO 7)** to **C14 (R5 Left)**.
   * Straight jumper from **Q7 (TB2 Pin 1 Data)** to **L7 (GPIO 1 ADC)**.
   * Parallel jumper from **Q9 (TB2 Pin 2 VCC)** to **L8 (GPIO 2 Power)**.
   * Jumper from **Q17 (TB1 TRIG)** to **L9 (GPIO 3)**.
   * Jumper from **O17 (Echo 3.3V)** to **L10 (GPIO 4)**.

---

## 8. Pre-Power Cold Testing Checklist (Multimeter DMM)

**DO NOT INSERT THE ESP32-C3 MODULE YET.** Verify these tests with a DMM in resistance mode:

| Test ID | Multimeter Probes | Expected Value | What It Verifies |
| :---: | :--- | :---: | :--- |
| **T1** | Socket L3 (5V) to Socket L4 (GND) | **$> 10\,\text{k}\Omega$** (Rising) | **No power short!** C3 charging curve confirms clean 5V rail. |
| **T2** | Socket L4 (GND) to Socket L5 (3V3) | **Open circuit ($\infty$)** | No 3.3V rail shorts. |
| **T3** | Socket L4 (GND) to TB1 Pin 4, TB2 Pin 3 | **$< 0.5\,\Omega$** | Sensor ground continuity through Row P bus. |
| **T4** | Socket L4 (GND) to Q1 Source (C20) | **$< 0.5\,\Omega$** | Power ground continuity through Star Link. |
| **T5** | Socket L3 (5V) to TB1 Pin 1, TB3 Pin 2 | **$< 0.5\,\Omega$** | 5V power distribution continuity. |
| **T6** | Socket L6 (GPIO 0) to Socket L4 (GND) | **$\approx 10.0\,\text{k}\Omega$** | Exact R2 resistance to ground (via M4-L4 bridge). |
| **T7** | Socket L6 (GPIO 0) to Socket L3 (5V) | **$\approx 10.0\,\text{k}\Omega$** | Exact R1 resistance to 5V rail (via N6). |
| **T8** | Socket F8 (GPIO 7) to Socket L4 (GND) | **$\approx 101\,\text{k}\Omega$** | Series R5 ($1\,\text{k}\Omega$) + R6 ($100\,\text{k}\Omega$) pull-down path. |
| **T9** | TB1 Pin 3 (Echo) to Socket L10 (GPIO 4)| **$\approx 1.0\,\text{k}\Omega$** | Upper Echo resistor R3. |
| **T10** | Socket L10 (GPIO 4) to Socket L4 (GND)| **$\approx 2.0\,\text{k}\Omega$** | Lower Echo resistor R4 to ground. |
| **T11** | TB3 Pin 1 (Drain) to TB3 Pin 2 (+5V) | **Diode Drop ($\sim 0.35\text{V}$)** | Diode D1 polarity check (red probe on TB3 Pin 1, black on Pin 2). |
