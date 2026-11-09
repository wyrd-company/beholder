// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-dart
// ---

/// Writes a Dart compiler report for `semantic-oracle audit --report`, from the
/// analyzer's resolved element model.
///
///     dart run report-dart.dart --project DIR --context ID --entry ENTRY --out FILE
///         --platform vm|js|wasm [--define NAME=VALUE]...
///         [--library-flag dart.library.NAME]... [--dataset FILE]
///
/// ENTRY is a context's `entry` parameter: `analysis` reports every library under
/// `lib/`, and a project-relative path reports the libraries that entry reaches
/// through imports and exports, with configurable directives selected by the
/// context's defines and library flags. The project tree gets no output: where the
/// project has no `.dart_tool/package_config.json`, the script writes one for a
/// project without dependencies to a temporary directory. A compile error in any
/// library the context compiles, in the project or in a package it reaches, is an
/// error: this script exits 2 and writes no report. Warnings, hints and lints are
/// not, as the compiler accepts them.
///
/// The analyzer drops a diagnostic that an `// ignore:` comment names, and the
/// compiler front end does not. So for a context with an entry, the script also
/// runs the context's real compiler on it, with the context's defines and with
/// its output in a temporary directory: `dart compile kernel` for vm, `dart
/// compile js` for js and `dart compile wasm` for wasm. Where it fails, the script
/// exits 2 with its output and writes no report. The compiler's `dart.library.*`
/// values are its platform's, which no `-D` overrides, so each one a configurable
/// directive of the context's libraries tests must agree with the context's
/// library flags, or the script exits 2: otherwise the compiler would check
/// libraries other than the ones reported. The `analysis` context runs no
/// compiler: its tool is `dart analyze`, which honours ignore comments as the
/// analyzer here does, so the report agrees with it.
///
/// The analyzer resolves against its own SDK on every platform, because the SDK
/// libraries a platform shares have the same public API. `--platform` has no
/// default, so that a js or wasm context cannot be checked as vm by omission. For
/// `--platform js` or `wasm`, each `dart:` library that any library of the context
/// imports or exports, in the project or in a package it depends on, must be one
/// the platform's target in the SDK's lib/libraries.json provides; the first that
/// is not exits 2 with no report, as the compiler rejects it. Members are reported
/// for project libraries only.
///
/// `--dataset` names the dataset whose context `--context` is, and defaults to the
/// project's `semantic-oracle.yml` where it exists. The context's `platform`,
/// `entry`, `defines` and `library-flags` must equal the arguments, or the script
/// exits 2 with no report.
library;

import 'dart:convert';
import 'dart:io';

import 'package:analyzer/dart/analysis/features.dart';
import 'package:analyzer/dart/analysis/results.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/constant/value.dart';
import 'package:analyzer/dart/element/element.dart';
import 'package:analyzer/dart/element/nullability_suffix.dart';
import 'package:analyzer/dart/element/type.dart';
import 'package:analyzer/diagnostic/diagnostic.dart';
import 'package:analyzer/error/error.dart' show DiagnosticType;
// The public collection takes no declared variables, which select a configurable
// directive's URI, and no package configuration file.
// ignore: implementation_imports
import 'package:analyzer/src/dart/analysis/analysis_context_collection.dart';
import 'package:path/path.dart' as p;
import 'package:yaml/yaml.dart';

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
  'index',
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

/// The mixins whose declaration writes an `on` clause.
final writtenOnClause = <Element>{};

/// The directory, outside the project tree, of a package configuration the script
/// writes and of the compiler's output.
Directory? temporary;

