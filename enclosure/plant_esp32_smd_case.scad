// =============================================================================
// Plant ESP32 Controller — Precision Enclosure with LED Status Window
// Compatible with: plant-esp32-smd.kicad_pcb (74.0 x 54.0 mm)
// =============================================================================
// Features:
// 1. Precise PCB Cradle: 4x reinforced M3 standoffs (67.0 x 47.0 mm pitch).
// 2. Beveled LED Viewing Window: Centered directly over the ESP32-C3 SuperMini
//    onboard Power (Red) and GPIO 8 Status (Blue) LEDs.
// 3. Diffuser / Acrylic Step: Underside pocket in lid for 1mm transparent lens.
// 4. Port Cutouts:
//    - Left Wall: USB-C port cutout for ESP32 SuperMini.
//    - Top Wall: Full clearance for TB1 (Ultra), TB2 (Soil), TB3 (Pump).
//    - Right Wall: Clearance for TB4 (Solar / Battery power).
// 5. Tool-Free Snap-Fit + Optional Corner Screws: Friction-fit lip with M3 screw pass.
// =============================================================================

/* [Render Mode] */
// Select what to render or export
part = "exploded"; // [base: Base Cradle Only, lid: Top Cover Only, both: Side-by-Side Printable, assembled: Assembled View, exploded: Exploded 3D Preview]

/* [PCB & Standoff Parameters] */
pcb_w          = 74.0;  // PCB width (X) in mm
pcb_h          = 54.0;  // PCB height (Y) in mm
pcb_thick      = 1.6;   // Standard FR4 thickness
hole_pitch_x   = 67.0;  // KiCad M3 hole pitch X (mm)
hole_pitch_y   = 47.0;  // KiCad M3 hole pitch Y (mm)
hole_d         = 3.2;   // Mounting hole diameter

/* [Enclosure Sizing] */
wall_thick     = 2.0;   // Perimeter wall thickness (mm)
floor_thick    = 2.5;   // Base floor thickness (mm)
lid_floor      = 2.0;   // Lid top face thickness (mm)
standoff_h     = 5.0;   // Standoff height below PCB (mm)
standoff_od    = 7.2;   // Standoff boss outer diameter (mm)
standoff_id    = 2.9;   // Pilot hole for M3 self-tapping screws / M3 heat-set inserts (mm)
interior_h     = 16.5;  // Internal clearance above standoffs for SuperMini socket & terminals
corner_r       = 4.0;   // Outer corner rounding radius (mm)
clearance      = 0.8;   // Tolerance around PCB perimeter (mm)

/* [Silicone Gel Adhesion Pattern] */
glue_channel_w = 2.0;   // Width of silicone channels (mm)
glue_channel_d = 1.0;   // Depth of silicone channels (mm)
glue_pitch     = 6.0;   // Grid spacing (mm)

/* [Port Cutout Dimensions] */
// Left Wall: USB-C cutout for ESP32-C3 SuperMini
usb_w          = 14.0;  // USB-C port cutout width (mm)
usb_h          = 9.0;   // USB-C port cutout height (mm)
usb_y          = -10.62;// Y position centered on SuperMini module

// Top Wall: Terminal Blocks TB1 (Ultra), TB2 (Soil), TB3 (Pump)
tb_top_w       = 62.0;  // Opening width covering TB1, TB2, TB3 (mm)
tb_top_h       = 10.5;  // Height of opening above PCB
tb_top_x       = -3.5;  // Centered over terminal group

// Right Wall: Terminal Block TB4 (Battery / Solar)
tb_right_w     = 18.0;  // Opening width for TB4 (mm)
tb_right_h     = 10.5;  // Opening height
tb_right_y     = -4.0;  // Position along right wall

/* [Status LED Viewing Window] */
led_win_w      = 20.0;  // Window width across ESP32 module (mm)
led_win_h      = 15.0;  // Window height across ESP32 module (mm)
led_win_x      = -18.5; // X center directly above ESP32 SuperMini status LEDs
led_win_y      = -10.6; // Y center directly above ESP32 SuperMini status LEDs
led_win_r      = 2.5;   // Corner radius of viewing window (mm)
bevel_extra    = 1.5;   // 45-degree chamfer expansion on outer surface (mm)

/* [Lid Lip & Fit] */
lid_lip_h      = 3.2;   // Height of retention lip entering the base (mm)
lid_clear      = 0.3;   // Friction-fit clearance (mm)
lid_lip_t      = 1.4;   // Thickness of inner retention lip (mm)

// Derived Dimensions
inner_w        = pcb_w + 2 * clearance;
inner_h        = pcb_h + 2 * clearance;
outer_w        = inner_w + 2 * wall_thick;
outer_h        = inner_h + 2 * wall_thick;
total_base_h   = floor_thick + standoff_h + interior_h;

lid_inner_w    = inner_w - 2 * lid_clear;
lid_inner_h    = inner_h - 2 * lid_clear;
lid_outer_w    = outer_w;
lid_outer_h    = outer_h;
lid_total_h    = lid_floor + lid_lip_h;

