#!/usr/bin/env python3
"""Falsification test. For each before/after pair, does a real consumer break, and which representation reports a change?
`breaks` comes from the compiler or runtime. `structured` facts are hand-written by the proposal's author under the proposed canonical rules.
`as-written` is the same tree without those rules: leaves keep the source spelling."""
import subprocess,re,os,json,shutil,sys,tempfile
def sh(cmd,cwd):
    r=subprocess.run(cmd,shell=True,cwd=cwd,capture_output=True,text=True); return r.returncode,(r.stdout+r.stderr)
C=[]
# as-written facts name the target by its canonical path, so a move shows as a change.
SPELLING={'union-order','array-spelling','any-spelling','optional-spelling','string-annotation','self-spelling'}
def case(id,lang,what,v1,v2,consumer,f1,f2):
    C.append(dict(id=id,lang=lang,what=what,v1=v1,v2=v2,consumer=consumer,f1=f1,f2=f2))
case('reformat','rust','line breaks in the parameter list',
 'pub fn read(path:&str,limit:u32)->u32{limit}\n','pub fn read(\n    path: &str,\n    limit: u32,\n) -> u32 {\n    limit\n}\n',
 'fn main(){let _:u32=lib::read("a",1);}',
 {'params':[('path','&str'),('limit','u32')],'returns':'u32'},{'params':[('path','&str'),('limit','u32')],'returns':'u32'})
case('import-alias','rust','the same type through a renamed import',
 'pub struct Thing;\npub fn f()->Thing{Thing}\n','pub struct Thing;\nuse self::Thing as T;\npub fn f()->T{Thing}\n',
 'fn main(){let _:lib::Thing=lib::f();}',{'returns':'{Thing}'},{'returns':'{Thing}'})
case('qualified','rust','short name against qualified path',
 'use std::path::Path;\npub fn f(p:&Path)->bool{p.exists()}\n','pub fn f(p:&std::path::Path)->bool{p.exists()}\n',
 'fn main(){let _:bool=lib::f(std::path::Path::new("a"));}',{'params':[('p','&{std.path.Path}')],'returns':'bool'},{'params':[('p','&{std.path.Path}')],'returns':'bool'})
case('self-spelling','rust','Self against the type name',
 'pub struct W;\nimpl W{pub fn new()->Self{W}}\n','pub struct W;\nimpl W{pub fn new()->W{W}}\n',
 'fn main(){let _:lib::W=lib::W::new();}',{'returns':'{W}'},{'returns':'{W}'})
case('same-name-other-type','rust','one spelling, the import selects another type',
 'pub mod a{pub struct Thing;}\npub mod b{pub struct Thing;}\nuse a::Thing;\npub fn f()->Thing{Thing}\n',
 'pub mod a{pub struct Thing;}\npub mod b{pub struct Thing;}\nuse b::Thing;\npub fn f()->Thing{Thing}\n',
 'fn main(){let _:lib::a::Thing=lib::f();}',{'returns':'{a.Thing}'},{'returns':'{b.Thing}'})
case('const-through-private','rust','a public constant follows a private constant',
 'const B:u32=1;\npub const A:u32=B+1;\n','const B:u32=2;\npub const A:u32=B+1;\n',
 'const _:() = assert!(lib::A==2);\nfn main(){}',{'type':'u32','value':'2'},{'type':'u32','value':'3'})
case('param-rename','rust','a parameter is renamed',
 'pub fn f(a:u32)->u32{a}\n','pub fn f(b:u32)->u32{b}\n','fn main(){let _=lib::f(1);}',
 {'params':[('a','u32')],'returns':'u32'},{'params':[('b','u32')],'returns':'u32'})
