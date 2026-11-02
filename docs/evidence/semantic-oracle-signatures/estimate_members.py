# Rough member counts per language. A grep estimate, except Python, which walks the syntax tree.
import ast,os,re,collections,subprocess,json
F=os.environ['FIXTURES']
def files(p,exts):
    for r,ds,fs in os.walk(p):
        ds[:]=[x for x in ds if x not in('vendor','node_modules','.dart_tool','obj','bin','dist-cjs','target','__pycache__','tests','test','benches','examples')]
        for f in fs:
            if f.endswith(exts): yield os.path.join(r,f)
# Python: ast count of public names and class members, packaging ignored
res={}
tot=0;per={}
for pr in sorted(os.listdir(F+'/python/projects')):
    n=0
    for f in files(f'{F}/python/projects/{pr}',('.py','.pyi')):
        rel=os.path.relpath(f,f'{F}/python/projects/{pr}')
        if any(s.startswith('_') and s not in('__init__.py','__init__.pyi') for s in rel.split('/')): continue
        try:t=ast.parse(open(f).read())
        except: continue
        for s in t.body:
            if isinstance(s,(ast.FunctionDef,ast.AsyncFunctionDef,ast.ClassDef,ast.Assign,ast.AnnAssign)):
                nm=getattr(s,'name',None) or (getattr(s,'target',None) or s.targets[0])
                nm=nm if isinstance(nm,str) else getattr(nm,'id','_')
                if nm.startswith('_'): continue
                n+=1
                if isinstance(s,ast.ClassDef):
                    n+=sum(1 for b in s.body if isinstance(b,(ast.FunctionDef,ast.AsyncFunctionDef,ast.Assign,ast.AnnAssign)))
    per[pr]=n
res['python']=sum(per.values())
print('python',res['python'],per)
pats={'rust':(r'^\s*pub\s+(?:(?:async|const|unsafe|extern(?:\s+"[^"]+")?)\s+)*(?:fn|struct|enum|trait|type|const|static|union)\b|^\s*pub\s+\w+\s*:',('.rs',)),
 'go':(r'^func(?:\s+\([^)]*\))?\s+[A-Z]|^type\s+[A-Z]|^(?:const|var)\s+[A-Z]|^\s+[A-Z]\w*\s',('.go',)),
 'csharp':(r'^\s*(?:\[[^\]]*\]\s*)*(?:public|protected)\s',('.cs',)),
 'dart':(r'^\s*(?:(?:abstract|final|sealed|base|static|const|late|external|factory)\s+)*(?:[A-Za-z_][\w<>,\?\[\] ]*\s+)?[a-z][\w]*\s*(?:\(|=|;)',('.dart',)),
 'typescript':(r'^\s*export\s',('.ts','.mts','.cts','.tsx'))}
for l,(p,ex) in pats.items():
    tot=0
    for pr in os.listdir(f'{F}/{l}/projects'):
        for f in files(f'{F}/{l}/projects/{pr}',ex):
            try:s=open(f,errors='ignore').read()
            except: continue
            tot+=len(re.findall(p,s,re.M))
    res[l]=tot; print(l,'grep estimate',tot)
print(res,sum(res.values()))
R=F+'/rust/projects/modules-visibility-and-resolution'
print('calibration rust pilot grep',sum(len(re.findall(pats['rust'][0],open(f,errors='ignore').read(),re.M)) for f in files(R,('.rs',))))