$fn = 40;

module rounded_rect_2d(w, h, r) {
    offset(r = r) offset(delta = -r) square([w, h], center = true);
}

// =============================================================================
// BASE CRADLE
// =============================================================================
module base_cradle() {
    difference() {
        union() {
            // Main Outer Box Body
            linear_extrude(height = total_base_h) {
                rounded_rect_2d(outer_w, outer_h, corner_r);
            }
            
            // 4x Standoff Bosses with conical base fillets
            for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
                for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                    translate([sx, sy, 0]) {
                        cylinder(d = standoff_od, h = floor_thick + standoff_h);
                        cylinder(d1 = standoff_od + 3.0, d2 = standoff_od, h = floor_thick + 2.0);
                    }
                }
            }
        }

        // Interior Cavity
        translate([0, 0, floor_thick]) {
            linear_extrude(height = total_base_h + 1) {
                rounded_rect_2d(inner_w, inner_h, max(1.0, corner_r - wall_thick));
            }
        }

        // Standoff Pilot Holes (M3)
        for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
            for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                translate([sx, sy, -0.5]) {
                    cylinder(d = standoff_id, h = floor_thick + standoff_h + 1.0);
                }
            }
        }

        // USB-C Cutout (Left Wall)
        translate([-outer_w/2 - 1, usb_y - usb_w/2, floor_thick + standoff_h]) {
            cube([wall_thick + 2, usb_w, usb_h]);
        }

        // Top Terminal Cutout (TB1, TB2, TB3 on Top Wall)
        translate([tb_top_x - tb_top_w/2, outer_h/2 - wall_thick - 1, floor_thick + standoff_h + 2.0]) {
            cube([tb_top_w, wall_thick + 2, tb_top_h + 10]);
        }

        // Right Terminal Cutout (TB4 on Right Wall)
        translate([outer_w/2 - wall_thick - 1, tb_right_y - tb_right_w/2, floor_thick + standoff_h + 2.0]) {
            cube([wall_thick + 2, tb_right_w, tb_right_h + 10]);
        }

        // Bottom Ventilation / Pin Snip Clearance Slots
        for (vx = [-18, 0, 18]) {
            translate([vx, 0, -1]) {
                linear_extrude(height = floor_thick + 2) {
                    rounded_rect_2d(8.0, 30.0, 3.0);
                }
            }
        }

        // =====================================================================
        // SILICONE ADHESION UNDERSURFACE PATTERN (Z=0)
        // =====================================================================
        // X-axis waffle channels
        for (x = [-outer_w/2 + 5 : glue_pitch : outer_w/2 - 5]) {
            translate([x - glue_channel_w/2, -outer_h/2 + 3, -0.05])
                cube([glue_channel_w, outer_h - 6, glue_channel_d + 0.05]);
        }
        // Y-axis waffle channels
        for (y = [-outer_h/2 + 5 : glue_pitch : outer_h/2 - 5]) {
            translate([-outer_w/2 + 3, y - glue_channel_w/2, -0.05])
                cube([outer_w - 6, glue_channel_w, glue_channel_d + 0.05]);
        }
        // Anchor wells at intersections for mechanical keying / suction
        for (x = [-outer_w/2 + 5 : glue_pitch : outer_w/2 - 5]) {
            for (y = [-outer_h/2 + 5 : glue_pitch : outer_h/2 - 5]) {
                translate([x, y, -0.05])
                    cylinder(d = 3.2, h = glue_channel_d + 0.15, $fn = 16);
            }
        }
        // Perimeter rivet thru-holes (silicone mushrooms through to lock physically)
        for (rx = [-outer_w/2 + 3.5, outer_w/2 - 3.5]) {
            for (ry = [-outer_h/2 + 10, 0, outer_h/2 - 10]) {
                translate([rx, ry, -1])
                    cylinder(d = 2.8, h = floor_thick + 2, $fn = 16);
            }
        }
    }
}

