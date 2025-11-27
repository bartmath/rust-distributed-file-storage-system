use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse::ParseStream, Ident, Token};
use std::vec::Vec;

#[proc_macro]
pub fn register_types(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TypeList);

    let mut id_impls = vec![];
    let mut serialize_fns = vec![];
    let mut deserialize_arms = vec![];

    for (id, ty) in input.types.iter().enumerate() {
        let fn_name = format_ident!("serialize_{}", ty.to_string().to_lowercase());
        let id_u8 = id as u8;

        // 1. TypeId impl (enforces Serialize+Deserialize bounds)
        id_impls.push(quote! {
            impl TypeId for #ty {
                const ID: u8 = #id_u8;
            }
        });

        // 2. Serialize helper
        serialize_fns.push(quote! {
            fn #fn_name(msg: &#ty, buf: &mut Vec<u8>) -> anyhow::Result<()> {
                buf.push(#id_u8);

                let options = bincode::DefaultOptions::new().allow_trailing_bytes().with_fixint_encoding();
                options.serialize_into(&mut *buf, msg)?;
                Ok(())
            }
        });

        // 3. Deserialize match arm
        deserialize_arms.push(quote! {
            #id_u8 => {
                Ok(bincode::deserialize::<#ty>(&payload)?)
            }
        });
    }

    let expanded = quote! {
        // TypeId impls
        #(#id_impls)*

        // Serialization helpers (one per type)
        #(#serialize_fns)*
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