case('alias-replaced','rust','a transparent alias is replaced by its target',
 'pub type Id=u32;\npub fn f(a:Id)->u32{a}\n','pub type Id=u32;\npub fn f(a:u32)->u32{a}\n','fn main(){let _=lib::f(1 as lib::Id);}',
 {'Id':'u32','params':[('a','{Id}')],'returns':'u32'},{'Id':'u32','params':[('a','u32')],'returns':'u32'})
case('alias-target','rust','an alias changes target; the signature that uses it keeps its text',
 'pub type Id=u32;\npub fn f(a:Id)->Id{a}\n','pub type Id=u64;\npub fn f(a:Id)->Id{a}\n','fn main(){let _:u32=lib::f(1u32);}',
 {'Id':'u32','params':[('a','{Id}')],'returns':'{Id}'},{'Id':'u64','params':[('a','{Id}')],'returns':'{Id}'})
case('impl-hidden','rust','the hidden type of impl Trait changes; bounds do not',
 'pub fn f()->impl Clone{1u32}\n','pub fn f()->impl Clone{1u64}\n','fn main(){let x=lib::f();let _=x.clone();}',
 {'returns':'impl {core.Clone}'},{'returns':'impl {core.Clone}'})
case('auto-trait-leak','rust','impl Trait leaks an auto trait of the hidden type',
 'pub fn f()->impl Clone{1u32}\n','pub fn f()->impl Clone{std::rc::Rc::new(1u32)}\n','fn s<T:Send>(_:T){}\nfn main(){s(lib::f());}',
 {'returns':'impl {core.Clone}','auto-traits':['RefUnwindSafe','Send','Sync','Unpin','UnwindSafe']},{'returns':'impl {core.Clone}','auto-traits':['RefUnwindSafe','Unpin','UnwindSafe']})
case('reformat','ts','line breaks in the parameter list',
 'export function read(path:string,limit:number):number{return limit}\n','export function read(\n  path: string,\n  limit: number,\n): number {\n  return limit;\n}\n',
 'import {read} from "./lib.js";const n:number=read("a",1);export {n};',{'params':[('path','string'),('limit','number')],'returns':'number'},{'params':[('path','string'),('limit','number')],'returns':'number'})
case('union-order','ts','union members are reordered',
 'export function f(x:string|number):void{}\n','export function f(x:number|string):void{}\n','import {f} from "./lib.js";f(1);',{'params':[('x','number | string')],'returns':'void'},{'params':[('x','number | string')],'returns':'void'})
case('array-spelling','ts','two spellings of one array type',
 'export function f(x:string[]):void{}\n','export function f(x:Array<string>):void{}\n','import {f} from "./lib.js";f(["a"]);',{'params':[('x','string[]')],'returns':'void'},{'params':[('x','string[]')],'returns':'void'})
case('inferred-return','ts','the return type follows the body; no annotation',
 'export function f(){return 1}\n','export function f(){return "a"}\n','import {f} from "./lib.js";const n:number=f();export {n};',{'returns':{'inferred':'number'}},{'returns':{'inferred':'string'}})
case('import-alias','ts','the same type through a renamed import',
 'export interface T{a:string}\nexport function f():T{return {a:""}}\n','export interface T{a:string}\nimport type {T as U} from "./lib.js";\nexport function f():U{return {a:""}}\n',
 'import {f,type T} from "./lib.js";const x:T=f();export {x};',{'returns':'{T}'},{'returns':'{T}'})
case('same-name-other-type','ts','one spelling, the import selects another type',
 'import type {Thing} from "./a.js";\nexport function f():Thing{return {a:""}}\n','import type {Thing} from "./b.js";\nexport function f():Thing{return {b:1}}\n',
 'import {f} from "./lib.js";const x:{a:string}=f();export {x};',{'returns':'{a.Thing}'},{'returns':'{b.Thing}'})
case('alias-replaced','ts','a transparent alias is replaced by its target',
 'export type Id=string;\nexport function f(a:Id):void{}\n','export type Id=string;\nexport function f(a:string):void{}\n','import {f,type Id} from "./lib.js";const i:Id="a";f(i);',{'Id':'string','params':[('a','{Id}')],'returns':'void'},{'Id':'string','params':[('a','string')],'returns':'void'})
