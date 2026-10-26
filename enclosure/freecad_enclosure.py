#!/usr/bin/env python3
"""
=============================================================================
Plant ESP32 Controller — Parametric FreeCAD Enclosure Script
Compatible with: FreeCAD 0.20+ / 1.0+ / 1.1+ and KiCadStepUp Workbench
PCB: plant-esp32-smd.kicad_pcb (74.0 mm x 54.0 mm)
=============================================================================
This script can be run:
  1. Inside FreeCAD GUI: Macro -> Run Macro -> select freecad_enclosure.py
  2. In FreeCAD Python Console: exec(open('freecad_enclosure.py').read())
  3. Headless via command-line: 
     QT_QPA_PLATFORM=offscreen /Applications/FreeCAD.app/Contents/MacOS/FreeCAD -c freecad_enclosure.py
=============================================================================
"""

import os
import sys
import math

# Add FreeCAD user Mod paths for KiCadStepUp workbench
_MOD_PATHS = [
    os.path.expanduser("~/Library/Application Support/FreeCAD/v1-1/Mod/kicadStepUpMod"),
    os.path.expanduser("~/Library/Application Support/FreeCAD/Mod/kicadStepUpMod"),
    os.path.expanduser("~/Library/Application Support/FreeCAD/v1-0/Mod/kicadStepUpMod"),
]
for p in _MOD_PATHS:
    if os.path.exists(p) and p not in sys.path:
        sys.path.append(p)

# --- FreeCAD Core API Imports ---
try:
    import FreeCAD as App
    import Part
except ImportError:
    print("[ERROR] This script must be run inside FreeCAD or via FreeCAD's Python environment.")
    print("        Usage: /Applications/FreeCAD.app/Contents/MacOS/FreeCAD -c freecad_enclosure.py")
    sys.exit(1)

# Check and initialize GUI for ViewProvider/GuiDocument support
gui_available = False
try:
    import FreeCADGui as Gui
    if not App.GuiUp:
        try:
            Gui.showMainWindow()
        except Exception:
            pass
    if App.GuiUp:
        gui_available = True
        print("[INFO] FreeCAD GUI module initialized.")
except Exception as e:
    print(f"[INFO] FreeCAD GUI not initialized: {e}")

# Check for KiCadStepUp workbench
# Note: kicadStepUptools requires an active OpenGL context for its ZoomFit thread.
# In offscreen/console mode, we use Part.read on the STEP model to prevent OpenGL segfaults.
is_offscreen = (os.environ.get('QT_QPA_PLATFORM') == 'offscreen')
has_ksu = False
ksu = None

if gui_available and not is_offscreen:
    try:
        import kicadStepUptools as ksu
        has_ksu = True
        print("[INFO] KiCadStepUp tools loaded in interactive GUI mode.")
    except Exception as e:
        try:
            import kicadStepUpCMD as ksu
            has_ksu = True
            print("[INFO] KiCadStepUpCMD loaded in interactive GUI mode.")
        except Exception:
            print(f"[INFO] KiCadStepUp not loaded ({e}); will use STEP assembly.")
else:
    print("[INFO] Headless/Offscreen execution: using STEP assembly for PCBA geometry.")

# =============================================================================
# 1. PARAMETRIC ENCLOSURE CONFIGURATION
# =============================================================================
# PCB Dimensions (from KiCad layout)
PCB_W = 74.0          # PCB Width (X)
PCB_H = 54.0          # PCB Height (Y)
PCB_THICKNESS = 1.6   # PCB Substrate thickness
HOLE_PITCH_X = 67.0   # M3 Hole spacing X (centers: +/- 33.5 mm)
HOLE_PITCH_Y = 47.0   # M3 Hole spacing Y (centers: +/- 23.5 mm)
HOLE_DIA = 3.2        # M3 Hole diameter on PCB

# Enclosure Tolerances & Dimensions
CLEARANCE_XY = 0.8    # Clearance between PCB and inner enclosure wall
WALL_THICKNESS = 2.4  # Main enclosure wall thickness
BASE_THICKNESS = 2.5  # Bottom floor plate thickness
STANDOFF_H = 5.0      # Standoff height below PCB (for lead/pin clearance)
STANDOFF_OD = 7.0     # Standoff boss outer diameter
STANDOFF_ID = 2.8     # Pilot hole for M3 self-tapping screws (4.0 for brass inserts)
BOX_DEPTH = 16.0      # Total inner height of the base box
CORNER_RADIUS = 4.0   # Exterior corner rounding fillet

