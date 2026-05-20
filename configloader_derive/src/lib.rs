extern crate proc_macro;

// it seems that TokenStream is the compiler-facing type
use proc_macro::TokenStream;
// ecosystem-facing type used in quote and syn, interesting.
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// Clean constants :)
const ATTR_SKIP: &str = "skip";
const ATTR_NESTED: &str = "nested";
const ATTR_DEFAULT: &str = "default";
const ATTR_ENV: &str = "env";
const ATTR_PREFIX: &str = "prefix";

#[proc_macro_derive(ConfigLoader, attributes(skip, nested, default, env, load_fn, prefix))]
pub fn config_loader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let top_level_prefix = match get_attr_string(&input.attrs, ATTR_PREFIX) {
        Ok(Some(prefix)) => prefix,
        Ok(None) => to_screaming_snake_case(&name.to_string()),
        Err(err) => return err.to_compile_error().into(),
    };

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
        
        let env_name = match has_attr(&field.attrs, ATTR_ENV) {
            true => match get_attr_string(&field.attrs, ATTR_ENV) {
                Ok(Some(env_name)) => env_name,
                Ok(None) => unreachable!("checked env attr presence"),
                Err(err) => return err.to_compile_error().into(),
            },
            false => to_screaming_snake_case(&field_name.to_string()),
        };

        let env_name_expr = env_name_expr(&env_name);
        let has_default = has_attr(&field.attrs, ATTR_DEFAULT);

        if has_attr(&field.attrs, ATTR_SKIP) {
            field_inits.push(quote! {
                #field_name: ::std::default::Default::default()
            });
            continue;
        }

        if has_attr(&field.attrs, ATTR_NESTED) {
            missing_checks.push(quote! {
                let nested_prefix = #env_name_expr;

                if let ::std::result::Result::Err(configloader::ConfigError::MissingVars(mut nested_missing_vars)) =
                    <#field_type as configloader::ConfigLoader>::load_with_prefix(&nested_prefix)
                {
                    missing_vars.append(&mut nested_missing_vars);
                }
            });

            field_inits.push(quote! {
                #field_name: {
                    let nested_prefix = #env_name_expr;

                    <#field_type as configloader::ConfigLoader>::load_with_prefix(&nested_prefix)?
                }
            });
            continue;
        }

        // Simple way of checking the existance of a given var. Omits defaulted tags.
        missing_checks.push(quote! {
            let env_name = #env_name_expr;

            if !#has_default && ::std::env::var_os(&env_name).is_none() {
                missing_vars.push(env_name);
            }
        });

        let true_val = match get_attr_string(&field.attrs, ATTR_DEFAULT) {
            Ok(Some(default)) => {
                quote! {
                    ::std::env::var(&env_name).unwrap_or_else(|_| #default.to_string())
                }
            }
            Ok(None) => {
                quote! {::std::env::var(&env_name)
                    .expect("checked required environment variable presence")
                }
            }
            Err(err) => return err.to_compile_error().into(),
        };

        field_inits.push(quote! {
            #field_name: {
                let env_name = #env_name_expr;
                let value = #true_val;
                value.parse::<#field_type>().map_err(|err| configloader::ConfigError::InvalidVar {
                    name: env_name,
                    message: err.to_string(),
                })?
            }
        });
    }

    let expanded = quote! {
        impl configloader::ConfigLoader for #name {
            fn load() -> ::std::result::Result<Self, configloader::ConfigError> {
                <Self as configloader::ConfigLoader>::load_with_prefix(#top_level_prefix)
            }


            fn load_with_prefix(prefix: &str) -> ::std::result::Result<Self, configloader::ConfigError> {
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

fn env_name_expr(env_name: &str) -> TokenStream2 {
    quote! {
        if prefix.is_empty() {
            #env_name.to_string()
        } else {
            ::std::format!("{}_{}", prefix, #env_name)
        }
    }
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn get_attr_string(attrs: &[syn::Attribute], name: &str) -> syn::Result<Option<String>> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(name))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .map(|lit| lit.value())
                .map_err(|_| syn::Error::new_spanned(attr, format!("expected #[{name}(\"...\")]")))
        })
        .transpose()
}

fn to_screaming_snake_case(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut prev_is_uppercase = false;

    while let Some(c) = chars.next() {
        if !res.is_empty() && c.is_uppercase() {
            let next_is_lowercase = chars.peek().map(|n| n.is_lowercase()).unwrap_or(false);
            if !prev_is_uppercase || next_is_lowercase {
                res.push('_');
            }
        }
        res.push(c.to_ascii_uppercase());
        prev_is_uppercase = c.is_uppercase();
    }
    res
}
