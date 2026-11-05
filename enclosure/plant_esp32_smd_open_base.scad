// =============================================================================
// Plant ESP32 Controller — Open-Frame Tank-Mount Sled (Rev 2.2)
// Compatible with: plant-esp32-smd.kicad_pcb (74.0 x 54.0 mm)
// =============================================================================
// Engineering Highlights:
// 1. Guaranteed PCB Fit & Zero Corner Binding:
//    - +0.6 mm clearance per side (+1.2 mm total breathing room).
//    - Pocket size: 75.2 x 55.2 mm (nominal PCB is 74.0 x 54.0 mm).
//    - Ø2.6 mm Dogbone corner relief drill at each inside corner vertex:
//      completely eliminates FDM nozzle bead corner fillets so sharp FR4 corners
//      never bind.
//    - 45° Lead-in funnel chamfers along all inner top edges of the corner guides
//      for effortless PCB drop-in.
// 2. Exact 2.0 mm Standoff Bosses:
//    - 4x reinforced M3 standoffs elevated exactly 2.0 mm above the base plate.
//    - Fully independent round bosses (Ø6.4 mm) with conical base gussets.
//    - Exact KiCad mounting pitch: 67.0 x 47.0 mm (±33.5, ±23.5 mm).
//    - Ø3.0 mm through-holes for M3 self-tapping screws or M3 heat-set inserts.
// 3. Silicone Gel Adhesion Underside Pattern:
//    - 2.0 mm wide x 1.0 mm deep cross-hatch waffle channels.
//    - Ø3.2 mm anchor suction wells at all grid intersections.
//    - 6x perimeter silicone rivet through-holes for permanent mechanical locking
//      to smooth/curved HDPE, PP, or acrylic water tanks.
// 4. Weight Reduction & Airflow:
//    - 3x central airflow slots prevent moisture trapping, allow cooling, and save filament.
// =============================================================================

/* [Render Mode] */
view_mode = "model"; // [model: 3D Base Sled Only (Printable), preview: Base + PCB Preview, underside: Undersurface Adhesive Pattern Inspection]

/* [PCB Parameters] */
pcb_w         = 74.0;  // PCB width (X) mm
pcb_h         = 54.0;  // PCB height (Y) mm
pcb_thick     = 1.6;   // FR4 thickness (mm)
hole_pitch_x  = 67.0;  // KiCad M3 hole pitch X (mm)
hole_pitch_y  = 47.0;  // KiCad M3 hole pitch Y (mm)
hole_d        = 3.2;   // Mounting hole diameter on PCB (mm)

/* [Fit & Tolerances] */
// Clearance offset around PCB perimeter (0.6 mm per side = 1.2 mm total breathing room)
pcb_clearance = 0.6;   // mm per side
pocket_w      = pcb_w + 2 * pcb_clearance; // 75.2 mm
pocket_h      = pcb_h + 2 * pcb_clearance; // 55.2 mm
dogbone_r     = 1.3;   // Dogbone corner relief radius (Ø2.6 mm)
chamfer_w     = 1.0;   // Top lead-in chamfer width (mm)

/* [Base Plate Dimensions] */
plate_floor   = 2.5;   // Plate floor thickness (mm)
standoff_h    = 2.0;   // Standoff height above floor (mm) — exactly 2.0 mm as requested
standoff_od   = 6.4;   // Standoff boss outer diameter (mm)
standoff_id   = 3.0;   // Pilot hole for M3 screws or M3 heat-set inserts (mm)

bracket_lip_h = 1.4;   // Height of corner guide lip above standoff (mm)
total_bracket_h = plate_floor + standoff_h + bracket_lip_h; // 5.9 mm

plate_w       = 82.0;  // Base plate width (mm)
plate_h       = 62.0;  // Base plate height (mm)
plate_corner_r= 3.5;   // Outer corner radius (mm)

bracket_arm_x = 12.0;  // Corner guide arm length in X (mm)
bracket_arm_y = 15.0;  // Corner guide arm length in Y (mm)

