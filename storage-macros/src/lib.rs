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

    let (send_arms, recv_arms) = match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            let mut send_arms = Vec::new();
            let mut recv_arms = Vec::new();

            for (idx, variant) in variants.iter().enumerate() {
                let variant_idx = idx as u8;
                let variant_name = &variant.ident;

                if let Fields::Unnamed(fields) = &variant.fields {
                    if fields.unnamed.len() == 1 {
                        let payload_ty = &fields.unnamed[0].ty;
                        send_arms.push(quote! {
                            #name::#variant_name(payload) => {
                                send.write_u8(#variant_idx).await?;
                                payload.send_payload(send).await?
                            }
                        });

                        recv_arms.push(quote! {
                            #variant_idx => anyhow::Ok(#name::#variant_name(#payload_ty::recv_payload(recv).await?)),
                        });
                    }
                }
            }

            (send_arms, recv_arms)
        }
        _ => panic!("Only enums with single unnamed field variants"),
    };

    let expanded = quote! {
        impl Message for #name {
            async fn send(&self, send: &mut SendStream) -> anyhow::Result<()> {
                match self {
                    #(#send_arms)*
                }
                anyhow::Ok(())
            }

            async fn recv(recv: &mut RecvStream) -> anyhow::Result<Self> {
                let variant_id = recv.read_u8().await?;
                match variant_id {
                    #(#recv_arms)*
                    _ => anyhow::bail!("Unknown variant ID: {}", variant_id),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
