use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemEnum, Lit, Meta, MetaNameValue, Variant};

fn parse_id(variant: &Variant) -> Lit {
    variant
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("id"))
        .map(|attr| match attr.parse_meta().unwrap() {
            Meta::NameValue(MetaNameValue { lit, .. }) => lit,
            _ => panic!("id must be a name-value attribute"),
        })
        .unwrap()
}

fn make_serialize_impl(input: &ItemEnum) -> TokenStream {
    let ident = &input.ident;

    let serialize_fields = input.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let id = parse_id(&variant);

        let field_names = variant.fields.iter().map(|field| {
            let ident = &field.ident;
            quote! { ref #ident }
        });

        let serialize_fields = variant.fields.iter().map(|field| {
            let ident = &field.ident;

            quote! {
                #ident.serialize(w)?;
            }
        });

        quote! {
            #ident::#v_ident { #(#field_names),* } => {
                (#id as u16).serialize(w)?;
                #(#serialize_fields)*
            }
        }
    });

    let deserialize_fields = input.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let id = parse_id(&variant);

        let deserialize_fields = variant.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;

            quote! {
                #ident: #ty::deserialize(r)?,
            }
        });

        quote! {
             #id => {
                Ok(#ident::#v_ident {
                    #(#deserialize_fields)*
                })
            }
        }
    });

    quote! {
        impl crate::serialize::Serialize for #ident {
            fn serialize<W: std::io::Write>(&self, w: &mut W) -> anyhow::Result<()> {
                match self {
                    #(#serialize_fields),*
                }
                Ok(())
            }

            fn deserialize<R: std::io::Read>(r: &mut R) -> anyhow::Result<Self> {
                let id = u16::deserialize(r)?;
                match id {
                    #(#deserialize_fields),*
                    _ => anyhow::bail!("unknown packet id: {}", id),
                }
            }
        }
    }
}

fn make_packet_enum(input: &ItemEnum) -> TokenStream {
    let variants = input.variants.iter().map(|variant| {
        let ident = &variant.ident;
        let fields = &variant.fields;
        quote! { #ident #fields }
    });

    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;

    quote! { #(#attrs)* #vis enum #ident { #(#variants),* } }
}

#[proc_macro_attribute]
pub fn packet(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as ItemEnum);
    let packet_enum = make_packet_enum(&input);
    let serialize_impl = make_serialize_impl(&input);
    let tokens = quote! {
        #packet_enum
        #serialize_impl
    };
    tokens.into()
}
