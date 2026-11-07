#!/usr/bin/env node
// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-typescript
// ---
//
// Writes a TypeScript compiler report for `semantic-oracle audit --report`.
//
// Emits declarations with the project's own options to memory, the same emit as
// `tsc --declaration`, then reads that emit and every declaration file of the project
// in the program with the compiler's API. The project tree gets no output. A program
// with an error is an error: this script exits 2 and writes no report.
//
//   report-typescript.cjs --project DIR --tsconfig FILE --context ID --out FILE
//
// TYPESCRIPT names the `typescript` module directory; otherwise the module comes
// from the project's node_modules or the global npm root.
"use strict";
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

function argument(name, fallback) {
  const at = process.argv.indexOf(`--${name}`);
  if (at >= 0) return process.argv[at + 1];
  if (fallback !== undefined) return fallback;
  console.error(`missing --${name}`);
  process.exit(2);
}

const project = path.resolve(argument("project"));
const context = argument("context");
const outFile = path.resolve(argument("out"));
const configFile = path.resolve(project, argument("tsconfig", "tsconfig.json"));

function loadTypescript() {
  const candidates = [process.env.TYPESCRIPT, path.join(project, "node_modules", "typescript")];
  for (const candidate of candidates) {
    if (candidate && fs.existsSync(candidate)) return require(candidate);
  }
  const root = execSync("npm root -g", { encoding: "utf8" }).trim();
  return require(path.join(root, "typescript"));
}
const ts = loadTypescript();

function fail(message) {
  console.error(`report-typescript: ${message}`);
  process.exit(2);
}

function printDiagnostics(diagnostics) {
  return ts.formatDiagnostics(diagnostics, {
    getCanonicalFileName: (name) => name,
    getCurrentDirectory: () => project,
    getNewLine: () => "\n",
  });
}

if (outFile === project || outFile.startsWith(project + path.sep)) {
  fail(`--out ${outFile} lies inside the project tree`);
}
fs.rmSync(outFile, { force: true });

const parsed = ts.getParsedCommandLineOfConfigFile(configFile, {}, {
  ...ts.sys,
  onUnRecoverableConfigFileDiagnostic: (d) => fail(ts.flattenDiagnosticMessageText(d.messageText, "\n")),
});
if (parsed.errors.length) fail(printDiagnostics(parsed.errors));

// The emit goes to memory, beside its source's path, so that what a declaration
// imports resolves as it does for a consumer of the project. Every emit destination
// of the project's own options is cleared.
const options = {
  ...parsed.options,
  declaration: true,
  emitDeclarationOnly: true,
  noEmit: false,
  noEmitOnError: false,
  outDir: project,
  rootDir: project,
  outFile: undefined,
  declarationDir: undefined,
  declarationMap: false,
  sourceMap: false,
  composite: false,
  incremental: false,
  tsBuildInfoFile: undefined,
};
const emitting = ts.createProgram(parsed.fileNames, options);
const emitted = new Map();
const compiled = new Set();
const result = emitting.emit(undefined, (name, text, _bom, _onError, sources) => {
  const normal = path.resolve(name);
  emitted.set(normal, text);
  for (const source of sources || []) compiled.add(path.resolve(source.fileName));
});
const diagnostics = ts.getPreEmitDiagnostics(emitting).concat(result.diagnostics);
const errors = diagnostics.filter((d) => d.category === ts.DiagnosticCategory.Error);
// A program that fails to compile stops the report: nothing it would state was checked.
if (errors.length) fail(`the program has errors:\n${printDiagnostics(errors)}`);
if (result.emitSkipped) fail("the compiler skipped its emit");

const isProjectFile = (file) => {
  const normal = path.resolve(file);
  return normal.startsWith(project + path.sep) && !normal.includes(`${path.sep}node_modules${path.sep}`);
};
// Every declaration file of the project in the program, however it got there.
const declarations = emitting
  .getSourceFiles()
  .map((file) => path.resolve(file.fileName))
  .filter((name) => /\.d\.(m|c)?ts$/.test(name) && isProjectFile(name));

