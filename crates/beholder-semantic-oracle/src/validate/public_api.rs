//! Public API recomputation: model validation criterion 7.
//!
//! The model requires every `public-api` entry to agree with the membership
//! recomputed from `exports`, `parent`, visibility and the profile's public
//! API rule. This module walks each distribution's consumer paths by the
//! profile rule, context by context, and compares the result with the
//! dataset's entries: the set of members, each member's complete path list,
//! and its access.
//!
//! The walk reads only dataset facts. Facts the profile cannot recompute
//! from source structure alone are inputs: the TypeScript entry module of
//! each subpath and mode (`typescript:entry`), and the visibility of each
//! export binding (`exports` `visibility`). Review confirms those against
//! configuration and source.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{list, names, text, value_in, Checker, Class, Contexts};
use crate::rules::PublicApiRule;

/// `all` or `subtypes`; a member reached by both is `all`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Access {
    All,
    Subtypes,
}

impl Access {
    fn as_str(self) -> &'static str {
        match self {
            Access::All => "all",
            Access::Subtypes => "subtypes",
        }
    }

    fn widest(self, other: Access) -> Access {
        self.min(other)
    }
}

/// One member in one context: its consumer paths and access.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Member {
    paths: BTreeSet<String>,
    access: Access,
}

/// `(distribution, entity)` → context → member.
type Membership<'a> = BTreeMap<(&'a str, &'a str), BTreeMap<&'a str, Member>>;

/// `(distribution, entity)` → context → the stating entry's index and its
/// member.
type Stated<'a> = BTreeMap<(&'a str, &'a str), BTreeMap<&'a str, (usize, Member)>>;

/// The facts the walk reads, indexed once.
struct Index<'a> {
    children: BTreeMap<&'a str, Vec<&'a Value>>,
    exports: BTreeMap<&'a str, Vec<(&'a Value, Contexts<'a>)>>,
    /// `aliases` relations: alias → (aliased entity, contexts).
    aliases: BTreeMap<&'a str, Vec<(&'a str, Contexts<'a>)>>,
    by_path: BTreeMap<&'a str, Vec<&'a Value>>,
}

/// A walk of one distribution in one context.
struct Walk<'c, 'a> {
    checker: &'c Checker<'a>,
    index: &'c Index<'a>,
    context: &'a str,
    distribution: &'a str,
    found: BTreeMap<&'a str, Member>,
    /// Containers on the current path, so that a cycle of re-exports ends.
    stack: Vec<&'a str>,
}

impl<'c, 'a> Walk<'c, 'a> {
    fn add(&mut self, id: &'a str, path: String, access: Access) {
        let member = self.found.entry(id).or_insert_with(|| Member {
            paths: BTreeSet::new(),
            access,
        });
        member.paths.insert(path);
        member.access = member.access.widest(access);
    }

