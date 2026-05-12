bl_info = {
    "name": "Sokoban 3D Tools",
    "author": "Sokoban3D",
    "version": (1, 0, 0),
    "blender": (3, 6, 0),
    "location": "View3D > Sidebar > Sokoban",
    "description": "Export models and metadata for Sokoban 3D",
    "category": "Import-Export",
}

import bpy
import os
from bpy.props import (
    StringProperty,
    BoolProperty,
    EnumProperty,
    FloatProperty,
)
from bpy.types import (
    Panel,
    Operator,
    PropertyGroup,
)

from . import exporter


# ============================================================
#  Per-object custom properties
# ============================================================

class SokobanObjectProperties(PropertyGroup):
    object_type: EnumProperty(
        name="Object Type",
        description="Type of this object in the game",
        items=[
            ("None", "None", "No object"),
            ("Wall", "Wall", "Wall"),
            ("CrackedWall", "Cracked Wall", "Cracked wall"),
            ("Box", "Box", "Standard box"),
            ("HeavyBox", "Heavy Box", "Heavy box"),
            ("FragileBox", "Fragile Box", "Fragile box"),
            ("IceBox", "Ice Box", "Ice box"),
            ("Bomb", "Bomb", "Bomb"),
            ("Spring", "Spring", "Spring"),
            ("Rock", "Rock", "Rock"),
            ("Player", "Player", "Player character"),
            ("Key", "Key", "Key"),
            ("Gate", "Gate", "Gate / Door"),
            ("Switch", "Switch", "Switch"),
            ("Pillar", "Pillar", "Pillar"),
            ("Mirror", "Mirror", "Mirror"),
            ("Magnet", "Magnet", "Magnet"),
            ("Spikes", "Spikes", "Spikes"),
        ],
        default="None",
    )

    scene_theme: EnumProperty(
        name="Scene Theme",
        description="Which scene theme this model belongs to",
        items=[
            ("common", "Common", "Shared across all scenes"),
            ("forest", "Forest", "Forest scene"),
            ("volcano", "Volcano", "Volcano scene"),
            ("ice_palace", "Ice Palace", "Ice palace scene"),
            ("sky_temple", "Sky Temple", "Sky temple scene"),
            ("ruins", "Ruins", "Ruins scene"),
            ("void", "Void", "Void scene"),
        ],
        default="common",
    )

    category: EnumProperty(
        name="Category",
        description="Asset category",
        items=[
            ("wall", "Wall", "Wall / obstacle"),
            ("box", "Box", "Pushable box"),
            ("floor", "Floor", "Floor tile"),
            ("item", "Item", "Functional item"),
            ("player", "Player", "Player model"),
        ],
        default="wall",
    )

    is_pushable: BoolProperty(
        name="Pushable",
        description="Whether this object can be pushed by the player",
        default=False,
    )

    model_height: FloatProperty(
        name="Model Height",
        description="Height of the model in meters",
        default=2.0,
        min=0.1,
        max=10.0,
    )

    display_name: StringProperty(
        name="Display Name",
        description="Human-readable display name",
        default="",
    )


# ============================================================
#  Export settings (scene-level)
# ============================================================

class SokobanExportSettings(PropertyGroup):
    export_path: StringProperty(
        name="Export Directory",
        description="Root directory for exported assets (assets/models/)",
        default="",
        subtype="DIR_PATH",
    )

    auto_meta: BoolProperty(
        name="Auto Generate meta.ron",
        description="Automatically generate meta.ron alongside each glb",
        default=True,
    )


# ============================================================
#  Operators
# ============================================================

class SOKOBAN_OT_export_selected(Operator):
    bl_idname = "sokoban.export_selected"
    bl_label = "Export Selected"
    bl_description = "Export selected object(s) as glb with meta.ron"

    def execute(self, context):
        settings = context.scene.sokoban_export
        selected = context.selected_objects

        if not selected:
            self.report({"WARNING"}, "No objects selected")
            return {"CANCELLED"}

        if not settings.export_path:
            self.report({"ERROR"}, "Set export directory first")
            return {"CANCELLED"}

        exported = 0
        for obj in selected:
            props = obj.sokoban
            theme = props.scene_theme
            category = props.category
            obj_type = props.object_type
            display_name = props.display_name or obj.name

            result = exporter.export_object(
                obj,
                export_dir=settings.export_path,
                scene_theme=theme,
                category=category,
                object_type=obj_type,
                display_name=display_name,
                is_pushable=props.is_pushable,
                model_height=props.model_height,
                auto_meta=settings.auto_meta,
            )
            if result:
                exported += 1

        self.report({"INFO"}, f"Exported {exported} object(s)")
        return {"FINISHED"}


