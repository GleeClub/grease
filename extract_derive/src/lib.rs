#![recursion_limit = "128"]

extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;

use syn::DeriveInput;
use proc_macro::TokenStream;

#[proc_macro_derive(Extract)]
pub fn extract(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
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
    };

    gen.into()
}
