// =============================================================================
// Plant ESP32 Controller — Open-Frame Tank-Mount Sled (Single-Piece Base)
// Compatible with: plant-esp32-smd.kicad_pcb (74.0 x 54.0 mm)
// =============================================================================
// Features:
// 1. Single-piece open frame: Electronics 100% exposed for instant access,
//    direct wiring, USB-C programming, and full LED visibility.
// 2. 2.0 mm Standoffs: 4x corner M3 bosses with 2.0 mm elevation above plate.
// 3. Silicone Adhesion Underside Pattern:
//    - 2.0 mm wide x 1.0 mm deep cross-hatch waffle channels for even spread.
//    - Ø3.2 mm anchor wells at grid intersections for mechanical interlocking.
//    - Perimeter silicone squeeze-out rivet holes to lock cured silicone.
//    - Optimized for sticking firmly to curved or flat plastic water tanks.
// =============================================================================

/* [Render Mode] */
view_mode = "model"; // [model: 3D Base Sled Only (Printable), preview: Base + PCB Preview, underside: Undersurface Adhesive Pattern Inspection]

/* [PCB & Standoff Parameters] */
pcb_w        = 74.0;  // PCB width (X) mm
pcb_h        = 54.0;  // PCB height (Y) mm
pcb_thick    = 1.6;   // FR4 thickness
hole_pitch_x = 67.0;  // KiCad M3 hole pitch X (mm)
hole_pitch_y = 47.0;  // KiCad M3 hole pitch Y (mm)
hole_d       = 3.2;   // Mounting hole diameter

/* [Base Plate Dimensions] */
plate_floor  = 2.5;   // Plate floor thickness (mm)
standoff_h   = 2.0;   // Standoff height (mm) — 2.0mm as requested
standoff_od  = 7.2;   // Standoff boss outer diameter (mm)
standoff_id  = 2.9;   // Pilot hole for M3 self-tapping screws or M3 heat-set inserts (mm)

margin       = 2.0;   // Perimeter margin around PCB (mm)
plate_w      = pcb_w + 2 * margin; // 78.0 mm
plate_h      = pcb_h + 2 * margin; // 58.0 mm
corner_r     = 4.0;   // Outer corner radius (mm)

/* [Silicone Gel Adhesion Pattern] */
channel_w    = 2.0;   // Width of glue channels (mm)
channel_d    = 1.0;   // Depth of glue channels (mm)
grid_pitch   = 6.0;   // Grid spacing (mm)

$fn = 36;

module rounded_rect_2d(w, h, r) {
    offset(r = r) offset(delta = -r) square([w, h], center = true);
}

// =============================================================================
// MAIN OPEN-FRAME SLED
// =============================================================================
module open_base_sled() {
    difference() {
        union() {
            // Main Rigid Plate Floor
            linear_extrude(height = plate_floor)
                rounded_rect_2d(plate_w, plate_h, corner_r);
            
            // 4x Standoff Bosses (2.0 mm height above floor)
            for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
                for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                    translate([sx, sy, plate_floor]) {
                        cylinder(d = standoff_od, h = standoff_h);
                        // Conical base reinforcement for layer-shear resistance
                        cylinder(d1 = standoff_od + 2.4, d2 = standoff_od, h = 1.0);
                    }
                }
            }

