#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Variant};
use syn_util::get_attribute_value;

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(StrNum, attributes(value))]
pub fn derive_strnum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let expanded = derive(input.data, &input.ident, &input.attrs);

    proc_macro::TokenStream::from(expanded)
}

fn derive(data: Data, enum_name: &Ident, _attrs: &Vec<Attribute>) -> TokenStream {
    let span = enum_name.span();

    match data {
        Data::Enum(data) => {
            let options: Vec<StringOption> =
                data.variants.into_iter().map(StringOption::from).collect();

            let has_fallback = match options.iter().filter(|option| option.catch_all).count() {
                0 => false,
                1 => true,
                _ => panic!("Only a single catch-all variant is supported"),
            };

            let match_arms: Vec<_> = options
                .iter()
                .map(|option| {
                    let span = option.span;
                    let ident = &option.ident;
                    let string = &option.name;
                    if option.catch_all {
                        quote_spanned! { span =>
                            _ => #enum_name::#ident(value.into())
                        }
                    } else {
                        quote_spanned! { span =>
                            #string => #enum_name::#ident
                        }
                    }
                })
                .collect();

            // quote! takes ownership of anything passed to it, so instead of cloning the match arms we grab 2 Iter's
            let match_arms_1 = match_arms.iter();
            let match_arms_2 = match_arms.iter();

            let from = if has_fallback {
                quote_spanned! { span =>
                    impl ::std::convert::From<String> for #enum_name {
                        fn from(value: String) -> Self {
                            match value.as_str() {
                                #(#match_arms_1 ,)*
                            }
                        }
                    }

                    impl ::std::convert::From<&str> for #enum_name {
                        fn from(value: &str) -> Self {
                            match value {
                                #(#match_arms_2 ,)*
                            }
                        }
                    }
                }
            } else {
                quote_spanned! { span =>
                    impl ::std::convert::TryFrom<String> for #enum_name {
                        type Error = String;

                        fn try_from(value: String) -> Result<Self, Self::Error> {
                            Ok(match value.as_str() {
                                #(#match_arms_1 ,)*
                                _ => return Err(value)
                            })
                        }
                    }

                    impl ::std::convert::TryFrom<&str> for #enum_name {
                        type Error = String;

                        fn try_from(value: &str) -> Result<Self, Self::Error> {
                            Ok(match value {
                                #(#match_arms_2 ,)*
                                _ => return Err(value.to_string())
                            })
                        }
                    }
                }
            };

            let display_arms = options.iter().map(|option| {
                let span = option.span;
                let ident = &option.ident;
                let string = &option.name;
                if option.catch_all {
                    quote_spanned! { span =>
                        #enum_name::#ident(value) => write!(f, "{}", value)
                    }
                } else {
                    quote_spanned! { span =>
                        #enum_name::#ident => write!(f, #string)
                    }
                }
            });

            let to_string_arms = options.iter().map(|option| {
                let span = option.span;
                let ident = &option.ident;
                let string = &option.name;
                if option.catch_all {
                    quote_spanned! { span =>
                        #enum_name::#ident(value) => value
                    }
                } else {
                    quote_spanned! { span =>
                        #enum_name::#ident => #string.to_string()
                    }
                }
            });

            let display = quote_spanned! { span =>
                impl ::std::fmt::Display for #enum_name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match self {
                            #(#display_arms ,)*
                        }
                    }
                }

                impl ::std::convert::From<#enum_name> for String {
                    fn from(from: #enum_name) -> String {
                        match from {
                            #(#to_string_arms ,)*
                        }
                    }
                }
            };

            quote_spanned! { span =>
                #from

                #display
            }
        }
        _ => panic!("Can only derive StrNum for enums"),
    }
}

struct StringOption {
    ident: Ident,
    name: String,
    catch_all: bool,
    span: Span,
}

impl From<Variant> for StringOption {
    fn from(variant: Variant) -> Self {
        let span = variant.span();
        let name: String = get_attribute_value(&variant.attrs, &["value"])
            .unwrap_or_else(|| variant.ident.to_string());
        let catch_all = match variant.fields {
            Fields::Unit => false,
            Fields::Named(_) => panic!("Only single unnamed enum field is supported"),
            Fields::Unnamed(ref fields) if fields.unnamed.len() > 1 => {
                panic!("Only a single unnamed enum field is supported")
            }
            Fields::Unnamed(_) => true,
        };

        StringOption {
            ident: variant.ident,
            name,
            catch_all,
            span,
        }
    }
}