function documentOf(file) {
  const normal = path.resolve(file);
  if (emitted.has(normal)) {
    const stem = path.relative(project, normal).split(path.sep).join("/").replace(/\.d\.(m|c)?ts$/, "");
    for (const suffix of [".ts", ".tsx", ".mts", ".cts"]) {
      if (fs.existsSync(path.join(project, stem + suffix))) return stem + suffix;
    }
  }
  if (normal.startsWith(project + path.sep)) {
    const relative = path.relative(project, normal).split(path.sep).join("/");
    const at = relative.indexOf("node_modules/");
    return at >= 0 ? relative.slice(at + "node_modules/".length) : relative;
  }
  const at = normal.lastIndexOf("node_modules" + path.sep);
  if (at >= 0) return normal.slice(at + "node_modules/".length).split(path.sep).join("/");
  const lib = normal.indexOf(`typescript${path.sep}lib${path.sep}`);
  if (lib >= 0) return normal.slice(lib).split(path.sep).join("/");
  return normal;
}

// A program over the emit and the project's declaration files. A compiled source is
// hidden, so that an import of it resolves to its emit.
const readingOptions = { ...parsed.options, noEmit: true };
const host = ts.createCompilerHost(readingOptions);
const hostFileExists = host.fileExists.bind(host);
const hostReadFile = host.readFile.bind(host);
const hostGetSourceFile = host.getSourceFile.bind(host);
host.fileExists = (name) =>
  emitted.has(path.resolve(name)) || (!compiled.has(path.resolve(name)) && hostFileExists(name));
host.readFile = (name) => emitted.get(path.resolve(name)) ?? hostReadFile(name);
host.getSourceFile = (name, version, ...rest) => {
  const text = emitted.get(path.resolve(name));
  return text !== undefined ? ts.createSourceFile(name, text, version, true) : hostGetSourceFile(name, version, ...rest);
};
host.writeFile = () => fail("the reading program tried to write");
const reading = ts.createProgram([...emitted.keys(), ...declarations], readingOptions, host);
const checker = reading.getTypeChecker();

// The string name of the ambient module around a declaration, if any.
function ambientModuleOf(declaration) {
  let found;
  for (let node = declaration; node; node = node.parent) {
    if (ts.isModuleDeclaration(node) && ts.isStringLiteral(node.name)) found = node.name.text;
  }
  return found;
}

function symbolPath(symbol) {
  let target = symbol;
  if (target.flags & ts.SymbolFlags.Alias) target = checker.getAliasedSymbol(target);
  const declaration = target.declarations && target.declarations[0];
  if (!declaration) return undefined;
  const document = documentOf(declaration.getSourceFile().fileName);
  const qualified = checker.getFullyQualifiedName(target).replace(/^"[^"]*"\./, "");
  const ambient = ambientModuleOf(declaration);
  if (ambient !== undefined && !ts.isExternalModule(declaration.getSourceFile())) {
    return `ambient:"${ambient}"#${qualified}`;
  }
  const isProject = !path.isAbsolute(document) && !document.startsWith("typescript/");
  const inTree = isProject && fs.existsSync(path.join(project, document));
  return `${inTree ? `"${document}"` : document}#${qualified}`;
}

function typeText(node) {
  if (!node) return undefined;
  const source = node.getSourceFile();
  const edits = [];
  (function visit(current) {
    let name;
    if (ts.isTypeReferenceNode(current)) name = current.typeName;
    else if (ts.isExpressionWithTypeArguments(current)) name = current.expression;
    else if (ts.isTypeQueryNode(current)) name = current.exprName;
    if (name) {
      const symbol = checker.getSymbolAtLocation(ts.isQualifiedName(name) ? name.right : name);
      const isParameter = symbol && symbol.flags & ts.SymbolFlags.TypeParameter;
      const written = symbol && !isParameter ? symbolPath(symbol) : undefined;
      if (written) edits.push([name.getStart(source), name.getEnd(), `@{${written}}`]);
    }
    ts.forEachChild(current, visit);
  })(node);
  let text = source.text.slice(node.getStart(source), node.getEnd());
  const base = node.getStart(source);
  for (const [start, end, replacement] of edits.sort((a, b) => b[0] - a[0])) {
    text = text.slice(0, start - base) + replacement + text.slice(end - base);
  }
  return text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\s+/g, " ").trim();
}