    fn entity(&self, id: &str) -> Option<&'a Value> {
        match self.checker.ids.get(id) {
            Some((Class::Entity, record)) => Some(*record),
            _ => None,
        }
    }

    fn exists(&self, id: &str) -> bool {
        self.checker
            .entity_contexts
            .get(id)
            .is_some_and(|contexts| contexts.contains(self.context))
    }

    /// The children of `id` that exist in the context.
    fn children(&self, id: &str) -> Vec<&'a Value> {
        self.index
            .children
            .get(id)
            .into_iter()
            .flatten()
            .copied()
            .filter(|child| text(child, "id").is_some_and(|c| self.exists(c)))
            .collect()
    }

    /// The `exports` relations from `id` that hold in the context.
    fn exports(&self, id: &str) -> Vec<&'a Value> {
        self.index
            .exports
            .get(id)
            .into_iter()
            .flatten()
            .filter(|(_, contexts)| contexts.contains(self.context))
            .map(|(relation, _)| *relation)
            .collect()
    }

    fn declared(&self, entity: &'a Value) -> Option<&'a str> {
        entity
            .get("declared-visibility")
            .and_then(|v| value_in(v, self.context))
            .and_then(|v| text(v, "value"))
    }

    fn language_kind(&self, entity: &'a Value) -> Option<&'a str> {
        entity
            .get("language-kind")
            .and_then(|v| value_in(v, self.context))
            .and_then(Value::as_str)
    }

    fn enter(&mut self, id: &'a str) -> bool {
        if self.stack.contains(&id) {
            return false;
        }
        self.stack.push(id);
        true
    }

    // ---------------------------------------------------------------- Rust

    /// A Rust module's namespace as a downstream crate sees it: its `pub`
    /// items, its `pub` re-exports and the names its `pub` globs add.
    fn rust_module(&mut self, module: &'a str, prefix: &str) {
        if !self.enter(module) {
            return;
        }
        for child in self.children(module) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter" | "extension") {
                continue;
            }
            if self.declared(child) != Some("pub") {
                continue;
            }
            if let (Some(id), Some(name)) = (text(child, "id"), text(child, "name")) {
                self.rust_member(id, format!("{prefix}::{name}"));
            }
        }
        for relation in self.exports(module) {
            if text(relation, "visibility") != Some("pub") {
                continue;
            }
            if let (Some(to), Some(name)) = (text(relation, "to"), text(relation, "name")) {
                self.rust_member(to, format!("{prefix}::{name}"));
            }
        }
        self.stack.pop();
    }

    fn rust_member(&mut self, id: &'a str, path: String) {
        // A path that returns to a container on it is not a consumer path.
        if self.stack.contains(&id) {
            return;
        }
        self.add(id, path.clone(), Access::All);
        let Some(entity) = self.entity(id) else {
            return;
        };
        match text(entity, "kind").unwrap_or_default() {
            "module" => self.rust_module(id, &path),
            "type" | "enum-member" => self.rust_type(id, entity, &path, false),
            // A type alias of a nominal type names that type's members.
            "type-alias" => {
                let aliased: Vec<&'a str> = self
                    .index
                    .aliases
                    .get(id)
                    .into_iter()
                    .flatten()
                    .filter(|(_, contexts)| contexts.contains(self.context))
                    .map(|(to, _)| *to)
                    .collect();
                for to in aliased {
                    if let Some(target) = self.entity(to) {
                        if text(target, "kind") == Some("type") && self.enter(id) {
                            self.rust_type(to, target, &path, true);
                            self.stack.pop();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// The members of a member type: enum variants and their fields, `pub`
    /// fields, trait items, and `pub` inherent associated items. Through a
    /// type alias, only the members a path names: variants and inherent
    /// associated items.
    fn rust_type(&mut self, id: &'a str, entity: &'a Value, path: &str, alias: bool) {
        if !self.enter(id) {
            return;
        }
        for child in self.children(id) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter" | "extension")
                || (alias && kind == "field")
            {
                continue;
            }
            if !matches!(self.declared(child), Some("pub" | "inherited")) {
                continue;
            }
            if let (Some(child_id), Some(name)) = (text(child, "id"), text(child, "name")) {
                self.rust_member(child_id, format!("{path}::{name}"));
            }
        }
        // Inherent associated items: children of an inherent `impl` block
        // whose canonical path continues the type's canonical path.
        if let Some(type_path) = text(entity, "path") {
            let unit = text(entity, "unit");
            let inherent: Vec<(&'a str, &'a str)> = self
                .index
                .by_path
                .iter()
                .filter_map(|(item_path, records)| {
                    let name = item_path.strip_prefix(type_path)?.strip_prefix("::")?;
                    (!name.contains("::")).then_some((name, records))
                })
                .flat_map(|(name, records)| records.iter().map(move |r| (name, *r)))
                .filter(|(_, item)| text(item, "unit") == unit)
                .filter(|(_, item)| {
                    text(item, "parent")
                        .and_then(|parent| self.entity(parent))
                        .is_some_and(|block| self.language_kind(block) == Some("impl"))
                })
                .filter(|(_, item)| self.declared(item) == Some("pub"))
                .filter_map(|(name, item)| {
                    let id = text(item, "id")?;
                    self.exists(id).then_some((name, id))
                })
                .collect();
            for (name, item) in inherent {
                self.add(item, format!("{path}::{name}"), Access::All);
            }
        }
        self.stack.pop();
    }

    // ---------------------------------------------------------- TypeScript

    /// A module's or namespace's export namespace: exported declarations
    /// and the names `export` specifiers and `export *` add.
    fn typescript_exports(&mut self, container: &'a str, prefix: &str, separator: &str) {
        if !self.enter(container) {
            return;
        }
        for child in self.children(container) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter") {
                continue;
            }
            let name = match self.declared(child) {
                Some("exported") => text(child, "name"),
                Some("exported-default") => Some("default"),
                _ => None,
            };
            if let (Some(id), Some(name)) = (text(child, "id"), name) {
                self.typescript_member(id, format!("{prefix}{separator}{name}"), Access::All);
            }
        }
        for relation in self.exports(container) {
            if let (Some(to), Some(name)) = (text(relation, "to"), text(relation, "name")) {
                self.typescript_member(to, format!("{prefix}{separator}{name}"), Access::All);
            }
        }
        self.stack.pop();
    }

    fn typescript_member(&mut self, id: &'a str, path: String, access: Access) {
        if self.stack.contains(&id) {
            return;
        }
        self.add(id, path.clone(), access);
        let Some(entity) = self.entity(id) else {
            return;
        };
        match text(entity, "kind").unwrap_or_default() {
            "module" | "namespace" => self.typescript_exports(id, &path, "."),
            "type" => self.typescript_type(id, &path, access),
            _ => {}
        }
    }

    /// The `public` and `protected` members of an exported class,
    /// interface or enum. Constructors and unnamed signatures have no
    /// member name a consumer writes.
    fn typescript_type(&mut self, id: &'a str, path: &str, access: Access) {
        if !self.enter(id) {
            return;
        }
        for child in self.children(id) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter") {
                continue;
            }
            if self.language_kind(child) == Some("constructor") {
                continue;
            }
            let member_access = match self.declared(child) {
                Some("public" | "inherited") => access,
                Some("protected") => Access::Subtypes,
                _ if kind == "enum-member" => access,
                _ => continue,
            };
            let Some((child_id, name)) = text(child, "id").zip(text(child, "name")) else {
                continue;
            };
            // A static member's segment is `static:<name>`.
            let segment = match self.language_kind(child) {
                Some(kind) if kind.starts_with("static-") => format!("static:{name}"),
                _ => name.to_owned(),
            };
            self.add(child_id, format!("{path}.{segment}"), member_access);
        }
        self.stack.pop();
    }

    // -------------------------------------------------------------- Python

    /// A public module's public names by the library interface rules.
    fn python_module(&mut self, module: &'a str, name: &str) {
        self.add(module, name.to_owned(), Access::All);
        let listed: Vec<&'a str> = self.checker.python_all_names(module, self.context);
        if !listed.is_empty() {
            for binding in listed {
                if let Some(target) = self.python_binding(module, name, binding) {
                    self.python_member(target, format!("{name}.{binding}"));
                }
            }
            return;
        }
        for child in self.children(module) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "module" | "parameter" | "type-parameter") {
                continue;
            }
            if let (Some(id), Some(binding)) = (text(child, "id"), text(child, "name")) {
                if !binding.starts_with('_') {
                    self.python_member(id, format!("{name}.{binding}"));
                }
            }
        }
        for relation in self.exports(module) {
            if text(relation, "visibility") != Some("re-export") {
                continue;
            }
            if let (Some(to), Some(binding)) = (text(relation, "to"), text(relation, "name")) {
                if !binding.starts_with('_') {
                    self.python_member(to, format!("{name}.{binding}"));
                }
            }
        }
    }

    /// The entity a module-level name denotes: a binding the module
    /// declares, an import binding, or a submodule.
    fn python_binding(&self, module: &'a str, module_name: &str, binding: &str) -> Option<&'a str> {
        let declared = self
            .children(module)
            .into_iter()
            .filter(|child| text(child, "kind") != Some("module"))
            .find(|child| text(child, "name") == Some(binding))
            .and_then(|child| text(child, "id"));
        let imported = || {
            self.exports(module)
                .into_iter()
                .find(|relation| text(relation, "name") == Some(binding))
                .and_then(|relation| text(relation, "to"))
        };
        let submodule = || {
            let path = format!("{module_name}.{binding}");
            let modules: Vec<(&'a str, bool)> = self
                .index
                .by_path
                .get(path.as_str())
                .into_iter()
                .flatten()
                .filter(|record| text(record, "kind") == Some("module"))
                .filter_map(|record| {
                    let id = text(record, "id")?;
                    self.exists(id).then(|| (id, self.python_stub(record)))
                })
                .collect();
            // A typing context reads the stub where one exists; a runtime
            // context reads the implementation.
            let stub = modules.iter().find(|(_, stub)| *stub).map(|(id, _)| *id);
            let implementation = modules.iter().find(|(_, stub)| !*stub).map(|(id, _)| *id);
            if self.checker.python_typing(self.context) {
                stub.or(implementation)
            } else {
                implementation
            }
        };
        declared.or_else(imported).or_else(submodule)
    }

    /// Whether a module entity is a stub: a unit of kind `stub` defines it.
    fn python_stub(&self, record: &'a Value) -> bool {
        self.checker.units_of(record).iter().any(|unit| {
            matches!(self.checker.ids.get(unit), Some((Class::Unit, u)) if text(u, "kind") == Some("stub"))
        })
    }

    fn python_member(&mut self, id: &'a str, path: String) {
        if self.stack.contains(&id) {
            return;
        }
        self.add(id, path.clone(), Access::All);
        if let Some(entity) = self.entity(id) {
            if text(entity, "kind") == Some("type") {
                self.python_class(id, &path);
            }
        }
    }

    /// A public class's members: names without a leading underscore, and
    /// dunder names.
    fn python_class(&mut self, id: &'a str, path: &str) {
        if !self.enter(id) {
            return;
        }
        for child in self.children(id) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter") {
                continue;
            }
            let (Some(child_id), Some(name)) = (text(child, "id"), text(child, "name")) else {
                continue;
            };
            if python_class_member(name) {
                self.python_member(child_id, format!("{path}.{name}"));
            }
        }
        // A class-level import binds a class attribute.
        for relation in self.exports(id) {
            if let (Some(to), Some(name)) = (text(relation, "to"), text(relation, "name")) {
                if python_class_member(name) {
                    self.python_member(to, format!("{path}.{name}"));
                }
            }
        }
        self.stack.pop();
    }
}