Directory scratch() =>
    temporary ??= Directory.systemTemp.createTempSync('report-dart-');

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
    // The value's position in `values`, which its `index` getter returns.
    if (element.isEnumConstant && enumElement is EnumElement) {
      found.slots['index'] = '${enumElement.constants.indexOf(element)}';
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
  var declaredGetter = getter != null && getter.isOriginDeclaration;
  var declaredSetter = setter != null && setter.isOriginDeclaration;
  // Each half has its own type, and a half the class does not declare comes from
  // its inherited interface, with the supertype's type arguments substituted.
  var owner = element.enclosingElement;
  ExecutableElement? inheritedGetter;
  ExecutableElement? inheritedSetter;
  if (owner is InterfaceElement && !element.isStatic) {
    var name = element.name!;
    if (!declaredGetter) {
      inheritedGetter = owner.getInheritedMember(
        Name.forLibrary(owner.library, name),
      );
    }
    if (!declaredSetter) {
      inheritedSetter = owner.getInheritedMember(
        Name.forLibrary(owner.library, '$name='),
      );
    }
  }
  if (declaredGetter) {
    found.slots['get.type'] = typeText(getter.returnType);
  } else if (inheritedGetter != null) {
    found.slots['get.type'] = typeText(inheritedGetter.returnType);
  }
  var inheritedParameter = inheritedSetter?.formalParameters.firstOrNull;
  if (!declaredSetter && inheritedParameter != null) {
    found.slots['set.type'] = typeText(inheritedParameter.type);
    if (inheritedParameter.isCovariant) {
      found.slots['set.modifiers[covariant]'] = 'covariant';
    }
  }
  if (declaredSetter) {
    var parameter = setter.formalParameters.firstOrNull;
    if (parameter != null) found.slots['set.type'] = typeText(parameter.type);
    if (parameter != null && parameter.isCovariant) {
      found.slots['set.modifiers[covariant]'] = 'covariant';
    }
  }
  var accessor = declaredGetter ? getter : (declaredSetter ? setter : null);
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
      // A consumer can neither call nor tear off an enum's generative
      // constructor, so it is no member; a factory constructor is.
      if (element is EnumElement && !constructor.isFactory) continue;
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
            if (element.isMixinClass) 'mixin',
            if (element.isSealed) 'sealed',
          ]);
        }
        if (element is MixinElement) {
          modifiers(found, [if (element.isBase) 'base']);
          var constraints = element.superclassConstraints;
          // A mixin without an `on` clause has the implicit constraint Object, and
          // no bounds; a written `on Object` is a bound.
          if (writtenOnClause.contains(element)) {
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

/// The libraries a library's imports and exports select in the context.
List<LibraryElement> reachedLibraries(LibraryElement library) => [
  for (var fragment in library.fragments)
    ...[
      for (var import in fragment.libraryImports) import.importedLibrary,
      for (var export in fragment.libraryExports) export.exportedLibrary,
    ].nonNulls,
];

/// A unit's `package:` URI, or a project file's project-relative path.
String unitName(ResolvedUnitResult unit) => unit.uri.scheme == 'file'
    ? p.relative(unit.path, from: project)
    : '${unit.uri}';

/// Whether the compiler front end rejects the code a diagnostic reports: a
/// compile-time or syntactic error. Warnings, hints and lints never fail the
/// report, whatever severity an analysis options file gives them.
bool rejected(Diagnostic diagnostic) =>
    switch (diagnostic.diagnosticCode.type) {
      DiagnosticType.COMPILE_TIME_ERROR ||
      DiagnosticType.SYNTACTIC_ERROR => true,
      _ => false,
    };

/// The `dart:` libraries a target of the SDK's lib/libraries.json provides,
/// with the targets it includes, each with its entry there.
Map<String, Map> platformLibraries(String target) {
  var sdk = p.dirname(p.dirname(Platform.resolvedExecutable));
  var file = File(p.join(sdk, 'lib', 'libraries.json'));
  if (!file.existsSync()) fail('the SDK has no ${file.path}');
  var specification = jsonDecode(file.readAsStringSync()) as Map;
  Map<String, Map> collect(String name) {
    var section = specification[name];
    if (section is! Map) fail('libraries.json has no target `$name`');
    var found = <String, Map>{};
    for (var include in (section['include'] as List?) ?? const []) {
      found.addAll(collect((include as Map)['target'] as String));
    }
    found.addAll(
      ((section['libraries'] as Map?) ?? const {}).cast<String, Map>(),
    );
    return found;
  }

  return collect(target);
}

String? argument(List<String> arguments, String name) {
  var at = arguments.indexOf('--$name');
  return at >= 0 && at + 1 < arguments.length ? arguments[at + 1] : null;
}

List<String> repeated(List<String> arguments, String name) => [
  for (var i = 0; i < arguments.length - 1; i++)
    if (arguments[i] == '--$name') arguments[i + 1],
];

/// The parameters of `context` in the dataset must equal the arguments.
void agreeWithDataset(
  String file,
  String context,
  Map<String, Object> arguments,
) {
  var dataset = loadYaml(File(file).readAsStringSync());
  var contexts = dataset is YamlMap ? dataset['contexts'] : null;
  if (contexts is! YamlList) fail('$file has no contexts');
  var record = contexts.cast<Object?>().firstWhere(
    (record) => record is YamlMap && record['id'] == context,
    orElse: () => fail('$file declares no context `$context`'),
  );
  var parameters = (record as YamlMap)['parameters'];
  if (parameters is! YamlMap) fail('context `$context` has no parameters');
  String text(Object? value) => value is YamlList
      ? jsonEncode([...value.map((item) => '$item')]..sort())
      : jsonEncode('$value');
  for (var MapEntry(:key, :value) in arguments.entries) {
    var stated = text(parameters[key]);
    var given = text(value is List ? YamlList.wrap(value) : value);
    if (stated != given) {
      fail(
        'context `$context` in $file has $key $stated, '
        'but the arguments give $given',
      );
    }
  }
}

/// The project's package configuration, or one for a project without
/// dependencies, written outside the tree.
String packageConfig() {
  var existing = File(p.join(project, '.dart_tool', 'package_config.json'));
  if (existing.existsSync()) return existing.path;
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
  var file = File(p.join(scratch().path, 'package_config.json'));
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

  // The analyzer resolves against its own SDK, whose shared libraries have the
  // same public API on every platform. A js or wasm context's libraries must
  // also be ones that platform provides, or the command would not compile.
  var platform = argument(arguments, 'platform') ?? fail('missing --platform');
  var (target, compile) = switch (platform) {
    'vm' => ('vm', 'kernel'),
    'js' => ('dart2js', 'js'),
    'wasm' => ('wasm', 'wasm'),
    _ => fail('--platform $platform is not vm, js or wasm'),
  };
  var libraries = platformLibraries(target);
  var provided = platform == 'vm' ? null : libraries.keys.toSet();
  // The `dart.library.*` values the compiler gives a configurable directive: true
  // for a library the target provides and lets a condition test.
  var conditional = {
    for (var MapEntry(:key, :value) in libraries.entries)
      if (value['supported'] != false &&
          value['support_conditional_import'] != false)
        'dart.library.$key',
  };

  var declared = <String, String>{};
  for (var define in repeated(arguments, 'define')) {
    var at = define.indexOf('=');
    if (at <= 0) fail('--define $define is not NAME=VALUE');
    declared[define.substring(0, at)] = define.substring(at + 1);
  }
  for (var flag in repeated(arguments, 'library-flag')) {
    declared[flag] = 'true';
  }
  var dataset =
      argument(arguments, 'dataset') ??
      (File(p.join(project, 'semantic-oracle.yml')).existsSync()
          ? p.join(project, 'semantic-oracle.yml')
          : null);
  if (dataset == null) {
    if (!arguments.contains('--standalone')) {
      fail(
        'no dataset to check context `$context` against: pass --dataset, or '
        '--standalone to report without that check',
      );
    }
    stderr.writeln(
      'report-dart: warning: --standalone, so context `$context` and its '
      'parameters are not checked against any dataset',
    );
  } else {
    agreeWithDataset(dataset, context, {
      'platform': platform,
      'entry': entry,
      'defines': repeated(arguments, 'define'),
      'library-flags': repeated(arguments, 'library-flag'),
    });
  }

  var packages = packageConfig();
  var collection = AnalysisContextCollectionImpl(
    includedPaths: [project],
    declaredVariables: declared,
    packageConfigFile: packages,
  );
  var session = collection.contextFor(project).currentSession;

  // The roots the context compiles: every library under lib/ for analysis, and
  // otherwise the entry.
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
  var roots = pending.toSet();

  // One walk reads the complete graph the context compiles: the roots and every
  // library they reach through imports and exports, the packages the project
  // depends on included, with configurable directives selected by the declared
  // variables. A library's parts are among its units. Each library is checked for
  // compile errors and, on js and wasm, for the `dart:` libraries it reaches.
  // Members are reported for project libraries only: the roots for analysis, and
  // every project library the entry reaches otherwise.
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
      if (entry != 'analysis') {
        for (var directive
            in unit.unit.directives.whereType<NamespaceDirective>()) {
          // Each tool takes the first configuration that holds, else the
          // directive's own URI. The analyzer reads the declared variables and
          // an empty tested value as `true`. The compiler compares as written
          // against its platform's `dart.library.*` values, which a define does
          // not override, and against `''` for any other name, whatever the
          // context defines.
          int? analyzerChoice, compilerChoice;
          var configurations = directive.configurations;
          for (var i = 0; i < configurations.length; i++) {
            var configuration = configurations[i];
            // The name's tokens include its periods.
            var name = configuration.name.tokens
                .map((token) => token.lexeme)
                .join();
            var tested = configuration.value?.stringValue ?? 'true';
            var analyzed = tested.isEmpty ? 'true' : tested;
            var compiledValue = name.startsWith('dart.library.')
                ? (conditional.contains(name) ? 'true' : null)
                : '';
            if (analyzerChoice == null && declared[name] == analyzed) {
              analyzerChoice = i;
            }
            if (compilerChoice == null && compiledValue == tested) {
              compilerChoice = i;
            }
          }
          if (analyzerChoice != compilerChoice) {
            String chosen(int? i) => i == null
                ? 'its default ${directive.uri.toSource()}'
                : '`${configurations[i].toSource()}`';
            fail(
              '${unitName(unit)}: the context\'s library flags and defines select '
              '${chosen(analyzerChoice)}, but `dart compile $compile` for '
              '$platform selects ${chosen(compilerChoice)}',
            );
          }
        }
      }
      for (var diagnostic in unit.diagnostics) {
        if (rejected(diagnostic)) {
          var location = unit.lineInfo.getLocation(diagnostic.offset);
          errors.add(
            '${unitName(unit)}:${location.lineNumber}:${location.columnNumber}: '
            '${diagnostic.message}',
          );
        }
      }
    }
    var element = result.element;
    for (var reached in reachedLibraries(element)) {
      var uri = reached.uri;
      if (uri.scheme == 'dart') {
        if (provided != null && !provided.contains(uri.path)) {
          fail(
            '${libraryUri(element)} reaches dart:${uri.path}, '
            'which the $platform platform does not provide',
          );
        }
      } else {
        pending.add(p.normalize(reached.firstFragment.source.fullName));
      }
    }
    var reports = entry == 'analysis'
        ? roots.contains(path)
        : p.isWithin(project, path);
    if (!reports) continue;
    reported.add(element);
    for (var unit in result.units) {
      for (var declaration
          in unit.unit.declarations.whereType<MixinDeclaration>()) {
        var mixin = declaration.declaredFragment?.element;
        if (declaration.onClause != null && mixin != null) {
          writtenOnClause.add(mixin);
        }
      }
    }
  }
  // A library that fails to compile stops the report: nothing it would state was
  // checked, and the compiler would state nothing.
  if (errors.isNotEmpty)
    fail('the analyzer reports errors:\n${errors.join('\n')}');
  // The analyzer drops an error an `// ignore:` comment names, and the compiler
  // does not: a context with an entry is reported only where its compiler
  // accepts it.
  if (entry != 'analysis') {
    var output = p.join(scratch().path, 'out', switch (platform) {
      'vm' => 'entry.dill',
      'js' => 'entry.js',
      _ => 'entry.wasm',
    });
    Directory(p.dirname(output)).createSync();
    var command = [
      'compile',
      compile,
      '--packages=$packages',
      for (var define in repeated(arguments, 'define')) '--define=$define',
      '--output=$output',
      entry,
    ];
    ProcessResult done;
    try {
      done = await Process.run(
        Platform.resolvedExecutable,
        command,
        workingDirectory: project,
      );
    } on ProcessException catch (error) {
      fail('cannot run `dart ${command.join(' ')}`: ${error.message}');
    }
    if (done.exitCode != 0) {
      fail(
        '`dart ${command.join(' ')}` exits ${done.exitCode}:\n'
        '${done.stdout}${done.stderr}',
      );
    }
  }
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
