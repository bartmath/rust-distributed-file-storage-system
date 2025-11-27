use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse::ParseStream, Ident, Token};
use std::vec::Vec;

#[proc_macro]
pub fn register_types(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TypeList);

    let mut id_impls = vec![];

    for (id, ty) in input.types.iter().enumerate() {
        id_impls.push(quote! {
            impl TypeId for #ty {
                const ID: u8 = #id as u8;
            }
        });
    }

    let expanded = quote! {
        #(#id_impls)*
    };

    TokenStream::from(expanded)
}

struct TypeList {
    types: Vec<Ident>,
}

impl syn::parse::Parse for TypeList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut types = Vec::new();
        types.push(input.parse()?);
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            types.push(input.parse()?);
        }
        Ok(TypeList { types })
    }
}
