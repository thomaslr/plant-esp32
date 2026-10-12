# Plant ESP32 Controller — Custom PCB Fabrication Package

This directory contains the production-ready KiCad 8/10 project, fabrication files (Gerbers & Drill), 3D visual renders, and ordering instructions for the **Plant ESP32 Automated Watering & Telemetry System**, specifically tailored for **KF301 Modular 5.00mm Pitch Screw Terminal Blocks**.

---

## 3D Board Previews

| Top View (Components & Silkscreen) | Isometric 3D View |
| :---: | :---: |
| ![Top View](pcb_render_top.png) | ![Isometric View](pcb_render_iso.png) |

| Bottom View (Solid Ground Plane Pour) |
| :---: |
| ![Bottom View](pcb_render_bottom.png) |

---

## Mechanical & Electrical Clearances (KF301 Verified)

The board layout has been explicitly configured for the mechanical envelope of **KF301-2P, KF301-3P, and KF301-4P** (or spliced 2P+2P) terminal blocks:
* **Pin Pitch**: Exactly **5.00 mm** (matching metric KF301 pin spacing).
* **Pad Hole Diameter**: **1.3 mm drill** (comfortably fits round or square KF301 solder pins).
* **Body Outline on Silkscreen**: Exactly **7.6 mm depth** ($-3.8\text{ mm}$ to $+3.8\text{ mm}$ relative to pin center).
* **Inter-Block Gaps**:
  * **2.5 mm physical gap** between TB2 (Soil) and TB1 (Ultrasonic) — plenty of room for interlocking side dovetails.
  * **2.0 mm physical gap** between TB1 (Ultrasonic) and TB3 (Pump).
* **Mounting Hole Clearances**:
  * **5.0 mm clearance** from TB2 outer edge to top-left M3 screw head (H1).
  * **6.5 mm clearance** from TB3 outer edge to top-right M3 screw head (H2).
  * Standoffs, washers, and screw heads will never touch or crowd the terminal housings.
* **Component Clearance Below Terminals**:
  * A clear **$4.5\text{ mm}$ corridor** exists below the plastic housing before reaching the nearest resistors/capacitors.
  * Fingers and small screwdrivers have comfortable access to terminal screws without risking damage to nearby components.
* **Pin Labels**:
  * Silkscreen labels (`[DATA]`, `[VCC]`, `[GND]`, `[5V]`, `[TRIG]`, `[ECHO]`, `[GND]`, `[+]`, `[-]`) are printed outside the housing footprint so they remain visible when wires are screwed down.

---

## PCB Specifications

| Parameter | Specification | Fabricator Default? |
| :--- | :--- | :--- |
| **Dimensions** | **74.0 mm × 54.0 mm** | Custom (auto-detected from Edge.Cuts) |
| **Layer Count** | **2 Layers** (Top: Signals & Power Bus, Bottom: Solid Ground Plane) | Yes |
| **Board Thickness** | **1.6 mm** | Yes (Standard) |
| **Base Material** | FR-4 Standard (Tg 130–140°C) | Yes |
| **Copper Weight** | **1 oz** (35 µm) | Yes |
| **Surface Finish** | **HASL Lead-Free** (or Lead-Free OSP / ENIG) | Recommended: HASL Lead-Free |
| **Solder Mask** | Any color (Green, Matte Black, Blue, Purple) | Yes |
| **Silkscreen** | White | Yes |
| **Mounting Holes** | **4× M3** (3.2 mm drill, 5.0 mm annular ring, grounded) | Yes |
| **Design Rule Check** | **0 DRC Violations • 0 Unconnected Nets** | 100% Clean |

---

## Bill of Materials (BOM) — 100% Through-Hole (THT)