function modifiersOf(node) {
  const flags = ts.getCombinedModifierFlags(node);
  const list = [];
  if (flags & ts.ModifierFlags.Abstract) list.push("abstract");
  if (node.questionToken) list.push("optional");
  if (flags & ts.ModifierFlags.Readonly) list.push("readonly");
  if (flags & ts.ModifierFlags.Static) list.push("static");
  return list;
}

const members = new Map();
function add(memberPath, slots) {
  if (!members.has(memberPath)) members.set(memberPath, { path: memberPath, slots: {}, unstated: [] });
  Object.assign(members.get(memberPath).slots, slots);
}

function signatureSlots(prefix, node) {
  const slots = {};
  const dot = prefix ? `${prefix}.` : "";
  (node.typeParameters || []).forEach((parameter, i) => {
    slots[`${dot}type-parameters[${i}].name`] = parameter.name.getText();
    slots[`${dot}type-parameters[${i}].kind`] = "type";
    if (parameter.constraint) slots[`${dot}type-parameters[${i}].bounds`] = typeText(parameter.constraint);
    if (parameter.default) slots[`${dot}type-parameters[${i}].default`] = typeText(parameter.default);
  });
  (node.parameters || []).forEach((parameter, i) => {
    if (ts.isIdentifier(parameter.name)) slots[`${dot}parameters[${i}].name`] = parameter.name.text;
    if (parameter.type) slots[`${dot}parameters[${i}].type`] = typeText(parameter.type);
    if (parameter.dotDotDotToken) slots[`${dot}parameters[${i}].mode`] = "rest";
    else if (parameter.questionToken) slots[`${dot}parameters[${i}].mode`] = "optional";
  });
  if (node.type) slots[`${dot}returns`] = typeText(node.type);
  return slots;
}

function modifierSlots(node) {
  return Object.fromEntries(modifiersOf(node).map((word) => [`modifiers[${word}]`, word]));
}

function memberOf(owner, node, groups) {
  if (ts.isConstructorDeclaration(node)) {
    groups.constructors.push(node);
  } else if (ts.isCallSignatureDeclaration(node)) {
    groups["call-signatures"].push(node);
  } else if (ts.isConstructSignatureDeclaration(node)) {
    groups["construct-signatures"].push(node);
  } else if (ts.isIndexSignatureDeclaration(node)) {
    groups["index-signatures"].push(node);
  } else if (node.name && (ts.isIdentifier(node.name) || ts.isStringLiteral(node.name))) {
    const memberPath = `${owner}.${node.name.text}`;
    if (ts.isMethodDeclaration(node) || ts.isMethodSignature(node)) {
      (groups.methods[memberPath] = groups.methods[memberPath] || []).push(node);
    } else if (ts.isPropertyDeclaration(node) || ts.isPropertySignature(node)) {
      add(memberPath, { ...(node.type ? { type: typeText(node.type) } : {}), ...modifierSlots(node) });
    } else if (ts.isGetAccessor(node)) {
      add(memberPath, { ...(node.type ? { "get.type": typeText(node.type) } : {}), ...modifierSlots(node) });
    } else if (ts.isSetAccessor(node)) {
      const type = node.parameters[0] && node.parameters[0].type;
      add(memberPath, { ...(type ? { "set.type": typeText(type) } : {}), ...modifierSlots(node) });
    }
  }
}

function functionSlots(memberPath, declarations) {
  const signatures = declarations.filter((node) => !(node.body));
  const nodes = signatures.length ? signatures : declarations;
  if (nodes.length > 1) {
    nodes.forEach((node, n) => add(memberPath, signatureSlots(`overloads[${n}]`, node)));
  } else {
    add(memberPath, { ...signatureSlots("", nodes[0]), ...modifierSlots(nodes[0]) });
  }
}

