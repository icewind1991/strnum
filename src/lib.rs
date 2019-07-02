extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident};
use syn_util::get_attribute_value;

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(StrNum, attributes(name))]
pub fn derive_strnum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let expanded = derive(input.data, &input.ident, &input.attrs);

    proc_macro::TokenStream::from(expanded)
}

fn derive(data: Data, enum_name: &Ident, _attrs: &Vec<Attribute>) -> TokenStream {
    let span = enum_name.span();

    match data {
        Data::Enum(data) => {
            let options: Vec<_> = data
                .variants
                .iter()
                .map(|variant| {
                    let name: String = get_attribute_value(&variant.attrs, &["name"])
                        .unwrap_or(variant.ident.to_string());
                    let catch_all = match &variant.fields {
                        Fields::Unit => false,
                        Fields::Named(_) => panic!("Only single unnamed enum field is supported"),
                        Fields::Unnamed(fields) if fields.unnamed.len() > 1 => {
                            panic!("Only a single unnamed enum field is supported")
                        }
                        Fields::Unnamed(_) => true,
                    };

                    StringOption {
                        ident: variant.ident.clone(),
                        name,
                        catch_all,
                    }
                })
                .collect();

            let has_fallback = match options.iter().filter(|option| option.catch_all).count() {
                0 => false,
                1 => true,
                _ => panic!("Only a single catch-all variant is supported"),
            };

            let mut match_arms: Vec<_> = options
                .iter()
                .map(|option| {
                    let ident = &option.ident;
                    let string = &option.name;
                    if has_fallback {
                        if option.catch_all {
                            quote_spanned! { span =>
                                _ => #enum_name::#ident(value)
                            }
                        } else {
                            quote_spanned! { span =>
                                #string => #enum_name::#ident
                            }
                        }
                    } else {
                        quote_spanned! { span =>
                            #string => Ok(#enum_name::#ident)
                        }
                    }
                })
                .collect();

            if !has_fallback {
                match_arms.push(quote_spanned! { span =>
                    _ => Err(value)
                });
            }

            let from = if has_fallback {
                quote_spanned! { span =>
                    impl ::std::convert::From<String> for #enum_name {
                        fn from(value: String) -> Self {
                            match value.as_str() {
                                #(#match_arms ,)*
                            }
                        }
                    }

                    impl ::std::convert::From<&str> for #enum_name {
                        fn from(value: &str) -> Self {
                            Self::from(value.to_string())
                        }
                    }
                }
            } else {
                quote_spanned! { span =>
                    impl ::std::convert::TryFrom<String> for #enum_name {
                        type Error = String;

                        fn try_from(value: String) -> Result<Self, Self::Error> {
                            match value.as_str() {
                                #(#match_arms ,)*
                            }
                        }
                    }

                    impl ::std::convert::TryFrom<&str> for #enum_name {
                        type Error = String;

                        fn try_from(value: &str) -> Result<Self, Self::Error> {
                            Self::try_from(value.to_string())
                        }
                    }
                }
            };

            let display_arms = options.iter().map(|option| {
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
}
