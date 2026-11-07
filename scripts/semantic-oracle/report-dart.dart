// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-dart
// ---

/// Writes a Dart compiler report for `semantic-oracle audit --report`, from the
/// analyzer's resolved element model.
///
///     dart run report-dart.dart --project DIR --context ID --entry ENTRY --out FILE
///         [--define NAME=VALUE]... [--library-flag dart.library.NAME]...
///
/// ENTRY is a context's `entry` parameter: `analysis` reports every library under
/// `lib/`, and a project-relative path reports the libraries that entry reaches
/// through imports and exports, with configurable directives selected by the
/// context's defines and library flags. The project tree gets no output: where the
/// project has no `.dart_tool/package_config.json`, the script writes one for a
/// project without dependencies to a temporary directory. A library with an error
/// is an error: this script exits 2 and writes no report.
library;

import 'dart:convert';
import 'dart:io';

import 'package:analyzer/dart/analysis/features.dart';
import 'package:analyzer/dart/analysis/results.dart';
import 'package:analyzer/dart/constant/value.dart';
import 'package:analyzer/dart/element/element.dart';
import 'package:analyzer/dart/element/nullability_suffix.dart';
import 'package:analyzer/dart/element/type.dart';
import 'package:analyzer/diagnostic/diagnostic.dart';
// The public collection takes no declared variables, which select a configurable
// directive's URI, and no package configuration file.
// ignore: implementation_imports
import 'package:analyzer/src/dart/analysis/analysis_context_collection.dart';
import 'package:path/path.dart' as p;

const covers = [
  'type-parameters[].name',
  'type-parameters[].kind',
  'type-parameters[].bounds',
  'receiver',
  'parameters[].name',
  'parameters[].type',
  'parameters[].mode',
  'parameters[].default',
  'returns',
  'modifiers[]',
  'underlying',
  'type',
  'bounds',
  'value',
  'get.type',
  'set.type',
  'set.modifiers[]',
];

/// The predeclared names the profile writes bare.
const bare = {
  'bool',
  'double',
  'dynamic',
  'int',
  'Never',
  'Null',
  'num',
  'Object',
  'String',
  'void',
};

/// The directory of a package configuration written outside the project tree.
Directory? temporary;

Never fail(String message) {
  temporary?.deleteSync(recursive: true);
  stderr.writeln('report-dart: $message');
  exit(2);
}

class Member {
  Member(this.path);
  final String path;
  final Map<String, String> slots = {};
  final List<String> unstated = [];
  Map<String, Object> toJson() => {
    'path': path,
    'slots': Map.fromEntries(
      slots.entries.toList()..sort((a, b) => a.key.compareTo(b.key)),
    ),
    'unstated': unstated,
  };
}

late String project;
late String packageName;
final members = <String, Member>{};

Member member(String path) => members.putIfAbsent(path, () => Member(path));

/// A library's canonical URI: `package:` and `dart:` URIs as they are, and a file
/// of the project by its project-relative path.
String libraryUri(LibraryElement library) {
  var uri = library.uri;
  if (uri.scheme == 'file') {
    return p.posix.joinAll(
      p.split(p.relative(uri.toFilePath(), from: project)),
    );
  }
  return uri.toString();
}

String typeArguments(List<DartType> arguments) =>
    arguments.isEmpty ? '' : '<${arguments.map(typeText).join(', ')}>';

String question(NullabilitySuffix suffix) =>
    suffix == NullabilitySuffix.question ? '?' : '';

