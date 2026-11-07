// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-csharp
// ---

// Writes a C# compiler report for `semantic-oracle audit --report`, from Roslyn.
//
// A .NET file-based app:
//
//   dotnet run --file report-csharp.cs -- --project DIR --entry FILE --context ID --out FILE
//       [--configuration NAME] [--property NAME=VALUE]... [--target-framework TFM]
//
// `--entry` is the context's `entry`, the solution or project its command builds,
// relative to DIR. The script copies DIR, without `bin` and `obj`, to a temporary
// directory, restores the entry there with MSBuild's API, opens it with
// MSBuildWorkspace, and reads each project's compilation. It writes nothing into the
// project tree. MSBuild and its SDK resolvers come from the newest installed .NET SDK
// (or the one `global.json` selects), whose version is the report's `release`.
// A project that fails to load or compile produces no report.
#:package Microsoft.Build.Locator@1.11.2
#:package Microsoft.CodeAnalysis.Workspaces.MSBuild@5.9.0
#:package Microsoft.CodeAnalysis.CSharp.Workspaces@5.9.0
#:property PublishAot=false
#:property JsonSerializerIsReflectionEnabledByDefault=true
#:property DisableMSBuildAssemblyCopyCheck=true
#:property Nullable=enable

using System.Collections.Immutable;
using System.Globalization;
using System.Reflection;
using System.Runtime.Loader;
using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.Build.Locator;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.MSBuild;

return Report.Run(args);

static class Report
{
    static readonly string[] Covers =
    [
        "type-parameters[].name", "type-parameters[].kind", "type-parameters[].bounds",
        "type-parameters[].variance",
        "parameters[].name", "parameters[].type", "parameters[].mode", "parameters[].default",
        "returns", "return-mode", "modifiers[]", "underlying", "type", "value",
        "get.type", "get.access", "get.modifiers[]", "set.type", "set.access", "set.modifiers[]",
        "init.type", "init.access", "init.modifiers[]", "add.type", "remove.type",
    ];

    sealed record Member(string Path, string Unit)
    {
        public SortedDictionary<string, string> Slots { get; } = new(StringComparer.Ordinal);
        public List<string> Unstated { get; } = [];
    }

    static readonly SortedDictionary<string, Member> Members = new(StringComparer.Ordinal);
    static string root = "";

