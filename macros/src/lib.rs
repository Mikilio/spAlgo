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
            let (insert, update, remove) = if let Some(_) = lookup {
                (
                    quote!(
                        self.lookup.insert(value,end);
                        #[cfg(debug_assertions)] dbg!(self.lookup.capacity());
                    ),
                    quote!(
                        let last = self.inner
                            .last()
                            .expect("pop_min called even though heap was empty")
                            .clone()
                            .value;
                        self.lookup.insert(last, 0);
                    ),
                    quote!(
                        self.lookup.remove(&min.value);
                        #[cfg(debug_assertions)] dbg!(self.lookup.capacity());
                    ),
                )
            } else {
                (quote!(), quote!(), quote!())
            };

            return TokenStream::from(quote!(
                impl PriorityQueue for #name {
                    type RefType = Self::Value;

                    type Key = u32;

                    type Value = Vertex;

                    #[inline]
                    fn is_empty(&self) -> bool {
                        self.inner.is_empty()
                    }

                    #[inline]
                    fn pop(&mut self) -> Option<(Self::Key, Self::Value)> {
                        if self.is_empty() {
                            return None;
                        }
                        #update
                        let min = self.inner.swap_remove(0);
                        #remove
                        self.bubble_down();
                        return Some((min.key, min.value));
                    }

                    #[inline]
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