# Adhesion Features for Flat Bottle
GLUE_GROOVE_W = 1.6   # Width of mechanical interlock glue channels
GLUE_GROOVE_D = 0.8   # Depth of glue channels on the rear face
GLUE_RIVET_DIA = 3.0  # Weep / rivet holes for silicone/glue mushrooming

# Connector Opening Positions (relative to PCB corner)
# USB-C: on left wall (X = -PCB_W/2), center Y = 16.5 mm from center, size 13x8mm
USB_WIDTH = 13.0
USB_HEIGHT = 8.0
USB_OFFSET_Y = -(PCB_H/2 - 37.5)  # Y position matching SuperMini USB-C port

# Top Terminal Block Opening: TB1, TB2, TB3 along top wall (Y = +PCB_H/2)
TOP_CUTOUT_W = 60.0
TOP_CUTOUT_H = 9.0

# Right Terminal Block Opening: TB4 along right wall (X = +PCB_W/2)
RIGHT_CUTOUT_W = 20.0
RIGHT_CUTOUT_H = 9.0

# Paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
STEP_PCB_PATH = os.path.join(SCRIPT_DIR, "plant-esp32-smd.step")
KICAD_PCB_PATH = os.path.join(SCRIPT_DIR, "..", "pcb", "smd", "plant-esp32-smd.kicad_pcb")

# Helper to configure object appearance and ensure it is visible in the GUI
def configure_view(obj, rgb_tuple, transparency=0):
    if hasattr(obj, "ViewObject") and obj.ViewObject is not None:
        try:
            obj.ViewObject.Visibility = True
            obj.ViewObject.ShapeColor = rgb_tuple
            if transparency > 0:
                obj.ViewObject.Transparency = transparency
            if hasattr(obj.ViewObject, "DisplayMode"):
                obj.ViewObject.DisplayMode = "Shaded"
        except Exception as e:
            print(f"[WARN] Failed to configure ViewObject for {obj.Name}: {e}")

# =============================================================================
# 2. FREECAD DOCUMENT SETUP
# =============================================================================
DOC_NAME = "PlantESP32_FreeCAD_Enclosure"
# Close existing document with same name if already open
if DOC_NAME in App.listDocuments():
    App.closeDocument(DOC_NAME)

doc = App.newDocument(DOC_NAME)
App.setActiveDocument(DOC_NAME)

print(f"[INFO] Initialized FreeCAD Document: {DOC_NAME}")

# =============================================================================
# 3. BUILD BOTTOM ENCLOSURE (BASE CRADLE)
# =============================================================================
inner_w = PCB_W + 2 * CLEARANCE_XY
inner_h = PCB_H + 2 * CLEARANCE_XY
outer_w = inner_w + 2 * WALL_THICKNESS
outer_h = inner_h + 2 * WALL_THICKNESS
total_h = BASE_THICKNESS + STANDOFF_H + BOX_DEPTH

# 3.1 Outer Shell
outer_box = Part.makeBox(outer_w, outer_h, total_h)
outer_box.translate(App.Vector(-outer_w/2, -outer_h/2, 0))

# 3.2 Inner Cavity (Pocket)
inner_box = Part.makeBox(inner_w, inner_h, total_h)
inner_box.translate(App.Vector(-inner_w/2, -inner_h/2, BASE_THICKNESS))

# Hollow cradle body
cradle = outer_box.cut(inner_box)

# 3.3 Add 4 Corner Standoff Bosses
hole_centers = [
    (-HOLE_PITCH_X/2, -HOLE_PITCH_Y/2),
    ( HOLE_PITCH_X/2, -HOLE_PITCH_Y/2),
    (-HOLE_PITCH_X/2,  HOLE_PITCH_Y/2),
    ( HOLE_PITCH_X/2,  HOLE_PITCH_Y/2)
]

for hx, hy in hole_centers:
    boss = Part.makeCylinder(STANDOFF_OD/2, BASE_THICKNESS + STANDOFF_H, App.Vector(hx, hy, 0), App.Vector(0, 0, 1))
    cradle = cradle.fuse(boss)

# 3.4 Drill Screw Pilot Holes in Standoffs
for hx, hy in hole_centers:
    hole = Part.makeCylinder(STANDOFF_ID/2, total_h + 5, App.Vector(hx, hy, -1), App.Vector(0, 0, 1))
    cradle = cradle.cut(hole)