    public static int Run(string[] args)
    {
        string? project = null, entry = null, context = null, outFile = null, framework = null;
        var configuration = "Debug";
        var properties = new Dictionary<string, string?>();
        for (var i = 0; i < args.Length; i++)
        {
            var value = i + 1 < args.Length ? args[i + 1] : null;
            switch (args[i])
            {
                case "--project": project = value; i++; break;
                case "--entry": entry = value; i++; break;
                case "--context": context = value; i++; break;
                case "--out": outFile = value; i++; break;
                case "--configuration": configuration = value!; i++; break;
                case "--target-framework": framework = value; i++; break;
                case "--property":
                    var split = value!.Split('=', 2);
                    properties[split[0]] = split.Length > 1 ? split[1] : "";
                    i++;
                    break;
                default: return Fail($"unknown argument {args[i]}");
            }
        }
        if (project is null || entry is null || context is null || outFile is null)
        {
            return Fail("--project, --entry, --context and --out are required");
        }
        var projectRoot = Path.GetFullPath(project);
        outFile = Path.GetFullPath(outFile);
        if (outFile.StartsWith(projectRoot + Path.DirectorySeparatorChar, StringComparison.Ordinal))
        {
            return Fail($"--out {outFile} lies inside the project tree");
        }
        File.Delete(outFile);
        properties["Configuration"] = configuration;

        var copy = Directory.CreateTempSubdirectory("semantic-oracle-csharp-").FullName;
        try
        {
            CopyTree(projectRoot, copy);
            root = copy;
            var entryPath = Path.Combine(copy, entry);
            var sdk = SelectSdk(projectRoot);
            if (!Restore(sdk, entryPath, properties))
            {
                return Fail($"restoring {entry} failed");
            }
            var release = sdk.Version.ToString();
            using var workspace = MSBuildWorkspace.Create(properties!);
            var solution = entry.EndsWith(".csproj", StringComparison.OrdinalIgnoreCase)
                ? workspace.OpenProjectAsync(entryPath).GetAwaiter().GetResult().Solution
                : workspace.OpenSolutionAsync(entryPath).GetAwaiter().GetResult();
            var failures = workspace.Diagnostics.Where(d => d.Kind == WorkspaceDiagnosticKind.Failure).ToList();
            if (failures.Count > 0)
            {
                return Fail(string.Join("\n", failures.Select(d => d.Message)));
            }
            foreach (var selected in Select(solution, framework))
            {
                var compilation = selected.GetCompilationAsync().GetAwaiter().GetResult()!;
                var errors = compilation.GetDiagnostics()
                    .Where(d => d.Severity == DiagnosticSeverity.Error).ToList();
                if (errors.Count > 0)
                {
                    // Nothing a failed compilation states was checked.
                    return Fail(string.Join("\n", errors.Take(20).Select(d => d.ToString().Replace(copy + "/", ""))));
                }
                ReadNamespace(compilation.Assembly.GlobalNamespace, compilation.AssemblyName ?? selected.Name);
            }
            var roslyn = typeof(CSharpCompilation).Assembly
                .GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? "";
            var report = new JsonObject
            {
                ["format"] = "semantic-oracle-compiler-report/1",
                ["language"] = "csharp",
                ["context"] = context,
                ["release"] = release,
                ["tool"] = new JsonObject { ["name"] = "Roslyn", ["version"] = roslyn.Split('+')[0] + " (.NET SDK " + release + ")" },
                ["covers"] = new JsonArray(Covers.Select(c => (JsonNode)c!).ToArray()),
                ["members"] = new JsonArray(Members.Values.Select(m => (JsonNode)new JsonObject
                {
                    ["path"] = m.Path,
                    ["unit"] = new JsonObject { ["name"] = m.Unit },
                    ["slots"] = new JsonObject(m.Slots.Select(s => KeyValuePair.Create(s.Key, (JsonNode?)s.Value))),
                    ["unstated"] = new JsonArray(m.Unstated.Select(u => (JsonNode)u!).ToArray()),
                }).ToArray()),
            };
            File.WriteAllText(outFile, report.ToJsonString(new JsonSerializerOptions { WriteIndented = true }));
            Console.WriteLine($"{{\"members\": {Members.Count}}}");
            return 0;
        }
        finally
        {
            Directory.Delete(copy, true);
        }
    }

    static int Fail(string message)
    {
        Console.Error.WriteLine("report-csharp: " + message);
        return 2;
    }

    static void CopyTree(string from, string to)
    {
        Directory.CreateDirectory(to);
        foreach (var file in Directory.EnumerateFiles(from))
        {
            File.Copy(file, Path.Combine(to, Path.GetFileName(file)));
        }
        foreach (var directory in Directory.EnumerateDirectories(from))
        {
            var name = Path.GetFileName(directory);
            if (name is "bin" or "obj" or ".git")
            {
                continue;
            }
            CopyTree(directory, Path.Combine(to, name));
        }
    }

    // The .NET SDK that `dotnet` resolves from the project directory, as `global.json` selects.
    static VisualStudioInstance SelectSdk(string projectRoot) =>
        MSBuildLocator.QueryVisualStudioInstances(new VisualStudioInstanceQueryOptions
        {
            DiscoveryTypes = DiscoveryType.DotNetSdk,
            WorkingDirectory = projectRoot,
        }).First();

