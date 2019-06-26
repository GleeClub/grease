#![recursion_limit = "128"]

extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::iter::FromIterator;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Extract)]
pub fn extract(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let extract = quote!(crate::extract::Extract);
    let error = quote!(crate::error::GreaseError);
    let result = quote!(crate::error::GreaseResult);

    let gen = quote! {
        impl #extract for #name {
            fn extract(request: &cgi::Request) -> #result<Self> {
                serde_json::from_str(
                    std::str::from_utf8(request.body())
                        .map_err(|err| #error::BadRequest(
                            format!("request body was not a string: {}", err)
                        ))?
                ).map_err(|err| #error::BadRequest(
                    format!("couldn't deserialize body: {}", err)
                ))
            }
        }

        impl #extract for Vec<#name> {
            fn extract(request: &cgi::Request) -> #result<Self> {
                serde_json::from_str(
                    std::str::from_utf8(request.body())
                        .map_err(|err| #error::BadRequest(
                            format!("request body was not a string: {}", err)
                        ))?
                ).map_err(|err| #error::BadRequest(
                    format!("couldn't deserialize body: {}", err)
                ))
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let struct_ast: DeriveInput = syn::parse(input).unwrap();

    let from_row = quote!(mysql::prelude::FromRow);
    let row_type = quote!(mysql::Row);
    let error = quote!(mysql::FromRowError);

    let name = &struct_ast.ident;
    let fields = match &struct_ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let num_fields = fields.len();
    let field_mappings: TokenStream2 =
        TokenStream2::from_iter(fields.iter().enumerate().map(|(index, field)| {
            let field_name = &field.ident.clone().expect("all fields must be named");
            let field_type = &field.ty;

            quote! {
                #field_name: (if let Some(val) = row.take(#index) {
                    <#field_type>::from_value_opt(val).map_err(|_err| #error(row.clone()))
                } else {
                    Err(#error(row.clone()))
                })?,
            }
        }));

    let gen = quote! {
        impl #from_row for #name {
            #[inline]
            fn from_row(row: #row_type) -> Self {
                use mysql_enum::mysql::prelude::FromValue as _;

                match Self::from_row_opt(row) {
                    Ok(x) => x,
                    Err(#error(row)) => panic!(
                        "Couldn't convert {:?} to type #name. (see FromRow documentation)",
                        row
                    ),
                }
            }
            fn from_row_opt(mut row: #row_type) -> Result<Self, #error> {
                use mysql_enum::mysql::prelude::FromValue as _;

                if row.len() != #num_fields {
                    return Err(#error(row));
                }
                Ok(Self {
                    #field_mappings
                })
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(TableName, attributes(table_name))]
pub fn table_name(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let trait_name = quote!(crate::db::traits::TableName);
    let table = ast
        .attrs
        .into_iter()
        .filter_map(|option| {
            match option.parse_meta().expect("couldn't parse as meta") {
                // Match '#[ident = lit]' attributes. Match guard makes it '#[prefix = lit]'
                syn::Meta::NameValue(syn::MetaNameValue {
                    ref ident, ref lit, ..
                }) if ident == "table_name" => {
                    if let syn::Lit::Str(lit) = lit {
                        Some(lit.value())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .next()
        .expect("no `table_name` attribute provided");

    (quote! {
        impl #trait_name for #name {
            #[inline]
            fn table_name() -> &'static str {
                #table
            }
        }
    })
    .into()
}

#[proc_macro_derive(FieldNames, attributes(rename))]
pub fn field_names(input: TokenStream) -> TokenStream {
    let struct_ast: DeriveInput = syn::parse(input).unwrap();

    let trait_name = quote!(crate::db::traits::FieldNames);
    let name = &struct_ast.ident;
    let fields = match &struct_ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    if fields.len() == 0 {
        panic!("at least one field is required");
    }
    let field_names: TokenStream2 = TokenStream2::from_iter(fields.iter().map(|field| {
        let rename = field
            .attrs
            .iter()
            .filter_map(
                |attr| match attr.parse_meta().expect("couldn't parse as meta") {
                    syn::Meta::NameValue(syn::MetaNameValue {
                        ref ident, ref lit, ..
                    }) if ident == "rename" => {
                        if let syn::Lit::Str(lit) = lit {
                            Some(lit.value())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
            )
            .next();
        let field_name = rename
            .or(field.ident.clone().map(|name| name.to_string()))
            .map(|name| if let Some('`') = name.chars().next() {
                name
            } else {
                format!("`{}`", name)
            })
            .expect("all fields must be named");

        quote! {
            #field_name,
        }
    }));

    let gen = quote! {
        impl #trait_name for #name {
            #[inline]
            fn field_names() -> &'static[&'static str] {
                &[ #field_names ]
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(Insertable, attributes(rename))]
pub fn insertable(input: TokenStream) -> TokenStream {
    let struct_ast: DeriveInput = syn::parse(input).unwrap();

    let trait_name = quote!(crate::db::traits::Insertable);
    let name = &struct_ast.ident;
    let fields = match &struct_ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    if fields.len() == 0 {
        panic!("at least one field is required");
    }
    let field_sets = TokenStream2::from_iter(fields.iter().map(|field| {
        let rename = field
            .attrs
            .iter()
            .filter_map(
                |attr| match attr.parse_meta().expect("couldn't parse as meta") {
                    syn::Meta::NameValue(syn::MetaNameValue {
                        ref ident, ref lit, ..
                    }) if ident == "rename" => {
                        if let syn::Lit::Str(lit) = lit {
                            Some(lit.value())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
            )
            .next();
        let db_field_name = rename
            .or(field.ident.clone().map(|name| name.to_string()))
            .map(|name| if let Some('`') = name.chars().next() {
                name
            } else {
                format!("`{}`", name)
            })
            .expect("all fields must be named");
        let rs_field_name = field.ident.clone()
            .expect("all fields must be named");

        quote! {
            .set(#db_field_name, &self.#rs_field_name.to_value().as_sql(true))
        }
    }));

    let gen = quote! {
        impl #trait_name for #name {
            fn insert<C: crate::db::connection::Connection>(&self, conn: &mut C) -> crate::error::GreaseResult<()> {
                use crate::db::traits::TableName;

                conn.insert(
                    &pinto::query_builder::Insert::new(Self::table_name())
                        #field_sets
                )
            }

            fn insert_multiple<C: crate::db::connection::Connection>(to_insert: &[Self], conn: &mut C) -> crate::error::GreaseResult<()> {
                for item in to_insert {
                    item.insert(conn)?;
                }

                Ok(())
            }

            fn insert_returning_id<C: crate::db::connection::Connection>(&self, conn: &mut C) -> crate::error::GreaseResult<i32> {
                use crate::db::traits::TableName;

                conn.insert_returning_id(
                    &pinto::query_builder::Insert::new(Self::table_name())
                        #field_sets
                )
            }
        }
    };

    gen.into()
}
