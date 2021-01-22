extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, Ident, ItemStruct, ItemType, ItemUnion};

fn try_parse_ident(item: TokenStream) -> Result<Ident, TokenStream> {
    syn::parse_macro_input::parse::<ItemStruct>(item.clone())
        .map(|ident| ident.ident)
        .or_else(|_| {
            syn::parse_macro_input::parse::<ItemType>(item.clone()).map(|ident| ident.ident)
        })
        .or_else(|_| syn::parse_macro_input::parse::<ItemUnion>(item).map(|ident| ident.ident))
        .map_err(|err| TokenStream::from(err.to_compile_error()))
}

#[proc_macro_attribute]
pub fn assert_size(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut base = item.clone();
    let ident = match try_parse_ident(item) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let sz = parse_macro_input!(attr as Expr);
    let checker = TokenStream::from(quote! {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{ const ASSERT: bool = core::mem::size_of::<#ident>() == #sz; ASSERT } as usize] = [];
    });
    base.extend(checker);
    base
}

struct ArgTwo {
    lhs: syn::Expr,
    rhs: syn::Expr,
}

impl syn::parse::Parse for ArgTwo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs: syn::Expr = input.parse()?;
        input.parse::<syn::token::Comma>()?;
        let rhs: syn::Expr = input.parse()?;
        if input.is_empty() {
            Ok(Self { lhs, rhs })
        } else {
            Err(syn::Error::new(input.span(), "EOF expect"))
        }
    }
}

#[proc_macro]
pub fn const_assert_eq(item: TokenStream) -> TokenStream {
    let ArgTwo { lhs, rhs } = syn::parse_macro_input!(item as ArgTwo);
    TokenStream::from(quote! {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{ const ASSERT: bool = #lhs == #rhs; ASSERT } as usize] = [];
    })
}
