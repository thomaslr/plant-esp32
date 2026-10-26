// =============================================================================
// Plant ESP32 Controller — Universal Flat Bottle/Tank PCB Mount
// Fits: plant-esp32-smd.kicad_pcb (74.0 mm x 54.0 mm)
// =============================================================================
// Designed for adhesion to flat plastic cleaning fluid bottles, tanks, or walls
// using silicone sealant, hot glue, or double-sided foam VHB tape.
// =============================================================================

// --- [CUSTOMIZER PARAMETERS] -------------------------------------------------

/* [PCB Dimensions] */
pcb_width           = 74.0; // [60:0.5:100]
pcb_height          = 54.0; // [40:0.5:80]
hole_pitch_x        = 67.0; // [50:0.5:90]
hole_pitch_y        = 47.0; // [30:0.5:70]

/* [Mount Profile] */
base_thickness      = 2.8;  // Base plate thickness (mm)
standoff_height     = 5.0;  // Clearance between PCB underside & baseplate (mm)
standoff_od         = 7.2;  // Standoff boss outer diameter (mm)
mount_type          = "self_tap"; // ["self_tap": M3 self-tapping screw (2.8mm), "heat_set": M3 brass insert (4.0mm), "m3_loose": M3 clearance hole (3.4mm)]

/* [Adhesion & Ventilation] */
flange_margin       = 5.0;  // Extra border width for adhesive surface area (mm)
corner_radius       = 4.5;  // Outer corner rounding radius (mm)
enable_glue_grooves = true; // Textured mechanical interlock grid on the back face
glue_groove_w       = 1.6;  // Width of glue channels (mm)
glue_groove_depth   = 0.8;  // Depth of glue channels (mm)
enable_glue_rivets  = true; // Through-holes that let glue/silicone mushroom through
enable_center_vent  = true; // Large center cutout for airflow & through-hole pin clearance

/* [Resolution] */
$fn = 48;

// --- [DERIVED VARIABLES] -----------------------------------------------------
mount_w = pcb_width + 2 * flange_margin;   // 84.0 mm
mount_h = pcb_height + 2 * flange_margin;  // 64.0 mm

screw_hole_d = (mount_type == "heat_set") ? 4.0 :
               (mount_type == "self_tap") ? 2.8 : 3.4;

vent_w = hole_pitch_x - standoff_od - 4.0; // ~55.8 mm
vent_h = hole_pitch_y - standoff_od - 4.0; // ~35.8 mm

// =============================================================================
// 2D HELPER PROFILES
// =============================================================================
module rounded_rect_2d(w, h, r) {
    offset(r = r) offset(delta = -r) {
        square([w, h], center = true);
    }
}

// =============================================================================
// MAIN ASSEMBLY
// =============================================================================
module pcb_mount() {
    difference() {
        // --- 1. POSITIVE SOLID BODY ---
        union() {
            // Main Baseplate
            linear_extrude(height = base_thickness) {
                rounded_rect_2d(mount_w, mount_h, corner_radius);
            }
            
            // 4x Standoff Bosses
            for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
                for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                    translate([sx, sy, 0]) {
                        cylinder(d = standoff_od, h = base_thickness + standoff_height);
                        // Structural fillet/cone at base of boss for strength
                        cylinder(d1 = standoff_od + 2.5, d2 = standoff_od, h = base_thickness + 1.5);
                    }
                }
            }
        }

        // --- 2. NEGATIVE CUTOUTS (HOLES & CLEARANCES) ---
        
        // 4x PCB Screw Holes
        for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
            for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                translate([sx, sy, -1]) {
                    // Screw hole through the entire standoff
                    cylinder(d = screw_hole_d, h = base_thickness + standoff_height + 2);
                }
            }
        }

        // Center Ventilation & Component Lead Clearance Window
        if (enable_center_vent) {
            translate([0, 0, -1]) {
                linear_extrude(height = base_thickness + 2) {
                    rounded_rect_2d(vent_w, vent_h, 3.0);
                }
            }
        }

        // Glue Mushroom Rivet Holes (Allows silicone/hot glue to mushroom through for a mechanical anchor)
        if (enable_glue_rivets) {
            for (rx = [-(mount_w/2 - 3.5), (mount_w/2 - 3.5)]) {
                for (ry = [-(mount_h/2 - 3.5), (mount_h/2 - 3.5)]) {
                    translate([rx, ry, -1]) {
                        cylinder(d = 3.0, h = base_thickness + 2);
                        // Top countersink so the mushroom head lies flat
                        translate([0, 0, base_thickness - 0.8 + 1]) {
                            cylinder(d1 = 3.0, d2 = 5.0, h = 1.0);
                        }
                    }
                }
            }
            // Midpoint side rivet holes
            for (rx = [-(mount_w/2 - 3.5), (mount_w/2 - 3.5)]) {
                translate([rx, 0, -1]) {
                    cylinder(d = 3.0, h = base_thickness + 2);
                    translate([0, 0, base_thickness - 0.8 + 1]) {
                        cylinder(d1 = 3.0, d2 = 5.0, h = 1.0);
                    }
                }
            }
        }

        // Rear Glue / Silicone Grid Channels (Mechanical Interlock Grooves)
        if (enable_glue_grooves) {
            // Recessed channels on the bottom (Z = -0.01 to glue_groove_depth)
            for (gx = [-(mount_w/2 - 6) : 6.0 : (mount_w/2 - 6)]) {
                translate([gx - glue_groove_w/2, -mount_h/2, -0.05]) {
                    cube([glue_groove_w, mount_h, glue_groove_depth + 0.05]);
                }
            }
            for (gy = [-(mount_h/2 - 6) : 6.0 : (mount_h/2 - 6)]) {
                translate([-mount_w/2, gy - glue_groove_w/2, -0.05]) {
                    cube([mount_w, glue_groove_w, glue_groove_depth + 0.05]);
                }
            }
        }

        // Chamfer / Wire entry notches along top and bottom
        // Top edge wire relief notch
        translate([0, mount_h/2, base_thickness]) {
            rotate([45, 0, 0]) {
                cube([mount_w - 10, 3, 3], center = true);
            }
        }
    }
}

// Render the mount ready for printing (flat side down on Z=0)
pcb_mount();
