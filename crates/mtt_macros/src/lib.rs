use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, ItemEnum, Meta, MetaNameValue, Variant, Expr};

fn parse_id(variant: &Variant) -> Expr {
    variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("id"))
        .map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue { value, .. }) => value,
            _ => panic!("id must be a name-value attribute"),
        })
        .unwrap()
        .clone()
}

fn make_packet_impls(input: &ItemEnum) -> TokenStream {
    let ident = &input.ident;

    let serialize_variants = input.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let id = parse_id(variant);

        quote! {
            #ident::#v_ident(pkt) => {
                (#id as u16).serialize(w)?;
                pkt.serialize(w)?;
            }
        }
    });

    let deserialize_variants = input.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let id = parse_id(variant);
        let ty = variant.fields.iter().next().unwrap();

        quote! {
             #id => Ok(#ident::#v_ident(#ty::deserialize(r)?)),
        }
    });

    let from_impls = input.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let ty = variant.fields.iter().next().unwrap();

        quote! {
            impl From<#ty> for #ident {
                fn from(v: #ty) -> Self {
                    #ident::#v_ident(v)
                }
            }
        }
    });

    quote! {
        impl mtt_serialize::Serialize for #ident {
            fn serialize<W: std::io::Write>(&self, w: &mut W) -> anyhow::Result<()> {
                match self {
                    #(#serialize_variants),*
                }
                Ok(())
            }

            fn deserialize<R: std::io::Read>(r: &mut R) -> anyhow::Result<Self> {
                let id = u16::deserialize(r)?;
                match id {
                    #(#deserialize_variants)*
                    _ => anyhow::bail!("unknown packet id: {}", id),
                }
            }
        }

        #(#from_impls)*
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

// TODO: Remove packet specifics and make it a generic "impl Serialize for enum" macro
// TODO: Convert to proc_derive macro
#[proc_macro_attribute]
pub fn packet(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as ItemEnum);
    let packet_enum = make_packet_enum(&input);
    let serialize_impl = make_packet_impls(&input);
    let tokens = quote! {
        #packet_enum
        #serialize_impl
    };

    tokens.into()
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data = if let Data::Struct(data) = input.data {
        data
    } else {
        panic!("#[derive(Serialize)] only works with structs");
    };

    let serialize_fields = data.fields.iter().map(|field| {
        let ident = &field.ident;

        quote! {
            self.#ident.serialize(w)?;
        }
    });

    let deserialize_fields = data.fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;

        quote! {
            #ident: #ty::deserialize(r)?,
        }
    });

    let tokens = quote! {
        impl #impl_generics mtt_serialize::Serialize for #ident #ty_generics #where_clause {
            fn serialize<W: std::io::Write>(&self, w: &mut W) -> anyhow::Result<()> {
                #(#serialize_fields)*
                Ok(())
            }

            fn deserialize<R: std::io::Read>(r: &mut R) -> anyhow::Result<Self> {
                Ok(#ident {
                    #(#deserialize_fields)*
                })
            }
        }
    };

    tokens.into()
}
