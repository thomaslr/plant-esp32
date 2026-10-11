# ESP32-C3 SuperMini 15-Column Breadboard Guide (Top-Half Staggered Terminals)

This guide documents the physical build for your **15-column mini breadboard** with dual power rails (Top and Bottom), matching the exact board from your photo.

### Key Layout Highlights:
1. **Staggered Top-Half Terminal Blocks (Zero Column Overlap)**:
   * **TB1 (4-pin Ultrasonic)** sits in **Row J** across odd columns: **Cols 9, 11, 13, 15** (facing **UPWARDS** off the top edge).
   * **TB2 (3-pin Soil Sensor)** sits in **Row F** across even columns: **Cols 10, 12, 14** (facing **DOWNWARDS** towards the center trench).
   * Wires enter TB1 from the top edge, and enter TB2 from the open center trench—giving completely unobstructed screwdriver and wiring access to both blocks!
2. **Bottom-Half Isolation for Pump**:
   * **TB3 (2-pin DC Pump)** sits in **Row A** across **Cols 13, 15** (facing **DOWNWARDS** off the bottom edge).
   * The entire bottom half (Cols 8–10, 12) is left open and clean, isolating the pump power circuit from sensitive analog signals.
3. **Hardware Pin Assignments**:
   * **`GPIO 0` (Col 4 Top)**: 5V Supply Rail Monitor (10k/10k divider + 100nF filter cap).
   * **`GPIO 1` (Col 5 Top)**: Soil Moisture Analog In (`AOUT`).
   * **`GPIO 2` (Col 6 Top)**: Switched Soil Moisture Power (`VCC`) — duty-cycled 3.3V to eliminate 7mA idle draw and galvanic corrosion.
   * **`GPIO 3` (Col 7 Top)**: Ultrasonic `TRIG` Output.
   * **`GPIO 4` (Col 8 Top)**: Ultrasonic `ECHO` Input (via 1k/2k 5V-to-3.3V divider).
   * **`GPIO 7` (Col 6 Bottom)**: Pump MOSFET Gate (via 1k gate resistor and 100k pull-down).