case('untyped-const','go','an untyped constant becomes typed; the value text is unchanged',
 'package lib\nconst A = 1\n','package lib\nconst A int = 1\n','package main\nimport "m/lib"\nvar _ int64 = lib.A\nfunc main(){}\n',{'type':'untyped int','value':'1'},{'type':'int','value':'1'})
case('iota-shift','go','an iota value moves when a constant is inserted above it',
 'package lib\nconst (\n\tA = iota\n\tB\n)\n','package lib\nconst (\n\tZ = iota\n\tA\n\tB\n)\n','package main\nimport "m/lib"\nconst _ = uint(1 - lib.B)\nfunc main(){}\n',{'A':('untyped int','0'),'B':('untyped int','1')},{'Z':('untyped int','0'),'A':('untyped int','1'),'B':('untyped int','2')})
case('any-spelling','go','any against interface{}',
 'package lib\nfunc F(x any){}\n','package lib\nfunc F(x interface{}){}\n','package main\nimport "m/lib"\nfunc main(){lib.F(1)}\n',{'params':[('x','any')]},{'params':[('x','any')]})
case('param-group','go','grouped parameters',
 'package lib\nfunc F(a int, b int){}\n','package lib\nfunc F(a, b int){}\n','package main\nimport "m/lib"\nfunc main(){lib.F(1,2)}\n',{'params':[('a','int'),('b','int')]},{'params':[('a','int'),('b','int')]})
case('reformat','python','line breaks in the parameter list',
 'def read(path:str,limit:int)->int:\n    return limit\n','def read(\n    path: str,\n    limit: int,\n) -> int:\n    return limit\n','import lib\nlib.read("a",1)',{'params':[('path','str'),('limit','int')],'returns':'int'},{'params':[('path','str'),('limit','int')],'returns':'int'})
case('optional-spelling','python','Optional[int] against int | None',
 'from typing import Optional\ndef f(a:Optional[int])->None:\n    pass\n','def f(a:int|None)->None:\n    pass\n','import lib\nlib.f(1)',{'params':[('a','int | None')],'returns':'None'},{'params':[('a','int | None')],'returns':'None'})
case('string-annotation','python','a quoted annotation',
 'def f(a:int)->None:\n    pass\n','def f(a:"int")->None:\n    pass\n','import lib\nlib.f(1)',{'params':[('a','int')],'returns':'None'},{'params':[('a','int')],'returns':'None'})
case('keyword-rename','python','a parameter is renamed; keyword callers break',
 'def f(a):\n    return a\n','def f(b):\n    return b\n','import lib\nlib.f(a=1)',{'params':[('a',None)]},{'params':[('b',None)]})
case('default-through-name','python','a default follows a private name',
 '_D=1\ndef f(a=_D):\n    return a\n','_D=2\ndef f(a=_D):\n    return a\n','import lib\nassert lib.f()==1',{'params':[('a',None,'present')]},{'params':[('a',None,'present')]})
case('unannotated-return','python','an unannotated return follows the body',
 'def f():\n    return 1\n','def f():\n    return "a"\n','import lib\nassert lib.f()+1==2',{},{})
case('moved-behind-reexport','rust','a type moves between private modules; the re-export keeps its consumer path',
 'mod d1{pub struct Grid;}\npub use d1::Grid;\npub fn f()->Grid{Grid}\n','mod d2{pub struct Grid;}\npub use d2::Grid;\npub fn f()->Grid{Grid}\n',
 'fn main(){let _:lib::Grid=lib::f();}',{'returns':'@{lib::Grid}'},{'returns':'@{lib::Grid}'})
