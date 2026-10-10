# ESP32-C3 SuperMini 15-Column Breadboard Guide (All-Outward Terminals & Swapped G0/G1)

This guide documents the physical build for your **15-column mini breadboard** with dual power rails (Top and Bottom), matching the exact board from your photo.

In this layout:
1. **G0 and G1 Swapped**:
   * **`GPIO 0` (Col 4)** measures the **5V Supply Monitor** via the $10\,\text{k}\Omega / 10\,\text{k}\Omega$ divider and $100\,\text{nF}$ filter cap.
   * **`GPIO 1` (Col 5)** measures the **Soil Moisture Sensor** analog signal (`AOUT`).
2. **100% Outward-Facing Terminals**:
   * **TB1 (4-pin Ultrasonic)** sits in **Row J** facing **UPWARDS** off the top edge.
   * **TB2 (3-pin Soil Sensor)** sits in **Row A** facing **DOWNWARDS** off the bottom edge.
   * **TB3 (2-pin DC Pump)** sits in **Row A** facing **DOWNWARDS** off the bottom edge.
3. **Direct Component Leads**: Every resistor, capacitor, diode, and transistor has explicit pin-to-pin connections with zero floating leads.

The visual, interactive diagram is saved at:
👉 [circuit_diagram.html](file:///Users/robertlongbottom/dev/plant-esp32/circuit_diagram.html) (open in any web browser).

Previous versions are preserved in `circuit_diagram.backup.html` and `breadboard_circuit_guide.backup.md`.

---

## 1. Passive Components & Functional Reference

| Designator | Component Value | Location / Connects Between | Exact Circuit Function |
| :---: | :--- | :--- | :--- |
| **R1** | **$10\,\text{k}\Omega$ Resistor** | Top (+) 5V Rail $\to$ **Col 4 Row H** | **Voltage Divider Top Leg**: Steps down the 5V USB rail into the safe 0–3.1V range for ESP32 ADC1. |
| **R2** | **$10\,\text{k}\Omega$ Resistor** | **Col 4 Row G** $\to$ Top (-) GND Rail | **Voltage Divider Bottom Leg**: Creates the exact $1/2$ division ratio ($5.0\text{V} \to 2.50\text{V}$). |
| **C1** | **$100\,\text{nF}$ Ceramic ("104")** | **Col 4 Row I** $\to$ Top (-) GND Rail | **ADC Filter Capacitor**: Anti-aliasing and low-pass filter; stabilizes ADC1 sampling against power ripple. |
| **R3** | **$1\,\text{k}\Omega$ Resistor** | **Col 13 Row I** $\to$ **Col 8 Row I** | **Echo Stepper Upper Leg**: Limits current and acts as the top resistor in the Echo voltage divider. |
| **R4** | **$2\,\text{k}\Omega$ Resistor** | **Col 8 Row H** $\to$ Top (-) GND Rail | **Echo Stepper Lower Leg**: Steps the 5V Echo pulse down to safe $3.33\,\text{V}$ to protect `GPIO 4`. |
| **C2** | **$100\,\text{nF}$ Ceramic ("104")** | **Col 10 Row C** $\to$ **Col 12 Row C** | **Soil Sensor RF Filter**: Placed directly across AOUT and GND to absorb ESP32 Wi-Fi antenna RF noise. |
| **R5** | **$1\,\text{k}\Omega$ Resistor** | **Col 6 Row A** $\to$ **Col 11 Row C** | **Gate Damping Resistor**: Dampens high-frequency ringing and limits gate charging current from `GPIO 7`. |
| **R6** | **$100\,\text{k}\Omega$ Resistor** | **Col 11 Row D** $\to$ Bottom (-) GND Rail | **Gate Pull-Down Resistor**: Pulls Gate to 0V during bootloader/reset so the pump stays strictly OFF. |
| **D1** | **1N5819 Schottky Diode** | **Col 13 Row B** $\to$ **Col 15 Row B** | **Flyback Snubber Diode**: Clamps inductive voltage spikes from pump motor coils when the MOSFET shuts off. |
| **C3** | **$470\,\mu\text{F}$ Electrolytic** | Top (+) 5V Rail $\to$ Top (-) GND Rail | **Bulk Reservoir Capacitor**: Buffers motor inrush current spikes ($>1\text{A}$) to prevent ESP32 brownouts. |

---

## 2. Power Rail Allocation

Bridge the top and bottom blue ground lines together (under the board) for a single solid ground plane:

| Rail | Voltage | Supply Source | Connects To |
| :--- | :---: | :--- | :--- |
| **Top Rail (+)** | **+5V (USB VBUS)** | ESP32 Pin 1 (Col 1 Top) | • TB1 Pin 1 Ultrasonic 5V (Col 9 Top)<br>• TB3 Pin 2 Pump +5V (Col 15 Bottom)<br>• Top leg of 10kΩ Supply Divider (R1)<br>• Bulk Capacitor C3 (+) |
| **Top Rail (-)** | **Common GND** | ESP32 Pin 2 (Col 2 Top) | • TB1 Pin 4 Ultrasonic GND (Col 15 Top)<br>• R2 (10kΩ) and C1 (100nF) ground legs<br>• R4 (2kΩ) Echo stepper ground leg<br>• Bulk Capacitor C3 (-) |
| **Bottom Rail (+)**| **+3.3V System** | ESP32 Pin 3 (Col 3 Top) | • TB2 Pin 1 Soil Sensor 3.3V (Col 8 Bottom) |
| **Bottom Rail (-)**| **Common GND** | Bridged to Top GND | • TB2 Pin 3 Soil Sensor GND (Col 12 Bottom)<br>• MOSFET Source (Col 14 Bottom)<br>• R6 (100kΩ) Gate pull-down resistor |

---

## 3. Complete 15-Column Breadboard Map

| Col | Upper Half (Rows F–J) | Lower Half (Rows A–E) | Function & Exact Connections |
| :---: | :--- | :--- | :--- |
| **1** | **ESP32 5V In (Row F)** | ESP32 GPIO 21 (Row B) | • Jumper from Col 1 Top into **Top (+) 5V Rail** |
| **2** | **ESP32 GND (Row F)** | ESP32 GPIO 20 (Row B) | • Jumper from Col 2 Top into **Top (-) GND Rail** |
| **3** | **ESP32 3V3 Out (Row F)**| ESP32 GPIO 10 (Row B) | • Jumper from Col 3 Top into **Bottom (+) 3.3V Rail** |
| **4** | **ESP32 GPIO 0 (Row F)** | ESP32 GPIO 9 (Row B) | • **5V Supply Monitor Tap**: Connected to R1 (10k), R2 (10k), and C1 (100nF) |
| **5** | **ESP32 GPIO 1 (Row F)** | ESP32 GPIO 8 (Row B) | • **Soil Moisture Analog In**: Jumper from Col 10 Bottom (TB2 AOUT) |
| **6** | **ESP32 GPIO 2 (Row F)** | **ESP32 GPIO 7 (Row B)**| • Row A on Col 6: R5 (1kΩ) Gate Resistor running to Col 11 Row C (Gate) |
| **7** | **ESP32 GPIO 3 (Row F)** | ESP32 GPIO 6 (Row B) | • Ultrasonic TRIG Out: Jumper running to Col 11 Top (TB1 TRIG) |
| **8** | **ESP32 GPIO 4 (Row F)** | **TB2 Pin 1: Soil 3.3V (Row A)**| • Top: 1kΩ from Col 13, 2kΩ to Top GND<br>• Bottom: Jumper to Bottom (+) 3.3V Rail |
| **9** | **TB1 Pin 1: Ultrasonic 5V (Row J)** | *(Open spacer)* | • Top: Jumper into **Top (+) 5V Rail** |
| **10** | *(Open spacer)* | **TB2 Pin 2: Soil AOUT (Row A)** | • Bottom: Jumper to Col 5 Top (GPIO 1)<br>• C2 (100nF) across Col 10 &amp; Col 12 |
| **11** | **TB1 Pin 2: TRIG (Row J)** | **MOSFET Gate (Rows C–D)** | • Top: Jumper from Col 7 Top (GPIO 3)<br>• Bottom: R5 (1kΩ) from G7, R6 (100kΩ) to GND |
| **12** | *(Open spacer)* | **TB2 Pin 3: Soil GND (Row A)** | • Bottom: Jumper to Bottom (-) GND Rail |
| **13** | **TB1 Pin 3: ECHO (Row J)** | **TB3 Pin 1: Pump Drain (Row A)** &bull; **MOSFET Drain (Rows C–D)** | • Top: R3 (1kΩ) running to Col 8 Top (GPIO 4)<br>• Bottom: **Direct internal share!** TB3 Drain and MOSFET Drain share Col 13 |
| **14** | *(Open spacer)* | **MOSFET Source (Rows C–D)** | • Bottom: Jumper to Bottom (-) GND Rail |
| **15** | **TB1 Pin 4: GND (Row J)** | **TB3 Pin 2: Pump +5V (Row A)** | • Top: Jumper into **Top (-) GND Rail**<br>• Bottom: Jumper to Top (+) 5V Rail<br>• D1 (1N5819) across Col 13 &amp; Col 15 |

---

## 4. Step-by-Step Assembly Checklist

1. **Bridge Grounds**: Connect the Top (-) Rail and Bottom (-) Rail under the board for unified zero-noise ground.
2. **Mount the ESP32-C3 SuperMini**:
   * Align top pins in **Row F, Cols 1–8** and bottom pins in **Row B, Cols 1–8** (USB-C facing left).
   * Col 1 Top (5V) $\to$ Top (+) 5V Rail.
   * Col 2 Top (GND) $\to$ Top (-) GND Rail.
   * Col 3 Top (3V3) $\to$ Bottom (+) 3.3V Rail.
3. **Mount the Terminal Blocks (100% Outward Facing)**:
   * **TB1 (4-pin Ultrasonic)** in **Row J** (Cols 9, 11, 13, 15) with wire entries facing **UPWARDS**.
   * **TB2 (3-pin Soil)** in **Row A** (Cols 8, 10, 12) with wire entries facing **DOWNWARDS**.
   * **TB3 (2-pin Pump)** in **Row A** (Cols 13, 15) with wire entries facing **DOWNWARDS**.
4. **Insert the Swapped 5V Supply Divider on Col 4 Top**:
   * **R1 (10kΩ)** plugs from Top (+) 5V Rail into Col 4 Row H.
   * **R2 (10kΩ)** plugs from Col 4 Row G into Top (-) GND Rail.
   * **C1 (100nF)** plugs from Col 4 Row I into Top (-) GND Rail.
5. **Connect Soil Sensor on Col 5 Top (GPIO 1)**:
   * Run a jumper wire from **Col 10 Row A (TB2 AOUT)** across to **Col 5 Row F (GPIO 1)**.
   * Insert **C2 (100nF)** across Col 10 Row C and Col 12 Row C (directly across AOUT & GND).
   * Jumper TB2 Pin 1 (Col 8) to Bottom (+) 3.3V Rail; Jumper TB2 Pin 3 (Col 12) to Bottom (-) GND Rail.
6. **Connect Ultrasonic Sensor on Top Edge**:
   * Jumper Col 9 (5V) to Top (+) 5V Rail; Jumper Col 15 (GND) to Top (-) GND Rail.
   * Jumper Col 7 Top (GPIO 3) to Col 11 Top (TB1 TRIG).
   * Insert **R3 (1kΩ)** from Col 13 Row I to Col 8 Row I.
   * Insert **R4 (2kΩ)** from Col 8 Row H into Top (-) GND Rail.
7. **Insert the Pump MOSFET & Snubber**:
   * Insert TO-220 MOSFET into Rows C–D: Gate in Col 11, Drain in Col 13, Source in Col 14.
   * Notice: Col 13 automatically connects the MOSFET Drain to TB3 Pin 1!
   * Insert **D1 (1N5819)** across Col 13 Row B and Col 15 Row B (stripe to Col 15).
   * Connect **R5 (1kΩ)** from Col 6 Row A (GPIO 7) to Col 11 Row C (Gate).
   * Connect **R6 (100kΩ)** from Col 11 Row D into Bottom (-) GND Rail.
   * Jumper Col 14 (Source) to Bottom (-) GND Rail; Jumper Col 15 (Pump 5V) to Top (+) 5V Rail.
   * Insert **C3 (470µF)** directly across Top (+) 5V and (-) GND rails near Col 15.
