extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::{parse_macro_input, Fields};

#[proc_macro_derive(PriorityQueue)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let lookup = fields.named.iter().find(|field| {
                let name = field.ident.as_ref().unwrap();
                name.to_string() == "lookup"
            });

            let name = input.ident;
            let (insert, remove) = if let Some(_) = lookup {
                (
                    quote!(self.lookup.insert(value,end);),
                    quote!(self.lookup.remove(&min.value);),
                )
            } else {
                (quote!(), quote!())
            };

            return TokenStream::from(quote!(
                impl PriorityQueue for #name {
                    type RefType = Self::Value;

                    type Key = u32;

                    type Value = Vertex;

                    fn is_empty(&self) -> bool {
                        self.inner.is_empty()
                    }

                    fn pop(&mut self) -> (Self::Key, Self::Value) {
                        let min = self.inner.swap_remove(0);
                        self.bubble_down();
                        #remove
                        return (min.key, min.value);
                    }

                    fn push(&mut self, key: Self::Key, value: Self::Value) -> Self::RefType {
                        self.inner.push(Item { key, value });
                        let end = self.inner.len() - 1;
                        #insert
                        self.bubble_up(end);
                        value
                    }
                }
            ));
        }
    }

    // Catchall if we don't match on the structure we don't want
    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields and one 'subtrees' field can match",
        )
        .to_compile_error(),
    )
}