case('earlier-reexport-added','rust','a re-export adds an earlier-sorting path; the old path stays',
 'pub mod z{pub struct Widget;}\npub use z::Widget;\npub fn f()->Widget{Widget}\n','pub mod z{pub struct Widget;}\npub use z::Widget;\npub use z::Widget as Alias;\npub fn f()->Widget{Widget}\n',
 'fn main(){let _:lib::Widget=lib::f();}',{'returns':'@{lib::Widget}'},{'returns':'@{lib::Widget}'})
# Slot regressions: real breaks that an earlier slot list had no slot for. Not in the comparison table,
# because the text extractor reads declaration headers and these changes sit in bodies.
REG=[]
def reg(*a):
    n=len(C); case(*a); REG.append(C.pop(n))
reg('trait-binding','rust','an implementation changes an associated type',
 'pub trait R{type K;}\npub struct E;\nimpl R for E{type K=u32;}\n','pub trait R{type K;}\npub struct E;\nimpl R for E{type K=u64;}\n',
 'use lib::R;\nfn main(){let _:<lib::E as R>::K=1u32;}',{'bindings':[('K','u32')]},{'bindings':[('K','u64')]})
reg('inherent-impl-bound','rust','an inherent implementation gains a bound',
 'pub struct C<T>(pub T);\nimpl<T> C<T>{pub fn u(self){}}\n','pub struct C<T>(pub T);\nimpl<T:Clone> C<T>{pub fn u(self){}}\n',
 'struct N;\nfn main(){lib::C(N).u();}',{'impl':{'bounds':None}},{'impl':{'bounds':'Clone'}})
reg('constructor','ts','a class constructor changes a parameter type',
 'export class I{constructor(v:string){}}\n','export class I{constructor(v:number){}}\n',
 'import {I} from "./lib.js";new I("a");',{'constructors':[('v','string')]},{'constructors':[('v','number')]})
reg('call-signature','ts','an interface call signature changes',
 'export interface H{(v:string):string}\n','export interface H{(v:number):string}\n',
 'import type {H} from "./lib.js";declare const h:H;h("a");',{'call':[('v','string')]},{'call':[('v','number')]})
reg('setter-type','ts','a setter narrows; the getter does not change',
 'export class B{get v():string{return ""} set v(x:string|number){}}\n','export class B{get v():string{return ""} set v(x:string){}}\n',
 'import {B} from "./lib.js";new B().v=1;',{'get':'string','set':'number | string'},{'get':'string','set':'string'})
reg('optional-member','ts','an optional member becomes required',
 'export interface S{v?:string}\n','export interface S{v:string}\n',
 'import type {S} from "./lib.js";const s:S={};export {s};',{'type':'string','modifiers':['optional']},{'type':'string','modifiers':[]})
reg('literal-overload-order','ts','call signatures of a type literal swap',
 'export type H={(v:string):string;(v:number):number}\n','export type H={(v:number):number;(v:string):string}\n',
 'import type {H} from "./lib.js";const r:ReturnType<H>=1;export {r};',{'type':'{ (v: string): string; (v: number): number }'},{'type':'{ (v: number): number; (v: string): string }'})
reg('array-length','go','a defined array type changes length',
 'package lib\ntype P [2]int\n','package lib\ntype P [3]int\n','package main\nimport "m/lib"\nvar _ lib.P = [2]int{}\nfunc main(){}\n',{'underlying':'[2]int'},{'underlying':'[3]int'})
reg('constraint-terms','go','a named constraint loses a term',
 'package lib\ntype N interface{ ~int | ~string }\nfunc F[T N](x T){}\n','package lib\ntype N interface{ ~int }\nfunc F[T N](x T){}\n','package main\nimport "m/lib"\nfunc main(){lib.F("a")}\n',{'underlying':'interface { ~int | ~string }'},{'underlying':'interface { ~int }'})
