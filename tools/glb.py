"""Read the embedded, uncompressed GLB accessors used by N:SIDE asset checks."""

import json
import struct


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_glb(path):
    data = path.read_bytes()
    require(len(data) >= 28, f"{path}: truncated GLB")
    magic, version, length, json_size, kind = struct.unpack_from("<5I", data)
    require(magic == 0x46546C67 and version == 2 and length == len(data), f"{path}: invalid GLB header")
    require(kind == 0x4E4F534A and 20 + json_size + 8 <= length, f"{path}: invalid JSON chunk")
    document = json.loads(data[20:20 + json_size])
    require(document.get("asset", {}).get("version") == "2.0", "expected glTF 2.0 asset")
    require(not {"KHR_draco_mesh_compression", "EXT_meshopt_compression"} & set(document.get("extensionsUsed", [])), "compressed accessors are unsupported; export uncompressed GLB")
    binary_size, binary_kind = struct.unpack_from("<2I", data, 20 + json_size)
    require(binary_kind == 0x004E4942, f"{path}: expected embedded binary chunk")
    binary = data[28 + json_size:]
    require(len(binary) == binary_size, f"{path}: truncated binary chunk")
    require(len(document.get("buffers", [])) == 1 and "uri" not in document["buffers"][0], "expected one embedded GLB buffer")
    declared = document["buffers"][0]["byteLength"]
    require(0 <= len(binary) - declared <= 3, "binary length differs from declared buffer")
    return document, binary[:declared]


def values(document, binary, accessor):
    require("sparse" not in accessor, "sparse accessors are unsupported; export dense character accessors")
    require(isinstance(accessor["bufferView"], int) and 0 <= accessor["bufferView"] < len(document["bufferViews"]), "invalid accessor buffer view")
    view = document["bufferViews"][accessor["bufferView"]]
    require(view["buffer"] == 0, "expected embedded buffer 0")
    count = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}[accessor["type"]]
    component = accessor["componentType"]
    code = {5120: "b", 5121: "B", 5122: "h", 5123: "H", 5125: "I", 5126: "f"}[component]
    format_ = "<" + code * count
    size = struct.calcsize(format_)
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride", size)
    require(stride >= size and accessor["count"] > 0 and start >= view.get("byteOffset", 0) >= 0, "invalid accessor offset/stride/count")
    end = start + stride * (accessor["count"] - 1) + size
    require(end <= view.get("byteOffset", 0) + view["byteLength"] <= len(binary), "accessor exceeds its buffer view")
    rows = [struct.unpack_from(format_, binary, start + i * stride) for i in range(accessor["count"])]
    if accessor.get("normalized"):
        require(component in (5120, 5121, 5122, 5123), "invalid normalized component type")
        divisor = {5120: 127, 5121: 255, 5122: 32767, 5123: 65535}[component]
        rows = [tuple(max(-1.0, value / divisor) for value in row) for row in rows]
    return rows
