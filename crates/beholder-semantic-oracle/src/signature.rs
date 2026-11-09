// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-signatures
// ---
//! The slots of a signature record, in the model's slot paths.
//!
//! The model's comparison unit is one slot: its entity, its alternative and its
//! slot path. [`slots`] lists them from a record, with each slot's class and the
//! declaration that states it, for the validator and for adapters.

use serde_json::Value;

/// A slot's class, as a report's `slot-class` groups it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotClass {
    Shape,
    DeclaredType,
    InferredType,
    Value,
    AutoTrait,
}

impl SlotClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shape => "shape",
            Self::DeclaredType => "declared-type",
            Self::InferredType => "inferred-type",
            Self::Value => "value",
            Self::AutoTrait => "auto-trait",
        }
    }
}

/// The declaration that states a slot: the model's Source declarations table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source<'a> {
    Member,
    Overload(usize),
    /// `constructors[n]`, `call-signatures[n]`, `construct-signatures[n]` or
    /// `index-signatures[n]`.
    Group(&'a str, usize),
    /// The `impl` slot's `extension` entity.
    Impl,
    /// A `bases` entry, whose source depends on the language.
    Base,
    /// An `implementations` entry's `extension` entity.
    Implementation,
}

/// One slot of a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot<'a> {
    /// The model's slot path, such as `parameters[1].type`.
    pub path: String,
    /// The JSON Pointer of the slot in its record, without a leading record path.
    pub pointer: String,
    pub class: SlotClass,
    /// The slot's value as text; a whole element for a set slot.
    pub text: String,
    /// Whether the slot is `{inferred}`.
    pub inferred: bool,
    /// Whether this slot is a `default`.
    pub is_default: bool,
    pub source: Source<'a>,
    /// The index into `alternatives`.
    pub alternative: Option<usize>,
}

#[derive(Clone)]
struct Frame<'a> {
    prefix: String,
    pointer: String,
    source: Source<'a>,
    alternative: Option<usize>,
}

impl<'a> Frame<'a> {
    fn child(&self, key: &str) -> Frame<'a> {
        let dot = if self.prefix.is_empty() { "" } else { "." };
        Frame {
            prefix: format!("{}{dot}{key}", self.prefix),
            pointer: format!("{}/{key}", self.pointer),
            ..self.clone()
        }
    }

    fn indexed(&self, key: &str, index: usize) -> Frame<'a> {
        let dot = if self.prefix.is_empty() { "" } else { "." };
        Frame {
            prefix: format!("{}{dot}{key}[{index}]", self.prefix),
            pointer: format!("{}/{key}/{index}", self.pointer),
            ..self.clone()
        }
    }

    fn keyed(&self, key: &str, index: usize, item: &str) -> Frame<'a> {
        let dot = if self.prefix.is_empty() { "" } else { "." };
        Frame {
            prefix: format!("{}{dot}{key}[{item}]", self.prefix),
            pointer: format!("{}/{key}/{index}", self.pointer),
            ..self.clone()
        }
    }
}

/// Every slot of a record, in a stable order.
pub fn slots(record: &Value) -> Vec<Slot<'_>> {
    let mut out = Vec::new();
    let frame = Frame {
        prefix: String::new(),
        pointer: String::new(),
        source: Source::Member,
        alternative: None,
    };
    walk(record, &frame, &mut out);
    if let Some(alternatives) = record.get("alternatives").and_then(Value::as_array) {
        for (i, alternative) in alternatives.iter().enumerate() {
            let frame = Frame {
                pointer: format!("/alternatives/{i}"),
                alternative: Some(i),
                ..frame.clone()
            };
            walk(alternative, &frame, &mut out);
        }
    }
    out
}

fn shape<'a>(frame: &Frame<'a>, key: &str, value: Option<&Value>, out: &mut Vec<Slot<'a>>) {
    let Some(value) = value else {
        return;
    };
    let frame = frame.child(key);
    let text = value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned);
    out.push(Slot {
        path: frame.prefix,
        pointer: frame.pointer,
        class: SlotClass::Shape,
        text,
        inferred: false,
        is_default: false,
        source: frame.source,
        alternative: frame.alternative,
    });
}

fn typed<'a>(frame: &Frame<'a>, key: &str, value: Option<&Value>, out: &mut Vec<Slot<'a>>) {
    let Some(value) = value else {
        return;
    };
    let frame = frame.child(key);
    let (text, inferred) = unwrap(value);
    out.push(Slot {
        path: frame.prefix,
        pointer: frame.pointer,
        class: if inferred {
            SlotClass::InferredType
        } else {
            SlotClass::DeclaredType
        },
        text,
        inferred,
        is_default: false,
        source: frame.source,
        alternative: frame.alternative,
    });
}

