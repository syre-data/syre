//! Derive the `settings_manager#Settings` trait.
use crate::common::IDENT;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use std::convert::TryFrom;
use syn::parse::Error as ParseError;
use syn::{Data, Expr, Fields, Ident, Lit, Path, Type};

const PRIORITY_IDENT: &str = "priority";
const FILE_LOCK_IDENT: &str = "file_lock";

// *********************
// *** Settings Info ***
// *********************

#[derive(Default)]
struct SettingsDataInfo {
    pub data_field: Option<Ident>,
    pub priority: Option<Ident>,
    pub file_lock: Option<Ident>,
}

impl<'a> SettingsDataInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

struct SettingsInfoBuilder(HashMap<Path, SettingsDataInfo>);
impl SettingsInfoBuilder {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn set_data_field(&mut self, key: Path, value: Ident) -> Result<(), ParseError> {
        let info = self.0.entry(key).or_insert(SettingsDataInfo::new());
        if info.data_field.is_some() {
            return Err(ParseError::new(
                Span::call_site(),
                "data field already defined",
            ));
        }

        let _ = info.data_field.insert(value);
        Ok(())
    }

    pub fn set_priority(&mut self, key: Path, value: Ident) -> Result<(), ParseError> {
        let info = self.0.entry(key).or_insert(SettingsDataInfo::new());
        if info.priority.is_some() {
            return Err(ParseError::new(
                Span::call_site(),
                "priority already defined",
            ));
        }

        let _ = info.priority.insert(value);
        Ok(())
    }

    pub fn set_file_lock(&mut self, key: Path, value: Ident) -> Result<(), ParseError> {
        let info = self.0.entry(key).or_insert(SettingsDataInfo::new());
        if info.file_lock.is_some() {
            return Err(ParseError::new(
                Span::call_site(),
                "file lock already defined",
            ));
        }

        let _ = info.file_lock.insert(value);
        Ok(())
    }
}

struct SettingsInfo {
    ty: Path,
    data_field: Ident,
    priority: Ident,
    file_lock: Ident,
}

impl TryFrom<SettingsInfoBuilder> for Vec<SettingsInfo> {
    type Error = ParseError;

    fn try_from(builder: SettingsInfoBuilder) -> Result<Self, Self::Error> {
        let mut info = Self::with_capacity(builder.0.len());
        for (key, data) in builder.0.into_iter() {
            let Some(data_field) = data.data_field else {
                return Err(ParseError::new(
                    Span::call_site(),
                    format!("`data_field` not set for `{key:?}`"),
                ));
            };

            let Some(priority) = data.priority else {
                return Err(ParseError::new(
                    Span::call_site(),
                    format!("`priority` not set for `{key:?}`"),
                ));
            };

            let Some(file_lock) = data.file_lock else {
                return Err(ParseError::new(
                    Span::call_site(),
                    format!("`file_lock` not set for `{key:?}`"),
                ));
            };

            info.push(SettingsInfo {
                ty: key,
                data_field,
                priority,
                file_lock,
            });
        }

        Ok(info)
    }
}

// **************
// *** Derive ***
// **************

pub(crate) fn impl_settings(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    // get marked fields
    let Data::Struct(obj) = &ast.data else {
        panic!("`Settings` can only be derived for structs.");
    };

    let fields = match &obj.fields {
        Fields::Named(fields) => &fields.named,
        Fields::Unnamed(fields) => &fields.unnamed,
        Fields::Unit => {
            panic!("cannot derive `Settings` for unit structs");
        }
    };

    let mut settings_info = SettingsInfoBuilder::new();

    for field in fields.into_iter() {
        let attrs = &field.attrs;
        for attr in attrs.iter() {
            if !attr.path().is_ident(IDENT) {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let field_ident = field
                    .ident
                    .clone()
                    .expect("could not get `file_lock` field ident");

                if meta.path.is_ident(FILE_LOCK_IDENT) {
                    let data_type: Expr = meta
                        .value()
                        .expect("could not get data type for `file_lock`")
                        .parse()
                        .expect("could not parse data type for `file_lock`");

                    let Expr::Lit(data_type) = data_type else {
                        return Err(meta.error("could not parse `file_lock`"));
                    };

                    let Lit::Str(data_type) = data_type.lit else {
                        return Err(meta.error("could not parse `file_lock`"));
                    };

                    let data_type: Path = syn::parse_str(&data_type.value())
                        .expect("could not parse `file_lock` value as `Path`");

                    settings_info.set_file_lock(data_type, field_ident)?;
                } else if meta.path.is_ident(PRIORITY_IDENT) {
                    let Type::Path(data_type) = &field.ty else {
                        panic!("invalid data type");
                    };

                    let data_type = data_type.path.clone();

                    let data_field = field_ident;
                    let priority: Expr = meta
                        .value()
                        .expect("could not get `priority` value")
                        .parse()
                        .expect("could not parse `priority`");

                    let Expr::Lit(priority) = priority else {
                                return Err(meta.error("could not parse `priority`"));
                            };

                    let Lit::Str(priority) = priority.lit else {
                                return Err(meta.error("could not parse `priority`"));
                            };

                    let priority = Ident::new(&priority.value(), Span::call_site());

                    settings_info.set_data_field(data_type.clone(), data_field)?;
                    settings_info.set_priority(data_type, priority)?;
                } else {
                    return Err(meta.error(&format!(
                        "invalid field attribute `{:}`",
                        meta.path
                            .get_ident()
                            .expect("could not get field attribute")
                    )));
                }

                Ok(())
            })
            .expect("could not parse attirbute data");
        }
    }

    let settings_info: Vec<SettingsInfo> = settings_info.try_into().expect("invalid setting");
    let mut impls = Vec::with_capacity(settings_info.len());
    for SettingsInfo {
        ty: data_type,
        data_field: data_field_ident,
        file_lock: file_lock_field,
        priority,
    } in settings_info.iter()
    {
        let gen = quote! {
            impl #impl_generics Settings<#data_type> for #name #ty_generics #where_clause {
                fn settings(&self) -> std::borrow::Cow<#data_type> {
                    std::borrow::Cow::Borrowed(&self.#data_field_ident)
                }

                fn file(&self) -> &File {
                    &*self.#file_lock_field
                }

                fn file_mut(&mut self) -> &mut File {
                    &mut *self.#file_lock_field
                }

                fn file_lock(&self) -> &FlockLock<File> {
                    &self.#file_lock_field
                }

                fn priority(&self) -> settings_manager::Priority {
                    settings_manager::Priority::#priority
                }
            }
        };

        impls.push(gen);
    }

    let gen = quote! {
        #(#impls) *
    };

    gen.into()
}
