use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, parse_macro_input};

/// `Message` trait derivation for an enum to act as a network protocol dispatcher.
///
/// # Functionality
/// Macro transforms a Rust `enum` into a binary format for sending and receiving messages.
///
/// - **Format:** `[1 byte Variant ID] + [Payload Data]`
/// - **Sending:** Matches the enum variant, writes its index as a `u8` ID, and delegates
///   serialization to the inner payload's `send_payload` method.
/// - **Receiving:** Reads a `u8` ID from the stream, matches it to the correct enum variant,
///   delegates deserialization to the inner payload's `recv_payload`, and wraps it in the variant.
///
/// # Constraints
/// - The enum must only contain variants with a single unnamed field (payload of the message).
/// - The variants are indexed based on their order in the definition (0, 1, 2, ...).
#[proc_macro_derive(Message)]
pub fn derive_message_payload_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Collecting variant names, indices and payload types
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Only enums with single unnamed field variants are supported"),
    };

    let mut variant_idxs = Vec::new();
    let mut variant_names = Vec::new();
    let mut payload_types = Vec::new();

    for (idx, variant) in variants.iter().enumerate() {
        if let Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() == 1 {
                variant_idxs.push(idx as u8);
                variant_names.push(&variant.ident);
                payload_types.push(&fields.unnamed[0].ty);
            } else {
                panic!("Enum variants must have exactly one field");
            }
        } else {
            panic!("Enum variants must be unnamed (tuple-like)");
        }
    }

    // Implement sending and receiving message
    let expanded = quote! {
        impl crate::common::messages::messages::Message for #name {
            async fn send(&self, send: &mut ::quinn::SendStream) -> ::anyhow::Result<()> {
                match self {
                    #(
                        #name::#variant_names(payload) => {
                            send.write_u8(#variant_idxs).await?;
                            crate::common::messages::payload::MessagePayload::send_payload(payload, send).await?
                        }
                    )*
                }
                ::anyhow::Ok(())
            }

            async fn recv(recv: &mut ::quinn::RecvStream) -> ::anyhow::Result<Self> {
                use ::tokio::io::AsyncReadExt;
                let variant_id = recv.read_u8().await?;
                match variant_id {
                    #(
                        #variant_idxs => {
                            let payload = <#payload_types as crate::common::messages::payload::MessagePayload>::recv_payload(recv).await?;
                            ::anyhow::Ok(#name::#variant_names(payload))
                        }
                    )*
                    _ => ::anyhow::bail!("Unknown variant ID: {}", variant_id),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