# 3.5 Center Ventilation / Lead Clearance Window in Floor
vent_w = HOLE_PITCH_X - STANDOFF_OD - 4.0
vent_h = HOLE_PITCH_Y - STANDOFF_OD - 4.0
vent_box = Part.makeBox(vent_w, vent_h, BASE_THICKNESS + 4)
vent_box.translate(App.Vector(-vent_w/2, -vent_h/2, -2))
cradle = cradle.cut(vent_box)

# 3.6 Connector Cutouts
# USB-C Cutout (Left Wall)
usb_cut = Part.makeBox(WALL_THICKNESS + 4, USB_WIDTH, USB_HEIGHT)
usb_cut.translate(App.Vector(-outer_w/2 - 2, USB_OFFSET_Y - USB_WIDTH/2, BASE_THICKNESS + STANDOFF_H))
cradle = cradle.cut(usb_cut)

# Top Wire Cutout (TB1, TB2, TB3 - Top Wall)
top_cut = Part.makeBox(TOP_CUTOUT_W, WALL_THICKNESS + 4, TOP_CUTOUT_H)
top_cut.translate(App.Vector(-TOP_CUTOUT_W/2, outer_h/2 - WALL_THICKNESS - 2, BASE_THICKNESS + STANDOFF_H + 4))
cradle = cradle.cut(top_cut)

# Right Wire Cutout (TB4 External Power - Right Wall)
right_cut = Part.makeBox(WALL_THICKNESS + 4, RIGHT_CUTOUT_W, RIGHT_CUTOUT_H)
right_cut.translate(App.Vector(outer_w/2 - WALL_THICKNESS - 2, -(PCB_H/2 - 37.0) - RIGHT_CUTOUT_W/2, BASE_THICKNESS + STANDOFF_H + 3))
cradle = cradle.cut(right_cut)

# 3.7 Rear Glue Interlock Channels (Waffle Grid on Z = 0)
for gx in [x for x in range(int(-outer_w/2 + 6), int(outer_w/2 - 6), 7)]:
    slot = Part.makeBox(GLUE_GROOVE_W, outer_h, GLUE_GROOVE_D + 0.1)
    slot.translate(App.Vector(gx - GLUE_GROOVE_W/2, -outer_h/2, -0.05))
    cradle = cradle.cut(slot)

for gy in [y for y in range(int(-outer_h/2 + 6), int(outer_h/2 - 6), 7)]:
    slot = Part.makeBox(outer_w, GLUE_GROOVE_W, GLUE_GROOVE_D + 0.1)
    slot.translate(App.Vector(-outer_w/2, gy - GLUE_GROOVE_W/2, -0.05))
    cradle = cradle.cut(slot)

# 3.8 Glue Rivet Mushroom Holes
rivet_pts = [
    (-outer_w/2 + 4, -outer_h/2 + 4),
    ( outer_w/2 - 4, -outer_h/2 + 4),
    (-outer_w/2 + 4,  outer_h/2 - 4),
    ( outer_w/2 - 4,  outer_h/2 - 4),
    (-outer_w/2 + 4,  0),
    ( outer_w/2 - 4,  0)
]
for rx, ry in rivet_pts:
    rhole = Part.makeCylinder(GLUE_RIVET_DIA/2, BASE_THICKNESS + 2, App.Vector(rx, ry, -1), App.Vector(0, 0, 1))
    cradle = cradle.cut(rhole)

# Add Base to FreeCAD Document
base_obj = doc.addObject("Part::Feature", "FreeCAD_Enclosure_Base")
base_obj.Shape = cradle
configure_view(base_obj, (0.2, 0.4, 0.8))  # Industrial blue
print("[INFO] Created FreeCAD_Enclosure_Base feature.")

# =============================================================================
# 4. BUILD SPLASH COVER (LID)
# =============================================================================
LID_LIP_H = 3.5
LID_WALL = 2.0
LID_CLEARANCE = 0.35  # Snap-on printing tolerance

lid_inner_w = outer_w + 2 * LID_CLEARANCE
lid_inner_h = outer_h + 2 * LID_CLEARANCE
lid_outer_w = lid_inner_w + 2 * LID_WALL
lid_outer_h = lid_inner_h + 2 * LID_WALL
lid_depth = 5.0 + LID_LIP_H

lid_outer = Part.makeBox(lid_outer_w, lid_outer_h, lid_depth)
lid_outer.translate(App.Vector(-lid_outer_w/2, -lid_outer_h/2, 0))

