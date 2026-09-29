"""Preserve this batch's real run metadata and visual hashes before required cleanup."""
from pathlib import Path
import hashlib,json,shutil
root=Path(__file__).resolve().parents[4]
out=Path(__file__).resolve().parent
names=['task045-ground-r5-baseline', 'task045-ground-r5-no-normal', 'task045-ground-r5-solid-color', 'task045-ground-r5-rotated-pair', 'task045-ground-r5-local-patches', 'task045-ground-r5-local-angles', 'task049-understory-before-r1', 'task049-residential-before-r2', 'task047-material-before-r1', 'task049-understory-after-r1', 'task049-residential-after-r2', 'task047-material-after-r1', 'task047-material-motion-r1', 'task049-understory-detail-r1', 'task047-ling-motion-r1', 'task047-ling-orbit-r1']
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
    results.append({'name':name,'status':run['status'],'runtime_exit_code':run.get('runtime_exit_code'),'frames':len(samples),'native_checks':len(state.get('checks',[])), 'wrapper_checks':sum(c.get('source') == 'capture wrapper' for c in run.get('checks',[])),'visual_files':len(visuals)})
(out/'summary.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(results,ensure_ascii=False))