/// A type in the profile's canonical spelling, with `@{path}` for each name.
String typeText(DartType type) {
  var alias = type.alias;
  if (alias != null) {
    var element = alias.element;
    return '@{${libraryUri(element.library)}::${element.name}}'
        '${typeArguments(alias.typeArguments)}${question(alias.nullabilitySuffix)}';
  }
  switch (type) {
    case DynamicType():
      return 'dynamic';
    case VoidType():
      return 'void';
    case NeverType():
      return 'Never${question(type.nullabilitySuffix)}';
    case TypeParameterType():
      return '${type.element.name}${question(type.nullabilitySuffix)}';
    case InterfaceType():
      var element = type.element;
      var library = libraryUri(element.library);
      var name = library == 'dart:core' && bare.contains(element.name)
          ? element.name!
          : '@{$library::${element.name}}';
      return '$name${typeArguments(type.typeArguments)}'
          '${question(type.nullabilitySuffix)}';
    case FunctionType():
      return '${functionTypeText(type)}${question(type.nullabilitySuffix)}';
    case RecordType():
      var parts = [
        for (var field in type.positionalFields) typeText(field.type),
      ];
      if (type.namedFields.isNotEmpty) {
        var named = [
          for (var field in type.namedFields)
            '${typeText(field.type)} ${field.name}',
        ];
        parts.add('{${named.join(', ')}}');
      }
      var trailing =
          type.positionalFields.length == 1 && type.namedFields.isEmpty
          ? ','
          : '';
      return '(${parts.join(', ')}$trailing)${question(type.nullabilitySuffix)}';
  }
  return type.getDisplayString();
}

String typeParametersText(List<TypeParameterElement> parameters) {
  if (parameters.isEmpty) return '';
  var texts = [
    for (var parameter in parameters)
      parameter.bound == null
          ? parameter.name!
          : '${parameter.name} extends ${typeText(parameter.bound!)}',
  ];
  return '<${texts.join(', ')}>';
}

/// `R Function<T>(A, [B], {C c, required D d})`: a positional parameter's label is
/// omitted, and a named parameter's label stays.
String functionTypeText(FunctionType type) {
  var positional = <String>[];
  var optional = <String>[];
  var named = <String>[];
  for (var parameter in type.formalParameters) {
    var text = typeText(parameter.type);
    if (parameter.isNamed) {
      named.add(
        '${parameter.isRequiredNamed ? 'required ' : ''}$text ${parameter.name}',
      );
    } else if (parameter.isOptionalPositional) {
      optional.add(text);
    } else {
      positional.add(text);
    }
  }
  var parts = [...positional];
  if (optional.isNotEmpty) parts.add('[${optional.join(', ')}]');
  if (named.isNotEmpty) parts.add('{${named.join(', ')}}');
  return '${typeText(type.returnType)} Function'
      '${typeParametersText(type.typeParameters)}(${parts.join(', ')})';
}

/// A constant in the model's value forms, or null where it has none.
String? valueText(DartObject? value) {
  if (value == null || value.isNull) return value == null ? null : 'null';
  var type = value.type;
  if (type == null) return null;
  if (type.isDartCoreBool) return '${value.toBoolValue()}';
  if (type.isDartCoreInt) return '${value.toIntValue()}';
  if (type.isDartCoreDouble) return '${value.toDoubleValue()}';
  if (type.isDartCoreString) return jsonEncode(value.toStringValue());
  var element = type.element;
  if (element is EnumElement) {
    var index = value.getField('index')?.toIntValue();
    var constants = element.constants;
    if (index != null && index < constants.length) {
      return '@{${libraryUri(element.library)}::${element.name}.${constants[index].name}}';
    }
  }
  return null;
}

void typeParameters(Member found, List<TypeParameterElement> parameters) {
  for (var (i, parameter) in parameters.indexed) {
    found.slots['type-parameters[$i].name'] = parameter.name!;
    found.slots['type-parameters[$i].kind'] = 'type';
    if (parameter.bound != null) {
      found.slots['type-parameters[$i].bounds'] = typeText(parameter.bound!);
    }
  }
}

