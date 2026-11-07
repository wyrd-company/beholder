// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-csharp
// ---

// Writes a C# compiler report for `semantic-oracle audit --report`, from the
// assemblies the context's build emits, as the .NET SDK's own Roslyn reads them.
//
// A .NET file-based app:
//
//   dotnet run --file report-csharp.cs -- --project DIR --entry FILE --context ID --out FILE
//       --target-framework TFM [--configuration NAME] [--property NAME=VALUE]...
//
// The arguments are the context's parameters: `--entry` is its `entry`, the solution or
// project its command builds, relative to DIR. The script copies DIR, without `bin` and
// `obj`, to a temporary directory and writes nothing into the project tree. There, with
// MSBuild's API from the .NET SDK that `dotnet` resolves for DIR:
//
// 1. it restores and builds the entry as the context's command does, with a binary log
//    that records each task's inputs; a build that fails writes no report;
// 2. it reads each `Csc` run from the log: its project instance, that instance's
//    evaluated framework and configuration, its output assembly and its references;
// 3. for each compiled project it takes the instance at the framework that NuGet
//    nearest-framework selection picks for the context's `target-framework`, leaves out
//    a project with no compatible framework, and stops when the build did not compile
//    the instance the context requires;
// 4. it loads that instance's implementation assembly, with its references, into the
//    SDK's own Roslyn, from `<sdk>/Roslyn/bincore`, and reads every signature slot from
//    the metadata symbols: what a consumer compiles against.
//
// The report's `release` is the SDK's version, and `tool` names the Roslyn build that
// read. Nothing is compiled again, so generated code, build-time properties and the
// configuration and framework of each referenced project are the build's own.
#:package Microsoft.Build.Locator@1.11.2
#:package Microsoft.CodeAnalysis.CSharp@5.9.0
#:property PublishAot=false
#:property JsonSerializerIsReflectionEnabledByDefault=true
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

return Bootstrap.Start(args);

/// Runs the report in a load context whose Roslyn is the SDK's own, so that the SDK
/// compiler checks, never the compiler package this app is built against. Nothing here
/// names a Roslyn type, so the default context never loads one.
static class Bootstrap
{
    public static int Start(string[] args)
    {
        var at = Array.IndexOf(args, "--project");
        var directory = at >= 0 && at + 1 < args.Length ? Path.GetFullPath(args[at + 1]) : Directory.GetCurrentDirectory();
        // The .NET SDK that `dotnet` resolves from the project directory, as `global.json` selects.
        var sdk = MSBuildLocator.QueryVisualStudioInstances(new VisualStudioInstanceQueryOptions
        {
            DiscoveryTypes = DiscoveryType.DotNetSdk,
            WorkingDirectory = directory,
        }).FirstOrDefault();
        if (sdk is null)
        {
            Console.Error.WriteLine("report-csharp: no .NET SDK found");
            return 2;
        }
        var bincore = Path.Combine(sdk.MSBuildPath, "Roslyn", "bincore");
        var self = typeof(Bootstrap).Assembly.Location;
        var context = new RoslynLoadContext(bincore, self);
        var report = context.LoadFromAssemblyPath(self).GetType("Report", true)!;
        return (int)report.GetMethod("Run")!.Invoke(null, [args, sdk.MSBuildPath, sdk.Version.ToString()])!;
    }

