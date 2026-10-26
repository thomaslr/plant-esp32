// =============================================================================
// Plant ESP32 Controller — 2-Piece Splash-Proof Bottle Enclosure
// Compatible with: plant-esp32-smd.kicad_pcb (74x54mm)
// =============================================================================
// Features:
// - Deep protective cradle with cutouts for USB-C and screw terminal wires
// - 4x M3 corner standoffs with lead clearance
// - Flat adhesive back with waffle glue channels and rivet holes for bottle mounting
// - Snap-on splash cover/lid to shield electronics from water drips
// =============================================================================

/* [Render Mode] */
part = "exploded"; // ["base": Bottom Cradle Only, "lid": Top Cover Only, "both": Printable Side-by-Side, "exploded": Exploded 3D Assembly Preview]

/* [PCB Parameters] */
pcb_w          = 74.0;
pcb_h          = 54.0;
hole_pitch_x   = 67.0;
hole_pitch_y   = 47.0;

/* [Enclosure Sizing] */
wall_thick     = 2.4;  // Perimeter wall thickness (mm)
floor_thick    = 2.5;  // Bottom base thickness (mm)
standoff_h     = 5.0;  // Standoff height below PCB (mm)
standoff_od    = 7.0;  // Standoff outer diameter (mm)
standoff_id    = 2.8;  // Pilot hole for M3 self-tapping screws (mm)
box_interior_h = 16.0; // Interior wall height above standoffs (mm)
corner_r       = 4.0;  // Exterior corner rounding radius (mm)

/* [Adhesion Grid] */
glue_channel_w = 1.6;  // Width of glue channels (mm)
glue_channel_d = 0.8;  // Depth of glue channels (mm)

/* [Port Openings] */
usb_w          = 13.0; // USB-C cutout width (mm)
usb_h          = 8.0;  // USB-C cutout height (mm)
usb_y_center   = -(pcb_h/2 - 37.5); // Y offset from center to SuperMini USB-C port

top_cutout_w   = 62.0; // Top terminal blocks opening width (mm)
top_cutout_h   = 9.5;  // Top opening height (mm)

right_cutout_w = 20.0; // Right terminal block (TB4) cutout width (mm)
right_cutout_h = 9.5;  // Right cutout height (mm)

$fn = 40;

// Derived Dimensions
inner_w = pcb_w + 1.6;
inner_h = pcb_h + 1.6;
outer_w = inner_w + 2 * wall_thick;
outer_h = inner_h + 2 * wall_thick;
total_box_h = floor_thick + standoff_h + box_interior_h;

module rounded_rect_2d(w, h, r) {
    offset(r = r) offset(delta = -r) {
        square([w, h], center = true);
    }
}