void parameters(Member found, List<FormalParameterElement> parameters) {
  for (var (i, parameter) in parameters.indexed) {
    var at = 'parameters[$i]';
    // A wildcard parameter, `_` from language version 3.7, binds no name.
    var name = parameter.name;
    var wildcard =
        name == '_' &&
        parameter.library!.featureSet.isEnabled(Feature.wildcard_variables);
    if (name != null && !wildcard) found.slots['$at.name'] = name;
    found.slots['$at.type'] = typeText(parameter.type);
    if (parameter.isOptionalPositional) found.slots['$at.mode'] = 'optional';
    if (parameter.isOptionalNamed) found.slots['$at.mode'] = 'named';
    if (parameter.isRequiredNamed) found.slots['$at.mode'] = 'required-named';
    if (parameter.isOptional) {
      if (!parameter.hasDefaultValue) {
        found.slots['$at.default'] = 'null';
      } else {
        var value = valueText(parameter.computeConstantValue());
        if (value != null) {
          found.slots['$at.default'] = value;
        } else {
          found.unstated.add('$at.default');
        }
      }
    }
  }
}

void modifiers(Member found, Iterable<String> words, {String prefix = ''}) {
  for (var word in words) {
    found.slots['${prefix}modifiers[$word]'] = word;
  }
}

void executable(String path, ExecutableElement element) {
  var found = member(path);
  typeParameters(found, element.typeParameters);
  parameters(found, element.formalParameters);
  if (element is! ConstructorElement) {
    found.slots['returns'] = typeText(element.returnType);
  }
  modifiers(found, [
    if (element.isAbstract) 'abstract',
    if (element is ConstructorElement && element.isConst) 'const',
    if (element.isExternal) 'external',
    if (element.isStatic && element.enclosingElement is InstanceElement)
      'static',
  ]);
}

/// A field or top-level variable: a `const`, a field entity, or the accessor pair
/// that explicit getters and setters declare.
void property(String path, PropertyInducingElement element) {
  if (element is FieldElement &&
      (element.isEnumConstant || element.isOriginEnumValues)) {
    var found = member(path);
    if (element.isOriginEnumValues)
      found.slots['type'] = typeText(element.type);
    var enumElement = element.enclosingElement;
    if (element.isEnumConstant && enumElement.typeParameters.isNotEmpty) {
      found.slots['type'] = typeText(element.type);
    }
    return;
  }
  var found = member(path);
  if (element.isOriginDeclaration) {
    found.slots['type'] = typeText(element.type);
    if (element.isConst) {
      var value = valueText(element.computeConstantValue());
      if (value != null) found.slots['value'] = value;
      return;
    }
    modifiers(found, [
      if (element is FieldElement && element.isAbstract) 'abstract',
      if (element is FieldElement && element.isCovariant) 'covariant',
      if (element is FieldElement && element.isExternal) 'external',
      if (element is TopLevelVariableElement && element.isExternal) 'external',
      if (element.isFinal) 'final',
      if (element.isLate) 'late',
      if (element.isStatic && element is FieldElement) 'static',
    ]);
    return;
  }
  var getter = element.getter;
  var setter = element.setter;
  if (getter != null && getter.isOriginDeclaration) {
    found.slots['get.type'] = typeText(getter.returnType);
  }
  if (setter != null && setter.isOriginDeclaration) {
    var parameter = setter.formalParameters.firstOrNull;
    if (parameter != null) found.slots['set.type'] = typeText(parameter.type);
    if (parameter != null && parameter.isCovariant) {
      found.slots['set.modifiers[covariant]'] = 'covariant';
    }
  }
  var accessor = getter != null && getter.isOriginDeclaration ? getter : setter;
  if (accessor != null) {
    modifiers(found, [
      if (accessor.isAbstract) 'abstract',
      if (accessor.isExternal) 'external',
      if (accessor.isStatic && accessor.enclosingElement is InstanceElement)
        'static',
    ]);
  }
}

void instanceMembers(String path, InstanceElement element) {
  for (var field in element.fields) {
    property('$path.${field.name}', field);
  }
  for (var method in element.methods) {
    executable('$path.${method.name}', method);
  }
  if (element is InterfaceElement) {
    for (var constructor in element.constructors) {
      var name = constructor.name;
      executable(
        '$path.${name == null || name == 'new' ? 'new' : name}',
        constructor,
      );
    }
  }
}

