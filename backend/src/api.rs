//! Типы, сгенерированные из OpenAPI-спецификации (см. build.rs).
//! Руками не правится: источник — контракт в `contracts/`.

pub mod api_types {
    include!(concat!(env!("OUT_DIR"), "/openapi_types.rs"));
}