    /// Loads MSBuild from the SDK into its own load context, so that the MSBuild
    /// assemblies the workspace package brings never answer for the SDK's, and runs
    /// the `Restore` target. MSBuild is called by reflection because no MSBuild
    /// assembly is compiled into this app.
    static bool Restore(VisualStudioInstance sdk, string entryPath, Dictionary<string, string?> properties)
    {
        Environment.SetEnvironmentVariable("MSBUILD_EXE_PATH", Path.Combine(sdk.MSBuildPath, "MSBuild.dll"));
        Environment.SetEnvironmentVariable("MSBuildExtensionsPath", sdk.MSBuildPath);
        Environment.SetEnvironmentVariable("MSBuildSDKsPath", Path.Combine(sdk.MSBuildPath, "Sdks"));
        // The workload resolver does not load outside the SDK's own host. Workload
        // projects (Android, iOS, MAUI) are out of reach of this report.
        Environment.SetEnvironmentVariable("MSBuildEnableWorkloadResolver", "false");
        var context = new SdkLoadContext(sdk.MSBuildPath);
        var build = context.LoadFromAssemblyName(new AssemblyName("Microsoft.Build"));
        var framework = context.LoadFromAssemblyName(new AssemblyName("Microsoft.Build.Framework"));
        var global = new Dictionary<string, string?>(properties)
        {
            ["MSBuildRestoreSessionId"] = Guid.NewGuid().ToString(),
        };
        var collection = Activator.CreateInstance(build.GetType("Microsoft.Build.Evaluation.ProjectCollection", true)!, [global])!;
        var verbosity = Enum.Parse(framework.GetType("Microsoft.Build.Framework.LoggerVerbosity", true)!, "Quiet");
        var logger = Activator.CreateInstance(build.GetType("Microsoft.Build.Logging.ConsoleLogger", true)!, [verbosity])!;
        logger.GetType().GetProperty("Parameters")!.SetValue(logger, "ErrorsOnly");
        var parameters = Activator.CreateInstance(build.GetType("Microsoft.Build.Execution.BuildParameters", true)!, [collection])!;
        var loggers = Array.CreateInstance(framework.GetType("Microsoft.Build.Framework.ILogger", true)!, 1);
        loggers.SetValue(logger, 0);
        parameters.GetType().GetProperty("Loggers")!.SetValue(parameters, loggers);
        var request = Activator.CreateInstance(
            build.GetType("Microsoft.Build.Execution.BuildRequestData", true)!,
            [entryPath, global, null, new[] { "Restore" }, null])!;
        var manager = build.GetType("Microsoft.Build.Execution.BuildManager", true)!
            .GetProperty("DefaultBuildManager")!.GetValue(null)!;
        var result = manager.GetType().GetMethod("Build", [parameters.GetType(), request.GetType()])!
            .Invoke(manager, [parameters, request])!;
        return result.GetType().GetProperty("OverallResult")!.GetValue(result)!.ToString() == "Success";
    }

    sealed class SdkLoadContext(string directory) : AssemblyLoadContext("sdk-msbuild")
    {
        protected override Assembly? Load(AssemblyName name)
        {
            var path = Path.Combine(directory, name.Name + ".dll");
            return File.Exists(path) ? LoadFromAssemblyPath(path) : null;
        }
    }

    /// One project per project file. A multi-targeting project loads once per
    /// framework; the context's `target-framework` picks the inner build.
    static IEnumerable<Project> Select(Solution solution, string? framework)
    {
        foreach (var group in solution.Projects.Where(p => p.Language == LanguageNames.CSharp).GroupBy(p => p.FilePath))
        {
            var projects = group.ToList();
            if (projects.Count == 1)
            {
                yield return projects[0];
                continue;
            }
            var match = projects.FirstOrDefault(p => framework is not null && p.Name.EndsWith($"({framework})", StringComparison.Ordinal))
                ?? throw new InvalidOperationException($"{group.Key} targets several frameworks; pass --target-framework");
            yield return match;
        }
    }

    // ------------------------------------------------------------------ paths

