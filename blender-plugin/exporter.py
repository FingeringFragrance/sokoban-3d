import bpy
import os
import json
import math


# ============================================================
#  Export a single object
# ============================================================

def export_object(
    obj,
    export_dir: str,
    scene_theme: str,
    category: str,
    object_type: str,
    display_name: str,
    is_pushable: bool,
    model_height: float,
    auto_meta: bool = True,
) -> bool:
    """Export one Blender object as glb + meta.ron. Returns True on success."""

    # Build filename: {scene_theme}_{category}_{name}.glb
    safe_name = display_name.lower().replace(" ", "_")
    safe_name = "".join(c for c in safe_name if c.isalnum() or c == "_")
    filename = f"{scene_theme}_{category}_{safe_name}"

    # Target directory: {export_dir}/{scene_theme}/
    target_dir = os.path.join(export_dir, scene_theme)
    os.makedirs(target_dir, exist_ok=True)

    glb_path = os.path.join(target_dir, f"{filename}.glb")
    meta_path = os.path.join(target_dir, f"{filename}.meta.ron")

    # Select only this object
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj

    # Export glb
    try:
        bpy.ops.export_scene.gltf(
            filepath=glb_path,
            use_selection=True,
            export_format="GLB",
            export_apply=True,
            export_texcoords=True,
            export_normals=True,
            export_materials="EXPORT",
            export_animations=True,
            export_image_format="AUTO",
        )
    except Exception as e:
        print(f"[Sokoban] Export failed for {obj.name}: {e}")
        return False

    # Generate meta.ron
    if auto_meta:
        animations = []
        if obj.animation_data and obj.animation_data.action:
            for track in obj.animation_data.nla_tracks:
                for strip in track.strips:
                    animations.append(strip.name)
            # Also check the active action name
            if obj.animation_data.action.name not in animations:
                animations.insert(0, obj.animation_data.action.name)

        meta = {
            "model_path": f"{filename}.glb",
            "display_name": display_name,
            "display_name_key": f"obj.{scene_theme}.{safe_name}",
            "category": category,
            "object_type": object_type,
            "scene_theme": scene_theme,
            "model_height": round(model_height, 2),
            "is_pushable": is_pushable,
            "animations": animations,
        }

        # Write as RON-style (human-readable)
        ron_content = format_as_ron(meta)
        try:
            with open(meta_path, "w", encoding="utf-8") as f:
                f.write(ron_content)
        except Exception as e:
            print(f"[Sokoban] Failed to write meta.ron for {obj.name}: {e}")

    print(f"[Sokoban] Exported: {glb_path}")
    return True


def format_as_ron(data: dict) -> str:
    """Format a dict as a RON-like string matching the AssetMeta struct."""
    lines = []
    lines.append("(")
    for key, value in data.items():
        if isinstance(value, str):
            lines.append(f'    {key}: "{value}",')
        elif isinstance(value, bool):
            lines.append(f"    {key}: {'true' if value else 'false'},")
        elif isinstance(value, (int, float)):
            lines.append(f"    {key}: {value},")
        elif isinstance(value, list):
            items = ", ".join(f'"{v}"' for v in value)
            lines.append(f"    {key}: [{items}],")
    lines.append(")")
    return "\n".join(lines) + "\n"


# ============================================================
#  Material check
# ============================================================

VALID_TEXTURE_SIZES = {512, 1024}


def check_material_issues(obj) -> list:
    """Check an object's materials for common issues. Returns list of issue strings."""
    issues = []

    if not obj.data.materials:
        issues.append("No materials assigned")
        return issues

    for mat in obj.data.materials:
        if mat is None:
            issues.append("Empty material slot")
            continue

        if not mat.use_nodes:
            issues.append(f"'{mat.name}': not using nodes")
            continue

        # Check for Principled BSDF
        has_principled = False
        for node in mat.node_tree.nodes:
            if node.type == "BSDF_PRINCIPLED":
                has_principled = True
                break

        if not has_principled:
            issues.append(f"'{mat.name}': no Principled BSDF node")

        # Check texture sizes
        for node in mat.node_tree.nodes:
            if node.type == "TEX_IMAGE" and node.image:
                img = node.image
                w, h = img.size[0], img.size[1]
                if w > 0 and h > 0:
                    if w not in VALID_TEXTURE_SIZES or h not in VALID_TEXTURE_SIZES:
                        issues.append(
                            f"'{mat.name}' / '{img.name}': "
                            f"texture size {w}x{h} (expected 512 or 1024)"
                        )

    return issues


# ============================================================
#  Reference grid
# ============================================================

def create_reference_grid(context, name: str, cell_count: int = 10):
    """Create a 2m x 2m reference grid in the scene."""
    verts = []
    edges = []
    idx = 0
    cell = 2.0  # 1 grid cell = 2m in Blender

    for i in range(cell_count + 1):
        # Vertical line
        verts.append((i * cell, 0, 0))
        verts.append((i * cell, 0, cell_count * cell))
        edges.append((idx, idx + 1))
        idx += 2

        # Horizontal line
        verts.append((0, 0, i * cell))
        verts.append((cell_count * cell, 0, i * cell))
        edges.append((idx, idx + 1))
        idx += 2

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, edges, [])
    mesh.update()

    grid_obj = bpy.data.objects.new(name, mesh)
    context.collection.objects.link(grid_obj)

    # Make it non-selectable
    grid_obj.hide_select = True
    grid_obj.display_type = "WIRE"

    return grid_obj