// =============================================================================
// STATUS LID (TOP COVER WITH LED VIEWING WINDOW)
// =============================================================================
module status_lid() {
    difference() {
        union() {
            // Main Top Plate
            linear_extrude(height = lid_floor) {
                rounded_rect_2d(lid_outer_w, lid_outer_h, corner_r);
            }
            
            // Alignment / Snap-Fit Perimeter Rim (extends down into base cavity)
            translate([0, 0, lid_floor]) {
                difference() {
                    linear_extrude(height = lid_lip_h) {
                        rounded_rect_2d(lid_inner_w, lid_inner_h, max(1.0, corner_r - wall_thick));
                    }
                    translate([0, 0, -0.1]) {
                        linear_extrude(height = lid_lip_h + 0.2) {
                            rounded_rect_2d(lid_inner_w - 2*lid_lip_t, lid_inner_h - 2*lid_lip_t, 1.0);
                        }
                    }
                }
            }
        }

        // --- STATUS LED VIEWING WINDOW ---
        // Through-cutout with beveled 45-degree chamfer on top face
        translate([led_win_x, led_win_y, -0.5]) {
            hull() {
                translate([0, 0, 0])
                    linear_extrude(height = 0.1)
                        rounded_rect_2d(led_win_w + 2*bevel_extra, led_win_h + 2*bevel_extra, led_win_r + bevel_extra);
                translate([0, 0, lid_floor + lid_lip_h + 1.0])
                    linear_extrude(height = 0.1)
                        rounded_rect_2d(led_win_w, led_win_h, led_win_r);
            }
        }

        // Acrylic / Diffuser Insert Recess (Underside step for clear lens)
        translate([led_win_x, led_win_y, lid_floor - 0.8]) {
            linear_extrude(height = lid_lip_h + 2) {
                rounded_rect_2d(led_win_w + 4.0, led_win_h + 4.0, led_win_r + 1.0);
            }
        }

        // Top Terminal Wire Exit Hood / Relief
        translate([tb_top_x - tb_top_w/2 + 2, lid_outer_h/2 - wall_thick - 2, -1]) {
            cube([tb_top_w - 4, wall_thick + 4, lid_total_h + 2]);
        }
        
        // Right Terminal Wire Relief
        translate([lid_outer_w/2 - wall_thick - 2, tb_right_y - tb_right_w/2 + 1, -1]) {
            cube([wall_thick + 4, tb_right_w - 2, lid_total_h + 2]);
        }

        // M3 Corner Screw Holes through lid (countersunk on top face)
        for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
            for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                translate([sx, sy, -1]) {
                    cylinder(d = 3.4, h = lid_total_h + 2); // M3 clearance pass
                    translate([0, 0, 0.5])
                        cylinder(d1 = 6.4, d2 = 3.4, h = 1.8); // M3 countersunk head
                }
            }
        }
        
        // Debossed Title Text on Lid Surface
        translate([12.0, 10.0, -0.1]) {
            rotate([180, 0, 0])
                linear_extrude(height = 0.6)
                    text("PLANT ESP32", size = 4.2, font = "Liberation Sans:style=Bold", halign = "center", valign = "center");
        }
        translate([led_win_x, led_win_y + led_win_h/2 + 2.8, -0.1]) {
            rotate([180, 0, 0])
                linear_extrude(height = 0.6)
                    text("STATUS", size = 2.4, font = "Liberation Sans:style=Bold", halign = "center", valign = "center");
        }
    }
}

// =============================================================================
// PCB & ESP32 MOCKUP (FOR FIT & ALIGNMENT VERIFICATION)
// =============================================================================
module pcb_mockup() {
    translate([0, 0, floor_thick + standoff_h]) {
        // Green PCB Substrate
        color([0.15, 0.55, 0.25, 0.85])
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
        translate([led_win_x, led_win_y, pcb_thick + 8.5]) {
            // Module PCB
            color([0.12, 0.12, 0.12])
                cube([22.5, 17.8, 1.2], center = true);
            
            // RF Metal Shielding Can
            color([0.75, 0.75, 0.78])
                translate([3.0, 0, 1.2])
                    cube([12.0, 14.0, 2.0], center = true);
            
            // USB-C Receptacle
            color([0.8, 0.8, 0.85])
                translate([-11.0, 0, 1.4])
                    cube([7.5, 8.8, 3.2], center = true);
            
            // Red Power LED (illuminated)
            color([1.0, 0.1, 0.1])
                translate([-4.5, 4.0, 0.8])
                    cube([1.8, 1.0, 0.8], center = true);
            
            // Blue Status LED (GPIO 8)
            color([0.1, 0.5, 1.0])
                translate([-4.5, -4.0, 0.8])
                    cube([1.8, 1.0, 0.8], center = true);
        }

        // Blue Terminal Blocks (KF301)
        color([0.15, 0.35, 0.85]) {
            // TB2 (Soil)
            translate([-25.3, 27.0 - 7.5, pcb_thick + 5.0])
                cube([15.0, 7.5, 10.0], center = true);
            // TB1 (Ultrasonic)
            translate([-3.7, 27.0 - 7.5, pcb_thick + 5.0])
                cube([20.0, 7.5, 10.0], center = true);
            // TB3 (Pump)
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
if (part == "base") {
    base_cradle();
} else if (part == "lid") {
    // Flipped flat for 3D printing build plate
    rotate([180, 0, 0]) status_lid();
} else if (part == "both") {
    // Side by side for printing both parts in one run
    translate([-outer_w/2 - 6, 0, 0]) base_cradle();
    translate([outer_w/2 + 6, 0, lid_total_h]) rotate([180, 0, 0]) status_lid();
} else if (part == "assembled") {
    color([0.25, 0.45, 0.75]) base_cradle();
    pcb_mockup();
    color([0.88, 0.88, 0.90, 0.92]) translate([0, 0, total_base_h]) status_lid();
} else if (part == "exploded") {
    // Base in slate blue
    color([0.25, 0.45, 0.75]) base_cradle();
    // PCB assembly inside
    pcb_mockup();
    // Hovering Lid above
    color([0.88, 0.88, 0.90, 0.92]) translate([0, 0, total_base_h + 18.0]) status_lid();
}