fn valued<'a>(
    frame: &Frame<'a>,
    key: &str,
    value: Option<&Value>,
    is_default: bool,
    out: &mut Vec<Slot<'a>>,
) {
    let Some(value) = value else {
        return;
    };
    let frame = frame.child(key);
    let (text, inferred) = unwrap(value);
    out.push(Slot {
        path: frame.prefix,
        pointer: frame.pointer,
        class: SlotClass::Value,
        text,
        inferred,
        is_default,
        source: frame.source,
        alternative: frame.alternative,
    });
}

fn unwrap(value: &Value) -> (String, bool) {
    match value.get("inferred") {
        Some(inner) => (inner.as_str().unwrap_or_default().to_owned(), true),
        None => (value.as_str().unwrap_or_default().to_owned(), false),
    }
}

fn set<'a>(
    frame: &Frame<'a>,
    key: &str,
    record: &Value,
    class: SlotClass,
    out: &mut Vec<Slot<'a>>,
) {
    let Some(items) = record.get(key).and_then(Value::as_array) else {
        return;
    };
    for (i, item) in items.iter().enumerate() {
        let text = item.as_str().unwrap_or_default();
        let element = crate::canonical::auto_trait_key(text);
        let frame = frame.keyed(key, i, element);
        out.push(Slot {
            path: frame.prefix,
            pointer: frame.pointer,
            class,
            text: text.to_owned(),
            inferred: false,
            is_default: false,
            source: frame.source,
            alternative: frame.alternative,
        });
    }
}

fn items<'v>(record: &'v Value, key: &str) -> &'v [Value] {
    record
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn type_parameters<'a>(record: &Value, frame: &Frame<'a>, out: &mut Vec<Slot<'a>>) {
    for (i, parameter) in items(record, "type-parameters").iter().enumerate() {
        let frame = frame.indexed("type-parameters", i);
        shape(&frame, "name", parameter.get("name"), out);
        shape(&frame, "kind", parameter.get("kind"), out);
        typed(&frame, "type", parameter.get("type"), out);
        typed(&frame, "bounds", parameter.get("bounds"), out);
        typed(&frame, "default", parameter.get("default"), out);
        shape(&frame, "variance", parameter.get("variance"), out);
        set(&frame, "modifiers", parameter, SlotClass::Shape, out);
    }
}

fn parameters<'a>(record: &Value, frame: &Frame<'a>, out: &mut Vec<Slot<'a>>) {
    for (i, parameter) in items(record, "parameters").iter().enumerate() {
        let frame = frame.indexed("parameters", i);
        shape(&frame, "name", parameter.get("name"), out);
        typed(&frame, "type", parameter.get("type"), out);
        shape(&frame, "mode", parameter.get("mode"), out);
        valued(&frame, "default", parameter.get("default"), true, out);
    }
}

/// The written key of a `bases` or `implementations` entry.
pub fn base_key(entry: &Value) -> String {
    let ty = entry
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match entry.get("self-type").and_then(Value::as_str) {
        Some(self_type) => format!("{ty}, {self_type}"),
        None => ty.to_owned(),
    }
}

fn bases<'a>(
    record: &Value,
    key: &str,
    source: Source<'a>,
    frame: &Frame<'a>,
    out: &mut Vec<Slot<'a>>,
) {
    for (i, entry) in items(record, key).iter().enumerate() {
        let frame = Frame {
            source,
            ..frame.keyed(key, i, &base_key(entry))
        };
        typed(&frame, "type", entry.get("type"), out);
        shape(&frame, "position", entry.get("position"), out);
        type_parameters(entry, &frame, out);
        typed(&frame, "self-type", entry.get("self-type"), out);
        typed(&frame, "where", entry.get("where"), out);
        for (k, binding) in items(entry, "bindings").iter().enumerate() {
            let name = binding
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let frame = frame.keyed("bindings", k, name);
            type_parameters(binding, &frame, out);
            typed(&frame, "where", binding.get("where"), out);
            typed(&frame, "type", binding.get("type"), out);
            valued(&frame, "value", binding.get("value"), false, out);
            typed(&frame, "returns", binding.get("returns"), out);
            set(&frame, "auto-traits", binding, SlotClass::AutoTrait, out);
        }
    }
}

fn accessor<'a>(record: &Value, key: &str, frame: &Frame<'a>, out: &mut Vec<Slot<'a>>) {
    let Some(accessor) = record.get(key) else {
        return;
    };
    let frame = frame.child(key);
    typed(&frame, "type", accessor.get("type"), out);
    shape(&frame, "access", accessor.get("access"), out);
    set(&frame, "modifiers", accessor, SlotClass::Shape, out);
}

