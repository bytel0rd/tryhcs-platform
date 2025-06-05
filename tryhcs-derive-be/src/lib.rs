use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field, Fields, Ident};

#[proc_macro_derive(Encrypted, attributes(deterministic, randomized))]
pub fn derive_encrypted(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let encrypted_name = format_ident!("Encrypted{}", name);

    let fields = match input.data {
        Data::Struct(ref data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Encrypted struct fields: all Vec<u8>
    let encrypted_fields = fields.iter().map(|field| {
        let fname = &field.ident;
        let ftype = &field.ty;
        let is_deterministic = has_attr(field, "deterministic");
        let is_randomized = has_attr(field, "randomized");

        match (is_deterministic, is_randomized) {
            (true, _) | (_, true) => quote! {
                pub #fname: Vec<u8>,
            },
            _ => quote! {
                pub #fname: #ftype,
            },
        }
    });

    // Encryption logic per field
    let encrypt_stmts = fields.iter().map(|field| {
        let fname = &field.ident;
        let is_deterministic = has_attr(field, "deterministic");
        let is_randomized = has_attr(field, "randomized");

        match (is_deterministic, is_randomized) {
            (true, false) => quote! {
                #fname: self.#fname.encrypt_deterministic(encryptor)?,
            },
            (false, true) => quote! {
                #fname: self.#fname.encrypt_randomized(encryptor)?,
            },
            _ => quote! { #fname:  self.#fname.clone(), },
        }
    });

    let expanded = quote! {
        pub struct #encrypted_name {
            #(#encrypted_fields)*
        }

        impl Encrypted for #name {
            type Output = #encrypted_name;

            fn encrypt(&self, encryptor: &Encryptor) -> eyre::Result<Self::Output> {
                Ok(#encrypted_name {
                    #(#encrypt_stmts)*
                })
            }
        }
    };

    expanded.into()
}

fn has_attr(field: &Field, attr_name: &str) -> bool {
    field.attrs.iter().any(|attr| attr.path.is_ident(attr_name))
}