reg('private-field-send','rust','a private field change removes Send from a public struct',
 'pub struct S{x:u32}\n','pub struct S{x:std::rc::Rc<u32>}\n',
 'fn s<T:Send>(){}\nfn main(){s::<lib::S>();}',{'auto-traits':['RefUnwindSafe','Send','Sync','Unpin','UnwindSafe']},{'auto-traits':['RefUnwindSafe','Unpin','UnwindSafe']})
reg('tuple-component-send','rust','one opaque component of a tuple loses Send; the whole tuple does not change',
 'pub fn make()->(impl Clone,impl Clone){(1u8,std::rc::Rc::new(()))}\n','pub fn make()->(impl Clone,impl Clone){(std::rc::Rc::new(()),std::rc::Rc::new(()))}\n',
 'fn s<T:Send>(_:T){}\nfn main(){s(lib::make().0);}',{'auto-traits':['#0 RefUnwindSafe', '#0 Send', '#0 Sync', '#0 Unpin', '#0 UnwindSafe', '#1 RefUnwindSafe', '#1 Unpin', '#1 UnwindSafe']},{'auto-traits':['#0 RefUnwindSafe', '#0 Unpin', '#0 UnwindSafe', '#1 RefUnwindSafe', '#1 Unpin', '#1 UnwindSafe']})
reg('async-output-send','rust','an async function\'s opaque output loses Send; its future does not change',
 'pub async fn make()->impl Clone{1u8}\n','pub async fn make()->impl Clone{std::rc::Rc::new(())}\n',
 'fn s<T:Send>(_:T){}\nasync fn c(){s(lib::make().await);}\nfn main(){let _=c();}',{'auto-traits':['#0 RefUnwindSafe', '#0 Send', '#0 Sync', '#0 UnwindSafe', '#1 RefUnwindSafe', '#1 Send', '#1 Sync', '#1 Unpin', '#1 UnwindSafe']},{'auto-traits':['#0 RefUnwindSafe', '#0 Send', '#0 Sync', '#0 UnwindSafe', '#1 RefUnwindSafe', '#1 Unpin', '#1 UnwindSafe']})
reg('trait-impl-opaque-send','rust','a trait implementation\'s opaque return loses Send; the trait does not change',
 'pub trait Tr{fn make()->impl Clone;}\npub struct Item;\nimpl Tr for Item{fn make()->impl Clone{1u8}}\n','pub trait Tr{fn make()->impl Clone;}\npub struct Item;\nimpl Tr for Item{fn make()->impl Clone{std::rc::Rc::new(())}}\n',
 'use lib::Tr;\nfn s<T:Send>(_:T){}\nfn main(){s(lib::Item::make());}',{'bindings':[('make',['RefUnwindSafe', 'Send', 'Sync', 'Unpin', 'UnwindSafe'])]},{'bindings':[('make',['RefUnwindSafe', 'Unpin', 'UnwindSafe'])]})
reg('provided-opaque-send','rust','a trait method\'s default body loses Send on its opaque return',
 'pub trait Tr{fn make()->impl Clone{1u8}}\npub struct Item;\nimpl Tr for Item{}\n','pub trait Tr{fn make()->impl Clone{std::rc::Rc::new(())}}\npub struct Item;\nimpl Tr for Item{}\n',
 'use lib::Tr;\nfn s<T:Send>(_:T){}\nfn main(){s(lib::Item::make());}',{'auto-traits':['RefUnwindSafe', 'Send', 'Sync', 'Unpin', 'UnwindSafe']},{'auto-traits':['RefUnwindSafe', 'Unpin', 'UnwindSafe']})