lid_inner = Part.makeBox(lid_inner_w, lid_inner_h, LID_LIP_H + 1)
lid_inner.translate(App.Vector(-lid_inner_w/2, -lid_inner_h/2, 2.0))

lid_shape = lid_outer.cut(lid_inner)

# Wire exit slots on lid top
lid_wire_slot = Part.makeBox(TOP_CUTOUT_W - 10, LID_WALL + 4, 3.5)
lid_wire_slot.translate(App.Vector(-(TOP_CUTOUT_W - 10)/2, lid_outer_h/2 - LID_WALL - 2, 0))
lid_shape = lid_shape.cut(lid_wire_slot)

# Position Lid hovering above base for clear exploded visualization
lid_obj = doc.addObject("Part::Feature", "FreeCAD_Enclosure_Lid")
lid_obj.Shape = lid_shape
lid_obj.Placement.Base = App.Vector(0, 0, total_h + 12.0)  # Exploded 12mm above base
configure_view(lid_obj, (0.85, 0.85, 0.85), transparency=20)
print("[INFO] Created FreeCAD_Enclosure_Lid feature.")

# =============================================================================
# 5. IMPORT PCB MODEL (KiCadStepUp or Direct STEP)
# =============================================================================
pcb_loaded = False
pcb_obj = None

if has_ksu and os.path.exists(KICAD_PCB_PATH):
    try:
        print(f"[INFO] Importing PCB via KiCadStepUp: {KICAD_PCB_PATH}")
        ksu.open(KICAD_PCB_PATH, insert=True)
        pcb_loaded = True
        print("[INFO] PCB successfully loaded via KiCadStepUp.")
    except Exception as e:
        print(f"[WARN] KiCadStepUp import failed: {e}. Falling back to STEP file.")

if not pcb_loaded and os.path.exists(STEP_PCB_PATH):
    try:
        print(f"[INFO] Loading PCB STEP assembly: {STEP_PCB_PATH}")
        pcb_shape = Part.read(STEP_PCB_PATH)
        pcb_obj = doc.addObject("Part::Feature", "FreeCAD_Plant_ESP32_PCBA")
        pcb_obj.Shape = pcb_shape
        # Center PCB and place on standoffs
        pcb_obj.Placement.Base = App.Vector(-PCB_W/2, -PCB_H/2, BASE_THICKNESS + STANDOFF_H)
        configure_view(pcb_obj, (0.05, 0.45, 0.20))  # PCB green
        pcb_loaded = True
        print("[INFO] Successfully placed PCBA into enclosure assembly.")
    except Exception as e:
        print(f"[WARN] STEP import failed: {e}")

# =============================================================================
# 6. EXPORT MESHES & STEP ASSEMBLED GEOMETRY
# =============================================================================
doc.recompute()

out_base_step     = os.path.join(SCRIPT_DIR, "freecad_enclosure_base.step")
out_base_stl      = os.path.join(SCRIPT_DIR, "freecad_enclosure_base.stl")
out_lid_step      = os.path.join(SCRIPT_DIR, "freecad_enclosure_lid.step")
out_lid_stl       = os.path.join(SCRIPT_DIR, "freecad_enclosure_lid.stl")
out_assembly_step = os.path.join(SCRIPT_DIR, "freecad_enclosure_assembly.step")
out_fcstd         = os.path.join(SCRIPT_DIR, "freecad_enclosure_assembly.FCStd")

# Save FreeCAD Document with full GUI and ViewProvider data
doc.saveAs(out_fcstd)
print(f"[SUCCESS] Saved FreeCAD Document with GuiDocument: {out_fcstd}")

# Export individual parts
Part.export([base_obj], out_base_step)
Part.export([base_obj], out_base_stl)
Part.export([lid_obj], out_lid_step)
Part.export([lid_obj], out_lid_stl)

# Export complete assembly
export_list = [base_obj, lid_obj]
if pcb_obj is not None:
    export_list.append(pcb_obj)
Part.export(export_list, out_assembly_step)

print(f"[SUCCESS] Exported Base STEP:     {out_base_step}")
print(f"[SUCCESS] Exported Base STL:      {out_base_stl}")
print(f"[SUCCESS] Exported Lid STEP:      {out_lid_step}")
print(f"[SUCCESS] Exported Lid STL:       {out_lid_stl}")
print(f"[SUCCESS] Exported Assembly STEP: {out_assembly_step}")
print("[DONE] FreeCAD enclosure generation complete.")

if os.environ.get('QT_QPA_PLATFORM') == 'offscreen':
    os._exit(0)
else:
    sys.exit(0)
