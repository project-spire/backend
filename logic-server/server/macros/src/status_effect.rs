use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Meta};

pub fn expand_derive_status_effect(input: DeriveInput) -> Result<TokenStream, Error> {
    let data = match input.data {
        Data::Enum(data) => data,
        _ => return Err(Error::new_spanned(
            input,
            "StatusEffect can only be derived for enums"
        ))
    };
    let name = input.ident;

    let mut kind_arms = Vec::new();
    for variant in &data.variants {
        let variant_name = &variant.ident;

        let mut kind = None;
        for attr in &variant.attrs {
            match &attr.meta {
                Meta::Path(path) if path.is_ident("buff") =>
                    kind = Some(quote! { StatusEffectKind::Buff }),
                Meta::Path(path) if path.is_ident("debuff") =>
                    kind = Some(quote! { StatusEffectKind::Debuff }),
                Meta::Path(path) if path.is_ident("passive") =>
                    kind = Some(quote! { StatusEffectKind::Passive }),
                Meta::Path(path) if path.is_ident("curse") =>
                    kind = Some(quote! { StatusEffectKind::Curse }),
                _ => {},
            }
        }

        let kind = kind.ok_or_else(|| Error::new_spanned(
            variant_name,
            "StatusEffect variant must have a kind attribute (buff, debuff, passive, curse)"
        ))?;

        match &variant.fields {
            Fields::Unit => {
                kind_arms.push(quote! { Self::#variant_name => #kind, })
            },
            Fields::Named(_) | Fields::Unnamed(_) => {
                kind_arms.push(quote! { Self::#variant_name { .. } => #kind, })
            },
        }
    }

    let out = quote! {
        impl #name {
            pub fn kind(&self) -> StatusEffectKind {
                match self {
                    #(#kind_arms)*
                }
            }
        }
    };

    Ok(out.into())
}
