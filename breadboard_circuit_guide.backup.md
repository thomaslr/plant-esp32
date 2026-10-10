# ESP32-C3 SuperMini Quarter-Size Breadboard Guide (Cols 1–18 + Power Rails)

This updated guide incorporates the **two sets of power rails (Top and Bottom)** along the sides of your 18-column quarter-size breadboard. Having dedicated power rails frees up columns, eliminates jumper clutter, and provides dedicated low-noise power distribution.

The interactive visual diagram is available at:
👉 [circuit_diagram.html](file:///Users/robertlongbottom/dev/plant-esp32/circuit_diagram.html) (open in any web browser).

---

## 1. Power Rail Allocation

Connecting the GND rails together (e.g., under the board with a jumper wire) ensures a single solid ground plane across all components:

| Rail | Color Stripe | Assigned Voltage | Fed From | Powers |
| :--- | :---: | :--- | :--- | :--- |
| **Top Rail (+)** | Red | **+5V (USB VBUS)** | ESP32 Pin 1 (5V) | • HC-SR04 Ultrasonic Sensor (TB1)<br>• Submersible DC Pump (TB3)<br>• Top leg of 10kΩ Supply Divider |
| **Top Rail (-)** | Blue | **Common GND** | ESP32 Pin 2 (GND) | • TB1 Ultrasonic GND<br>• TB2 Soil Sensor GND<br>• 10kΩ Divider & 100nF filter ground<br>• 2kΩ Echo Stepper ground |
| **Bottom Rail (+)**| Red/Gold | **+3.3V System** | ESP32 Pin 3 (3V3) | • Capacitive Soil Moisture Sensor (TB2) |
| **Bottom Rail (-)**| Blue | **Common GND** | Bridged to Top GND | • Logic-Level N-FET Source<br>• 100kΩ Gate pull-down resistor |

---

## 2. Master Breadboard Column Map (Cols 1–18)

| Col | Top Section (Rows A–E) | Bottom Section (Rows F–J) | Connections & Conditioning |
| :---: | :--- | :--- | :--- |
| **1** | **ESP32 5V In (Row E)** | ESP32 GPIO 21 (Row F) | • Jumper directly into **Top (+) 5V Rail** |
| **2** | **ESP32 GND (Row E)** | ESP32 GPIO 20 (Row F) | • Jumper directly into **Top (-) GND Rail** |
| **3** | **ESP32 3V3 Out (Row E)**| ESP32 GPIO 10 (Row F) | • Jumper directly into **Bottom (+) 3.3V Rail**<br>• Row F: E-Stop Button |
| **4** | **ESP32 GPIO 0 (Row E)** | ESP32 GPIO 9 (Row F) | • Soil Moisture Analog In (Jumper from Col 15) |
| **5** | **ESP32 GPIO 1 (Row E)** | ESP32 GPIO 8 (Row F) | • 10kΩ resistor runs directly into **Top (+) 5V Rail**<br>• 10kΩ resistor runs directly into **Top (-) GND Rail**<br>• 100nF cap runs directly into **Top (-) GND Rail** |
| **6** | **ESP32 GPIO 2 (Row E)** | **ESP32 GPIO 7 (Row F)** | • Bottom Row: 1kΩ Gate Resistor to Col 16 Bottom |
| **7** | **ESP32 GPIO 3 (Row E)** | ESP32 GPIO 6 (Row F) | • HC-SR04 TRIG Out (Jumper to Col 11) |
| **8** | **ESP32 GPIO 4 (Row E)** | ESP32 GPIO 5 (Row F) | • 1kΩ resistor from Col 12 (TB1 Echo)<br>• 2kΩ resistor directly into **Top (-) GND Rail** |
| **9** | *(Open)* | *(Open)* | Open spacer column |
| **10** | **TB1 Pin 1: Ultrasonic 5V** | *(Open)* | • Jumper directly into **Top (+) 5V Rail** |
| **11** | **TB1 Pin 2: TRIG** | *(Open)* | • Jumper from Col 7 Top (GPIO 3) |
| **12** | **TB1 Pin 3: ECHO (5V)** | *(Open)* | • 1kΩ resistor runs from here to Col 8 Top (GPIO 4) |
| **13** | **TB1 Pin 4: GND** | *(Open)* | • Jumper directly into **Top (-) GND Rail** |
| **14** | **TB2 Pin 1: Soil 3.3V** | *(Open)* | • Jumper directly into **Bottom (+) 3.3V Rail** |
| **15** | **TB2 Pin 2: Soil AOUT** | *(Open)* | • Jumper to Col 4 Top (GPIO 0)<br>• 100nF filter cap across Col 15 & Col 16 |
| **16** | **TB2 Pin 3: Soil GND** | **Pump N-FET: Pin 1 (GATE)** | • Top: Jumper directly into **Top (-) GND Rail**<br>• Bottom: 1kΩ from G7, 100kΩ to **Bottom (-) GND Rail** |
| **17** | **TB3 Pin 1: Pump (+5V)** | **Pump N-FET: Pin 2 (DRAIN)**| • Top: Jumper directly into **Top (+) 5V Rail**<br>• Bottom: N-FET Drain jumpered to Col 18 Top |
| **18** | **TB3 Pin 2: Pump (DRAIN)**| **Pump N-FET: Pin 3 (SOURCE)**| • Top: Return from pump<br>• Bottom: Source jumpered into **Bottom (-) GND Rail**<br>• 1N5819 diode across Col 17 & Col 18 Top |

---

## 3. Direct Rail-Connected Signal Conditioning

Having power rails eliminates complex daisy-chaining:

1. **5V Voltage Divider on Col 5**:
   * One 10kΩ resistor plugs from **Col 5** straight into the **Top (+) 5V Rail**.
   * One 10kΩ resistor plugs from **Col 5** straight into the **Top (-) GND Rail**.
   * One 100nF ceramic cap plugs from **Col 5** straight into the **Top (-) GND Rail**.
   * *Zero jumper wires required!*
2. **Echo Stepper on Col 8**:
   * The 1kΩ resistor spans from **Col 12 (TB1 Echo)** to **Col 8 (GPIO 4)**.
   * The 2kΩ resistor plugs from **Col 8** straight into the **Top (-) GND Rail**.
3. **Pump Driver on Cols 16–18 Bottom**:
   * **Gate (Col 16)**: 100kΩ resistor plugs straight into the **Bottom (-) GND Rail**.
   * **Source (Col 18)**: Plugs straight into the **Bottom (-) GND Rail**.
4. **Bulk 470µF Decoupling Capacitor**:
   * Plugs directly into the **Top (+) 5V Rail** and **Top (-) GND Rail** right beside Terminal Block 3 (Pump).

---

## 4. Assembly Order

1. **Bridge Grounds**:
   * Run a jumper wire between the Top (-) Rail and Bottom (-) Rail (under or around the breadboard edge) to establish a common ground.
2. **Mount the ESP32-C3 SuperMini**:
   * Insert across the center trough in **Cols 1 to 8**.
   * Jumper Pin 1 (Col 1) to Top (+) 5V Rail.
   * Jumper Pin 2 (Col 2) to Top (-) GND Rail.
   * Jumper Pin 3 (Col 3) to Bottom (+) 3.3V Rail.
3. **Mount Terminal Blocks (Row A)**:
   * **TB1 (Ultrasonic)** into Cols 10, 11, 12, 13.
   * **TB2 (Soil)** into Cols 14, 15, 16.
   * **TB3 (Pump)** into Cols 17, 18.
4. **Connect Terminal Power**:
   * TB1 Pin 1 (Col 10) $\to$ Top (+) 5V Rail.
   * TB1 Pin 4 (Col 13) $\to$ Top (-) GND Rail.
   * TB2 Pin 1 (Col 14) $\to$ Bottom (+) 3.3V Rail.
   * TB2 Pin 3 (Col 16) $\to$ Top (-) GND Rail.
   * TB3 Pin 1 (Col 17) $\to$ Top (+) 5V Rail.
5. **Drop in the Passive Components**:
   * Col 5 resistors & 100nF cap directly to Top rails.
   * Col 12 to Col 8 1kΩ resistor; Col 8 2kΩ resistor to Top (-) GND Rail.
   * 100nF capacitor across Col 15 and Col 16.
   * 1N5819 diode across Col 17 and Col 18 (Cathode/stripe in Col 17).
   * 470µF bulk capacitor directly across Top (+) and (-) rails.
6. **Insert Pump N-FET (Cols 16–18 Bottom)**:
   * Gate (Col 16): 1kΩ from Col 6 Bottom (GPIO 7), 100kΩ to Bottom (-) GND Rail.
   * Drain (Col 17): Jumper up to Col 18 Top (TB3 Pin 2).
   * Source (Col 18): Jumper to Bottom (-) GND Rail.
7. **Signal Lines**:
   * Col 7 Top (GPIO 3 TRIG) to Col 11 Top (TB1 TRIG).
   * Col 15 Top (TB2 AOUT) to Col 4 Top (GPIO 0).