void library(LibraryElement library) {
  var uri = libraryUri(library);
  var unnamed = 0;
  for (var element in library.children) {
    switch (element) {
      case ExtensionElement():
        var name = element.name ?? 'extension#${++unnamed}';
        var path = '$uri::$name';
        var found = member(path);
        typeParameters(found, element.typeParameters);
        found.slots['receiver'] = typeText(element.extendedType);
        instanceMembers(path, element);
      case InterfaceElement():
        var path = '$uri::${element.name}';
        var found = member(path);
        typeParameters(found, element.typeParameters);
        if (element is ClassElement) {
          modifiers(found, [
            if (element.isAbstract) 'abstract',
            if (element.isBase) 'base',
            if (element.isFinal) 'final',
            if (element.isInterface) 'interface',
            if (element.isSealed) 'sealed',
          ]);
        }
        if (element is MixinElement) {
          modifiers(found, [if (element.isBase) 'base']);
          var constraints = element.superclassConstraints;
          if (!(constraints.length == 1 &&
              constraints.single.isDartCoreObject)) {
            var texts = constraints.map(typeText).toList()..sort();
            found.slots['bounds'] = texts.join(', ');
          }
        }
        if (element is ExtensionTypeElement) {
          found.slots['underlying'] = typeText(element.representation.type);
        }
        instanceMembers(path, element);
      case TypeAliasElement():
        var found = member('$uri::${element.name}');
        typeParameters(found, element.typeParameters);
        found.slots['type'] = typeText(element.aliasedType);
      case TopLevelFunctionElement():
        executable('$uri::${element.name}', element);
      case TopLevelVariableElement():
        property('$uri::${element.name}', element);
    }
  }
}

String? argument(List<String> arguments, String name) {
  var at = arguments.indexOf('--$name');
  return at >= 0 && at + 1 < arguments.length ? arguments[at + 1] : null;
}

List<String> repeated(List<String> arguments, String name) => [
  for (var i = 0; i < arguments.length - 1; i++)
    if (arguments[i] == '--$name') arguments[i + 1],
];

/// A package configuration for a project without dependencies, outside the tree.
String? packageConfig() {
  var existing = File(p.join(project, '.dart_tool', 'package_config.json'));
  if (existing.existsSync()) return null;
  var pubspec = File(p.join(project, 'pubspec.yaml')).readAsStringSync();
  if (RegExp(
    r'^(dev_)?dependencies:\s*\n\s+\S',
    multiLine: true,
  ).hasMatch(pubspec)) {
    fail(
      '$project has dependencies and no .dart_tool/package_config.json; run `dart pub get`',
    );
  }
  var constraint = RegExp(
    r'''^\s+sdk:\s*['"]?[\^>=]*\s*(\d+)\.(\d+)''',
    multiLine: true,
  ).firstMatch(pubspec);
  if (constraint == null) fail('pubspec.yaml states no SDK constraint');
  var directory = temporary = Directory.systemTemp.createTempSync(
    'report-dart-',
  );
  var file = File(p.join(directory.path, 'package_config.json'));
  file.writeAsStringSync(
    jsonEncode({
      'configVersion': 2,
      'packages': [
        {
          'name': packageName,
          'rootUri': Uri.directory(project).toString(),
          'packageUri': 'lib/',
          'languageVersion': '${constraint[1]}.${constraint[2]}',
        },
      ],
    }),
  );
  return file.path;
}

