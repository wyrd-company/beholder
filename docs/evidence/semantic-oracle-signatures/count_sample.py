# Counts the sample's slots by class. Run with the samples as JSON beside this script's sample/ directory.
import json,collections
LISTS=('parameters','type-parameters')
GROUPS=('overloads','constructors','call-signatures','construct-signatures','index-signatures')
def walk(x,c):
    for k,v in x.items():
        if k in('entity','contexts'):continue
        if k in GROUPS:
            for o in v: walk(o,c)
        elif k in LISTS:
            for p in v:
                for a,b in p.items():
                    c['name' if a=='name' else 'type' if a in('type','bounds') else 'other']+=1
        elif k in('type','returns'):
            c['inferred' if isinstance(v,dict) else 'type']+=1
        elif k in('get','set'):
            walk(v,c)
        else: c['other']+=len(v) if isinstance(v,list) else 1
T=collections.Counter()
for l in ['rust','typescript','python']:
    r=json.load(open(f'sample/{l}.json'))['signatures']; c=collections.Counter()
    for x in r: walk(x,c)
    print(l,'records',len(r),'without slots',sum(1 for x in r if not set(x)-{'entity','contexts'}),'slots',sum(c.values()),dict(c)); T+=c
print('all',sum(T.values()),dict(T))
