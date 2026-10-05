//! Генерация серверных типов контракта из OpenAPI-спецификации.
//!
//! Источник правды — `openapi/openapi.json` (эмитится TypeSpec рядом с YAML).
//! Сгенерированный файл руками не правится: правится контракт в `contracts/`.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../openapi/openapi.json");
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let raw = fs::read_to_string(&spec_path)?;
    let doc: serde_json::Value = serde_json::from_str(&raw)?;
    let schemas = doc["components"]["schemas"]
        .as_object()
        .ok_or("no components.schemas in spec")?;

    let defs = schemas
        .iter()
        .map(|(name, value)| {
            let schema = serde_json::from_value::<schemars::schema::Schema>(value.clone())?;
            Ok((name.clone(), schema))
        })
        .collect::<Result<Vec<_>, serde_json::Error>>()?;

    let mut settings = typify::TypeSpaceSettings::default();
    settings.with_struct_builder(true);
    let mut type_space = typify::TypeSpace::new(&settings);
    type_space.add_ref_types(defs)?;

    let file = syn::parse2::<syn::File>(type_space.to_stream())?;
    let formatted = format!(
        "// @generated from openapi/openapi.json — do not edit\n{}\n",
        prettyplease::unparse(&file)
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out_dir.join("openapi_types.rs"), formatted)?;
    Ok(())
}