impl<'c, 'a> Walk<'c, 'a> {
    // ------------------------------------------------------------------ Go

    /// An exported package-level declaration, and for an exported type its
    /// exported fields and methods, including promoted ones.
    fn go_package(&mut self, package: &'a str) {
        for child in self.children(package) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter") {
                continue;
            }
            if self.declared(child) != Some("exported") {
                continue;
            }
            let (Some(id), Some(path)) = (text(child, "id"), text(child, "path")) else {
                continue;
            };
            self.add(id, path.to_owned(), Access::All);
            if kind == "type" {
                self.go_type(id, path);
            }
        }
    }

    /// The exported fields and methods of an exported type: its own, then
    /// those promoted through embedded fields at the shallowest depth at
    /// which a name occurs once. A name the type declares itself, or that
    /// occurs twice at one depth, is not promoted.
    fn go_type(&mut self, id: &'a str, path: &str) {
        let mut seen: BTreeSet<&'a str> = BTreeSet::new();
        // Each embedding path to a type at this depth, as the chain of types
        // it passes. One type reached by two paths, as in a diamond, counts
        // twice, so its members are ambiguous at that depth.
        let mut level: Vec<Vec<&'a str>> = vec![vec![id]];
        while !level.is_empty() {
            let mut names: BTreeMap<&'a str, Vec<&'a Value>> = BTreeMap::new();
            for chain in &level {
                let owner = chain[chain.len() - 1];
                for child in self.children(owner) {
                    if matches!(text(child, "kind"), Some("parameter" | "type-parameter")) {
                        continue;
                    }
                    if let Some(name) = text(child, "name") {
                        names.entry(name).or_default().push(child);
                    }
                }
            }
            for (name, members) in &names {
                if !seen.insert(name) || members.len() != 1 {
                    continue;
                }
                let member = members[0];
                if self.declared(member) == Some("exported") {
                    if let Some(member_id) = text(member, "id") {
                        self.add(member_id, format!("{path}.{name}"), Access::All);
                    }
                }
            }
            let next: Vec<Vec<&'a str>> = level
                .iter()
                .flat_map(|chain| {
                    let owner = chain[chain.len() - 1];
                    self.checker
                        .go_embedded(owner, self.context)
                        .into_iter()
                        // An embedding cycle ends where a path returns to a
                        // type it passed.
                        .filter(|embedded| !chain.contains(embedded))
                        .map(|embedded| {
                            let mut longer = chain.clone();
                            longer.push(embedded);
                            longer
                        })
                })
                .collect();
            level = next;
        }
    }

    // ---------------------------------------------------------------- Dart

    /// A public library's export namespace and the public members of the
    /// classes, mixins, enums, extensions and extension types it exports.
    fn dart_library(&mut self, library: &'a str, uri: &str) {
        for child in self.children(library) {
            let kind = text(child, "kind").unwrap_or_default();
            if matches!(kind, "parameter" | "type-parameter") {
                continue;
            }
            if self.declared(child) != Some("public") {
                continue;
            }
            if let (Some(id), Some(name)) = (text(child, "id"), text(child, "name")) {
                self.dart_member(id, format!("{uri}::{name}"));
            }
        }
        for relation in self.exports(library) {
            if let (Some(to), Some(name)) = (text(relation, "to"), text(relation, "name")) {
                self.dart_member(to, format!("{uri}::{name}"));
            }
        }
    }

    fn dart_member(&mut self, id: &'a str, path: String) {
        self.add(id, path.clone(), Access::All);
        let Some(entity) = self.entity(id) else {
            return;
        };
        if !matches!(text(entity, "kind"), Some("type" | "extension")) {
            return;
        }
        for child in self.children(id) {
            if matches!(text(child, "kind"), Some("parameter" | "type-parameter")) {
                continue;
            }
            let (Some(child_id), Some(name)) = (text(child, "id"), text(child, "name")) else {
                continue;
            };
            if !name.starts_with('_') {
                self.add(child_id, format!("{path}.{name}"), Access::All);
            }
        }
    }
}