Future<void> main(List<String> arguments) async {
  var projectArgument =
      argument(arguments, 'project') ?? fail('missing --project');
  var context = argument(arguments, 'context') ?? fail('missing --context');
  var entry = argument(arguments, 'entry') ?? fail('missing --entry');
  var outArgument = argument(arguments, 'out') ?? fail('missing --out');
  project = p.normalize(p.absolute(projectArgument));
  var out = p.normalize(p.absolute(outArgument));
  if (p.isWithin(project, out) || out == project) {
    fail('--out $out lies inside the project tree');
  }
  var stale = File(out);
  if (stale.existsSync()) stale.deleteSync();
  var pubspec = File(p.join(project, 'pubspec.yaml'));
  if (!pubspec.existsSync()) fail('$project has no pubspec.yaml');
  packageName =
      RegExp(
        r'^name:\s*(\S+)',
        multiLine: true,
      ).firstMatch(pubspec.readAsStringSync())?[1] ??
      fail('pubspec.yaml has no name');

  var declared = <String, String>{};
  for (var define in repeated(arguments, 'define')) {
    var at = define.indexOf('=');
    if (at <= 0) fail('--define $define is not NAME=VALUE');
    declared[define.substring(0, at)] = define.substring(at + 1);
  }
  for (var flag in repeated(arguments, 'library-flag')) {
    declared[flag] = 'true';
  }

  var collection = AnalysisContextCollectionImpl(
    includedPaths: [project],
    declaredVariables: declared,
    packageConfigFile: packageConfig(),
  );
  var session = collection.contextFor(project).currentSession;

  // The libraries the context compiles: every library under lib/ for analysis,
  // and otherwise the entry's closure over imports and exports.
  var pending = <String>[];
  if (entry == 'analysis') {
    var lib = Directory(p.join(project, 'lib'));
    if (lib.existsSync()) {
      pending.addAll(
        lib
            .listSync(recursive: true)
            .whereType<File>()
            .map((file) => p.normalize(file.path))
            .where((path) => path.endsWith('.dart'))
            .toList()
          ..sort(),
      );
    }
  } else {
    var path = p.normalize(p.join(project, entry));
    if (!File(path).existsSync()) fail('the entry $entry does not exist');
    pending.add(path);
  }

  var seen = <String>{};
  var reported = <LibraryElement>[];
  var errors = <String>[];
  while (pending.isNotEmpty) {
    var path = pending.removeAt(0);
    if (!seen.add(path)) continue;
    var result = await session.getResolvedLibrary(path);
    if (result is NotLibraryButPartResult) continue;
    if (result is! ResolvedLibraryResult) fail('cannot resolve $path: $result');
    for (var unit in result.units) {
      for (var diagnostic in unit.diagnostics) {
        if (diagnostic.severity == Severity.error) {
          var location = unit.lineInfo.getLocation(diagnostic.offset);
          errors.add(
            '${p.relative(unit.path, from: project)}:${location.lineNumber}:'
            '${location.columnNumber}: ${diagnostic.message}',
          );
        }
      }
    }
    var element = result.element;
    reported.add(element);
    if (entry == 'analysis') continue;
    for (var fragment in element.fragments) {
      var reached = [
        for (var import in fragment.libraryImports) import.importedLibrary,
        for (var export in fragment.libraryExports) export.exportedLibrary,
      ];
      for (var library in reached.nonNulls) {
        var uri = library.uri;
        var file = library.firstFragment.source.fullName;
        if (uri.scheme != 'dart' && p.isWithin(project, file))
          pending.add(p.normalize(file));
      }
    }
  }
  // A library that fails to analyze stops the report: nothing it would state was
  // checked.
  if (errors.isNotEmpty)
    fail('the analyzer reports errors:\n${errors.join('\n')}');
  for (var element in reported) {
    library(element);
  }
  await collection.dispose();
  temporary?.deleteSync(recursive: true);

  var version = Platform.version.split(' ').first;
  var paths = members.keys.toList()..sort();
  var report = {
    'format': 'semantic-oracle-compiler-report/1',
    'language': 'dart',
    'context': context,
    'release': version,
    'tool': {
      'name': 'package:analyzer',
      'version': '14.7.0, Dart SDK $version',
    },
    'covers': covers,
    'members': [for (var path in paths) members[path]!.toJson()],
  };
  File(out)
      .writeAsStringSync(const JsonEncoder.withIndent(' ').convert(report));
  stdout.writeln(jsonEncode({'members': paths.length}));
}
