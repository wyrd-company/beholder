# Independent check of the Python sample. The runtime reports what a call site sees.
import os,sys,inspect,json,importlib,typing,subprocess
P=os.environ['FIXTURES']+'/python/projects/diagnostics-toolkit'
sys.path.insert(0,P)
rec=json.load(open('sample/python.json'))['signatures']
ok=bad=0; out=[]
for s in rec:
    if not('parameters' in s or 'returns' in s or 'overloads' in s): continue
    if s.get('contexts')==['typing']:
        continue
    mod,*rest=s['entity'].split('.')
    m=importlib.import_module('diagnostics_toolkit.'+mod)
    o=m
    for r in rest: o=getattr(o,r)
    sig=inspect.signature(o)
    got=[(n,p.annotation if p.annotation is not inspect._empty else None, p.default if p.default is not inspect._empty else '<none>') for n,p in sig.parameters.items()]
    fact=[(p['name'],p.get('type'),p.get('default','<none>')) for p in s.get('parameters',[])]
    fix=lambda t:None if t is None else str(t).replace('@{ext.typing.Any}','Any').replace('@{static_vs_runtime.StatusLiteral}','StatusLiteral').replace('@{ext.warnings.WarningMessage}','warnings.WarningMessage')
    ra=sig.return_annotation; ra=None if ra is inspect._empty else ra
    same=[(a,fix(b),c) for a,b,c in fact]==[(a,str(b) if b is not None else None,('"%s"'%c if isinstance(c,str) and c!='<none>' else ('None' if c is None else c))) for a,b,c in got] and fix(s.get('returns'))==(None if ra is None else str(ra))
    ok+=same; bad+=not same
    if not same: out.append((s['entity'],fact,got,s.get('returns'),ra))
print('python probe agree',ok,'differ',bad);[print(' ',x) for x in out]