The visual, interactive diagram is saved at:
👉 [circuit_diagram.html](file:///Users/robertlongbottom/dev/plant-esp32/circuit_diagram.html) (open in any web browser).

Previous layout versions are preserved in `circuit_diagram.backup.html` and `breadboard_circuit_guide.backup.md`.

---

## 1. Passive Components & Functional Reference

| Designator | Component Value | Location / Connects Between | Exact Circuit Function |
| :---: | :--- | :--- | :--- |
| **R1** | **$10\,\text{k}\Omega$ Resistor** | Top (+) 5V Rail $\to$ **Col 4 Row H** | **Voltage Divider Top Leg**: Steps down the 5V USB rail into the safe 0–3.1V range for ESP32 ADC1. |
| **R2** | **$10\,\text{k}\Omega$ Resistor** | **Col 4 Row G** $\to$ Top (-) GND Rail | **Voltage Divider Bottom Leg**: Creates the exact $1/2$ division ratio ($5.0\text{V} \to 2.50\text{V}$). |
| **C1** | **$100\,\text{nF}$ Ceramic ("104")** | **Col 4 Row I** $\to$ Top (-) GND Rail | **ADC Filter Capacitor**: Anti-aliasing and low-pass filter; stabilizes ADC1 sampling against power ripple. |
| **R3** | **$1\,\text{k}\Omega$ Resistor** | **Col 13 Row I** $\to$ **Col 8 Row I** | **Echo Stepper Upper Leg**: Limits current and acts as the top resistor in the Echo voltage divider. |
| **R4** | **$2\,\text{k}\Omega$ Resistor** | **Col 8 Row H** $\to$ Top (-) GND Rail | **Echo Stepper Lower Leg**: Steps the 5V Echo pulse down to safe $3.33\,\text{V}$ to protect `GPIO 4`. |
| **C2** | **$100\,\text{nF}$ Ceramic ("104")** | **Col 12 Row H** $\to$ **Col 14 Row H** | **Soil Sensor RF Filter**: Placed directly across AOUT and GND to absorb ESP32 Wi-Fi antenna RF noise. |
| **R5** | **$1\,\text{k}\Omega$ Resistor** | **Col 6 Row A** $\to$ **Col 11 Row C** | **Gate Damping Resistor**: Dampens high-frequency ringing and limits gate charging current from `GPIO 7`. |
| **R6** | **$100\,\text{k}\Omega$ Resistor** | **Col 11 Row D** $\to$ Bottom (-) GND Rail | **Gate Pull-Down Resistor**: Pulls Gate to 0V during bootloader/reset so the pump stays strictly OFF. |
| **Q1** | **FQP30N06L N-MOSFET** | **Rows C–D (Cols 11, 13, 14)** | **Pump Low-Side Power Switch**: Logic-level TO-220 MOSFET ($V_{DSS}=60\text{V}, I_D=32\text{A}, V_{GS(th)}\le 2.5\text{V}$). Pin 1 (Gate, Col 11), Pin 2 (Drain, Col 13), Pin 3 (Source, Col 14). Fully driven by 3.3V logic level without heating. |
| **D1** | **1N5819 Schottky Diode** | **Col 13 Row B** $\to$ **Col 15 Row B** | **Flyback Snubber Diode**: Clamps reverse inductive voltage spikes from pump motor coils when the MOSFET shuts off. |
| **C3** | **$470\,\mu\text{F}$ Electrolytic** | Top (+) 5V Rail $\to$ Top (-) GND Rail | **Bulk Reservoir Capacitor**: Buffers motor inrush current spikes ($>1\text{A}$) to prevent ESP32 brownouts. |

---

## 2. Power Rail Allocation

Bridge the top and bottom blue ground lines together (under the board) for a single solid ground plane:

| Rail | Voltage | Supply Source | Connects To |
| :--- | :---: | :--- | :--- |
| **Top Rail (+)** | **+5V (USB VBUS)** | ESP32 Pin 1 (Col 1 Top) | • TB1 Pin 1 Ultrasonic 5V (Col 9 Top)<br>• TB3 Pin 2 Pump +5V (Col 15 Bottom)<br>• Top leg of 10kΩ Supply Divider (R1)<br>• Bulk Capacitor C3 (+) |
| **Top Rail (-)** | **Common GND** | ESP32 Pin 2 (Col 2 Top) | • TB1 Pin 4 Ultrasonic GND (Col 15 Top)<br>• TB2 Pin 3 Soil GND (Col 14 Top)<br>• R2 (10kΩ) and C1 (100nF) ground legs<br>• R4 (2kΩ) Echo stepper ground leg<br>• Bulk Capacitor C3 (-) |
| **Bottom Rail (+)**| **+3.3V System (Aux)** | ESP32 Pin 3 (Col 3 Top) | Auxiliary 3.3V rail (unswitched) |
| **Bottom Rail (-)**| **Common GND** | Bridged to Top GND | • MOSFET Source (Col 14 Bottom)<br>• R6 (100kΩ) Gate pull-down resistor |

---

## 3. Complete 15-Column Breadboard Map

| Col | Upper Half (Rows F–J) | Lower Half (Rows A–E) | Function & Exact Connections |
| :---: | :--- | :--- | :--- |
| **1** | **ESP32 5V In (Row F)** | ESP32 GPIO 21 (Row B) | • Jumper from Col 1 Top into **Top (+) 5V Rail** |
| **2** | **ESP32 GND (Row F)** | ESP32 GPIO 20 (Row B) | • Jumper from Col 2 Top into **Top (-) GND Rail** |
| **3** | **ESP32 3V3 Out (Row F)**| ESP32 GPIO 10 (Row B) | • Jumper from Col 3 Top into **Bottom (+) 3.3V Rail** |
| **4** | **ESP32 GPIO 0 (Row F)** | ESP32 GPIO 9 (Row B) | • **5V Supply Monitor Tap**: Connected to R1 (10k), R2 (10k), and C1 (100nF) |
| **5** | **ESP32 GPIO 1 (Row F)** | ESP32 GPIO 8 (Row B) | • **Soil Moisture Analog In**: Jumper from Col 12 Row G (TB2 AOUT) |
| **6** | **ESP32 GPIO 2 (Row F)** | **ESP32 GPIO 7 (Row B)**| • **Switched Soil VCC (GPIO 2)**: Jumper from Col 6 Row G to Col 10 Row G (TB2 VCC)<br>• Row A on Col 6: R5 (1kΩ) Gate Resistor running to Col 11 Row C (Gate) |
| **7** | **ESP32 GPIO 3 (Row F)** | ESP32 GPIO 6 (Row B) | • Ultrasonic TRIG Out: Jumper running to Col 11 Top (TB1 TRIG) |
| **8** | **ESP32 GPIO 4 (Row F)** | *(Open spacer)* | • Top: R3 (1kΩ) from Col 13 Row I, R4 (2kΩ) to Top (-) GND Rail |
| **9** | **TB1 Pin 1: Ultrasonic 5V (Row J)** | *(Open spacer)* | • Top: Jumper from Col 9 into **Top (+) 5V Rail** |
| **10** | **TB2 Pin 1: Soil Switched VCC (Row F)** | *(Open spacer)* | • Top: Jumper from Col 10 Row G to Col 6 Row G (ESP32 GPIO 2 Switched Power) |
| **11** | **TB1 Pin 2: TRIG (Row J)** | **Q1 Gate: FQP30N06L Pin 1 (Rows C–D)** | • Top: Jumper from Col 7 Top (GPIO 3)<br>• Bottom: R5 (1kΩ) from G7, R6 (100kΩ) to GND |
| **12** | **TB2 Pin 2: Soil AOUT (Row F)** | *(Open spacer)* | • Top: Jumper from Col 12 Row G to Col 5 Row G (ESP32 GPIO 1)<br>• C2 (100nF) across Col 12 Row H &amp; Col 14 Row H |
| **13** | **TB1 Pin 3: ECHO (Row J)** | **TB3 Pin 1: Pump Drain (Row A)** &bull; **Q1 Drain: FQP30N06L Pin 2 (Rows C–D)** | • Top: R3 (1kΩ) from Col 13 Row I running to Col 8 Row I (GPIO 4)<br>• Bottom: **Direct internal share!** TB3 Drain and MOSFET Drain share Col 13 |
| **14** | **TB2 Pin 3: Soil GND (Row F)** | **Q1 Source: FQP30N06L Pin 3 (Rows C–D)** | • Top: Jumper from Col 14 Row G into **Top (-) GND Rail**<br>• Bottom: Jumper from Col 14 Row C into **Bottom (-) GND Rail** |
| **15** | **TB1 Pin 4: GND (Row J)** | **TB3 Pin 2: Pump +5V (Row A)** | • Top: Jumper into **Top (-) GND Rail**<br>• Bottom: Jumper to Top (+) 5V Rail<br>• D1 (1N5819) across Col 13 &amp; Col 15 in Row B |

---

## 4. Step-by-Step Assembly Checklist

1. **Bridge Grounds**: Connect the Top (-) Rail and Bottom (-) Rail under the board for unified zero-noise ground.
2. **Mount the ESP32-C3 SuperMini**:
   * Align top pins in **Row F, Cols 1–8** and bottom pins in **Row B, Cols 1–8** (USB-C facing left).
   * Col 1 Top (5V) $\to$ Top (+) 5V Rail.
   * Col 2 Top (GND) $\to$ Top (-) GND Rail.
   * Col 3 Top (3V3) $\to$ Bottom (+) 3.3V Rail.
3. **Mount the Terminal Blocks**:
   * **TB1 (4-pin Ultrasonic)** in **Row J** (Cols 9, 11, 13, 15) with wire entries facing **UPWARDS** (off top edge).
   * **TB2 (3-pin Soil)** in **Row F** (Cols 10, 12, 14) with wire entries facing **DOWNWARDS** (towards center trench).
   * **TB3 (2-pin Pump)** in **Row A** (Cols 13, 15) with wire entries facing **DOWNWARDS** (off bottom edge).
4. **Insert the Swapped 5V Supply Divider on Col 4 Top**:
   * **R1 (10kΩ)** plugs from Top (+) 5V Rail into Col 4 Row H.
   * **R2 (10kΩ)** plugs from Col 4 Row G into Top (-) GND Rail.
   * **C1 (100nF)** plugs from Col 4 Row I into Top (-) GND Rail.
5. **Connect Soil Sensor on Row F (Cols 10, 12, 14)**:
   * **Switched Power**: Run a short jumper wire from **Col 10 Row G (TB2 Pin 1 / Soil VCC)** across to **Col 6 Row G (ESP32 GPIO 2)**.
   * **Analog In**: Run a jumper wire from **Col 12 Row G (TB2 Pin 2 / Soil AOUT)** across to **Col 5 Row G (ESP32 GPIO 1)**.
   * **Ground**: Run a jumper wire from **Col 14 Row G (TB2 Pin 3 / Soil GND)** into the **Top (-) GND Rail**.
   * **RF Filter**: Insert **C2 (100nF)** across **Col 12 Row H** and **Col 14 Row H** (directly across AOUT &amp; GND).
6. **Connect Ultrasonic Sensor on Row J (Cols 9, 11, 13, 15)**:
   * Jumper Col 9 (5V) to Top (+) 5V Rail; Jumper Col 15 (GND) to Top (-) GND Rail.
   * Jumper Col 7 Top (GPIO 3) to Col 11 Row I (TB1 TRIG).
   * Insert **R3 (1kΩ)** from Col 13 Row I to Col 8 Row I.
   * Insert **R4 (2kΩ)** from Col 8 Row H into Top (-) GND Rail.
7. **Insert the Pump MOSFET (FQP30N06L) & Snubber in Bottom Half**:
   * Insert the **FQP30N06L TO-220 MOSFET** into Rows C–D (metal heatsink tab facing UPWARDS/inward toward the center trench, and front printed label facing DOWNWARDS toward Row A):
     * **Pin 1 (Gate)** into **Col 11 Row C**
     * **Pin 2 (Drain)** into **Col 13 Row C** (notice: Col 13 automatically connects Drain directly to TB3 Pin 1!)
     * **Pin 3 (Source)** into **Col 14 Row C**
   * Insert **D1 (1N5819)** across Col 13 Row B and Col 15 Row B (cathode stripe to Col 15 / +5V).
   * Connect **R5 (1kΩ)** from Col 6 Row A (GPIO 7) to Col 11 Row C (Gate).
   * Connect **R6 (100kΩ)** from Col 11 Row D into Bottom (-) GND Rail.
   * Jumper Col 14 Row C (Source) into Bottom (-) GND Rail; Jumper Col 15 (Pump 5V) to Top (+) 5V Rail.
   * Insert **C3 (470µF)** directly across Top (+) 5V and (-) GND rails near Col 15.

---

## 5. Technical Note: FQP30N06L Logic-Level MOSFET Performance

* **Why FQP30N06L?** The **"L"** suffix designates **Logic-Level Gate Drive**. Standard power MOSFETs (such as IRF540N or standard FQP30N06) have high threshold voltages ($V_{GS(th)} = 2\text{V}\text{--}4\text{V}$) and require $10\text{V}$ on the gate to fully turn on; driving them directly with a 3.3V microcontroller leaves them in the linear resistive region, causing severe pump voltage drop and excessive heat.
* **Low Threshold Voltage ($V_{GS(th)}$)**: $1.0\text{V}$ minimum, $2.5\text{V}$ maximum (typically $\sim 1.5\text{V}$). The ESP32-C3’s $3.3\text{V}$ digital output easily exceeds this threshold.
* **Ultra-Low On-Resistance ($R_{DS(on)}$)**: At $V_{GS} = 3.3\text{V}$, $R_{DS(on)} < 0.05\,\Omega$.
  * For our 5V submersible pump drawing $\approx 0.35\text{A}$, the voltage drop across the switch is $\Delta V = I \times R_{DS(on)} \approx 0.35\text{A} \times 0.05\,\Omega \approx 17.5\,\text{mV}$ (virtually zero loss; pump gets full 5V).
  * Power dissipated as heat: $P = I^2 \times R_{DS(on)} \approx (0.35)^2 \times 0.05 \approx 6.1\,\text{mW}$ (runs completely cold to the touch; no heatsink required).
* **Pinout (TO-220 Front View, legs pointing down)**:
  1. **Pin 1 (Left): Gate** $\to$ Col 11
  2. **Pin 2 (Center): Drain** $\to$ Col 13 (internally tied to metal tab)
  3. **Pin 3 (Right): Source** $\to$ Col 14
