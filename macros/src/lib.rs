extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields};
use syn::{DeriveInput, Type};

#[proc_macro_derive(Tree)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let subtree_field = fields
                .named
                .iter()
                .find(|field| {
                    let name = field.ident.as_ref().unwrap();
                    name.to_string() == "subtrees"
                })
                .unwrap();

            let name = input.ident;
            let list = if let Type::Path(path) = &subtree_field.ty {
                &path.path.segments.first().unwrap().ident
            } else {
                panic!()
            };

            return TokenStream::from(quote!(
            impl Tree for #name {
                type Item = #name;

                fn new(v: Vertex) -> Self {
                    #name {
                        elem: v,
                        parent: None,
                        subtrees: #list::new(),
                    }
                }

                fn elem(&self) -> Vertex {
                    self.elem
                }

                fn parent(&self) -> Option<Link<Self>> {
                    match &self.parent {
                        Some(location) => Some(location.clone()),
                        None => None,
                    }
                }

                fn mend(a_tree: Link<Self>, b_tree: Link<Self>, a: u32, b: u32) -> Link<Self> {
                    if a > b {
                        a_tree.borrow_mut().parent = Some(b_tree.clone());
                        b_tree.borrow_mut().subtrees.extend(once(a_tree));
                        b_tree
                    } else {
                        b_tree.borrow_mut().parent = Some(a_tree.clone());
                        a_tree.borrow_mut().subtrees.extend(once(b_tree));
                        a_tree
                    }
                }
            }));
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