            // Low Corner Alignment Brackets (1.5 mm guide lips to position PCB easily)
            for (cx = [-pcb_w/2, pcb_w/2]) {
                for (cy = [-pcb_h/2, pcb_h/2]) {
                    translate([cx, cy, plate_floor]) {
                        difference() {
                            cylinder(r = 5.0, h = standoff_h + 1.2);
                            translate([cx > 0 ? -10 : 0, cy > 0 ? -10 : 0, -1])
                                cube([10, 10, standoff_h + 4]);
                        }
                    }
                }
            }
        }

        // 4x M3 Screw Pilot Holes through standoffs and plate
        for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
            for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                translate([sx, sy, -1])
                    cylinder(d = standoff_id, h = plate_floor + standoff_h + 2);
            }
        }

        // Center Weight-Reduction & Airflow Openings
        for (vx = [-18, 0, 18]) {
            translate([vx, 0, -1])
                linear_extrude(height = plate_floor + 2)
                    rounded_rect_2d(8.0, 30.0, 3.0);
        }

        // =====================================================================
        // SILICONE ADHESION PATTERN (UNDERSIDE Z=0)
        // =====================================================================
        // X-axis waffle channels
        for (x = [-plate_w/2 + 5 : grid_pitch : plate_w/2 - 5]) {
            translate([x - channel_w/2, -plate_h/2 + 3, -0.05])
                cube([channel_w, plate_h - 6, channel_d + 0.05]);
        }
        // Y-axis waffle channels
        for (y = [-plate_h/2 + 5 : grid_pitch : plate_h/2 - 5]) {
            translate([-plate_w/2 + 3, y - channel_w/2, -0.05])
                cube([plate_w - 6, channel_w, channel_d + 0.05]);
        }
        // Anchor wells at intersections for mechanical keying / suction
        for (x = [-plate_w/2 + 5 : grid_pitch : plate_w/2 - 5]) {
            for (y = [-plate_h/2 + 5 : grid_pitch : plate_h/2 - 5]) {
                translate([x, y, -0.05])
                    cylinder(d = 3.2, h = channel_d + 0.15, $fn = 16);
            }
        }
        // Perimeter rivet thru-holes (silicone mushrooms through to lock physically)
        for (rx = [-plate_w/2 + 3.2, plate_w/2 - 3.2]) {
            for (ry = [-plate_h/2 + 10, 0, plate_h/2 - 10]) {
                translate([rx, ry, -1])
                    cylinder(d = 2.8, h = plate_floor + 2, $fn = 16);
            }
        }
    }
}

// =============================================================================
// PCB & COMPONENT MOCKUP (FOR PREVIEW)
// =============================================================================
module pcb_mockup() {
    translate([0, 0, plate_floor + standoff_h]) {
        // Green PCB Substrate
        color([0.15, 0.55, 0.25, 0.9])
        difference() {
            linear_extrude(height = pcb_thick)
                square([pcb_w, pcb_h], center = true);
            for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
                for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                    translate([sx, sy, -0.5])
                        cylinder(d = hole_d, h = pcb_thick + 1);
                }
            }
        }

        // ESP32-C3 SuperMini Module
        translate([-18.5, -10.6, pcb_thick + 8.5]) {
            color([0.12, 0.12, 0.12])
                cube([22.5, 17.8, 1.2], center = true);
            color([0.75, 0.75, 0.78])
                translate([3.0, 0, 1.2])
                    cube([12.0, 14.0, 2.0], center = true);
            color([0.8, 0.8, 0.85])
                translate([-11.0, 0, 1.4])
                    cube([7.5, 8.8, 3.2], center = true);
            // Red Power LED
            color([1.0, 0.1, 0.1])
                translate([-4.5, 4.0, 0.8])
                    cube([1.8, 1.0, 0.8], center = true);
            // Blue Status LED (GPIO 8)
            color([0.1, 0.5, 1.0])
                translate([-4.5, -4.0, 0.8])
                    cube([1.8, 1.0, 0.8], center = true);
        }

        // Blue KF301 Screw Terminal Blocks
        color([0.15, 0.35, 0.85]) {
            // TB2 (Soil Moisture)
            translate([-25.3, 27.0 - 7.5, pcb_thick + 5.0])
                cube([15.0, 7.5, 10.0], center = true);
            // TB1 (Ultrasonic)
            translate([-3.7, 27.0 - 7.5, pcb_thick + 5.0])
                cube([20.0, 7.5, 10.0], center = true);
            // TB3 (Water Pump)
            translate([18.0, 27.0 - 7.5, pcb_thick + 5.0])
                cube([10.0, 7.5, 10.0], center = true);
            // TB4 (Battery / Solar)
            translate([37.0 - 7.5, -4.0, pcb_thick + 5.0])
                cube([7.5, 15.0, 10.0], center = true);
        }
    }
}

// =============================================================================
// OUTPUT SELECTOR
// =============================================================================
if (view_mode == "model") {
    // Printable Sled
    open_base_sled();
} else if (view_mode == "underside") {
    // Flipped to inspect silicone adhesion grid
    rotate([180, 0, 0])
        open_base_sled();
} else if (view_mode == "preview") {
    // Sled in Slate Blue with Seated PCB
    color([0.25, 0.45, 0.75])
        open_base_sled();
    pcb_mockup();
}
