# Checks that every @{id} in a sample record has a `type` reference fact from the member or its parameters.
import json,re,sys,collections
lang=sys.argv[1]; d=json.load(open(lang+'.json')); rec=json.load(open(sys.argv[2]))['signatures']
ids={e['id'] for e in d['entities']}|{e['id'] for e in d.get('externals') or []}
parent={e['id']:e.get('parent') for e in d['entities']}
refs=collections.defaultdict(list)
for r in d['references']:
    if r.get('role') in('type',) : refs[r['from']].append(r.get('target'))
def strs(o):
    if isinstance(o,str): yield o
    elif isinstance(o,dict):
        for a,v in o.items():
            if a!='inferred': yield from strs(v)
    elif isinstance(o,list):
        for v in o: yield from strs(v)
tot=ok=bad=0;notid=[];missref=[]; nokey=0;unit=0;
for s in rec:
    e=s['entity']
    if e not in ids: notid.append(e); continue
    tok=re.findall(r'@\{([^}]+)\}',' '.join(strs({k:v for k,v in s.items() if k!='entity'})))
    tot+=1
    have=collections.Counter(t for k,v in refs.items() if k==e or parent.get(k)==e for t in v)
    want=collections.Counter(tok)
    for t in want:
        if t not in ids: notid.append(t)
    miss=[t for t in want if t not in have or have[t]<want[t]]
    extra=[t for t in have if t not in want]
    if miss: missref.append((e,miss))
    if extra: missref.append((e,'not-in-fact',extra))
    if not miss and not extra: ok+=1
print(lang,'records with ids in key',tot,'agree with reference facts',ok); print(' ids not in key',sorted(set(notid))); print(' disagreements',missref)
