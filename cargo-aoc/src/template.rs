use tera::Tera;

macro_rules! get_builtin(($path: literal) => {
    ($path, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/template/", $path)))
});

const STATIC_TEMPLATES: &[(&str, &str)] = &[
    get_builtin!("Cargo-bench.toml.tpl"),
    get_builtin!("Cargo-run.toml.tpl"),
    get_builtin!("input.rs.tpl"),
    get_builtin!("benches/aoc_benchmark.rs.tpl"),
    get_builtin!("benches/gen_impl.rs.tpl"),
    get_builtin!("benches/gen.rs.tpl"),
    get_builtin!("benches/impl.rs.tpl"),
    get_builtin!("benches/part.rs.tpl"),
    get_builtin!("src/main.rs.tpl"),
    get_builtin!("src/runner.rs.tpl"),
];

pub fn get_tera() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_templates(STATIC_TEMPLATES.iter().map(|t| t.clone()))
        .expect("Invalid built in templates?");
    tera
}