impl<'a> Checker<'a> {
    /// The types a Go type embeds, by `go:embeds`, in a context.
    fn go_embedded(&self, owner: &str, context: &str) -> Vec<&'a str> {
        self.extension_facts("go:embeds")
            .iter()
            .filter(|fact| text(fact, "struct") == Some(owner))
            .filter(|fact| {
                self.declared(fact)
                    .unwrap_or_else(|| self.built_contexts(owner))
                    .contains(context)
            })
            .filter_map(|fact| text(fact, "type"))
            .collect()
    }

    fn extension_facts(&self, kind: &str) -> &'a [Value] {
        self.dataset
            .get("extensions")
            .and_then(|e| e.get(kind))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// C#: every entity whose `external` access is `all` or `subtypes`,
    /// under its own path, and every forwarded type.
    fn csharp_members(&self, context: &'a str, membership: &mut Membership<'a>) {
        let distribution_of = |unit: &str| match self.ids.get(unit) {
            Some((Class::Unit, record)) => text(record, "distribution"),
            _ => None,
        };
        for entity in self.list("entities") {
            let (Some(id), Some(path)) = (text(entity, "id"), text(entity, "path")) else {
                continue;
            };
            // Namespaces are not members, and an extension block is not a
            // name a consumer writes; its members are.
            if matches!(
                text(entity, "kind"),
                Some("namespace" | "extension" | "parameter" | "type-parameter")
            ) || !self.entity_contexts(id).contains(context)
            {
                continue;
            }
            let Some(external) = entity
                .get("effective-visibility")
                .and_then(|v| value_in(v, context))
                .and_then(|v| v.get("external"))
            else {
                continue;
            };
            let accesses: Vec<&Value> = match external.as_array() {
                Some(items) => items.iter().collect(),
                None => vec![external],
            };
            let access = if accesses.iter().any(|a| a.as_str() == Some("all")) {
                Access::All
            } else if accesses.iter().any(|a| a.as_str() == Some("subtypes")) {
                Access::Subtypes
            } else {
                continue;
            };
            for unit in self.units_of(entity) {
                if let Some(distribution) = distribution_of(unit) {
                    membership.entry((distribution, id)).or_default().insert(
                        context,
                        Member {
                            paths: BTreeSet::from([path.to_owned()]),
                            access,
                        },
                    );
                }
            }
        }
        for fact in self.extension_facts("csharp:type-forward") {
            let (Some(unit), Some(forwarded)) = (text(fact, "unit"), text(fact, "type")) else {
                continue;
            };
            let holds = self
                .declared(fact)
                .unwrap_or_else(|| self.built_contexts(unit))
                .contains(context);
            let path = self.ids.get(forwarded).and_then(|(_, r)| text(r, "path"));
            if let (true, Some(distribution), Some(path)) = (holds, distribution_of(unit), path) {
                membership
                    .entry((distribution, forwarded))
                    .or_default()
                    .insert(
                        context,
                        Member {
                            paths: BTreeSet::from([path.to_owned()]),
                            access: Access::All,
                        },
                    );
            }
        }
    }

    /// Whether a Python context is a `typing` context.
    fn python_typing(&self, context: &str) -> bool {
        self.list("contexts")
            .iter()
            .find(|c| text(c, "id") == Some(context))
            .and_then(|c| c.get("parameters"))
            .and_then(|p| text(p, "mode"))
            == Some("typing")
    }

    /// The names a module's static `__all__` lists in a context.
    fn python_all_names(&self, module: &str, context: &str) -> Vec<&'a str> {
        let facts = self
            .dataset
            .get("extensions")
            .and_then(|e| e.get("python:all-member"))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        facts
            .iter()
            .filter(|fact| text(fact, "module") == Some(module))
            .filter(|fact| {
                self.declared(fact)
                    .unwrap_or_else(|| self.built_contexts(module))
                    .contains(context)
            })
            .filter_map(|fact| text(fact, "name"))
            .collect()
    }

    fn public_api_index(&self) -> Index<'a> {
        let mut children: BTreeMap<&'a str, Vec<&'a Value>> = BTreeMap::new();
        let mut by_path: BTreeMap<&'a str, Vec<&'a Value>> = BTreeMap::new();
        for entity in self.list("entities") {
            if let Some(parent) = text(entity, "parent") {
                children.entry(parent).or_default().push(entity);
            }
            if let Some(path) = text(entity, "path") {
                by_path.entry(path).or_default().push(entity);
            }
        }
        let mut exports: BTreeMap<&'a str, Vec<(&'a Value, Contexts<'a>)>> = BTreeMap::new();
        let mut aliases: BTreeMap<&'a str, Vec<(&'a str, Contexts<'a>)>> = BTreeMap::new();
        for relation in self.list("relations") {
            let Some(from) = text(relation, "from") else {
                continue;
            };
            let contexts = self
                .declared(relation)
                .unwrap_or_else(|| self.built_contexts(from));
            match (text(relation, "kind"), text(relation, "to")) {
                (Some("exports"), _) => exports.entry(from).or_default().push((relation, contexts)),
                (Some("aliases"), Some(to)) => {
                    aliases.entry(from).or_default().push((to, contexts))
                }
                _ => {}
            }
        }
        Index {
            children,
            exports,
            aliases,
            by_path,
        }
    }

    /// The membership the profile rule yields, in every context.
    fn recompute_public_api(&self, index: &Index<'a>) -> Membership<'a> {
        let mut membership: Membership<'a> = BTreeMap::new();
        for context in self.all.iter().copied() {
            if self.rules.public_api == PublicApiRule::CSharp {
                self.csharp_members(context, &mut membership);
                continue;
            }
            for (distribution, entries) in self.public_api_entries(context) {
                let mut walk = Walk {
                    checker: self,
                    index,
                    context,
                    distribution,
                    found: BTreeMap::new(),
                    stack: Vec::new(),
                };
                for entry in entries {
                    match entry {
                        Entry::RustCrate { root, name } => {
                            walk.rust_member(root, name.to_owned());
                        }
                        Entry::TypeScript { module, prefix } => {
                            walk.typescript_exports(module, &prefix, "");
                        }
                        Entry::PythonModule { module, name } => {
                            walk.python_module(module, name);
                        }
                        Entry::GoPackage { root } => walk.go_package(root),
                        Entry::DartLibrary { root, uri } => walk.dart_library(root, uri),
                    }
                }
                for (id, member) in walk.found {
                    membership
                        .entry((walk.distribution, id))
                        .or_default()
                        .insert(context, member);
                }
            }
        }
        membership
    }

    /// Each distribution's starting points in a context, by the profile.
    fn public_api_entries(&self, context: &'a str) -> BTreeMap<&'a str, Vec<Entry<'a>>> {
        let mut entries: BTreeMap<&'a str, Vec<Entry<'a>>> = BTreeMap::new();
        let built = |unit: &str| {
            self.unit_contexts
                .get(unit)
                .is_some_and(|contexts| contexts.contains(context))
        };
        match self.rules.public_api {
            PublicApiRule::Rust => {
                for unit in self.list("units") {
                    let (Some(id), Some(distribution), Some(root), Some(name)) = (
                        text(unit, "id"),
                        text(unit, "distribution"),
                        text(unit, "root"),
                        text(unit, "name"),
                    ) else {
                        continue;
                    };
                    if matches!(text(unit, "kind"), Some("lib" | "proc-macro")) && built(id) {
                        entries
                            .entry(distribution)
                            .or_default()
                            .push(Entry::RustCrate { root, name });
                    }
                }
            }
            PublicApiRule::TypeScript => {
                let facts = self
                    .dataset
                    .get("extensions")
                    .and_then(|e| e.get("typescript:entry"))
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                for fact in facts {
                    let (Some(distribution), Some(mode), Some(subpath), Some(module)) = (
                        text(fact, "distribution"),
                        text(fact, "mode"),
                        text(fact, "subpath"),
                        text(fact, "module"),
                    ) else {
                        continue;
                    };
                    let holds = self
                        .declared(fact)
                        .unwrap_or_else(|| self.built_contexts(module))
                        .contains(context);
                    let Some((Class::Distribution, record)) = self.ids.get(distribution) else {
                        continue;
                    };
                    let (true, Some(package)) = (holds, text(record, "name")) else {
                        continue;
                    };
                    let rest = subpath.strip_prefix('.').unwrap_or(subpath);
                    entries
                        .entry(distribution)
                        .or_default()
                        .push(Entry::TypeScript {
                            module,
                            prefix: format!("{mode}:{package}{rest}#"),
                        });
                }
            }
            PublicApiRule::Python => {
                let typing = self.python_typing(context);
                let distributed: Vec<&'a Value> = self
                    .list("units")
                    .iter()
                    .filter(|unit| text(unit, "distribution").is_some())
                    .filter(|unit| text(unit, "id").is_some_and(built))
                    .collect();
                let stubbed: BTreeSet<(&str, &str)> = distributed
                    .iter()
                    .filter(|unit| text(unit, "kind") == Some("stub"))
                    .filter_map(|unit| Some((text(unit, "distribution")?, text(unit, "name")?)))
                    .collect();
                for unit in distributed {
                    let (Some(distribution), Some(name)) =
                        (text(unit, "distribution"), text(unit, "name"))
                    else {
                        continue;
                    };
                    let stub = text(unit, "kind") == Some("stub");
                    // Typing contexts read a module's stub where one exists;
                    // runtime contexts read the implementation.
                    let applies = if typing {
                        stub || !stubbed.contains(&(distribution, name))
                    } else {
                        !stub
                    };
                    if !applies || name.split('.').any(|part| part.starts_with('_')) {
                        continue;
                    }
                    let Some(module) = text(unit, "root") else {
                        continue;
                    };
                    entries
                        .entry(distribution)
                        .or_default()
                        .push(Entry::PythonModule { module, name });
                }
            }
            PublicApiRule::Go => {
                for unit in self.list("units") {
                    let (Some(id), Some(distribution), Some(root), Some(name)) = (
                        text(unit, "id"),
                        text(unit, "distribution"),
                        text(unit, "root"),
                        text(unit, "name"),
                    ) else {
                        continue;
                    };
                    let internal = name.split('/').any(|element| element == "internal");
                    if text(unit, "kind") == Some("package") && !internal && built(id) {
                        entries
                            .entry(distribution)
                            .or_default()
                            .push(Entry::GoPackage { root });
                    }
                }
            }
            PublicApiRule::Dart => {
                for unit in self.list("units") {
                    let (Some(id), Some(distribution), Some(root), Some(uri)) = (
                        text(unit, "id"),
                        text(unit, "distribution"),
                        text(unit, "root"),
                        text(unit, "name"),
                    ) else {
                        continue;
                    };
                    // `package:<package>/<path>`, outside `lib/src/`.
                    let public = uri
                        .strip_prefix("package:")
                        .and_then(|rest| rest.split_once('/'))
                        .is_some_and(|(_, path)| !path.starts_with("src/"));
                    if public && built(id) {
                        entries
                            .entry(distribution)
                            .or_default()
                            .push(Entry::DartLibrary { root, uri });
                    }
                }
            }
            PublicApiRule::CSharp => {}
        }
        entries
    }

    /// The dataset's `public-api` entries, by member and context.
    fn stated_public_api(&self) -> Stated<'a> {
        let mut stated: Stated<'a> = BTreeMap::new();
        for (i, entry) in self.list("public-api").iter().enumerate() {
            let (Some(distribution), Some(id)) =
                (text(entry, "distribution"), text(entry, "entity"))
            else {
                continue;
            };
            let contexts = self
                .declared(entry)
                .unwrap_or_else(|| match self.ids.get(id) {
                    Some((Class::Entity, _)) => self.built_contexts(id),
                    _ => self.all.clone(),
                });
            let access = match text(entry, "access") {
                Some("subtypes") => Access::Subtypes,
                _ => Access::All,
            };
            let paths: BTreeSet<String> = list(entry, "paths")
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            let by_context = stated.entry((distribution, id)).or_default();
            for context in contexts {
                by_context.insert(
                    context,
                    (
                        i,
                        Member {
                            paths: paths.clone(),
                            access,
                        },
                    ),
                );
            }
        }
        stated
    }

    /// Compares the recomputed membership with the dataset's entries.
    pub(super) fn check_public_api_recomputed(&mut self) {
        let index = self.public_api_index();
        let expected = self.recompute_public_api(&index);
        let stated = self.stated_public_api();

        // Findings that differ only in context are reported once.
        let mut grouped: BTreeMap<(String, String), Contexts<'a>> = BTreeMap::new();
        let keys: BTreeSet<(&'a str, &'a str)> =
            expected.keys().chain(stated.keys()).copied().collect();
        let none = BTreeMap::new();
        for key @ (distribution, id) in keys {
            let want = expected.get(&key).unwrap_or(&none);
            let have = stated.get(&key);
            for context in self.all.iter().copied() {
                let wanted = want.get(context);
                let had = have.and_then(|h| h.get(context));
                let (at, message) = match (wanted, had) {
                    (None, None) => continue,
                    (Some(member), None) => (
                        "/public-api".to_owned(),
                        format!(
                            "the profile rule makes `{id}` a member of `{distribution}` with \
                             paths {} and access `{}`; no entry states it",
                            path_list(&member.paths),
                            member.access.as_str()
                        ),
                    ),
                    // `model:context-scope` reports an entry whose entity
                    // does not exist in the context.
                    (None, Some(_))
                        if matches!(self.ids.get(id), Some((Class::Entity, _)))
                            && !self.entity_contexts(id).contains(context) =>
                    {
                        continue
                    }
                    (None, Some((i, _))) => (
                        format!("/public-api/{i}"),
                        format!(
                            "the profile rule gives `{id}` no consumer path in `{distribution}`"
                        ),
                    ),
                    (Some(member), Some((i, stated))) => {
                        if member.paths != stated.paths {
                            let missing: BTreeSet<String> =
                                member.paths.difference(&stated.paths).cloned().collect();
                            let extra: BTreeSet<String> =
                                stated.paths.difference(&member.paths).cloned().collect();
                            let mut parts = Vec::new();
                            if !missing.is_empty() {
                                parts.push(format!("missing {}", path_list(&missing)));
                            }
                            if !extra.is_empty() {
                                parts.push(format!("not consumer paths {}", path_list(&extra)));
                            }
                            grouped
                                .entry((
                                    format!("/public-api/{i}/paths"),
                                    format!(
                                        "the profile rule gives `{id}` the paths {}: {}",
                                        path_list(&member.paths),
                                        parts.join("; ")
                                    ),
                                ))
                                .or_default()
                                .insert(context);
                        }
                        if member.access != stated.access {
                            grouped
                                .entry((
                                    format!("/public-api/{i}/access"),
                                    format!(
                                        "the profile rule gives `{id}` access `{}`",
                                        member.access.as_str()
                                    ),
                                ))
                                .or_default()
                                .insert(context);
                        }
                        continue;
                    }
                };
                grouped.entry((at, message)).or_default().insert(context);
            }
        }
        if self.rules.public_api == PublicApiRule::Rust {
            self.check_rust_external_visibility(&expected, &mut grouped);
        }
        for ((at, message), contexts) in grouped {
            self.error(
                "model:public-api-recomputed",
                at,
                format!("{message} in {}", names(&contexts)),
            );
        }
    }

    /// Rust: `external` (and `project`) is `all` exactly when a downstream
    /// crate can name the entity by a consumer path.
    fn check_rust_external_visibility(
        &self,
        expected: &Membership<'a>,
        grouped: &mut BTreeMap<(String, String), Contexts<'a>>,
    ) {
        let members: BTreeMap<&'a str, BTreeSet<&'a str>> =
            expected
                .iter()
                .fold(BTreeMap::new(), |mut members, ((_, id), contexts)| {
                    members
                        .entry(*id)
                        .or_default()
                        .extend(contexts.keys().copied());
                    members
                });
        for (i, entity) in self.list("entities").iter().enumerate() {
            let (Some(id), Some(visibility)) =
                (text(entity, "id"), entity.get("effective-visibility"))
            else {
                continue;
            };
            let distributed = self.units_of(entity).iter().any(|unit| {
                matches!(self.ids.get(unit), Some((Class::Unit, u))
                    if text(u, "distribution").is_some()
                        && matches!(text(u, "kind"), Some("lib" | "proc-macro")))
            });
            // A trait implementation item is never a member; its effective
            // visibility is that of the trait item it implements.
            let trait_impl_item = text(entity, "parent")
                .and_then(|parent| self.ids.get(parent))
                .is_some_and(|(_, block)| {
                    block
                        .get("language-kind")
                        .is_some_and(|kind| kind.as_str() == Some("trait-impl"))
                });
            if !distributed || trait_impl_item {
                continue;
            }
            let named = members.get(id);
            for context in self.entity_contexts(id) {
                let Some(value) = value_in(visibility, context) else {
                    continue;
                };
                let open = named.is_some_and(|contexts| contexts.contains(context));
                for boundary in ["external", "project"] {
                    let stated = value.get(boundary).map(super::external_access);
                    let want = if open { "all" } else { "none" };
                    if stated.is_some_and(|stated| stated != want) {
                        grouped
                            .entry((
                                format!("/entities/{i}/effective-visibility"),
                                format!(
                                    "`{boundary}` is `{want}` by the consumer-path rule: the \
                                     recomputed public API {} `{id}`",
                                    if open { "names" } else { "does not name" }
                                ),
                            ))
                            .or_default()
                            .insert(context);
                    }
                }
            }
        }
    }
}

/// One starting point of a distribution's consumer paths.
enum Entry<'a> {
    /// A `lib` or `proc-macro` crate root and its crate name.
    RustCrate { root: &'a str, name: &'a str },
    /// A TypeScript entry module and its `<mode>:<specifier>#` prefix.
    TypeScript { module: &'a str, prefix: String },
    /// A public Python module and its module name.
    PythonModule { module: &'a str, name: &'a str },
    /// A Go `package` unit outside `internal` directories.
    GoPackage { root: &'a str },
    /// A public Dart library and its canonical URI.
    DartLibrary { root: &'a str, uri: &'a str },
}

/// A class member name the library interface rules make public: no leading
/// underscore, or a dunder name.
fn python_class_member(name: &str) -> bool {
    let dunder = name.len() > 4 && name.starts_with("__") && name.ends_with("__");
    !name.starts_with('_') || dunder
}

fn path_list(paths: &BTreeSet<String>) -> String {
    let quoted: Vec<String> = paths.iter().map(|p| format!("`{p}`")).collect();
    format!("[{}]", quoted.join(", "))
}
