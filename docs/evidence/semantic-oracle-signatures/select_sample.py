# Selects the seeded random sample. Reads <FIXTURES> and the answer keys as JSON in the current directory.
import json,random,ast,os,re
F=os.environ['FIXTURES']
random.seed(62)
KIND=('callable','type','field','constant','variable','type-alias','enum-member','property')
for n,rel in [('rust','rust/projects/modules-visibility-and-resolution'),('typescript','typescript/projects/package-publishing-ecosystem')]:
    d=json.load(open(n+'.json')); ents={e['id']:e for e in d['entities']}
    m=sorted({p['entity'] for p in d['public-api'] if ents.get(p['entity'],{}).get('kind') in KIND})
    pick=random.sample(m,18)
    print('==',n,len(m))
    for i in pick:
        e=ents[i]; a=e['definitions'][0]['at'] if e.get('definitions') else None
        line=open(f"{F}/{rel}/{a['path']}").read().split('\n')[a['start'][0]-1].strip() if a else '(no site)'
        print(f"{i} | {e['kind']}/{e['language-kind']} | {a['path'] if a else ''}:{a['start'][0] if a else ''} | {line}")
# Python public members per profile rule, computed by ast (selection only)
P=F+'/python/projects/diagnostics-toolkit'
pub=[]
for r,ds,fs in os.walk(P+'/diagnostics_toolkit'):
    if 'tests' in r: continue
    for f in fs:
        if not f.endswith('.py'): continue
        mod=os.path.relpath(os.path.join(r,f),P)
        if any(s.startswith('_') for s in mod.replace('.py','').split('/') if s!='__init__'): continue
        t=ast.parse(open(os.path.join(r,f)).read())
        for s in t.body:
            if isinstance(s,(ast.FunctionDef,ast.AsyncFunctionDef)) and not s.name.startswith('_'): pub.append((mod,s.name,s.lineno,'fn'))
            if isinstance(s,ast.ClassDef) and not s.name.startswith('_'):
                pub.append((mod,s.name,s.lineno,'class'))
                for b in s.body:
                    if isinstance(b,(ast.FunctionDef,ast.AsyncFunctionDef)) and (not b.name.startswith('_') or b.name.endswith('__')): pub.append((mod,s.name+'.'+b.name,b.lineno,'method'))
                    if isinstance(b,(ast.Assign,ast.AnnAssign)):
                        tg=b.targets[0] if isinstance(b,ast.Assign) else b.target
                        if isinstance(tg,ast.Name) and not tg.id.startswith('_'): pub.append((mod,s.name+'.'+tg.id,b.lineno,'classattr'))
            if isinstance(s,(ast.Assign,ast.AnnAssign)):
                tg=s.targets[0] if isinstance(s,ast.Assign) else s.target
                if isinstance(tg,ast.Name) and not tg.id.startswith('_'): pub.append((mod,tg.id,s.lineno,'var'))
print('== python members (modules excluded)',len(pub))
for x in random.sample(pub,18):
    print(x, '|', open(P+'/'+x[0]).read().split('\n')[x[2]-1].strip())
