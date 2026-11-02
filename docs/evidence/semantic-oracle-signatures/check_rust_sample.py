# Compares the Rust sample with rustdoc JSON. Run from the directory that holds rust.json, sample/ and rtarget/.
import json,glob,re,collections
K=json.load(open('rust.json')); ents={e['id']:e for e in K['entities']}; exts={e['id']:e for e in K['externals']}
PA={p['entity'] for p in K['public-api']}
allitems={} ; docs=[]
for f in glob.glob('rtarget/doc/*.json'):
    d=json.load(open(f)); docs.append(d)
    for i,it in d['index'].items():
        sp=it.get('span')
        if sp and it.get('name'):
            allitems.setdefault((sp['filename'],sp['begin'][0],it['name']),[]).append((d,it))
def pathof(d,i):
    p=d['paths'].get(str(i))
    if p: return '::'.join(p['path'])
    return None
def key_of(path):
    for e in list(ents.values())+list(exts.values()):
        if e.get('path')==path: return e['id']
def ty(d,t):
    if t is None: return '()'
    k=next(iter(t)); v=t[k]
    if k=='primitive': return v
    if k=='generic': return v
    if k=='tuple': return '('+', '.join(ty(d,x) for x in v)+')' if v else '()'
    if k=='borrowed_ref':
        l=v.get('lifetime'); return '&'+(l+' ' if l else '')+('mut ' if v.get('is_mutable') else '')+ty(d,v['type'])
    if k=='dyn_trait':
        return 'dyn '+' + '.join(ty(d,{'resolved_path':x['trait']}) for x in v['traits'])
    if k=='slice': return '['+ty(d,v)+']'
    if k=='array': return '['+ty(d,v['type'])+'; '+v['len']+']'
    if k=='resolved_path':
        a=v.get('args') or {}; ar=''
        if a and a.get('angle_bracketed') and a['angle_bracketed']['args']: ar='<'+', '.join(ty(d,x['type']) for x in a['angle_bracketed']['args'] if 'type' in x)+'>'
        p=pathof(d,v['id']); 
        return '@['+str(p)+']'+ar
    if k=='qualified_path':
        return '<'+ty(d,v['self_type'])+' as '+ty(d,{'resolved_path':v['trait']})+'>::'+v['name']
    return '?'+k
def fact_to_paths(s):
    def rep(m):
        e=ents.get(m.group(1)) or exts.get(m.group(1)); return '@['+(e['path'] if e else m.group(1))+']'
    return re.sub(r'@\{([^}]+)\}',rep,s)
rec=json.load(open('sample/rust.json'))['signatures']
agree=0;diff=[];skip=[]
for s in rec:
    if len(s)==1 or s['entity'] not in ents: continue
    e=ents[s['entity']]; defs=e.get('definitions') or []
    if not defs: skip.append(s['entity']); continue
    a=defs[0]['at']; name=e['name']
    c=allitems.get((a['path'],a['start'][0],name)) or allitems.get((a['path'],a['start'][0],name.replace('r#','')))
    if not c: skip.append(s['entity']); continue
    d,it=c[0]; inner=it['inner']; got={}
    if 'function' in inner:
        f=inner['function']; g=f['sig']
        got['returns']=ty(d,g.get('output'))
        ps=[]
        for n,t in g['inputs']:
            if n=='self': continue
            ps.append((n,ty(d,t)))
        got['params']=ps
        want={'returns':fact_to_paths(s.get('returns','()')),'params':[(p['name'],fact_to_paths(p['type'])) for p in s.get('parameters',[]) if p['name']!='self']}
        # Self in impl
        got['returns']=got['returns'].replace('@[None]','@[Self]')
    elif 'struct_field' in inner:
        got={'type':ty(d,inner['struct_field'])}; want={'type':fact_to_paths(s['type'])}
    elif 'constant' in inner:
        got={'value':inner['constant']['const']['value']}; want={'value':s['value']}
    elif 'variant' in inner:
        v=inner['variant'].get('discriminant'); got={'value':(v or {}).get('value')}; want={'value':s.get('value')}
    elif 'enum' in inner:
        got={'modifiers':['non_exhaustive'] if any('non_exhaustive' in str(x) for x in it.get('attrs',[])) else []}; want={'modifiers':s.get('modifiers',[])}
    elif 'type_alias' in inner:
        got={'type':ty(d,inner['type_alias']['type'])}; want={'type':fact_to_paths(s['type']).replace('@[modules_visibility_and_resolution::path_roots::PathTrait::Assoc]','Assoc')}
    else: skip.append((s['entity'],list(inner))); continue
    if got==want: agree+=1
    else: diff.append((s['entity'],got,want))
print('rust sample vs rustdoc json: agree',agree,'differ',len(diff),'skipped',skip)
for x in diff: print(' ',x)
# coverage of a compiler draft over the whole key
N=collections.Counter()
for i in PA:
    e=ents.get(i)
    if not e or e['kind'] not in('callable','field','constant','type-alias','enum-member'): continue
    N['members']+=1
    defs=e.get('definitions') or []
    if defs and allitems.get((defs[0]['at']['path'],defs[0]['at']['start'][0],e.get('name'))): N['found']+=1
print('rustdoc finds by site',N)