/* [Silicone Gel Adhesion Pattern] */
channel_w     = 2.0;   // Width of glue channels (mm)
channel_d     = 1.0;   // Depth of glue channels (mm)
grid_pitch    = 6.0;   // Grid spacing (mm)

$fn = 36;

module rounded_rect_2d(w, h, r) {
    offset(r = r) offset(delta = -r) square([w, h], center = true);
}

// =============================================================================
// CORNER ALIGNMENT GUIDES (2D PLANAR PROFILE)
// =============================================================================
module corner_guides_2d() {
    difference() {
        // Outer plate profile restricted to 4 corner boundary zones
        intersection() {
            rounded_rect_2d(plate_w, plate_h, plate_corner_r);
            union() {
                for (sx = [-1, 1]) {
                    for (sy = [-1, 1]) {
                        translate([sx * (plate_w/2 - bracket_arm_x/2), 
                                   sy * (plate_h/2 - bracket_arm_y/2)])
                            square([bracket_arm_x, bracket_arm_y], center = true);
                    }
                }
            }
        }
        
        // Subtract PCB pocket
        square([pocket_w, pocket_h], center = true);
        
        // Subtract 4x dogbone relief circles at pocket vertices
        for (sx = [-1, 1]) {
            for (sy = [-1, 1]) {
                translate([sx * pocket_w/2, sy * pocket_h/2])
                    circle(r = dogbone_r, $fn = 32);
            }
        }
    }
}

// =============================================================================
// MAIN OPEN-FRAME SLED
// =============================================================================
module open_base_sled() {
    difference() {
        union() {
            // 1. Rigid Base Floor Plate
            linear_extrude(height = plate_floor)
                rounded_rect_2d(plate_w, plate_h, plate_corner_r);
            
            // 2. 4x Corner Alignment Guides
            linear_extrude(height = total_bracket_h)
                corner_guides_2d();
                
            // 3. 4x Standoff Bosses (2.0 mm elevation above floor)
            for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
                for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                    translate([sx, sy, plate_floor]) {
                        cylinder(d = standoff_od, h = standoff_h);
                        // Conical base gusset for layer-shear strength
                        cylinder(d1 = standoff_od + 1.0, d2 = standoff_od, h = 0.8);
                    }
                }
            }
        }

        // --- SUBTRACTIONS ---

        // 4x M3 Standoff Pilot Holes (through standoff and floor)
        for (sx = [-hole_pitch_x/2, hole_pitch_x/2]) {
            for (sy = [-hole_pitch_y/2, hole_pitch_y/2]) {
                translate([sx, sy, -1])
                    cylinder(d = standoff_id, h = total_bracket_h + 2);
            }
        }

        // 45° Lead-in Funnel Chamfers at top of corner brackets
        for (sx = [-1, 1]) {
            for (sy = [-1, 1]) {
                cx = sx * pocket_w/2;
                cy = sy * pocket_h/2;
                
                // Chamfer along X arm
                translate([cx - sx * bracket_arm_x/2, cy, total_bracket_h])
                    rotate([sy * 45, 0, 0])
                        cube([bracket_arm_x * 2, chamfer_w * 2, chamfer_w * 2], center = true);
                        
                // Chamfer along Y arm
                translate([cx, cy - sy * bracket_arm_y/2, total_bracket_h])
                    rotate([0, -sx * 45, 0])
                        cube([chamfer_w * 2, bracket_arm_y * 2, chamfer_w * 2], center = true);
                        
                // Conical chamfer at the dogbone vertex
                translate([cx, cy, total_bracket_h - chamfer_w])
                    cylinder(r1 = dogbone_r, r2 = dogbone_r + chamfer_w, h = chamfer_w + 0.1);
            }
        }

        // Center Weight-Reduction & Airflow Openings
        for (vx = [-17, 0, 17]) {
            translate([vx, 0, -1])
                linear_extrude(height = plate_floor + 2)
                    rounded_rect_2d(8.0, 32.0, 3.0);
        }

        // =====================================================================
        // SILICONE GEL ADHESION PATTERN (UNDERSIDE Z=0)
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
            for (ry = [-plate_h/2 + 12, 0, plate_h/2 - 12]) {
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
        // Green FR4 PCB Substrate
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
