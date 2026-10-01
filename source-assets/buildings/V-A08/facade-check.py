"""Validate the actual V-A08 facade GLB, source-contract metadata and approach volumes."""
import hashlib
import json
import math
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(ROOT/"tools"))
from glb import read_glb, require, values
MATERIALS=ROOT/"source-assets/environment-kit/materials"
sys.path.insert(0,str(MATERIALS))


def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def dot(a,b):return sum(x*y for x,y in zip(a,b))
def length(a):return math.sqrt(dot(a,a))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    runtime=ROOT/"game/assets/environment/buildings/v-a08-facade.glb"
    document,binary=read_glb(runtime)
    require(document.get("scene")==0 and len(document["scenes"])==1 and document["scenes"][0]["nodes"]==[0],"expected Scene0 single root")
    require(len(document["nodes"])==1 and len(document["meshes"])==1,"unexpected object hierarchy")
    node=document["nodes"][0]
    require(not node.get("children") and "matrix" not in node,"unexpected parent matrix")
    require(node.get("translation",[0,0,0])==[0,0,0] and node.get("rotation",[0,0,0,1])==[0,0,0,1] and node.get("scale",[1,1,1])==[1,1,1],"runtime placement must use source anchor once")
    require(not any(document.get(key) for key in ("skins","animations","cameras","extensionsUsed")),"unexpected rig, animation, camera or material extension")
    require(len(document["materials"])<8,"facade exceeds 7-material budget")
    roles={m["name"]:m for m in document["materials"]}
    require(set(roles)=={"Plaster","Concrete","Metal","Glazing","WarmPanel","Ochre"},"material roles changed")
    require(len(document.get("images",[]))==4 and all("bufferView" in i and "uri" not in i for i in document["images"]),"two original colors and two shared baked normals must be embedded")
    source_files={};derived_normals={}
    registered=json.loads((ROOT/"source-assets/environment-kit/asset-manifest.json").read_text())["files"]
    for role,stem in (("Plaster","Plaster001"),("Concrete","Concrete034")):
        material=roles[role]
        for suffix,texture_info in (("Color",material["pbrMetallicRoughness"]["baseColorTexture"]),("NormalGL",material["normalTexture"])):
            image=document["images"][document["textures"][texture_info["index"]]["source"]]
            view=document["bufferViews"][image["bufferView"]]
            image_bytes=binary[view.get("byteOffset",0):view.get("byteOffset",0)+view["byteLength"]]
            source=ROOT/f"source-assets/environment-kit/materials/{stem}_1K-JPG_{suffix}.jpg"
            recorded=next((record for record in registered if record["source"]==str(source.relative_to(ROOT/"source-assets/environment-kit"))),None)
            require(recorded is not None and recorded["sha256"]==sha(source) and recorded.get("license")=="CC0-1.0","texture differs from AST-003 license/hash record")
            if suffix=="Color":
                require(hashlib.sha256(image_bytes).hexdigest()==sha(source),f"{role} changed original color bytes")
            else:
                derived=MATERIALS/f"{stem}-NormalGL-scale028.png"
                require(image["mimeType"]=="image/png" and hashlib.sha256(image_bytes).hexdigest()==sha(derived),"GLB normal differs from shared PNG")
                require(texture_info.get("scale",1)==1,"baked normal must use glTF scale 1")
                if "--source" not in sys.argv:
                    from PIL import Image
                    from bake_normals import baked_normal
                    expected=baked_normal(source,.28)
                    with Image.open(derived) as actual:
                        require(actual.mode==expected.mode and actual.size==expected.size and actual.tobytes()==expected.tobytes(),"normal bake differs from xy*.28, z unchanged, normalize")
                derived_normals[str(derived.relative_to(ROOT))]={"sha256":sha(derived),"source":str(source.relative_to(ROOT)),"source_sha256":sha(source),"baked_scale":.28,"gltf_scale":1}
            source_files[str(source.relative_to(ROOT))]=sha(source)
    all_positions=[];triangles=0;uv_ratio=[];triangle_list=[]
    repeats={"Plaster":(2,2),"Concrete":(1.1,.55)}
    for primitive in document["meshes"][0]["primitives"]:
        require(primitive.get("mode",4)==4,"nontriangle primitive")
        attr=primitive["attributes"]
        require({"POSITION","NORMAL","TEXCOORD_0"}<=attr.keys(),"missing positions/normals/UVs")
        read=lambda i:values(document,binary,document["accessors"][i])
        ps=read(attr["POSITION"]);ns=read(attr["NORMAL"]);uv=read(attr["TEXCOORD_0"])
        ids=[r[0] for r in read(primitive["indices"])]
        require(len(ids)%3==0 and all(0<=i<len(ps) for i in ids),"invalid indices")
        require(all(math.isfinite(v) for rows in (ps,ns,uv) for row in rows for v in row),"nonfinite mesh attribute")
        require(all(abs(length(n)-1)<1e-5 for n in ns),"nonunit normal")
        material=document["materials"][primitive["material"]]["name"]
        repeat=repeats.get(material,(1,1))
        for start in range(0,len(ids),3):
            a,b,c=ids[start:start+3];normal=cross(sub(ps[b],ps[a]),sub(ps[c],ps[a]));area=length(normal)
            require(area>1e-8,"degenerate exported triangle")
            require(all(dot(normal,ns[i])/area>.999 for i in (a,b,c)),"normal/winding mismatch")
            for i,j in ((a,b),(b,c),(c,a)):
                metric=length(sub(ps[j],ps[i]));mapped=length(tuple((uv[j][k]-uv[i][k])*repeat[k] for k in range(2)))
                ratio=mapped/metric
                require(abs(ratio-1)<.002,"UV stretch beyond floating-point tolerance")
                uv_ratio.append(ratio)
            triangle_list.append([ps[a],ps[b],ps[c]])
        triangles+=len(ids)//3;all_positions.extend(ps)
    require(0<triangles<12000,"facade triangle budget exceeded")
    bounds=[[min(p[i] for p in all_positions),max(p[i] for p in all_positions)] for i in range(3)]
    expected=[[-9.657,10.35],[0,10.06],[-6.755,7.157]]
    require(all(abs(v-e)<1e-4 for row,target in zip(bounds,expected) for v,e in zip(row,target)),f"unexpected world-facing envelope: {bounds}")
    # Coordinates in GLB Y-up, outside the closed door plane; roof/canopies stay above headroom
    volumes={"public_door_approach":([9.30,.02,-.9],[12,2.60,.9]),
             "resident_door_approach":([9.30,.02,3.8],[12,2.60,5.2]),
             "service_door_approach":([-11,.02,-.2],[-9.30,2.60,1.2])}
    for name,(lo,hi) in volumes.items():
        hits=[tri for tri in triangle_list if all(max(p[i] for p in tri)>lo[i]+1e-6 and min(p[i] for p in tri)<hi[i]-1e-6 for i in range(3))]
        require(not hits,f"{name} is occupied by {len(hits)} imported triangles")
    m=json.loads((ROOT/"source-assets/district-map/district.json").read_text())
    building=next(b for b in m["buildings"] if b["id"]=="V-A08")
    require(building["polygon"]==[[70,271],[88,271],[88,284],[70,284]] and building["elevation"]==30.021515 and building["height"]==10.4,"building source contract changed")
    require([m["nodes"][e["node"]] for e in building["design"]["entries"]]==[[88,277.5,30.021515],[88,273,30.021515],[70,277,30.021515]],"door nodes changed")
    report={"status":"PASS","triangles":triangles,"materials":len(roles),"primitives":len(document["meshes"][0]["primitives"]),"images":len(document["images"]),"runtime_bounds_xyz":bounds,"uv_metric_edge_ratio_range":[min(uv_ratio),max(uv_ratio)],"source_sha256":sha(HERE/"facade.blend"),"runtime_sha256":sha(runtime),"runtime_bytes":runtime.stat().st_size,"texture_sources":source_files,"clear_approach_volumes":volumes,"anchor_map_xyz":[79,277.5,30.021515],"runtime_validation":"NOT RUN by asset producer; closed door planes require approach tests to end outside the wall"}
    report["derived_normals"]=derived_normals
    report["normal_pixel_check"]="NOT RUN in --source mode; run system Python for pixels" if "--source" in sys.argv else "PASS"
    if "--source" in sys.argv:
        import bpy
        import bmesh
        bpy.ops.wm.open_mainfile(filepath=str(HERE/"facade.blend"))
        collection=bpy.data.collections["V_A08_EXPORT_Facade"]
        objects=list(collection.objects)
        require(len(objects)==len(bpy.context.scene.objects),"preview context leaked into source scene")
        require(bpy.context.scene["owner_id"]=="V-A08" and list(bpy.context.scene["pivot_map_xyz"])==[79,277.5,30.021515],"source origin metadata differs")
        require(json.loads(bpy.context.scene["source_building"])==building,"editable source was built for different map contract")
        require(json.loads(bpy.context.scene["door_nodes"])=={e["node"]:m["nodes"][e["node"]] for e in building["design"]["entries"]},"source doors differ from map")
        source_triangles=0
        for obj in objects:
            require(obj.type=="MESH" and not obj.modifiers and not obj.constraints and not obj.animation_data,"source should remain editable static mesh")
            require(tuple(obj.location)==(0,0,0) and tuple(obj.rotation_euler)==(0,0,0) and tuple(obj.scale)==(1,1,1),"source group moved from shared origin")
            require(obj.data.uv_layers.active is not None,"source group lacks UVs")
            require(all(math.isfinite(v) for vertex in obj.data.vertices for v in vertex.co),"source has nonfinite vertex")
            bm=bmesh.new();bm.from_mesh(obj.data)
            require(all(edge.is_manifold for edge in bm.edges),"source cuboid has open edge")
            require(all(face.calc_area()>1e-8 for face in bm.faces),"source has degenerate face")
            require(bm.calc_volume(signed=True)>0,"source group winding is inward")
            bm.free()
            obj.data.calc_loop_triangles();source_triangles+=len(obj.data.loop_triangles)
        require(source_triangles==triangles,"editable source and exported triangle counts differ")
        images=[image for image in bpy.data.images if image.type=="IMAGE" and image.size[0]>0]
        require(len(images)==4 and all(image.packed_file for image in images),"source must contain four packed textures")
        for role,stem in (("Plaster","Plaster001"),("Concrete","Concrete034")):
            shader=bpy.data.materials[role].node_tree.nodes["Principled BSDF"]
            require(len(shader.inputs["Normal"].links)==1,"source normal must be bound to shader")
            normal=shader.inputs["Normal"].links[0].from_node
            require(normal.type=="NORMAL_MAP" and not normal.inputs["Strength"].is_linked and normal.inputs["Strength"].default_value==1,"source normal must use Strength 1")
            require(len(normal.inputs["Color"].links)==1,"source normal must have one image input")
            texture=normal.inputs["Color"].links[0].from_node
            require(texture.type=="TEX_IMAGE" and texture.image and texture.image.packed_file,"source normal must use a packed image")
            image=texture.image;derived=MATERIALS/f"{stem}-NormalGL-scale028.png"
            require(image.colorspace_settings.name=="Non-Color" and Path(bpy.path.abspath(image.filepath)).resolve()==derived.resolve(),"source normal must reference shared Non-Color PNG")
            require(hashlib.sha256(image.packed_file.data).hexdigest()==sha(derived),"source packed normal differs from shared PNG")
        report["blender_source"]={"status":"PASS","version":bpy.app.version_string,"editable_mesh_groups":len(objects),"triangles":source_triangles,"packed_images":len(images),"windows":bpy.context.scene["window_count"]}
    print(json.dumps(report,indent=2))


if __name__=="__main__":main()
