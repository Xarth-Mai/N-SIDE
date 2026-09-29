"""Preserve this batch's real run metadata and visual hashes before required cleanup."""
from pathlib import Path
import hashlib,json,shutil
root=Path(__file__).resolve().parents[4]
out=Path(__file__).resolve().parent
names=['task045-cut-before-r4','task045-cut-r4','task045-cut-r4-taa-ssao','task045-missing-slope-shader','task045-missing-slope-shader-2','task049-forest-r1','task049-forest-root-r1','task049-forest-mature-r1','task049-forest-root-mature-r1','task047-character-r6','task047-character-r6-final']
results=[]
for name in names:
    source=root/'output/capture'/name
    if not (source/'run.json').exists(): continue
    target=out/name;target.mkdir(exist_ok=True)
    for filename in ('run.json','runtime.log','script.json'):
        if (source/filename).exists():shutil.copyfile(source/filename,target/filename)
    state=json.loads((source/'state.json').read_text()) if (source/'state.json').exists() else {}
    samples=state.pop('samples',[])
    ready=next((s for s in samples if s.get('world_ready')),samples[0] if samples else {})
    state['sample_count']=len(samples)
    state['first_ready_sample']=ready
    state['full_state_path']=str(source.relative_to(root)/'state.json')
    state['full_state_sha256']=hashlib.sha256((source/'state.json').read_bytes()).hexdigest() if (source/'state.json').exists() else None
    (target/'state-summary.json').write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n')
    visuals=[{'path':str(p.relative_to(root)),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(source.rglob('*')) if p.is_file() and p.suffix in ('.png','.mp4')]
    (target/'visual-files.json').write_text(json.dumps(visuals,indent=2)+'\n')
    run=json.loads((source/'run.json').read_text())
    results.append({'name':name,'status':run['status'],'runtime_exit_code':run.get('runtime_exit_code'),'frames':len(samples),'checks':len(state.get('checks',[])),'visual_files':len(visuals)})
(out/'summary.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(results,ensure_ascii=False))