def build(c,src):
    d=tempfile.mkdtemp(); l=c['lang']
    if l=='rust':
        open(d+'/lib.rs','w').write(src); open(d+'/m.rs','w').write(c['consumer'])
        rc,o=sh('rustc --edition 2024 --crate-type lib --crate-name lib lib.rs -A warnings && rustc --edition 2024 --extern lib=liblib.rlib -A warnings -L . m.rs',d)
    elif l=='ts':
        open(d+'/lib.ts','w').write(src); open(d+'/m.ts','w').write(c['consumer'])
        open(d+'/a.ts','w').write('export interface Thing{a:string}\n'); open(d+'/b.ts','w').write('export interface Thing{b:number}\n')
        open(d+'/package.json','w').write('{"type":"module"}')
        rc,o=sh('tsc --noEmit --strict --target es2022 --module nodenext --moduleResolution nodenext m.ts',d)
    elif l=='go':
        os.makedirs(d+'/lib'); open(d+'/lib/lib.go','w').write(src); open(d+'/main.go','w').write(c['consumer']); open(d+'/go.mod','w').write('module m\ngo 1.21\n')
        rc,o=sh('go build ./...',d)
    else:
        open(d+'/lib.py','w').write(src); open(d+'/m.py','w').write(c['consumer'])
        rc,o=sh('python3 m.py',d)
    shutil.rmtree(d); return rc,o
def decl_lines(l,src):
    out=[]
    if l=='rust':
        for m in re.finditer(r'pub (?:fn|const|type)[^{;]*(?:;|(?=\{))',src,re.S): out.append(m.group(0))
    elif l=='ts':
        for m in re.finditer(r'export (?:function|type|interface)[^{;]*(?:;|(?=\{))',src,re.S): out.append(m.group(0))
    elif l=='go':
        for m in re.finditer(r'^func [^{]*|^const [^\n(]*|^\t[A-Z][^\n]*',src,re.S|re.M): out.append(m.group(0))
    else:
        for m in re.finditer(r'^def .*?:\s*$',src,re.S|re.M): out.append(m.group(0))
    return out
norm=lambda s:re.sub(r'\s+','',s).replace(',)',')')
res=[]
for c in C:
    rc1,o1=build(c,c['v1']); rc2,o2=build(c,c['v2'])
    if rc1: print('V1 NOT VALID',c['id'],c['lang'],o1[:500])
    a=decl_lines(c['lang'],c['v1']); b=decl_lines(c['lang'],c['v2'])
    st=c['f1']!=c['f2']
    res.append(dict(lang=c['lang'],id=c['id'],what=c['what'],breaks=rc2!=0,text=a!=b,normalized=[norm(x) for x in a]!=[norm(x) for x in b],structured=st,as_written=st or c['id'] in SPELLING or c['id']=='moved-behind-reexport',v1ok=rc1==0))
json.dump(res,open('results.json','w'),indent=1)
print(f"{'lang':7}{'case':24}{'break':6}{'text':6}{'norm':6}{'aswr':6}{'canon':6} what")
for r in res: print(f"{r['lang']:7}{r['id']:24}{'Y' if r['breaks'] else '-':6}{'Y' if r['text'] else '-':6}{'Y' if r['normalized'] else '-':6}{'Y' if r['as_written'] else '-':6}{'Y' if r['structured'] else '-':6} {r['what']}")
for k in ['text','normalized','as_written','structured']:
    fp=[f"{r['lang']}:{r['id']}" for r in res if r[k] and not r['breaks']]; fn=[f"{r['lang']}:{r['id']}" for r in res if not r[k] and r['breaks']]
    print(f'{k}: false change {len(fp)}, missed {len(fn)}\n  false: {fp}\n  missed: {fn}')
regbad=[]
for c in REG:
    rc1,_=build(c,c['v1']); rc2,_=build(c,c['v2'])
    print(f"slot regression {c['lang']:7}{c['id']:24} valid={rc1==0} breaks={rc2!=0} slot-change={c['f1']!=c['f2']}")
    if rc1 or not rc2 or c['f1']==c['f2']: regbad.append(c['id'])
print('slot regressions',len(REG),'failed',regbad)
print('cases',len(res),'breaking',sum(r['breaks'] for r in res),'invalid v1',[r['id'] for r in res if not r['v1ok']])