// =============================================================================
// BASE CRADLE
// =============================================================================
module base_cradle() {
    difference() {
        union() {
            // Main Outer Box Body
            linear_extrude(height = total_box_h) {
                rounded_rect_2d(outer_w, outer_h, corner_r);
            }
            
            // 4x Standoff Bosses
            for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
                for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                    translate([sx, sy, 0]) {
                        cylinder(d = standoff_od, h = floor_thick + standoff_h);
                        cylinder(d1 = standoff_od + 2.5, d2 = standoff_od, h = floor_thick + 1.5);
                    }
                }
            }
        }

        // Cavity
        translate([0, 0, floor_thick]) {
            linear_extrude(height = total_box_h + 1) {
                rounded_rect_2d(inner_w, inner_h, corner_r - 1.0);
            }
        }

        // Screw Holes
        for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
            for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                translate([sx, sy, -1]) {
                    cylinder(d = standoff_id, h = total_box_h + 2);
                }
            }
        }

        // Center Ventilation / Pin Clearance Window
        vent_w = hole_pitch_x - standoff_od - 4.0;
        vent_h = hole_pitch_y - standoff_od - 4.0;
        translate([0, 0, -1]) {
            linear_extrude(height = floor_thick + 2) {
                rounded_rect_2d(vent_w, vent_h, 3.0);
            }
        }

        // USB-C Cutout (Left Wall)
        translate([-outer_w/2 - 1, usb_y_center - usb_w/2, floor_thick + standoff_h]) {
            cube([wall_thick + 2, usb_w, usb_h]);
        }

        // Top Terminal Cutout (Top Wall for TB1, TB2, TB3)
        translate([-top_cutout_w/2, outer_h/2 - wall_thick - 1, floor_thick + standoff_h + 3]) {
            cube([top_cutout_w, wall_thick + 2, top_cutout_h + 5]);
        }

        // Right Terminal Cutout (Right Wall for TB4)
        translate([outer_w/2 - wall_thick - 1, -(pcb_h/2 - 37.0) - right_cutout_w/2, floor_thick + standoff_h + 3]) {
            cube([wall_thick + 2, right_cutout_w, right_cutout_h + 5]);
        }

        // Rear Glue Interlock Grid
        for (gx = [-(outer_w/2 - 6) : 7.0 : (outer_w/2 - 6)]) {
            translate([gx - glue_channel_w/2, -outer_h/2, -0.05]) {
                cube([glue_channel_w, outer_h, glue_channel_d + 0.05]);
            }
        }
        for (gy = [-(outer_h/2 - 6) : 7.0 : (outer_h/2 - 6)]) {
            translate([-outer_w/2, gy - glue_channel_w/2, -0.05]) {
                cube([outer_w, glue_channel_w, glue_channel_d + 0.05]);
            }
        }

        // Perimeter Glue Rivet Holes
        for (rx = [-(outer_w/2 - 4), (outer_w/2 - 4)]) {
            for (ry = [-(outer_h/2 - 4), 0, (outer_h/2 - 4)]) {
                translate([rx, ry, -1]) {
                    cylinder(d = 3.0, h = floor_thick + 2);
                }
            }
        }
    }
}

// =============================================================================
// SPLASH LID (TOP COVER)
// =============================================================================
lid_lip_h = 3.5;
lid_wall  = 2.0;
lid_clear = 0.35; // Friction-fit clearance

lid_inner_w = outer_w + 2 * lid_clear;
lid_inner_h = outer_h + 2 * lid_clear;
lid_outer_w = lid_inner_w + 2 * lid_wall;
lid_outer_h = lid_inner_h + 2 * lid_wall;
lid_total_h = 2.0 + lid_lip_h;

module splash_lid() {
    difference() {
        linear_extrude(height = lid_total_h) {
            rounded_rect_2d(lid_outer_w, lid_outer_h, corner_r + 1.0);
        }
        // Inner pocket that slips over the base
        translate([0, 0, 2.0]) {
            linear_extrude(height = lid_lip_h + 2) {
                rounded_rect_2d(lid_inner_w, lid_inner_h, corner_r);
            }
        }
        // Top Wire Exit Slot (Protective Overhang Hood)
        translate([-top_cutout_w/2 + 5, lid_outer_h/2 - lid_wall - 1, 0]) {
            cube([top_cutout_w - 10, lid_wall + 3, lid_total_h + 2]);
        }
    }
}

// =============================================================================
// OUTPUT SELECTOR
// =============================================================================
if (part == "base") {
    base_cradle();
} else if (part == "lid") {
    // Flipped flat for 3D printing
    rotate([180, 0, 0]) splash_lid();
} else if (part == "both") {
    // Side by side for single print plate
    translate([-outer_w/2 - 5, 0, 0]) base_cradle();
    translate([lid_outer_w/2 + 5, 0, lid_total_h]) rotate([180, 0, 0]) splash_lid();
} else if (part == "exploded") {
    // Base in blue
    color([0.2, 0.45, 0.8]) base_cradle();
    // Lid hovering above in semi-transparent gray
    color([0.8, 0.8, 0.8, 0.85]) translate([0, 0, total_box_h + 12.0]) splash_lid();
}
