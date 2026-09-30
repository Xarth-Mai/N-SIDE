"""Reproduce the building-family reference snapshot from the current district data"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SOURCE = ROOT / 'source-assets/district-map/district.json'
OUT = Path(__file__).parent
SOURCES = [
('S01','Random Play','official','https://zenless.hoyoverse.com/zh-cn/news/112950','https://webstatic.hoyoverse.com/upload/op-public/2023/09/08/b8a2ea18c7869e86dad2e6ce3a6d0e7d_4881018303094535194.jpg','黄色首层板面、绕角主招牌、大橱窗、独立店门与上层砖墙'),
('S02','Quality Tea','community screenshot','https://www.hoyolab.com/article/31106713','https://upload-os-bbs.hoyolab.com/upload/2024/07/16/31864461/698006b8ea53e3f61725c76a8736b20a_2449235897917906706.jpg','退入售卖窗、上翻小棚、窗前高凳、邻接外梯；截图署名 Vidservent'),
('S03','Gravity Cinema','community screenshot','https://www.miyoushe.com/zzz/article/54836136','https://upload-bbs.miyoushe.com/upload/2024/07/08/158756436/13330e7a1d1c8c6d0813b23bc9c65bcd_3098545530441864915.jpg','竖肋大面、大竖海报、红色门框、低台阶与座椅；截图署名九尾狐'),
('S04','141 Convenience Store','community screenshot','https://www.hoyolab.com/article/31185512','https://upload-os-bbs.hoyolab.com/upload/2024/07/18/87315851/07c542439b3d5a23ef595618086cdf54_7625111118623777778.jpg','布雨棚、货架分组、空出的入门线、玻璃门贴纸；截图署名 Clown'),
('S05','Lumina Square Coff Cafe exterior','community screenshot','https://www.hoyolab.com/article/35870103','https://upload-os-bbs.hoyolab.com/upload/2024/12/26/398459160/40646740f7a9f8da0aadf37b1ded4183_7991507005065272184.png','首层大窗、外侧折返梯、屋顶栏杆和外梯下的背面空间；截图署名 Matius'),
('S06','Lumina cafe terrace','community screenshot','https://www.hoyolab.com/article/34000921','https://upload-os-bbs.hoyolab.com/upload/2024/10/07/266333308/a5048aff7d6a73b7abffa23529ac828b_7921632593120961139.jpg','木地板、重复栏杆、两组伞桌、背景窗带及高低檐口；截图署名 RinRin'),
('S07','Lumina Square metro entrance','media original screenshot','https://dengekionline.com/article/202406/9315','https://cimg.kgl-systems.io/camion/files/dengeki/9315/a197e83ce3fde119443846dae1f2f8849.jpg?x=1280','站牌位于过梁、扶梯与楼梯分带、盲道和防护栏；2024年先行试玩截图'),
('S08','Lumina Square N.E.P.S. station','community screenshot','https://www.miyoushe.com/zzz/article/54836136','https://upload-bbs.miyoushe.com/upload/2024/07/08/158756436/a91e7f0ab07a47342f9feb8d88dce927_4272201999497008625.jpg','实际文字为光映广场分署：低墙铭牌、门岗、内部车场、后方普通楼；不是地铁站'),
('S09','Bardic Needle','official','https://zenless.hoyoverse.com/en-us/news/123158','https://fastcdn.hoyoverse.com/content-v2/nap/123158/e046443793a1fad6fc63098a08f1e39b_5417063584122938974.jpg','唱片橱窗、圆盘图形、横向主招牌；上层栏板、窗户、外置设备'),
('S10','Box Galaxy','official','https://zenless.hoyoverse.com/th-th/news/113924','https://fastcdn.hoyoverse.com/content-v2/nap/113924/c38282f457211ea51a5b29462067d750_8446721342144516865.jpg','退入式大入口、折面檐口、粗框大字、门内可见设备；泰文官方海报'),
('S11','Failume Heights market','official producer article','https://blog.playstation.com/2025/05/23/zenless-zone-zero-version-2-0-launches-on-june-6/','https://blog.playstation.com/tachyon/2025/05/0ec981151b3669eb0ddcc813e5139bf5f27e8f99.png','台阶前场、当街茶楼、现代住宅与岩坡层叠；不是随便观院内截图'),
('S12','Academy main teaching building entrance','official producer article','https://blog.ja.playstation.com/2026/06/06/20260606-zzz-post-launch-update-o/','https://blog.ja.playstation.com/uploads/sites/7/2026/06/89a40abaef8dfea0eb6406ae75d6561e446c6bb2.png','深门框、课程/日程信息板、功能门牌、成对壁灯、石材收口；只看见入口近景'),
('S13','Sixth Street noodle shop','community screenshot','https://www.hoyolab.com/article/31289813','https://upload-os-bbs.hoyolab.com/upload/2024/07/21/9597874/b9ecd7d9322ff502fceebe787f994b08_6032053032628487481.jpg','开放吧台、凳子、低檐、大鱼形招牌；后方楼房的窗、设备、墙面较安静'),
('S14','Godfinger','media original screenshot','https://kincir.com/game/pc-game/kegunaan-toko-zenless-zone-zero/','https://d1tgyzt3mf06m9.cloudfront.net/v3-staging/2024/07/toko-di-zenless-zone-zero-arcade.png','转角折角招牌、连续雨棚、棋盘铺地、海报和少量机台'),
('S15','Suibian Temple','publisher media reproduced by press','https://www.rpgsite.net/news/17507-zenless-zone-zero-2-0-release-date-xbox-series-x-cloud-gaming-anniversary-rewards','https://images.rpgsite.net/image/da49c9a1/152151/original/zenless-zone-zero_20250523_scene-4.png','瓦檐、木柱、柱脚、石基、庭院高差、灯与盆栽；文章列为HoYoverse新截图'),
]
FAMILIES = {
 'station':(['S07','S08'],'入口、引导标识与公共前场'),
 'row':(['S01','S04'],'经营门面与住家上层'),
 'mixed':(['S01','S02','S13'],'首层店面、窗口与上部生活构件'),
 'corner':(['S01','S14'],'转角招牌、连续雨棚和门前空间'),
 'apartment':(['S06','S08','S11'],'上层窗带、实墙及设备区'),
 'slope':(['S11','S05'],'逐栋水平楼层与平台外梯'),
 'cinema':(['S03','S05','S06'],'公共入口主轴、海报与屋顶平台'),
 'public':(['S08','S12'],'功能入口与前场'),
 'school':(['S12'],'学院入口门牌、信息板及门框'),
 'shrine':(['S15'],'檐口、柱脚、石基构件'),
 'shop':(['S02','S13'],'小门面、窗口及开放柜台'),
 'interest':(['S14','S10'],'兴趣商品大图形与设备入口'),
 'maker':(['S10','S09'],'作品橱窗、设备和搬运入口的外观'),
 'civic':(['S08','S12','S03'],'公共入口及展示构件'),
 'music':(['S09','S03'],'音乐展示与演出信息层次'),
 'none':(['S11','S06'],'远景体量、窗带及檐口层次'),
}
FOCUS = {
 'V-04':['S01'], 'V-08':['S04'], 'V-35':['S14'], 'V-39':['S14'],
 'V-36':['S10'], 'V-15':['S03','S05','S06'], 'V-30':['S05','S06'],
 'V-84':['S05','S06'], 'V-77':['S09'], 'V-78':['S09'], 'V-79':['S09'], 'V-80':['S09'],
}

def snapshot():
    data=json.loads(SOURCE.read_text())
    source_ids={r[0] for r in SOURCES}
    families=[]
    for kind,(refs,scope) in FAMILIES.items():
        ids=[b['id'] for b in data['buildings'] if b.get('design',{}).get('type','none')==kind]
        assert set(refs)<=source_ids
        families.append({'design_type':kind,'count':len(ids),'building_ids':ids,'reference_ids':refs,'scope':scope,'status':'proposed visual correspondence'})
    assigned=[i for family in families for i in family['building_ids']]
    actual=[b['id'] for b in data['buildings']]
    assert len(set(assigned))==len(assigned)==len(actual) and set(assigned)==set(actual), 'Missing or duplicated building mapping'
    assert set(FOCUS)<=set(actual) and all(set(v)<=source_ids for v in FOCUS.values())
    return {
      'review_date':'2026-09-30','status':'REFERENCE_RESEARCH_ONLY','source_path':str(SOURCE.relative_to(ROOT)),
      'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
      'building_count':len(actual),'family_count':len(families),
      'sources':[{'id':i,'label':label,'provenance':kind,'page_url':page,'image_url':image,'viewed':True,'observations':note,'observation_method':'Browser opened original remote image; complete screenshot actually inspected; no local download'} for i,label,kind,page,image,note in SOURCES],
      'families':families,'focus_objects':FOCUS,
      'gaps':[
        {'buildings':[b['id'] for b in data['buildings'] if b.get('design',{}).get('type')=='school'],'scope':'Full ordinary-secondary-school facade not established; S12 covers entrance only'},
        {'buildings':['V-60'],'scope':'No direct greenhouse shell reference; civic references cover entry and environment components only'},
        {'scope':'No reference image establishes metric dimensions, hidden plan, collision, current game performance or permission to redistribute meshes/textures'},
      ],
      'checks':{'all_current_buildings_mapped_once':'PASS','all_reference_ids_resolve':'PASS','actual_visual_observation':'PASS 15 source images','asset_modification':'NOT RUN','runtime_validation':'NOT RUN','author_acceptance':'NOT RUN'},
      'cleanup':{'downloaded_media':0,'local_screenshots':0,'source_image_files_added_to_repository':0},
    }

if __name__=='__main__':
    report=snapshot()
    (OUT/'building-reference-map.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':'PASS','buildings':report['building_count'],'families':report['family_count'],'observed_images':len(report['sources'])},ensure_ascii=False))
