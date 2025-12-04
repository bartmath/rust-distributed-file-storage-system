use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(ChunkPayload)]
pub fn derive_chunk_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl MessagePayload for #name {
            async fn send_payload(&self, send: &mut SendStream) -> Result<()> {
                let metadata_bytes = bincode::serialize(&self)?;
                send.write_u32(metadata_bytes.len() as u32).await?;
                send.write_all(&metadata_bytes).await?;

                send.write_u64(self.data.len() as u64).await?;
                send.write_all(&self.data).await?;

                Ok(())
            }

            async fn recv_payload(recv: &mut RecvStream) -> Result<Self> {
                let len = recv.read_u32().await?;
                let mut buffer = vec![0u8; len as usize];
                recv.read_exact(&mut buffer).await?;
                let mut payload: Self = bincode::deserialize(&buffer)?;

                let data_len = recv.read_u64().await?;
                let mut data = vec![0u8; data_len as usize];
                recv.read_exact(&mut data).await?;
                payload.data = data;

                Ok(payload)
            }
        }
    };
    TokenStream::from(expanded)
}

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
                            #variant_idx => Ok(#name::#variant_name(#payload_ty::recv_payload(recv).await?)),
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
            async fn send(&self, send: &mut SendStream) -> Result<()> {
                match self {
                    #(#send_arms)*
                }
                Ok(())
            }

            async fn recv(recv: &mut RecvStream) -> Result<Self> {
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
