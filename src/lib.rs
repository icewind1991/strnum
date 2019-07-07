#![recursion_limit = "128"]
//! Store common string values in enum variants
//!
//! # StrNum
//!
//! StrNum can be derived for enum that contain a number of unit fields for every string option and
//! optionally one field containing a `String` as fallback option.
//!
//! If a fallback option is provided `From<String>` and `From<&'str>` is implemented for the enum,
//! if no fallback option is provided `TryFrom` is implemented instead.
//!
//! Additionally, `Display` and `Into<String>` is implemented for the enum.
//!
//! ## Examples
//!
//! ```
//! use strnum::StrNum;
//!
//! #[derive(StrNum, PartialEq, Debug)]
//! enum Cities {
//!     Amsterdam,
//!     #[value = "New York"] // you can overwrite the string value by attribute
//!     NewYork,
//!     Tokyo,
//!     Other(String)
//! }
//!
//! fn main() {
//!     let first = Cities::from("Amsterdam");
//!     assert_eq!(Cities::Amsterdam, first);
//!     assert_eq!("Amsterdam", String::from(first));
//!
//!     let second = Cities::from("Dublin");
//!     assert_eq!(Cities::Other("Dublin".to_string()), second);
//!     assert_eq!("Dublin", String::from(second));
//! }
//!```
//!
//! ```
//! use strnum::StrNum;
//! use std::convert::TryFrom;
//!
//! #[derive(StrNum, PartialEq, Debug)]
//! enum SupportedCities {
//!     Amsterdam,
//!     #[value = "New York"] // you can overwrite the string value by attribute
//!     NewYork,
//!     Tokyo
//! }
//!
//! fn main() {
//!     let first = SupportedCities::try_from("Amsterdam");
//!     assert_eq!(Ok(SupportedCities::Amsterdam), first);
//!     assert_eq!("Amsterdam", String::from(first.unwrap()));
//!
//!     let second = SupportedCities::try_from("Dublin");
//!     assert_eq!(true, second.is_err());
//! }
//!```

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