| Designator | Description / Value | Footprint / Pitch | Recommended Part / Notes |
| :--- | :--- | :--- | :--- |
| **ESP32** | ESP32-C3 SuperMini module | 2× 1x8 Female Pin Header (2.54 mm pitch, 15.24 mm spacing) | ESP32-C3 SuperMini |
| **TB1** | 4-Pin Screw Terminal Block | **KF301-4P** (or 2× KF301-2P spliced), 5.00 mm pitch | Ultrasonic Sensor (5V, TRIG, ECHO, GND) |
| **TB2** | 3-Pin Screw Terminal Block | **KF301-3P**, 5.00 mm pitch | Soil Moisture Sensor (DATA, VCC, GND) |
| **TB3** | 2-Pin Screw Terminal Block | **KF301-2P**, 5.00 mm pitch | 5V Submersible Pump (+5V, Drain -) |
| **Q1** | N-Channel Power MOSFET | TO-220-3 Vertical | **FQP30N06L** / IRLZ44N (Logic-Level, $V_{GS(th)} \le 2.5\text{V}$) |
| **D1** | Schottky Flyback Diode | DO-41 Horizontal (7.62 mm pitch) | **1N5819** (1A 40V Schottky) |
| **R1, R2** | 10 kΩ Resistor (1/4W) | Axial DIN0207 (7.62 mm pitch) | Supply Voltage Divider ($V_{in} \to$ GPIO0) |
| **R3** | 1 kΩ Resistor (1/4W) | Axial DIN0207 (7.62 mm pitch) | Echo Divider High-Side (Echo 5V $\to$ GPIO4) |
| **R4** | 2 kΩ Resistor (1/4W) | Axial DIN0207 (7.62 mm pitch) | Echo Divider Low-Side (GPIO4 $\to$ GND) |
| **R5** | 1 kΩ Resistor (1/4W) | Axial DIN0207 (7.62 mm pitch) | Gate In-Rush Current Limiter (GPIO7 $\to$ Gate) |
| **R6** | 100 kΩ Resistor (1/4W) | Axial DIN0207 (7.62 mm pitch) | Gate Pull-Down Bleeder (Gate $\to$ GND) |
| **C1** | 100 nF (0.1 µF) Ceramic Capacitor | Disc 2.5 mm pitch | ADC Smoothing Filter |
| **C2** | 100 nF (0.1 µF) Ceramic Capacitor | Disc 2.5 mm pitch | Soil Sensor Analog RF Noise Filter |
| **C3** | 100 µF Electrolytic Capacitor (16V–25V) | Radial D6.3 mm, 2.5 mm pitch | 5V Power Rail Transient Bulk Decoupling |
| **H1–H4** | 4× M3 Standoffs / Screws | 3.2 mm hole | M3 nylon or brass PCB standoffs (10–15 mm) |

---

## Terminal Block Wiring Reference

The silkscreen below each terminal block is labeled clearly:

```
[ Top Edge of PCB ]
  TB2 (Soil Moisture)          TB1 (Ultrasonic)           TB3 (Pump)
   [DATA] [VCC] [GND]       [5V] [TRIG] [ECHO] [GND]       [ + ] [ - ]
```

* **TB2 (Soil Sensor)**:
  * `DATA` $\to$ Sensor Analog Out (GPIO1)
  * `VCC` $\to$ Sensor VCC (GPIO2 switched power)
  * `GND` $\to$ Sensor GND
* **TB1 (Ultrasonic Sensor - HC-SR04 / RCWL-1601)**:
  * `5V` $\to$ Sensor VCC
  * `TRIG` $\to$ Sensor Trig (GPIO3)
  * `ECHO` $\to$ Sensor Echo (GPIO4 via 1k/2k level-divider)
  * `GND` $\to$ Sensor GND
* **TB3 (Submersible Water Pump)**:
  * `+` $\to$ Pump Positive lead (connected to +5V)
  * `-` $\to$ Pump Negative lead (switched to GND via MOSFET Drain)

---

## How to Order the PCB

### Ordering via JLCPCB (Recommended — 2 to 4 Day Delivery to Singapore / SEA)
1. Go to **[jlcpcb.com](https://jlcpcb.com/)** and click **"Add Gerber file"**.
2. Upload:
   ```
   pcb/plant-esp32_gerbers.zip
   ```
3. JLCPCB will automatically detect the dimensions (**74.0 mm × 54.0 mm**) and render a live 2D/3D board preview.
4. Verify standard options:
   * **Base Material**: FR-4
   * **Layers**: 2
   * **Dimensions**: 74 mm × 54 mm (auto-detected)
   * **PCB Qty**: 5 (minimum order)
   * **Thickness**: 1.6 mm
   * **Surface Finish**: **HASL Lead-Free**
   * **Copper Weight**: 1 oz
5. Total cost: **US \$2.00** for 5 boards (+ ~\$3–\$5 shipping to Singapore).

---

## Assembly / Soldering Sequence

1. **Step 1: Resistors (R1, R2, R3, R4, R5, R6)**
   * R1 & R2: 10 kΩ (Brown-Black-Orange)
   * R3 & R5: 1 kΩ (Brown-Black-Red)
   * R4: 2 kΩ (Red-Black-Red)
   * R6: 100 kΩ (Brown-Black-Yellow)
2. **Step 2: Schottky Diode (D1)**
   * Match the **white cathode band** to the stripe indicated on the silkscreen (facing Pin 1 / left).
3. **Step 3: Ceramic Capacitors (C1, C2)**
   * Non-polarized 100 nF disc capacitors.
4. **Step 4: ESP32 Pin Headers (J1, J2)**
   * 2× 1x8 female pin socket strips.
5. **Step 5: MOSFET (Q1)**
   * Metal tab faces top edge (matching the thick silkscreen line).
6. **Step 6: Bulk Electrolytic Capacitor (C3)**
   * Long lead is (+), white striped stripe is GND (-).
7. **Step 7: KF301 Terminal Blocks (TB1, TB2, TB3)**
   * Insert with the wire entry openings facing outward towards the top edge. Solder the sturdy 1.3 mm pins.
8. **Step 8: Stand-offs**
   * Fasten four M3 standoffs into H1–H4.