function declareType(memberPath, node) {
  add(memberPath, {});
  const groups = { constructors: [], "call-signatures": [], "construct-signatures": [], "index-signatures": [], methods: {} };
  (node.typeParameters || []).forEach((parameter, i) => {
    add(memberPath, {
      [`type-parameters[${i}].name`]: parameter.name.getText(),
      [`type-parameters[${i}].kind`]: "type",
    });
  });
  if (ts.isClassDeclaration(node) && ts.getCombinedModifierFlags(node) & ts.ModifierFlags.Abstract) {
    add(memberPath, { "modifiers[abstract]": "abstract" });
  }
  for (const child of node.members) memberOf(memberPath, child, groups);
  for (const group of ["constructors", "call-signatures", "construct-signatures", "index-signatures"]) {
    groups[group].forEach((signature, n) => {
      const flags = ts.getCombinedModifierFlags(signature);
      const access = flags & ts.ModifierFlags.Protected ? "protected" : flags & ts.ModifierFlags.Private ? "private" : undefined;
      add(memberPath, {
        ...signatureSlots(`${group}[${n}]`, signature),
        ...(access ? { [`${group}[${n}].access`]: access } : {}),
      });
    });
  }
  for (const [methodPath, declarations] of Object.entries(groups.methods)) functionSlots(methodPath, declarations);
}

function declare(memberPath, symbol) {
  const declarations = symbol.declarations || [];
  const functions = declarations.filter(ts.isFunctionDeclaration);
  if (functions.length) functionSlots(memberPath, functions);
  for (const node of declarations) {
    if (ts.isClassDeclaration(node) || ts.isInterfaceDeclaration(node)) {
      declareType(memberPath, node);
    } else if (ts.isTypeAliasDeclaration(node)) {
      add(memberPath, { type: typeText(node.type) });
      (node.typeParameters || []).forEach((parameter, i) => {
        add(memberPath, { [`type-parameters[${i}].name`]: parameter.name.getText(), [`type-parameters[${i}].kind`]: "type" });
      });
    } else if (ts.isVariableDeclaration(node) && node.type) {
      add(memberPath, { type: typeText(node.type) });
    } else if (ts.isEnumDeclaration(node)) {
      add(memberPath, {});
      for (const item of node.members) {
        const value = checker.getConstantValue(item);
        const child = `${memberPath}.${item.name.getText()}`;
        add(child, value === undefined ? {} : { value: JSON.stringify(value) });
      }
    }
  }
}

for (const file of reading.getSourceFiles()) {
  if (reading.isSourceFileDefaultLibrary(file) || !isProjectFile(file.fileName)) continue;
  const document = documentOf(file.fileName);
  const module = checker.getSymbolAtLocation(file);
  if (!module) continue;
  for (const symbol of checker.getExportsOfModule(module)) {
    let target = symbol;
    if (target.flags & ts.SymbolFlags.Alias) continue;
    declare(`"${document}"#${symbol.name}`, target);
  }
}

// A member of an ambient module that a project file declares.
for (const module of checker.getAmbientModules()) {
  const inProject = (module.declarations || []).some((d) => isProjectFile(d.getSourceFile().fileName));
  if (!inProject) continue;
  for (const symbol of checker.getExportsOfModule(module)) {
    if (symbol.flags & ts.SymbolFlags.Alias) continue;
    declare(`ambient:${module.name}#${symbol.name}`, symbol);
  }
}

const report = {
  format: "semantic-oracle-compiler-report/1",
  language: "typescript",
  context,
  tool: { name: "tsc --declaration", version: ts.version },
  release: ts.version,
  covers: [
    "parameters[].name", "parameters[].type", "parameters[].mode", "returns", "type",
    "get.type", "set.type", "value", "modifiers[]", "type-parameters[].name", "type-parameters[].kind",
    "type-parameters[].bounds", "type-parameters[].default", "overloads[].parameters[].name",
    "overloads[].parameters[].type", "overloads[].parameters[].mode", "overloads[].returns",
    "constructors[].parameters[].name", "constructors[].parameters[].type",
    "constructors[].parameters[].mode", "constructors[].access",
    "call-signatures[].parameters[].name", "call-signatures[].parameters[].type", "call-signatures[].returns",
    "construct-signatures[].parameters[].name", "construct-signatures[].parameters[].type", "construct-signatures[].returns",
    "index-signatures[].parameters[].name", "index-signatures[].parameters[].type", "index-signatures[].returns",
  ],
  members: [...members.values()].sort((a, b) => (a.path < b.path ? -1 : 1)),
};
fs.writeFileSync(outFile, JSON.stringify(report, null, 1));
console.log(JSON.stringify({ members: members.size }));
