#!/usr/bin/env python3
"""Places Chapter 1 content in assets/world.ldtk.

Idempotent: it (re)defines the entity types it needs and replaces every
Npc / Trigger / Object / extra-house instance with the tables below.
Run from the repository root:  python3 tools/build_ch1_map.py
Then open the map in LDtk 1.5.x (or run the game and press F2) to check.
"""

import json
import uuid
from pathlib import Path

WORLD = Path("assets/world.ldtk")
GRID = 16

# --- placement tables -------------------------------------------------------
# NPC / object positions are the bottom-centre (pivot 0.5, 1) in LDtk pixels.
# Trigger rectangles are (left, top, width, height).

NPCS = {
    "Village": [
        ("ong_mac_day", 280, 200), ("lien_day", 296, 256), ("to_dai_son", 744, 548),
        ("ly_duc_day", 600, 352), ("be_dau_day", 424, 544), ("thim_ba_day", 760, 212),
        ("thim_ba_raid", 568, 560), ("be_dau_raid", 312, 590), ("hac_y_raid", 470, 400),
        ("ong_mac_raid", 232, 232), ("do_cuong_raid", 292, 262),
        ("ly_duc_dawn", 520, 470), ("thim_ba_dawn", 600, 480), ("be_dau_dawn", 636, 480),
    ],
    "Forest": [("vo_trang", 504, 384), ("hoang_khai", 280, 512)],
    "Snowfield": [("tinh_an", 790, 150), ("lao_moc", 376, 496)],
    "Beach": [("chu_nam", 200, 320), ("lao_ha", 808, 240), ("lien_dusk", 480, 252)],
}

# (id, x, y, w, h) — w/h match the object's sprite size.
OBJECTS = {
    "Village": [
        ("ch1_ban_go", 316, 176, 32, 24), ("ch1_mo_ong_mac", 160, 232, 32, 32),
        ("ch1_lenh_bai", 300, 292, 16, 16),
        ("ch1_lua_1", 206, 176, 16, 16), ("ch1_lua_2", 700, 306, 16, 16),
        ("ch1_lua_3", 830, 514, 16, 16), ("ch1_lua_4", 790, 178, 16, 16),
        ("ch1_lua_5", 420, 300, 16, 16),
    ],
    "Forest": [
        ("ch1_herb_1", 250, 420, 16, 16), ("ch1_herb_2", 480, 300, 16, 16),
        ("ch1_herb_3", 830, 300, 16, 16), ("ch1_herb_4", 168, 590, 16, 16),
        ("ch1_xac_vo_trang", 540, 404, 32, 16),
    ],
    "Snowfield": [("ch1_mieu_son_than", 850, 120, 48, 48)],
    "Beach": [
        ("ch1_dieu_mac", 634, 198, 16, 16), ("ch1_go_dao_troi", 300, 334, 16, 16),
        ("ch1_hoa_dang_1", 440, 340, 16, 16), ("ch1_hoa_dang_2", 520, 342, 16, 16),
        ("ch1_hoa_dang_3", 584, 340, 16, 16),
    ],
}

TRIGGERS = {
    "Village": [
        ("ch1_raid_enter", 448, 560, 160, 48),
        ("ch1_hac_y_zone", 64, 376, 848, 24),
        ("ch1_raid_block_west", 20, 48, 30, 272),
        ("ch1_dusk_block_west", 20, 48, 30, 272),
        ("ch1_raid_block_south", 448, 606, 160, 16),
    ],
    "Forest": [
        ("ch1_dau_giay", 640, 160, 64, 64),
        ("ch1_da_tru_zone", 256, 160, 64, 64),
        ("ch1_lang_dem_zone", 840, 48, 104, 272),
        ("ch1_forest_block_east", 920, 48, 24, 272),
    ],
}

# Extra houses: (entity identifier, x, y)
HOUSES = {
    "Village": [("HouseSmall", 232, 176), ("HouseSmallPurple", 808, 176)],
}

PLAYER_START = {"Village": (232, 204)}

# Warp arrival tweaks: (level, to_level) -> (to_x, to_y)
WARP_ARRIVALS = {("Forest", "Village"): (64, 192)}

EXTRA_HOUSE_DEFS = {
    "HouseSmall": (480, 144, 80, 80, "#6FA86A"),
    "HouseSmallPurple": (480, 256, 80, 80, "#7B6FB0"),
    "HouseBig": (560, 112, 112, 112, "#7B6FB0"),
}


def next_uid(world):
    uid = world["nextUid"]
    world["nextUid"] += 1
    return uid


def string_field_def(world, identifier):
    return {
        "identifier": identifier, "doc": None, "__type": "String", "uid": next_uid(world),
        "type": "F_String", "isArray": False, "canBeNull": False, "arrayMinLength": None,
        "arrayMaxLength": None, "editorDisplayMode": "ValueOnly", "editorDisplayScale": 1,
        "editorDisplayPos": "Above", "editorLinkStyle": "StraightArrow", "editorDisplayColor": None,
        "editorAlwaysShow": False, "editorShowInWorld": True, "editorCutLongValues": True,
        "editorTextSuffix": None, "editorTextPrefix": None, "useForSmartColor": False,
        "exportToToc": False, "searchable": True, "min": None, "max": None, "regex": None,
        "acceptFileTypes": None, "defaultOverride": {"id": "V_String", "params": [""]},
        "textLanguageMode": None, "symmetricalRef": False, "autoChainRef": True,
        "allowOutOfLevelRef": True, "allowedRefs": "OnlySame", "allowedRefsEntityUid": None,
        "allowedRefTags": [], "tilesetUid": None,
    }