fn walk<'a>(record: &Value, frame: &Frame<'a>, out: &mut Vec<Slot<'a>>) {
    type_parameters(record, frame, out);
    typed(frame, "where", record.get("where"), out);
    if let Some(implementation) = record.get("impl") {
        let frame = Frame {
            source: Source::Impl,
            ..frame.child("impl")
        };
        type_parameters(implementation, &frame, out);
        typed(&frame, "self-type", implementation.get("self-type"), out);
        typed(&frame, "where", implementation.get("where"), out);
    }
    shape(frame, "abi", record.get("abi"), out);
    typed(frame, "receiver", record.get("receiver"), out);
    parameters(record, frame, out);
    typed(frame, "returns", record.get("returns"), out);
    shape(frame, "return-mode", record.get("return-mode"), out);
    set(frame, "modifiers", record, SlotClass::Shape, out);
    set(frame, "auto-traits", record, SlotClass::AutoTrait, out);
    for (n, overload) in items(record, "overloads").iter().enumerate() {
        let frame = Frame {
            source: Source::Overload(n),
            ..frame.indexed("overloads", n)
        };
        walk(overload, &frame, out);
    }
    bases(record, "bases", Source::Base, frame, out);
    bases(
        record,
        "implementations",
        Source::Implementation,
        frame,
        out,
    );
    typed(frame, "underlying", record.get("underlying"), out);
    for group in [
        "constructors",
        "call-signatures",
        "construct-signatures",
        "index-signatures",
    ] {
        for (n, entry) in items(record, group).iter().enumerate() {
            let frame = Frame {
                source: Source::Group(group_name(group), n),
                ..frame.indexed(group, n)
            };
            walk(entry, &frame, out);
            shape(&frame, "access", entry.get("access"), out);
        }
    }
    typed(frame, "type", record.get("type"), out);
    typed(frame, "bounds", record.get("bounds"), out);
    shape(frame, "tag", record.get("tag"), out);
    shape(frame, "index", record.get("index"), out);
    valued(frame, "value", record.get("value"), false, out);
    for key in ["get", "set", "init", "add", "remove"] {
        accessor(record, key, frame, out);
    }
    shape(frame, "delete", record.get("delete"), out);
}

fn group_name(group: &str) -> &'static str {
    match group {
        "constructors" => "constructors",
        "call-signatures" => "call-signatures",
        "construct-signatures" => "construct-signatures",
        _ => "index-signatures",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn slot_paths_follow_the_model() {
        let record = json!({
            "entity": "x",
            "type-parameters": [{"name": "T", "kind": "type", "bounds": "@{C}"}],
            "parameters": [{"name": "a", "type": {"inferred": "u8"}, "default": "1"}],
            "modifiers": ["static"],
            "auto-traits": ["#1 Send when T: Send"],
            "bases": [{"type": "@{T}<u8>", "self-type": "@{X}<T>", "where": "W",
                       "bindings": [{"name": "make", "returns": "u8"}]}],
            "constructors": [{"parameters": [{"name": "b", "type": "string"}]}],
        });
        let found: Vec<(String, SlotClass)> = slots(&record)
            .into_iter()
            .map(|s| (s.path, s.class))
            .collect();
        for (path, class) in [
            ("type-parameters[0].bounds", SlotClass::DeclaredType),
            ("parameters[0].type", SlotClass::InferredType),
            ("parameters[0].default", SlotClass::Value),
            ("modifiers[static]", SlotClass::Shape),
            ("auto-traits[#1 Send]", SlotClass::AutoTrait),
            ("bases[@{T}<u8>, @{X}<T>].where", SlotClass::DeclaredType),
            (
                "bases[@{T}<u8>, @{X}<T>].bindings[make].returns",
                SlotClass::DeclaredType,
            ),
            (
                "constructors[0].parameters[0].type",
                SlotClass::DeclaredType,
            ),
        ] {
            assert!(
                found.iter().any(|(p, c)| p == path && *c == class),
                "{path}: {found:#?}"
            );
        }
    }

    #[test]
    fn an_alternative_has_its_own_slots() {
        let record = json!({"entity": "x", "alternatives": [
            {"at": {}, "returns": "int"}, {"at": {}, "returns": "str"}]});
        let found = slots(&record);
        assert_eq!(found.len(), 2);
        assert_eq!(found[1].alternative, Some(1));
        assert_eq!(found[1].pointer, "/alternatives/1/returns");
    }
}
