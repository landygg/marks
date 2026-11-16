use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use fluent_syntax::parser::parse;
use std::collections::{BTreeMap, BTreeSet};

type Shape = BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)>;

fn read(lang: &str) -> String {
    let p = format!("{}/i18n/{lang}/marks_i18n.ftl", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(p).expect("catalog")
}

fn vars(p: &Pattern<String>, out: &mut BTreeSet<String>) {
    for el in &p.elements {
        if let PatternElement::Placeable { expression } = el {
            expr(expression, out);
        }
    }
}

fn expr(e: &Expression<String>, out: &mut BTreeSet<String>) {
    match e {
        Expression::Inline(i) => inline(i, out),
        Expression::Select { selector, variants } => {
            inline(selector, out);
            for v in variants {
                vars(&v.value, out);
            }
        }
    }
}

fn inline(i: &InlineExpression<String>, out: &mut BTreeSet<String>) {
    match i {
        InlineExpression::VariableReference { id } => {
            out.insert(id.name.clone());
        }
        InlineExpression::Placeable { expression } => expr(expression, out),
        InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline(a, out);
            }
        }
        _ => {}
    }
}

fn shape(src: &str) -> Shape {
    let res = parse(src.to_string()).expect("valid ftl");
    let mut m = Shape::new();
    for e in &res.body {
        if let Entry::Message(msg) = e {
            let mut v = BTreeSet::new();
            let mut attrs = BTreeSet::new();
            if let Some(p) = &msg.value {
                vars(p, &mut v);
            }
            for a in &msg.attributes {
                attrs.insert(a.id.name.clone());
                vars(&a.value, &mut v);
            }
            m.insert(msg.id.name.clone(), (attrs, v));
        }
    }
    m
}

#[test]
fn same_ids_attributes_and_variables() {
    assert_eq!(shape(&read("en")), shape(&read("es")));
}

#[test]
fn es_plural_many() {
    // Every `$n` selector must have a default `*[other]` variant.
    for lang in ["en", "es"] {
        let res = parse(read(lang)).expect("valid ftl");
        for e in &res.body {
            let Entry::Message(m) = e else { continue };
            let pats = m.value.iter().chain(m.attributes.iter().map(|a| &a.value));
            for p in pats {
                for el in &p.elements {
                    if let PatternElement::Placeable {
                        expression: Expression::Select { selector, variants },
                    } = el
                        && matches!(selector, InlineExpression::VariableReference { id } if id.name == "n")
                    {
                        let d = variants.iter().find(|v| v.default);
                        assert!(
                            d.is_some_and(|v| matches!(&v.key, fluent_syntax::ast::VariantKey::Identifier { name } if name == "other")),
                            "{lang}: {} sin *[other]",
                            m.id.name
                        );
                    }
                }
            }
        }
    }
}

/// Ids named by some `fl!` in `src`, excluding `#[cfg(test)]` modules (they come last).
fn used_ids() -> BTreeSet<String> {
    let dir = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let mut ids = BTreeSet::new();
    for entry in std::fs::read_dir(dir).expect("src") {
        let src = std::fs::read_to_string(entry.expect("entry").path()).expect("rs");
        let code = src.split("#[cfg(test)]").next().unwrap_or_default();
        for call in code.split("fl!(").skip(1) {
            if let Some(id) = call.split('"').nth(1) {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

#[test]
fn every_id_is_used() {
    let used = used_ids();
    let unused: Vec<_> = shape(&read("en"))
        .into_keys()
        .filter(|id| !used.contains(id))
        .collect();
    assert!(unused.is_empty(), "ids sin fl! en src: {unused:?}");
}
