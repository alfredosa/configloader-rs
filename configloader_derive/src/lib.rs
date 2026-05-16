extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// TODO env, default, load_fn
#[proc_macro_derive(ConfigLoader, attributes(skip, nested, default, env, load_fn))]
pub fn config_loader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields) => fields.named,
            other => {
                return syn::Error::new_spanned(other, "ConfigLoader only supports named fields")
                    .to_compile_error()
                    .into();
            }
        },
        other => {
            let _ = other;
            return syn::Error::new_spanned(&name, "ConfigLoader can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let mut missing_checks = Vec::new();
    let mut field_inits = Vec::new();

    for field in fields {
        let field_name = field.ident.expect("named field");
        let field_type = field.ty;
        // TODO: this can be wrong MyTest -> MYTEST
        // Maybe Snakifying it would be better but don't feel like implementing this yet.
        let env_name = field_name.to_string().to_uppercase();
        let has_default = has_attr(&field.attrs, "default");

        if has_attr(&field.attrs, "skip") {
            field_inits.push(quote! {
                #field_name: ::std::default::Default::default()
            });
            continue;
        }

        if has_attr(&field.attrs, "nested") {
            missing_checks.push(quote! {
                if let ::std::result::Result::Err(configloader::ConfigError::MissingVars(mut nested_missing_vars)) =
                    <#field_type as configloader::ConfigLoader>::load()
                {
                    missing_vars.append(&mut nested_missing_vars);
                }
            });

            field_inits.push(quote! {
                #field_name: <#field_type as configloader::ConfigLoader>::load()?
            });
            continue;
        }

        // Simple way of checking the existance of a given var. Omits defaulted tags.
        missing_checks.push(quote! {
            if !#has_default && ::std::env::var_os(#env_name).is_none() {
                missing_vars.push(#env_name);
            }
        });

        let true_val = match has_default {
            true => {
                let def = get_attr_string(&field.attrs, "default");
                quote! {#def}
            }
            false => {
                quote! {::std::env::var(#env_name)
                    .expect("checked required environment variable presence")
                }
            }
        };

        field_inits.push(quote! {
            #field_name: {
                let value = #true_val;
                value.parse::<#field_type>().map_err(|err| configloader::ConfigError::InvalidVar {
                    name: #env_name,
                    message: err.to_string(),
                })?
            }
        });
    }

    let expanded = quote! {
        impl configloader::ConfigLoader for #name {
            fn load() -> ::std::result::Result<Self, configloader::ConfigError> {
                let mut missing_vars = ::std::vec::Vec::new();

                #(#missing_checks)*

                if !missing_vars.is_empty() {
                    return ::std::result::Result::Err(configloader::ConfigError::MissingVars(missing_vars));
                }

                ::std::result::Result::Ok(Self {
                    #(#field_inits),*
                })
            }
        }
    };

    expanded.into()
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn get_attr_string(attrs: &[syn::Attribute], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(name))
        .and_then(|attr| attr.parse_args::<syn::LitStr>().ok())
        .map(|lit| lit.value())
}
