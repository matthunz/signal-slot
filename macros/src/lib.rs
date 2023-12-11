use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, parse_quote, token::Type, Ident, Item, ItemImpl, PatType};

#[proc_macro_attribute]
pub fn signal(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemImpl);

    let msg_ident = parse_macro_input!(attrs as Ident);

    let items = item.items.clone();
    let sender_items: Vec<_> = item
        .items
        .iter()
        .map(|item| match item {
            syn::ImplItem::Fn(fn_item) => {
                let mut fn_item = fn_item.clone();
                fn_item.sig.inputs[0] = parse_quote!(&self);
                let block = fn_item.block.clone();
                let fn_ident = &fn_item.sig.ident;
                let inputs: Vec<_> = fn_item
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Receiver(r) => None,
                        syn::FnArg::Typed(ty) => Some(ty.pat.to_token_stream()),
                    })
                    .collect();
                fn_item.block = parse_quote!({
                    let cx = self.handle;
                    self.handle.update(move |me| {
                        me.#fn_ident(#(#inputs)*)
                    });
                });
                fn_item
            }

            _ => todo!(),
        })
        .collect();

    let ident = format_ident!("{}", item.self_ty.to_token_stream().to_string());
    let sender_ident = format_ident!("{}Sender", &ident);
    let output = quote! {
        impl Object for #ident {
            type Message = #msg_ident;
            type Sender = #sender_ident;

            fn emit(&mut self, _msg: Self::Message) {}
        }

        impl #ident {
            #(#items)*
        }

        pub struct #sender_ident {
            handle: HandleState<#ident>,
        }

        impl From<HandleState<#ident>> for #sender_ident {
            fn from(value: HandleState<#ident>) -> Self {
                Self { handle: value }
            }
        }


        impl #sender_ident {
            #(#sender_items)*
        }
    };
    output.into_token_stream().into()
}