class SOKOBAN_OT_export_batch(Operator):
    bl_idname = "sokoban.export_batch"
    bl_label = "Batch Export"
    bl_description = "Export all objects in the sokoban_export collection"

    def execute(self, context):
        settings = context.scene.sokoban_export

        if not settings.export_path:
            self.report({"ERROR"}, "Set export directory first")
            return {"CANCELLED"}

        collection = bpy.data.collections.get("sokoban_export")
        if not collection:
            self.report({"WARNING"}, "No 'sokoban_export' collection found")
            return {"CANCELLED"}

        exported = 0
        for obj in collection.objects:
            if obj.type != "MESH":
                continue
            props = obj.sokoban
            theme = props.scene_theme
            category = props.category
            obj_type = props.object_type
            display_name = props.display_name or obj.name

            result = exporter.export_object(
                obj,
                export_dir=settings.export_path,
                scene_theme=theme,
                category=category,
                object_type=obj_type,
                display_name=display_name,
                is_pushable=props.is_pushable,
                model_height=props.model_height,
                auto_meta=settings.auto_meta,
            )
            if result:
                exported += 1

        self.report({"INFO"}, f"Batch exported {exported} object(s)")
        return {"FINISHED"}


class SOKOBAN_OT_check_materials(Operator):
    bl_idname = "sokoban.check_materials"
    bl_label = "Check Materials"
    bl_description = "Check selected objects for material compliance"

    def execute(self, context):
        selected = context.selected_objects
        if not selected:
            self.report({"WARNING"}, "No objects selected")
            return {"CANCELLED"}

        issues = []
        for obj in selected:
            if obj.type != "MESH":
                continue
            obj_issues = exporter.check_material_issues(obj)
            for issue in obj_issues:
                issues.append(f"{obj.name}: {issue}")

        if issues:
            for issue in issues:
                self.report({"WARNING"}, issue)
        else:
            self.report({"INFO"}, "All materials OK")

        return {"FINISHED"}


class SOKOBAN_OT_toggle_grid(Operator):
    bl_idname = "sokoban.toggle_grid"
    bl_label = "Toggle Grid"
    bl_description = "Show/hide 2m x 2m reference grid"

    _grid_obj_name = "SokobanRefGrid"

    def execute(self, context):
        existing = bpy.data.objects.get(self._grid_obj_name)
        if existing:
            bpy.data.objects.remove(existing, do_unlink=True)
            self.report({"INFO"}, "Grid removed")
        else:
            exporter.create_reference_grid(context, self._grid_obj_name)
            self.report({"INFO"}, "Grid created (2m x 2m cells)")
        return {"FINISHED"}


# ============================================================
#  Panels
# ============================================================

class SOKOBAN_PT_main_panel(Panel):
    bl_label = "Sokoban 3D"
    bl_idname = "SOKOBAN_PT_main_panel"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Sokoban"

    def draw(self, context):
        layout = self.layout
        settings = context.scene.sokoban_export

        # Export path
        layout.prop(settings, "export_path")
        layout.prop(settings, "auto_meta")

        layout.separator()

        # Export buttons
        row = layout.row(align=True)
        row.scale_y = 1.4
        row.operator("sokoban.export_selected", icon="EXPORT")
        row.operator("sokoban.export_batch", icon="FILE_FOLDER")

        layout.separator()

        # Tools
        row = layout.row(align=True)
        row.operator("sokoban.check_materials", icon="MATERIAL")
        row.operator("sokoban.toggle_grid", icon="GRID")


class SOKOBAN_PT_object_panel(Panel):
    bl_label = "Object Properties"
    bl_idname = "SOKOBAN_PT_object_panel"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Sokoban"
    bl_parent_id = "SOKOBAN_PT_main_panel"

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def draw(self, context):
        layout = self.layout
        obj = context.active_object
        props = obj.sokoban

        layout.prop(props, "object_type")
        layout.prop(props, "scene_theme")
        layout.prop(props, "category")
        layout.prop(props, "is_pushable")
        layout.prop(props, "model_height")
        layout.prop(props, "display_name")

        # Preview: expected filename
        if props.object_type != "None":
            layout.separator()
            name = props.display_name or obj.name
            safe_name = name.lower().replace(" ", "_")
            filename = f"{props.scene_theme}_{props.category}_{safe_name}"
            layout.label(text=f"→ {filename}.glb", icon="FILE")


# ============================================================
#  Registration
# ============================================================

classes = (
    SokobanObjectProperties,
    SokobanExportSettings,
    SOKOBAN_OT_export_selected,
    SOKOBAN_OT_export_batch,
    SOKOBAN_OT_check_materials,
    SOKOBAN_OT_toggle_grid,
    SOKOBAN_PT_main_panel,
    SOKOBAN_PT_object_panel,
)


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.Object.sokoban = bpy.props.PointerProperty(type=SokobanObjectProperties)
    bpy.types.Scene.sokoban_export = bpy.props.PointerProperty(type=SokobanExportSettings)


def unregister():
    del bpy.types.Scene.sokoban_export
    del bpy.types.Object.sokoban
    for cls in reversed(classes):
        bpy.utils.unregister_class(cls)


if __name__ == "__main__":
    register()