    static string NamespacePath(INamespaceSymbol? ns) =>
        ns is null || ns.IsGlobalNamespace ? "" : ns.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat).Replace("global::", "");

    static string TypePath(INamedTypeSymbol type)
    {
        type = type.OriginalDefinition;
        var prefix = type.ContainingType is { } outer ? TypePath(outer) : NamespacePath(type.ContainingNamespace);
        var name = type.Name + (type.Arity > 0 ? "`" + type.Arity : "");
        var path = prefix.Length == 0 ? name : prefix + "." + name;
        if (type.IsFileLocal && type.Locations.FirstOrDefault(l => l.IsInSource) is { } location)
        {
            path += "@" + Path.GetRelativePath(root, location.SourceTree!.FilePath).Replace('\\', '/');
        }
        return path;
    }

    /// A member's name in its path. An explicit interface implementation is
    /// `<interface>.<member>`, where the interface part is the implemented
    /// interface's metadata type path with its type arguments, so that `I<int>.M`
    /// and `I<string>.M` differ.
    static string MemberName(ISymbol member)
    {
        var implemented = member switch
        {
            IMethodSymbol m => m.ExplicitInterfaceImplementations.Cast<ISymbol>().FirstOrDefault(),
            IPropertySymbol p => p.ExplicitInterfaceImplementations.Cast<ISymbol>().FirstOrDefault(),
            IEventSymbol e => e.ExplicitInterfaceImplementations.Cast<ISymbol>().FirstOrDefault(),
            _ => null,
        };
        if (implemented is not null)
        {
            return MetadataTypePath(implemented.ContainingType) + "." + MemberName(implemented);
        }
        return member switch
        {
            IMethodSymbol { MethodKind: MethodKind.Constructor } => ".ctor",
            IMethodSymbol { MethodKind: MethodKind.StaticConstructor } => ".cctor",
            IMethodSymbol { MethodKind: MethodKind.Destructor } => "Finalize",
            IMethodSymbol { MethodKind: MethodKind.UserDefinedOperator or MethodKind.Conversion } m => m.MetadataName,
            IPropertySymbol { IsIndexer: true } => "this[]",
            _ => member.Name,
        };
    }

    /// The parameter list of an overloaded member: canonical type paths, with
    /// `ref`, `out` and `in` prefixes kept and `params` omitted.
    static string ParameterList(ImmutableArray<IParameterSymbol> parameters) =>
        "(" + string.Join(",", parameters.Select(p => RefPrefix(p.RefKind) + MetadataTypePath(p.Type))) + ")";

    static string RefPrefix(RefKind kind) => kind switch
    {
        RefKind.Ref => "ref ",
        RefKind.Out => "out ",
        RefKind.In => "in ",
        RefKind.RefReadOnlyParameter => "ref readonly ",
        _ => "",
    };

    static string MetadataTypePath(ITypeSymbol type) => type switch
    {
        IArrayTypeSymbol array => MetadataTypePath(array.ElementType) + "[" + new string(',', array.Rank - 1) + "]",
        IPointerTypeSymbol pointer => MetadataTypePath(pointer.PointedAtType) + "*",
        ITypeParameterSymbol parameter => parameter.Name,
        INamedTypeSymbol named => TypePath(named) + (AllTypeArguments(named) is { Count: > 0 } arguments
            ? "<" + string.Join(",", arguments.Select(MetadataTypePath)) + ">"
            : ""),
        _ => type.ToDisplayString(),
    };

    static List<ITypeSymbol> AllTypeArguments(INamedTypeSymbol type)
    {
        var arguments = type.ContainingType is { } outer ? AllTypeArguments(outer) : [];
        // A generic type named inside its own declaration, as `I<T>` in `where T : I<T>`,
        // is its own definition and still has its type arguments.
        if (!type.IsUnboundGenericType)
        {
            arguments.AddRange(type.TypeArguments);
        }
        return arguments;
    }

    // -------------------------------------------------------------- type text

    static readonly Dictionary<SpecialType, string> Keywords = new()
    {
        [SpecialType.System_Boolean] = "bool", [SpecialType.System_Byte] = "byte",
        [SpecialType.System_Char] = "char", [SpecialType.System_Decimal] = "decimal",
        [SpecialType.System_Double] = "double", [SpecialType.System_Single] = "float",
        [SpecialType.System_Int32] = "int", [SpecialType.System_Int64] = "long",
        [SpecialType.System_IntPtr] = "nint", [SpecialType.System_UIntPtr] = "nuint",
        [SpecialType.System_Object] = "object", [SpecialType.System_SByte] = "sbyte",
        [SpecialType.System_Int16] = "short", [SpecialType.System_String] = "string",
        [SpecialType.System_UInt32] = "uint", [SpecialType.System_UInt64] = "ulong",
        [SpecialType.System_UInt16] = "ushort", [SpecialType.System_Void] = "void",
    };

    /// The effective nullable annotation of a reference type occurrence: `?`
    /// annotated, nothing when not annotated, and `~` oblivious.
    static string Annotation(ITypeSymbol type)
    {
        if (type.IsValueType || type.SpecialType == SpecialType.System_Void)
        {
            return "";
        }
        return type.NullableAnnotation switch
        {
            NullableAnnotation.Annotated => "?",
            NullableAnnotation.None when type is not ITypeParameterSymbol || type.IsReferenceType => "~",
            _ => "",
        };
    }

    static string TypeText(ITypeSymbol type)
    {
        switch (type)
        {
            case IArrayTypeSymbol array:
                return TypeText(array.ElementType) + "[" + new string(',', array.Rank - 1) + "]" + Annotation(array);
            case IPointerTypeSymbol pointer:
                return TypeText(pointer.PointedAtType) + "*";
            case ITypeParameterSymbol parameter:
                return parameter.Name + Annotation(parameter);
            case IDynamicTypeSymbol:
                return "dynamic" + Annotation(type);
            case INamedTypeSymbol named:
                if (named.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T)
                {
                    return TypeText(named.TypeArguments[0]) + "?";
                }
                if (named.IsTupleType)
                {
                    var elements = named.TupleElements.Select(e =>
                        TypeText(e.Type) + (e.IsExplicitlyNamedTupleElement ? " " + e.Name : ""));
                    return "(" + string.Join(", ", elements) + ")" + Annotation(named);
                }
                if (Keywords.TryGetValue(named.SpecialType, out var keyword))
                {
                    return keyword + Annotation(named);
                }
                var arguments = AllTypeArguments(named);
                return "@{" + TypePath(named) + "}"
                    + (arguments.Count > 0 ? "<" + string.Join(", ", arguments.Select(TypeText)) + ">" : "")
                    + Annotation(named);
            default:
                return type.ToDisplayString();
        }
    }

    // ------------------------------------------------------------------ values

    /// A constant in the oracle's value forms, or null where this report cannot spell it.
    static string? ValueText(object? value, ITypeSymbol type)
    {
        if (type.TypeKind == TypeKind.Enum)
        {
            return null;
        }
        return value switch
        {
            null when !type.IsValueType || type.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T => "null",
            null => null,
            string text => JsonSerializer.Serialize(text),
            bool flag => flag ? "true" : "false",
            char => null,
            double number => double.IsFinite(number) ? number.ToString("R", CultureInfo.InvariantCulture).Replace("E+", "e").Replace("E", "e") : null,
            float number => float.IsFinite(number) ? number.ToString("R", CultureInfo.InvariantCulture).Replace("E+", "e").Replace("E", "e") : null,
            decimal number => number.ToString(CultureInfo.InvariantCulture),
            IFormattable number => number.ToString(null, CultureInfo.InvariantCulture),
            _ => null,
        };
    }

    // ------------------------------------------------------------------- slots

    static Member Add(string path, string unit)
    {
        if (!Members.TryGetValue(path, out var member))
        {
            member = new Member(path, unit);
            Members[path] = member;
        }
        return member;
    }

    static void ReadNamespace(INamespaceSymbol ns, string unit)
    {
        foreach (var type in ns.GetTypeMembers())
        {
            ReadType(type, unit);
        }
        foreach (var inner in ns.GetNamespaceMembers())
        {
            ReadNamespace(inner, unit);
        }
    }

    static void ReadType(INamedTypeSymbol type, string unit)
    {
        if (type.IsImplicitlyDeclared && type.Name.StartsWith('<'))
        {
            return;
        }
        var path = TypePath(type);
        var member = Add(path, unit);
        TypeParameters(member, "", type.TypeParameters);
        if (type.TypeKind == TypeKind.Enum)
        {
            member.Slots["underlying"] = TypeText(type.EnumUnderlyingType!);
        }
        foreach (var modifier in TypeModifiers(type))
        {
            member.Slots[$"modifiers[{modifier}]"] = modifier;
        }
        var included = type.GetMembers().Where(m => Included(type, m)).ToList();
        var names = included.GroupBy(MemberName).ToDictionary(g => g.Key, g => g.Count());
        foreach (var child in included)
        {
            var name = MemberName(child);
            if (names[name] > 1)
            {
                name += child switch
                {
                    IMethodSymbol m => ParameterList(m.Parameters),
                    IPropertySymbol p => ParameterList(p.Parameters),
                    _ => "",
                };
            }
            ReadMember(child, Add(path + "." + name, unit));
        }
        foreach (var nested in type.GetTypeMembers())
        {
            ReadType(nested, unit);
        }
    }

    /// The members that are entities: declared members, and the synthetic members the
    /// profile lists. An accessor is part of its property or event; a backing field and
    /// an unspeakable member are not entities; a struct's implicit parameterless
    /// constructor is one only when the struct declares no instance constructor.
    static bool Included(INamedTypeSymbol type, ISymbol member)
    {
        if (member is INamedTypeSymbol || member.Name.StartsWith('<'))
        {
            return false;
        }
        if (member is IMethodSymbol method)
        {
            if (method.MethodKind is MethodKind.PropertyGet or MethodKind.PropertySet
                or MethodKind.EventAdd or MethodKind.EventRemove or MethodKind.EventRaise)
            {
                return false;
            }
            if (method is { MethodKind: MethodKind.Constructor, IsImplicitlyDeclared: true, Parameters.Length: 0 })
            {
                return type.TypeKind != TypeKind.Enum && !type.InstanceConstructors.Any(c => !c.IsImplicitlyDeclared);
            }
        }
        if (member is IFieldSymbol field && (field.IsImplicitlyDeclared || field.AssociatedSymbol is not null))
        {
            return false;
        }
        return true;
    }

    static IEnumerable<string> TypeModifiers(INamedTypeSymbol type)
    {
        switch (type.TypeKind)
        {
            case TypeKind.Class:
                if (type.IsStatic)
                {
                    yield return "static";
                    yield break;
                }
                if (type.IsAbstract)
                {
                    yield return "abstract";
                }
                if (type.IsSealed)
                {
                    yield return "sealed";
                }
                break;
            case TypeKind.Struct:
                if (type.IsReadOnly)
                {
                    yield return "readonly";
                }
                break;
        }
    }

    /// A member's modifiers. `readonly` on a member of a readonly struct is implied by
    /// the struct and is not stated again.
    static IEnumerable<string> MemberModifiers(ISymbol member)
    {
        if (member.IsStatic && member is not IFieldSymbol { IsConst: true })
        {
            yield return "static";
        }
        if (member.IsAbstract)
        {
            yield return "abstract";
        }
        if (member.IsVirtual)
        {
            yield return "virtual";
        }
        if (member.IsOverride)
        {
            yield return "override";
        }
        if (member.IsSealed && member.ContainingType.TypeKind != TypeKind.Struct)
        {
            yield return "sealed";
        }
        var readOnly = member switch
        {
            IFieldSymbol field => field.IsReadOnly,
            IMethodSymbol method => method.IsReadOnly && !method.ContainingType.IsReadOnly,
            _ => false,
        };
        if (readOnly)
        {
            yield return "readonly";
        }
        if (member is IPropertySymbol { IsRequired: true } or IFieldSymbol { IsRequired: true })
        {
            yield return "required";
        }
        // A volatile field's metadata signature carries `volatile`.
        if (member is IFieldSymbol { IsVolatile: true })
        {
            yield return "volatile";
        }
    }

    static void ReadMember(ISymbol symbol, Member member)
    {
        switch (symbol)
        {
            case IMethodSymbol method:
                if (method.MethodKind is MethodKind.StaticConstructor or MethodKind.Destructor)
                {
                    return;
                }
                if (method.MethodKind == MethodKind.Ordinary || method.MethodKind == MethodKind.DelegateInvoke
                    || method.MethodKind == MethodKind.ExplicitInterfaceImplementation)
                {
                    TypeParameters(member, "", method.TypeParameters);
                }
                Parameters(member, "", method.Parameters, method.IsExtensionMethod);
                if (method.MethodKind != MethodKind.Constructor)
                {
                    member.Slots["returns"] = TypeText(method.ReturnType);
                    ReturnMode(member, method.RefKind);
                }
                // A delegate's members are runtime-implemented `public virtual` members
                // by the language's rule, which no declaration can change.
                if (method.ContainingType.TypeKind != TypeKind.Delegate)
                {
                    Modifiers(member, "", MemberModifiers(method));
                }
                break;
            case IPropertySymbol property:
                if (property.IsIndexer)
                {
                    Parameters(member, "", property.Parameters, false);
                }
                ReturnMode(member, property.RefKind);
                Accessor(member, "get", property, property.GetMethod);
                Accessor(member, property.SetMethod?.IsInitOnly == true ? "init" : "set", property, property.SetMethod);
                Modifiers(member, "", MemberModifiers(property));
                break;
            case IEventSymbol @event:
                member.Slots["add.type"] = TypeText(@event.Type);
                member.Slots["remove.type"] = TypeText(@event.Type);
                Modifiers(member, "", MemberModifiers(@event));
                break;
            case IFieldSymbol field when field.ContainingType.TypeKind == TypeKind.Enum:
                if (field.HasConstantValue)
                {
                    member.Slots["value"] = Convert.ToString(field.ConstantValue, CultureInfo.InvariantCulture)!;
                }
                break;
            case IFieldSymbol field:
                if (field.IsFixedSizeBuffer)
                {
                    member.Slots["type"] = TypeText(((IPointerTypeSymbol)field.Type).PointedAtType) + "[" + field.FixedSize + "]";
                }
                else
                {
                    member.Slots["type"] = TypeText(field.Type);
                }
                if (field.IsConst)
                {
                    if (ValueText(field.ConstantValue, field.Type) is { } value)
                    {
                        member.Slots["value"] = value;
                    }
                    else
                    {
                        member.Unstated.Add("value");
                    }
                    break;
                }
                Modifiers(member, "", MemberModifiers(field));
                break;
        }
    }

    static void Modifiers(Member member, string prefix, IEnumerable<string> modifiers)
    {
        foreach (var modifier in modifiers)
        {
            member.Slots[$"{prefix}modifiers[{modifier}]"] = modifier;
        }
    }

    static void ReturnMode(Member member, RefKind kind)
    {
        if (kind == RefKind.Ref)
        {
            member.Slots["return-mode"] = "ref";
        }
        else if (kind == RefKind.RefReadOnly)
        {
            member.Slots["return-mode"] = "ref readonly";
        }
    }

    static void Accessor(Member member, string key, IPropertySymbol property, IMethodSymbol? accessor)
    {
        if (accessor is null)
        {
            return;
        }
        member.Slots[$"{key}.type"] = TypeText(property.Type);
        if (accessor.DeclaredAccessibility != property.DeclaredAccessibility)
        {
            member.Slots[$"{key}.access"] = accessor.DeclaredAccessibility switch
            {
                Accessibility.Internal => "internal",
                Accessibility.Private => "private",
                Accessibility.ProtectedAndInternal => "private protected",
                Accessibility.Protected => "protected",
                Accessibility.ProtectedOrInternal => "protected internal",
                _ => accessor.DeclaredAccessibility.ToString(),
            };
        }
        if (accessor.IsReadOnly && property.ContainingType.TypeKind == TypeKind.Struct && !property.ContainingType.IsReadOnly)
        {
            member.Slots[$"{key}.modifiers[readonly]"] = "readonly";
        }
    }

    static void TypeParameters(Member member, string prefix, ImmutableArray<ITypeParameterSymbol> parameters)
    {
        for (var i = 0; i < parameters.Length; i++)
        {
            var parameter = parameters[i];
            var at = $"{prefix}type-parameters[{i}]";
            member.Slots[$"{at}.name"] = parameter.Name;
            member.Slots[$"{at}.kind"] = "type";
            if (Bounds(parameter) is { Length: > 0 } bounds)
            {
                member.Slots[$"{at}.bounds"] = bounds;
            }
            if (parameter.Variance != VarianceKind.None)
            {
                member.Slots[$"{at}.variance"] = parameter.Variance == VarianceKind.In ? "in" : "out";
            }
        }
    }

    /// A `where` clause: the primary constraint, the type constraints, `new()`, and
    /// `allows ref struct` last.
    static string Bounds(ITypeParameterSymbol parameter)
    {
        var parts = new List<string>();
        if (parameter.HasReferenceTypeConstraint)
        {
            parts.Add(parameter.ReferenceTypeConstraintNullableAnnotation == NullableAnnotation.Annotated ? "class?" : "class");
        }
        if (parameter.HasUnmanagedTypeConstraint)
        {
            parts.Add("unmanaged");
        }
        else if (parameter.HasValueTypeConstraint)
        {
            parts.Add("struct");
        }
        if (parameter.HasNotNullConstraint)
        {
            parts.Add("notnull");
        }
        parts.AddRange(parameter.ConstraintTypes.Select(TypeText));
        if (parameter.HasConstructorConstraint)
        {
            parts.Add("new()");
        }
        if (parameter.AllowsRefLikeType)
        {
            parts.Add("allows ref struct");
        }
        return string.Join(", ", parts);
    }

    static void Parameters(Member member, string prefix, ImmutableArray<IParameterSymbol> parameters, bool extension)
    {
        for (var i = 0; i < parameters.Length; i++)
        {
            var parameter = parameters[i];
            var at = $"{prefix}parameters[{i}]";
            member.Slots[$"{at}.name"] = parameter.Name;
            member.Slots[$"{at}.type"] = TypeText(parameter.Type);
            if (Mode(parameter, extension && i == 0) is { Length: > 0 } mode)
            {
                member.Slots[$"{at}.mode"] = mode;
            }
            if (parameter.HasExplicitDefaultValue)
            {
                if (ValueText(parameter.ExplicitDefaultValue, parameter.Type) is { } value)
                {
                    member.Slots[$"{at}.default"] = value;
                }
                else
                {
                    member.Unstated.Add($"{at}.default");
                }
            }
        }
    }

    /// The effective mode: `scoped out` is `out`, because an `out` parameter is always scoped.
    static string Mode(IParameterSymbol parameter, bool receiver)
    {
        var words = new List<string>();
        if (receiver)
        {
            words.Add("this");
        }
        if (parameter.ScopedKind != ScopedKind.None && parameter.RefKind != RefKind.Out)
        {
            words.Add("scoped");
        }
        words.Add(parameter.RefKind switch
        {
            RefKind.Ref => "ref",
            RefKind.Out => "out",
            RefKind.In => "in",
            RefKind.RefReadOnlyParameter => "ref readonly",
            _ => "",
        });
        if (parameter.IsParams)
        {
            words.Add("params");
        }
        return string.Join(" ", words.Where(w => w.Length > 0));
    }
}
