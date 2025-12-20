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

                let mut file = tokio::fs::File::open(&self.data).await?;
                if self.offset > 0 {
                    tokio::io::AsyncSeekExt::seek(&mut file, std::io::SeekFrom::Start(self.offset)).await?;
                }

                let mut buf = vec![0u8; 64 * 1024]; // 64kB
                let mut sent = 0u64;

                while sent < self.chunk_size {
                    let remaining = self.chunk_size - sent;
                    let to_read = std::cmp::min(buf.len() as u64, remaining) as usize;

                    let n = file.read(&mut buf[0..to_read]).await?;
                    if n == 0 {
                        anyhow::bail!("Chunk read to few bytes");
                    }

                    send.write_all(&buf[0..n]).await?;

                    sent += n as u64;
                }

                Ok(())
            }

            async fn recv_payload(recv: &mut RecvStream) -> Result<Self> {
                let len = recv.read_u32().await?;

                let mut buffer = vec![0u8; len as usize];
                recv.read_exact(&mut buffer).await?;
                let mut payload: Self = bincode::deserialize(&buffer)?;

                let data_len = payload.chunk_size;

                payload.data = TMP_STORAGE_ROOT
                    .get()
                    .expect("Temporary storage not initialized via config")
                    .join(payload.chunk_id.to_string());

                let file = tokio::fs::File::create(&payload.data).await?;
                file.set_len(data_len).await?;
                let mut writer = tokio::io::BufWriter::with_capacity(64 * 1024, file); // 64 kB
                let mut limited_recv = recv.take(data_len);
                tokio::io::copy(&mut limited_recv, &mut writer).await?;
                writer.flush().await?;

                writer.into_inner().sync_all().await?;

                Ok(payload)
            }
        }

        impl Drop for #name {
            fn drop(&mut self) {
                if self.data.exists() {
                    let _ = std::fs::remove_file(&self.data);
                }
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
            async fn send(&self, send: &mut SendStream) -> anyhow::Result<()> {
                match self {
                    #(#send_arms)*
                }
                Ok(())
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