    /// Resolves `Microsoft.CodeAnalysis*` from the SDK's Roslyn directory, and this app's
    /// own assembly into the same context.
    sealed class RoslynLoadContext(string bincore, string self) : AssemblyLoadContext("sdk-roslyn")
    {
        protected override Assembly? Load(AssemblyName name)
        {
            var roslyn = Path.Combine(bincore, name.Name + ".dll");
            if (name.Name!.StartsWith("Microsoft.CodeAnalysis", StringComparison.Ordinal) && File.Exists(roslyn))
            {
                return LoadFromAssemblyPath(roslyn);
            }
            if (name.Name == Path.GetFileNameWithoutExtension(self))
            {
                return LoadFromAssemblyPath(self);
            }
            return null;
        }
    }
}

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

    /// A unit's key attributes, as the dataset's unit record states them, so that the
    /// audit tells two units' members with one path apart.
    sealed record Unit(string Name, string AssemblyVersion, string Project);

    sealed record Member(string Path, Unit Unit)
    {
        public SortedDictionary<string, string> Slots { get; } = new(StringComparer.Ordinal);
        public List<string> Unstated { get; } = [];
    }

    /// By unit and path: one path can name a member in each of two units.
    static readonly SortedDictionary<string, Member> Members = new(StringComparer.Ordinal);
    static string root = "";

    public static int Run(string[] args, string sdkPath, string release)
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
        if (project is null || entry is null || context is null || outFile is null || framework is null)
        {
            return Fail("--project, --entry, --context, --target-framework and --out are required");
        }
        var projectRoot = Path.GetFullPath(project);
        outFile = Path.GetFullPath(outFile);
        if (outFile.StartsWith(projectRoot + Path.DirectorySeparatorChar, StringComparison.Ordinal))
        {
            return Fail($"--out {outFile} lies inside the project tree");
        }
        File.Delete(outFile);
        properties["Configuration"] = configuration;

        // The compiler that reads is the SDK's: refuse to report under another.
        var compiler = typeof(CSharpCompilation).Assembly;
        var bincore = Path.Combine(sdkPath, "Roslyn", "bincore");
        if (!string.Equals(Path.GetDirectoryName(compiler.Location), bincore.TrimEnd(Path.DirectorySeparatorChar), StringComparison.Ordinal))
        {
            return Fail($"Roslyn loaded from {compiler.Location}, not the SDK's {bincore}");
        }
        var roslyn = compiler.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? "";

        var copy = Directory.CreateTempSubdirectory("semantic-oracle-csharp-").FullName;
        try
        {
            var tree = Path.Combine(copy, "project");
            CopyTree(projectRoot, tree);
            root = tree;
            var entryPath = Path.Combine(tree, entry);
            var msbuild = new MSBuild(sdkPath);
            if (!msbuild.Build(entryPath, properties, ["Restore"], restore: true))
            {
                return Fail($"restoring {entry} failed");
            }
            // The context's own build, logged so that each compilation it ran can be read.
            var log = Path.Combine(copy, "build.binlog");
            if (!msbuild.Build(entryPath, properties, ["Build"], binaryLog: log))
            {
                return Fail($"building {entry} failed");
            }
            var compilations = msbuild.Compilations(log);
            var selected = Select(compilations, msbuild, framework, configuration, out var problem);
            if (problem is not null)
            {
                return Fail(problem);
            }
            if (selected.Count == 0)
            {
                return Fail($"no project of {entry} has a framework compatible with {framework}");
            }
            foreach (var built in selected)
            {
                var assembly = Read(built, out var error);
                if (assembly is null)
                {
                    return Fail(error!);
                }
                var unit = new Unit(assembly.Identity.Name, assembly.Identity.Version.ToString(), Relative(built.Project));
                ReadNamespace(assembly.GlobalNamespace, unit);
            }
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
                    ["unit"] = new JsonObject
                    {
                        ["name"] = m.Unit.Name,
                        ["assembly-version"] = m.Unit.AssemblyVersion,
                        ["project"] = m.Unit.Project,
                    },
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
        Console.Error.WriteLine("report-csharp: " + (root.Length > 0 ? message.Replace(root + "/", "") : message));
        return 2;
    }

    static string Relative(string path) => Path.GetRelativePath(root, path).Replace('\\', '/');

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

    /// One `Csc` run of the build: the project instance that ran it, that instance's
    /// evaluated framework and configuration, and the task's inputs.
    sealed record Built(string Project, string TargetFramework, string TargetFrameworks, string Configuration,
        string OutputAssembly, IReadOnlyList<string> References);

    /// For each project the build compiled, the compilation the context means: the
    /// instance at the framework that NuGet nearest-framework selection picks for the
    /// context's `target-framework` from the project's frameworks. A project with no
    /// compatible framework is not built in the context. When the build compiled that
    /// framework in more than one configuration, the context's configuration decides.
    /// An instance the rule requires that the build did not compile stops the report.
    static List<Built> Select(List<Built> compilations, MSBuild msbuild, string framework, string configuration, out string? problem)
    {
        problem = null;
        var selected = new List<Built>();
        foreach (var group in compilations.GroupBy(c => c.Project).OrderBy(g => g.Key, StringComparer.Ordinal))
        {
            var declared = group.Select(c => c.TargetFrameworks).FirstOrDefault(t => t.Length > 0)
                ?? group.First().TargetFramework;
            var candidates = declared.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
                .Distinct().ToList();
            if (candidates.Count == 0)
            {
                candidates = group.Select(c => c.TargetFramework).Distinct().ToList();
            }
            var nearest = msbuild.Nearest(framework, candidates);
            if (nearest is null)
            {
                continue;
            }
            var instances = group.Where(c => string.Equals(c.TargetFramework, nearest, StringComparison.OrdinalIgnoreCase))
                .DistinctBy(c => c.OutputAssembly).ToList();
            if (instances.Count == 0)
            {
                problem = $"the build compiled {Relative(group.Key)} only at {string.Join(", ", group.Select(c => c.TargetFramework).Distinct())}, not at {nearest}, which the context selects for {framework}";
                return [];
            }
            if (instances.Count > 1)
            {
                instances = instances.Where(c => c.Configuration == configuration).ToList();
                if (instances.Count != 1)
                {
                    problem = $"the build compiled {Relative(group.Key)} at {nearest} in several configurations, none of them uniquely {configuration}";
                    return [];
                }
            }
            selected.Add(instances[0]);
        }
        return selected;
    }

    /// The assembly a build emitted, as Roslyn reads it from metadata: the
    /// implementation assembly, with the references its compilation used.
    static IAssemblySymbol? Read(Built built, out string? error)
    {
        error = null;
        if (!File.Exists(built.OutputAssembly))
        {
            error = $"the build emitted no {Relative(built.OutputAssembly)}";
            return null;
        }
        var output = MetadataReference.CreateFromFile(built.OutputAssembly);
        var references = built.References
            .Where(r => File.Exists(r))
            .Select(r => (MetadataReference)MetadataReference.CreateFromFile(r))
            .Prepend(output)
            .ToList();
        // Private and internal members too, so that paths and overloads see every member.
        var options = new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary)
            .WithMetadataImportOptions(MetadataImportOptions.All);
        var compilation = CSharpCompilation.Create("semantic-oracle-report", [], references, options);
        if (compilation.GetAssemblyOrModuleSymbol(output) is not IAssemblySymbol assembly)
        {
            error = $"Roslyn could not read {Relative(built.OutputAssembly)}";
            return null;
        }
        return assembly;
    }

    /// MSBuild from the SDK, in its own load context and called by reflection, so that no
    /// MSBuild assembly of this app answers for the SDK's.
    sealed class MSBuild
    {
        readonly Assembly build;
        readonly Assembly framework;
        readonly Assembly nuget;

        public MSBuild(string sdkPath)
        {
            Environment.SetEnvironmentVariable("MSBUILD_EXE_PATH", Path.Combine(sdkPath, "MSBuild.dll"));
            Environment.SetEnvironmentVariable("MSBuildExtensionsPath", sdkPath);
            Environment.SetEnvironmentVariable("MSBuildSDKsPath", Path.Combine(sdkPath, "Sdks"));
            // The workload resolver does not load outside the SDK's own host. Workload
            // projects (Android, iOS, MAUI) are out of reach of this report.
            Environment.SetEnvironmentVariable("MSBuildEnableWorkloadResolver", "false");
            var context = new SdkLoadContext(sdkPath);
            build = context.LoadFromAssemblyName(new AssemblyName("Microsoft.Build"));
            framework = context.LoadFromAssemblyName(new AssemblyName("Microsoft.Build.Framework"));
            nuget = context.LoadFromAssemblyName(new AssemblyName("NuGet.Frameworks"));
        }

        static void WriteError(string message) => Console.Error.Write(message.Replace(root + "/", ""));

        static void SetColor(ConsoleColor color) { }

        static void ResetColor() { }

        public bool Build(string path, Dictionary<string, string?> properties, string[] targets, bool restore = false, string? binaryLog = null)
        {
            var global = new Dictionary<string, string?>(properties);
            if (restore)
            {
                global["MSBuildRestoreSessionId"] = Guid.NewGuid().ToString();
            }
            var collection = Activator.CreateInstance(build.GetType("Microsoft.Build.Evaluation.ProjectCollection", true)!, [global])!;
            var verbosity = Enum.Parse(framework.GetType("Microsoft.Build.Framework.LoggerVerbosity", true)!, "Quiet");
            // The build's errors go to standard error, where a failed report says why.
            var write = Delegate.CreateDelegate(build.GetType("Microsoft.Build.Logging.WriteHandler", true)!,
                typeof(MSBuild).GetMethod(nameof(WriteError), BindingFlags.NonPublic | BindingFlags.Static)!);
            var setColor = Delegate.CreateDelegate(build.GetType("Microsoft.Build.Logging.ColorSetter", true)!,
                typeof(MSBuild).GetMethod(nameof(SetColor), BindingFlags.NonPublic | BindingFlags.Static)!);
            var resetColor = Delegate.CreateDelegate(build.GetType("Microsoft.Build.Logging.ColorResetter", true)!,
                typeof(MSBuild).GetMethod(nameof(ResetColor), BindingFlags.NonPublic | BindingFlags.Static)!);
            var console = Activator.CreateInstance(build.GetType("Microsoft.Build.Logging.ConsoleLogger", true)!,
                [verbosity, write, setColor, resetColor])!;
            console.GetType().GetProperty("Parameters")!.SetValue(console, "ErrorsOnly");
            var loggerType = framework.GetType("Microsoft.Build.Framework.ILogger", true)!;
            var list = new List<object> { console };
            if (binaryLog is not null)
            {
                var binary = Activator.CreateInstance(build.GetType("Microsoft.Build.Logging.BinaryLogger", true)!)!;
                binary.GetType().GetProperty("Parameters")!.SetValue(binary, binaryLog);
                list.Add(binary);
            }
            var loggers = Array.CreateInstance(loggerType, list.Count);
            for (var i = 0; i < list.Count; i++)
            {
                loggers.SetValue(list[i], i);
            }
            var parameters = Activator.CreateInstance(build.GetType("Microsoft.Build.Execution.BuildParameters", true)!, [collection])!;
            parameters.GetType().GetProperty("Loggers")!.SetValue(parameters, loggers);
            parameters.GetType().GetProperty("LogTaskInputs")!.SetValue(parameters, true);
            var request = Activator.CreateInstance(
                build.GetType("Microsoft.Build.Execution.BuildRequestData", true)!,
                [path, global, null, targets, null])!;
            var manager = build.GetType("Microsoft.Build.Execution.BuildManager", true)!
                .GetProperty("DefaultBuildManager")!.GetValue(null)!;
            var result = manager.GetType().GetMethod("Build", [parameters.GetType(), request.GetType()])!
                .Invoke(manager, [parameters, request])!;
            collection.GetType().GetMethod("UnloadAllProjects")!.Invoke(collection, null);
            return result.GetType().GetProperty("OverallResult")!.GetValue(result)!.ToString() == "Success";
        }

        static object? Get(object target, string property) => target.GetType().GetProperty(property)?.GetValue(target);

        static int Id(object args, string property) => (int)Get(Get(args, "BuildEventContext")!, property)!;

        static readonly List<object> Events = [];

        static void Collect(object sender, object args) => Events.Add(args);

        /// Every `Csc` run in a binary log, with its project instance's evaluated
        /// framework and configuration.
        public List<Built> Compilations(string binaryLog)
        {
            Events.Clear();
            var source = Activator.CreateInstance(build.GetType("Microsoft.Build.Logging.BinaryLogReplayEventSource", true)!)!;
            var raised = source.GetType().GetEvent("AnyEventRaised")!;
            raised.AddEventHandler(source, Delegate.CreateDelegate(raised.EventHandlerType!,
                typeof(MSBuild).GetMethod(nameof(Collect), BindingFlags.NonPublic | BindingFlags.Static)!));
            source.GetType().GetMethod("Replay", [typeof(string)])!.Invoke(source, [binaryLog]);

            var evaluations = new Dictionary<int, Dictionary<string, string>>();
            var projects = new Dictionary<int, (string File, int Evaluation)>();
            var tasks = new Dictionary<(int, int), Dictionary<string, List<string>>>();
            var csc = new List<(int Project, int Task)>();
            foreach (var e in Events)
            {
                switch (e.GetType().Name)
                {
                    case "ProjectEvaluationFinishedEventArgs":
                        var values = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
                        foreach (var property in (System.Collections.IEnumerable?)Get(e, "Properties") ?? Array.Empty<object>())
                        {
                            var (key, value) = property switch
                            {
                                System.Collections.DictionaryEntry entry => (entry.Key?.ToString(), entry.Value?.ToString()),
                                _ => ((Get(property, "Key") ?? Get(property, "Name"))?.ToString(),
                                      (Get(property, "Value") ?? Get(property, "EvaluatedValue"))?.ToString()),
                            };
                            if (key is not null)
                            {
                                values[key] = value ?? "";
                            }
                        }
                        evaluations[Id(e, "EvaluationId")] = values;
                        break;
                    case "ProjectStartedEventArgs":
                        projects[Id(e, "ProjectContextId")] = ((string)Get(e, "ProjectFile")!, Id(e, "EvaluationId"));
                        break;
                    case "TaskStartedEventArgs" when (string?)Get(e, "TaskName") == "Csc":
                        csc.Add((Id(e, "ProjectContextId"), Id(e, "TaskId")));
                        break;
                    case "TaskParameterEventArgs" when Get(e, "Kind")?.ToString() == "TaskInput":
                        var key2 = (Id(e, "ProjectContextId"), Id(e, "TaskId"));
                        if (!tasks.TryGetValue(key2, out var inputs))
                        {
                            inputs = [];
                            tasks[key2] = inputs;
                        }
                        var items = ((System.Collections.IEnumerable?)Get(e, "Items") ?? Array.Empty<object>()).Cast<object>()
                            .Select(i => i is string s ? s : (string?)Get(i, "ItemSpec") ?? i.ToString()!).ToList();
                        inputs[(string)Get(e, "ParameterName")!] = items;
                        break;
                }
            }
            var found = new List<Built>();
            foreach (var (projectId, taskId) in csc)
            {
                if (!projects.TryGetValue(projectId, out var started) || !tasks.TryGetValue((projectId, taskId), out var inputs)
                    || !inputs.TryGetValue("OutputAssembly", out var output) || output.Count == 0)
                {
                    continue;
                }
                var evaluated = evaluations.GetValueOrDefault(started.Evaluation) ?? [];
                var directory = Path.GetDirectoryName(started.File)!;
                found.Add(new Built(
                    started.File,
                    evaluated.GetValueOrDefault("TargetFramework") ?? "",
                    evaluated.GetValueOrDefault("TargetFrameworks") ?? "",
                    evaluated.GetValueOrDefault("Configuration") ?? "",
                    Path.GetFullPath(output[0], directory),
                    inputs.GetValueOrDefault("References")?.Select(r => Path.GetFullPath(r, directory)).ToList() ?? []));
            }
            return found;
        }

        /// The framework NuGet nearest-framework selection picks for `target` among
        /// `candidates`, or null when none is compatible.
        public string? Nearest(string target, List<string> candidates)
        {
            var frameworkType = nuget.GetType("NuGet.Frameworks.NuGetFramework", true)!;
            var parse = frameworkType.GetMethod("Parse", [typeof(string)])!;
            var reducer = Activator.CreateInstance(nuget.GetType("NuGet.Frameworks.FrameworkReducer", true)!)!;
            var nearest = reducer.GetType().GetMethod("GetNearest",
                [frameworkType, typeof(IEnumerable<>).MakeGenericType(frameworkType)])!;
            var parsed = Array.CreateInstance(frameworkType, candidates.Count);
            for (var i = 0; i < candidates.Count; i++)
            {
                parsed.SetValue(parse.Invoke(null, [candidates[i]]), i);
            }
            var chosen = nearest.Invoke(reducer, [parse.Invoke(null, [target]), parsed]);
            return chosen is null ? null : candidates[Array.IndexOf(parsed, chosen)];
        }
    }

    sealed class SdkLoadContext(string directory) : AssemblyLoadContext("sdk-msbuild")
    {
        protected override Assembly? Load(AssemblyName name)
        {
            var path = Path.Combine(directory, name.Name + ".dll");
            return File.Exists(path) ? LoadFromAssemblyPath(path) : null;
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

    /// A canonical metadata type path, by the profile's Paths table.
    static string MetadataTypePath(ITypeSymbol type)
    {
        switch (type)
        {
            case IArrayTypeSymbol array:
                // A jagged array keeps its rank specifiers in source order: `int[][,]`
                // is an array of `int[,]`.
                var ranks = new List<int>();
                ITypeSymbol element = array;
                while (element is IArrayTypeSymbol inner)
                {
                    ranks.Add(inner.Rank);
                    element = inner.ElementType;
                }
                return MetadataTypePath(element) + string.Concat(ranks.Select(r => "[" + new string(',', r - 1) + "]"));
            case IPointerTypeSymbol pointer:
                return MetadataTypePath(pointer.PointedAtType) + "*";
            case ITypeParameterSymbol parameter when parameter.TypeParameterKind == TypeParameterKind.Method:
                return "``" + parameter.Ordinal;
            case ITypeParameterSymbol parameter:
                // Counted from 0 across every enclosing type, outer first.
                var offset = 0;
                for (var outer = parameter.DeclaringType?.ContainingType; outer is not null; outer = outer.ContainingType)
                {
                    offset += outer.TypeParameters.Length;
                }
                return "`" + (offset + parameter.Ordinal);
            case IDynamicTypeSymbol:
                return "System.Object";
            case IFunctionPointerTypeSymbol pointer:
                return FunctionPointerPath(pointer.Signature);
            case INamedTypeSymbol named:
                named = named.TupleUnderlyingType ?? named;
                return TypePath(named) + (AllTypeArguments(named) is { Count: > 0 } arguments
                    ? "<" + string.Join(",", arguments.Select(MetadataTypePath)) + ">"
                    : "");
            default:
                return type.ToDisplayString();
        }
    }

    /// `delegate* <convention><parameters..., return>`.
    static string FunctionPointerPath(IMethodSymbol signature)
    {
        var convention = signature.CallingConvention switch
        {
            System.Reflection.Metadata.SignatureCallingConvention.Default => "managed",
            System.Reflection.Metadata.SignatureCallingConvention.CDecl => "unmanaged[Cdecl]",
            System.Reflection.Metadata.SignatureCallingConvention.StdCall => "unmanaged[Stdcall]",
            System.Reflection.Metadata.SignatureCallingConvention.ThisCall => "unmanaged[Thiscall]",
            System.Reflection.Metadata.SignatureCallingConvention.FastCall => "unmanaged[Fastcall]",
            _ => signature.UnmanagedCallingConventionTypes.IsEmpty
                ? "unmanaged"
                : "unmanaged[" + string.Join(",", signature.UnmanagedCallingConventionTypes
                    .Select(t => t.Name.StartsWith("CallConv", StringComparison.Ordinal) ? t.Name["CallConv".Length..] : t.Name)
                    .Order(StringComparer.Ordinal)) + "]",
        };
        var parts = signature.Parameters.Select(p => RefPrefix(p.RefKind) + MetadataTypePath(p.Type)).ToList();
        parts.Add(signature.RefKind switch
        {
            RefKind.Ref => "ref ",
            RefKind.RefReadOnly => "ref readonly ",
            _ => "",
        } + MetadataTypePath(signature.ReturnType));
        return "delegate* " + convention + "<" + string.Join(",", parts) + ">";
    }

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

    static Member Add(string path, Unit unit)
    {
        var key = unit.Project + "\n" + unit.Name + "\n" + path;
        if (!Members.TryGetValue(key, out var member))
        {
            member = new Member(path, unit);
            Members[key] = member;
        }
        return member;
    }

    static void ReadNamespace(INamespaceSymbol ns, Unit unit)
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

    /// An extension block's path: `<static class>.extension(<receiver type>)`.
    static string ExtensionPath(INamedTypeSymbol block) =>
        TypePath(block.ContainingType) + ".extension(" + MetadataTypePath(block.ExtensionParameter!.Type) + ")";

    /// The methods the compiler emits in a static class to implement its extension
    /// blocks' members. They are artifacts of the lowering, not entities.
    static HashSet<ISymbol> ExtensionImplementations(INamedTypeSymbol type)
    {
        var found = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
        foreach (var block in type.GetTypeMembers().Where(t => t.IsExtension))
        {
            foreach (var member in block.GetMembers())
            {
                var methods = member switch
                {
                    IMethodSymbol m => [m],
                    IPropertySymbol p => new[] { p.GetMethod, p.SetMethod },
                    _ => [],
                };
                foreach (var method in methods)
                {
                    if (method?.AssociatedExtensionImplementation is { } implementation)
                    {
                        found.Add(implementation);
                    }
                }
            }
        }
        return found;
    }

    static void ReadType(INamedTypeSymbol type, Unit unit)
    {
        if (type.IsExtension)
        {
            // An extension block is no member; its members are, under its path.
            ReadMembers(type, ExtensionPath(type), unit);
            return;
        }
        // Metadata holds types the compiler emits: unspeakable `<...>` types, which
        // include file-local types under their mangled names, and the attribute types it
        // embeds. None is an entity a consumer can name.
        if (type.Name.StartsWith('<') || type.GetAttributes().Any(a => a.AttributeClass?.ToDisplayString() == "Microsoft.CodeAnalysis.EmbeddedAttribute"))
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
        ReadMembers(type, path, unit);
        foreach (var nested in type.GetTypeMembers())
        {
            ReadType(nested, unit);
        }
    }

    static void ReadMembers(INamedTypeSymbol type, string path, Unit unit)
    {
        var artifacts = ExtensionImplementations(type);
        var included = type.GetMembers().Where(m => Included(type, m) && !artifacts.Contains(m)).ToList();
        var names = included.GroupBy(MemberName).ToDictionary(g => g.Key, g => g.Count());
        foreach (var child in included)
        {
            var name = MemberName(child);
            if (names[name] > 1)
            {
                name += child switch
                {
                    // A generic method's name carries its arity, because `M()` and
                    // `M<T>()` can overload each other.
                    IMethodSymbol m => (m.Arity > 0 ? "``" + m.Arity : "") + ParameterList(m.Parameters),
                    IPropertySymbol p => ParameterList(p.Parameters),
                    _ => "",
                };
            }
            ReadMember(child, Add(path + "." + name, unit));
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
            // An instance field of a readonly struct is readonly by the struct; a static
            // field's readonly is its own choice.
            IFieldSymbol field => field.IsReadOnly && !(field.ContainingType.IsReadOnly && !field.IsStatic),
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
        // A class type constraint is primary and comes first; the interface and type
        // parameter constraints after it are sorted.
        var types = parameter.ConstraintTypes;
        parts.AddRange(types.Where(t => t.TypeKind == TypeKind.Class).Select(TypeText));
        parts.AddRange(types.Where(t => t.TypeKind != TypeKind.Class).Select(TypeText).Order(StringComparer.Ordinal));
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
