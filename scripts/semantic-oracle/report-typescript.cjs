#!/usr/bin/env node
// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-typescript
// ---
//
// Writes a TypeScript compiler report for `semantic-oracle audit --report`.
//
// Emits declarations with the project's own options into a temporary directory,
// the same emit as `tsc --declaration`, then reads that emit with the compiler's
// API. The project tree gets no output.
//
//   report-typescript.cjs --project DIR --tsconfig FILE --context ID --out FILE
//
// TYPESCRIPT names the `typescript` module directory; otherwise the module comes
// from the project's node_modules or the global npm root.
"use strict";
const fs = require("fs");
const os = require("os");
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

const parsed = ts.getParsedCommandLineOfConfigFile(configFile, {}, {
  ...ts.sys,
  onUnRecoverableConfigFileDiagnostic: (d) => {
    console.error(ts.flattenDiagnosticMessageText(d.messageText, "\n"));
    process.exit(2);
  },
});
const outDir = fs.mkdtempSync(path.join(os.tmpdir(), "semantic-oracle-ts-"));
const options = {
  ...parsed.options,
  declaration: true,
  emitDeclarationOnly: true,
  noEmit: false,
  outDir,
  rootDir: project,
  declarationMap: false,
  composite: false,
  incremental: false,
  tsBuildInfoFile: undefined,
};
const emitting = ts.createProgram(parsed.fileNames, options);
emitting.emit();

// Maps an emitted declaration file back to the document that a dataset names.
function documentOf(file) {
  const normal = path.resolve(file);
  if (normal.startsWith(outDir + path.sep)) {
    const relative = path.relative(outDir, normal).split(path.sep).join("/");
    const stem = relative.replace(/\.d\.(m|c)?ts$/, "");
    for (const suffix of [".ts", ".tsx", ".mts", ".cts"]) {
      if (fs.existsSync(path.join(project, stem + suffix))) return stem + suffix;
    }
    return relative;
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

function walkFiles(directory, found = []) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) walkFiles(full, found);
    else if (/\.d\.(m|c)?ts$/.test(entry.name)) found.push(full);
  }
  return found;
}

const roots = walkFiles(outDir).concat(parsed.fileNames.filter((name) => /\.d\.(m|c)?ts$/.test(name)));
const reading = ts.createProgram(roots, { ...parsed.options, noEmit: true, types: parsed.options.types });
const checker = reading.getTypeChecker();

function symbolPath(symbol) {
  let target = symbol;
  if (target.flags & ts.SymbolFlags.Alias) target = checker.getAliasedSymbol(target);
  const declaration = target.declarations && target.declarations[0];
  if (!declaration) return undefined;
  const document = documentOf(declaration.getSourceFile().fileName);
  const isProject = !path.isAbsolute(document) && !document.startsWith("typescript/") && !document.includes("/node_modules/");
  const qualified = checker.getFullyQualifiedName(target).replace(/^"[^"]*"\./, "");
  const local = document.startsWith("typescript/") || document.includes("@types/") ? `${document}#${qualified}` : `${isProject && fs.existsSync(path.join(project, document)) ? `"${document}"` : document}#${qualified}`;
  return local;
}

// A node's text, with each name that denotes a declaration as `@{path}`.
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
  if (reading.isSourceFileFromExternalLibrary(file) || reading.isSourceFileDefaultLibrary(file)) continue;
  const document = documentOf(file.fileName);
  const module = checker.getSymbolAtLocation(file);
  if (!module) continue;
  for (const symbol of checker.getExportsOfModule(module)) {
    let target = symbol;
    if (target.flags & ts.SymbolFlags.Alias) continue;
    declare(`"${document}"#${symbol.name}`, target);
  }
}

const report = {
  format: "semantic-oracle-compiler-report/1",
  language: "typescript",
  context,
  tool: { name: "tsc --declaration", version: ts.version },
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
fs.rmSync(outDir, { recursive: true, force: true });
console.log(JSON.stringify({ members: members.size }));