def entity_def(world, template, identifier, **overrides):
    d = json.loads(json.dumps(template))
    d.update(identifier=identifier, uid=next_uid(world), fieldDefs=[])
    d.update(overrides)
    return d


def ensure_defs(world):
    defs = world["defs"]["entities"]
    by_id = {d["identifier"]: d for d in defs}
    warp, house, npc = by_id["Warp"], by_id["House"], by_id["Npc"]

    # Npc: a single `id` field referencing data/npcs.
    if [f["identifier"] for f in npc["fieldDefs"]] != ["id"]:
        npc["fieldDefs"] = [string_field_def(world, "id")]

    if "Trigger" not in by_id:
        d = entity_def(world, warp, "Trigger", color="#E05555", width=32, height=32)
        d["fieldDefs"] = [string_field_def(world, "id")]
        defs.append(d)
    if "Object" not in by_id:
        d = entity_def(world, warp, "Object", color="#55C0E0", width=16, height=16,
                       pivotX=0.5, pivotY=1, resizableX=True, resizableY=True)
        d["fieldDefs"] = [string_field_def(world, "id")]
        defs.append(d)
    for ident, (x, y, w, h, color) in EXTRA_HOUSE_DEFS.items():
        if ident not in by_id:
            d = entity_def(world, house, ident, color=color, width=w, height=h,
                           tileRect={"tilesetUid": 1, "x": x, "y": y, "w": w, "h": h})
            defs.append(d)
    return {d["identifier"]: d for d in defs}


def field_instance(field_def, value):
    return {
        "__identifier": field_def["identifier"], "__type": "String", "__value": value,
        "__tile": None, "defUid": field_def["uid"],
        "realEditorValues": [{"id": "V_String", "params": [value]}],
    }


def instance(defn, level, x, y, w=None, h=None, id_value=None):
    w = w or defn["width"]
    h = h or defn["height"]
    px, py = defn["pivotX"], defn["pivotY"]
    left, top = x - w * px, y - h * py
    inst = {
        "__identifier": defn["identifier"],
        "__grid": [int(left // GRID), int(top // GRID)],
        "__pivot": [px, py],
        "__tags": [],
        "__tile": defn.get("tileRect"),
        "__smartColor": defn["color"],
        "iid": str(uuid.uuid4()),
        "width": w,
        "height": h,
        "defUid": defn["uid"],
        "px": [x, y],
        "fieldInstances": [],
        "__worldX": level["worldX"] + x,
        "__worldY": level["worldY"] + y,
    }
    if id_value is not None:
        inst["fieldInstances"].append(field_instance(defn["fieldDefs"][0], id_value))
    return inst


def main():
    world = json.loads(WORLD.read_text())
    defs = ensure_defs(world)
    replaced = {"Npc", "Trigger", "Object", *EXTRA_HOUSE_DEFS}
    for level in world["levels"]:
        name = level["identifier"]
        layer = next(l for l in level["layerInstances"] if l["__type"] == "Entities")
        entities = [e for e in layer["entityInstances"] if e["__identifier"] not in replaced]
        for e in entities:
            if e["__identifier"] == "Player" and name in PLAYER_START:
                x, y = PLAYER_START[name]
                e.update(px=[x, y], __worldX=level["worldX"] + x, __worldY=level["worldY"] + y,
                         __grid=[(x - 16) // GRID, (y - 32) // GRID])
            if e["__identifier"] == "Warp":
                to_level = next(f["__value"] for f in e["fieldInstances"] if f["__identifier"] == "to_level")
                if (name, to_level) in WARP_ARRIVALS:
                    tx, ty = WARP_ARRIVALS[(name, to_level)]
                    for f in e["fieldInstances"]:
                        if f["__identifier"] == "to_x":
                            f["__value"] = tx
                            f["realEditorValues"] = [{"id": "V_Int", "params": [tx]}]
                        if f["__identifier"] == "to_y":
                            f["__value"] = ty
                            f["realEditorValues"] = [{"id": "V_Int", "params": [ty]}]
        for ident, x, y in HOUSES.get(name, []):
            entities.append(instance(defs[ident], level, x, y))
        for npc_id, x, y in NPCS.get(name, []):
            entities.append(instance(defs["Npc"], level, x, y, id_value=npc_id))
        for obj_id, x, y, w, h in OBJECTS.get(name, []):
            entities.append(instance(defs["Object"], level, x, y, w, h, id_value=obj_id))
        for trig_id, x, y, w, h in TRIGGERS.get(name, []):
            entities.append(instance(defs["Trigger"], level, x, y, w, h, id_value=trig_id))
        layer["entityInstances"] = entities
    WORLD.write_text(json.dumps(world, indent="\t", ensure_ascii=False) + "\n")
    print("world.ldtk updated; nextUid =", world["nextUid"])


if __name__ == "__main__":
    main()
