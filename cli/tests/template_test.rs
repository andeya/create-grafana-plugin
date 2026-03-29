//! Template rendering: substitution, conditionals, derived fields (9.5).

use create_grafana_plugin::config::{PluginType, ProjectConfig};
use create_grafana_plugin::template::{TemplateContext, render_string};

fn sample_config() -> ProjectConfig {
    ProjectConfig {
        name: "my-plugin".to_string(),
        description: "Test plugin".to_string(),
        author: "Jane Doe".to_string(),
        org: "acme".to_string(),
        plugin_type: PluginType::Panel,
        has_wasm: true,
        has_docker: false,
        has_mock: false,
    }
}

#[test]
fn render_string_substitutes_variables() {
    let cfg = sample_config();
    let ctx = TemplateContext::from_config(&cfg);
    let out = render_string(
        "id={{ plugin_id }} name={{ plugin_name }} org={{ org }}",
        &ctx,
    )
    .expect("render");
    assert_eq!(out, "id=acme-my-plugin name=my-plugin org=acme");
}

#[test]
fn render_string_conditional_includes_when_true() {
    let cfg = sample_config();
    let ctx = TemplateContext::from_config(&cfg);
    let tpl = "{% if has_wasm %}WASM{% else %}NO{% endif %}";
    assert_eq!(render_string(tpl, &ctx).expect("render"), "WASM");
}

#[test]
fn render_string_conditional_excludes_when_false() {
    let mut cfg = sample_config();
    cfg.has_wasm = false;
    let ctx = TemplateContext::from_config(&cfg);
    let tpl = "{% if has_wasm %}WASM{% else %}NO{% endif %}";
    assert_eq!(render_string(tpl, &ctx).expect("render"), "NO");
}

#[test]
fn template_context_derived_fields() {
    let cfg = ProjectConfig {
        name: "foo-bar-baz".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        org: "orgx".to_string(),
        plugin_type: PluginType::Datasource,
        has_wasm: true,
        has_docker: true,
        has_mock: true,
    };
    let ctx = TemplateContext::from_config(&cfg);
    assert_eq!(ctx.plugin_id, "orgx-foo-bar-baz");
    assert_eq!(ctx.crate_name, "foo_bar_baz");
    assert_eq!(ctx.pascal_case_name, "FooBarBaz");
    assert_eq!(ctx.plugin_type, "datasource");
    assert!(!ctx.current_year.is_empty());
    assert!(!ctx.today.is_empty());
}
