extern crate syn;
extern crate quote;
extern crate serde;

use serde::{Deserialize, from_str};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[proc_macro_derive(Extract)]
pub fn extract(input: TokenStream) -> TokenStream {
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect(&format!("Couldn't read the file: {}", &template_location));

    // Build the impl
    let gen = quote! {
        impl Extract for #name {
            fn extract(request: &cgi::Request) -> GreaseResult<Self> {
                #contents
            }
        }
    };

    // Return the generated impl
    gen.into()
}