use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Type};

#[proc_macro_derive(Parameters)]
pub fn derive_parameter_set(input: TokenStream) -> TokenStream {
    // Parse the input token stream.
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Convert each field to a corresponding serialize and deserialize expression.
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };
    let serializers = fields.iter().map(|f| {
        let field_name = &f.ident;
        if let Type::Array(_) = &f.ty {
            // Arrays must be serialized individually into a vector.
            quote! { (
                stringify!(#field_name).to_owned(),
                SerializedParameter::List(
                    self.#field_name.iter().map(Parameter::serialize).collect()
                )
            ) }
        } else {
            quote! { (stringify!(#field_name).to_owned(), self.#field_name.serialize()) }
        }
    });
    let deserializers = fields.iter().map(|f| {
        let field_name = &f.ident;
        if let Type::Array(_) = &f.ty {
            // Arrays must individually unpack each element.
            quote! {
                if let SerializedParameter::List(ps) = &params[stringify!(#field_name)] {
                    for (field, param) in self.#field_name.iter().zip(ps) {
                        field.deserialize(param);
                    }
                }
            }
        } else {
            quote! { self.#field_name.deserialize(&params[stringify!(#field_name)]); }
        }
    });

    // Generate the serialize/deserialize impls.
    TokenStream::from(quote! {
        #[automatically_derived]
        impl #struct_name {
            fn serialize(&self) -> HashMap<String, SerializedParameter> {
                HashMap::from([
                    #(#serializers),*
                ])
            }
            fn deserialize(&self, params: &HashMap<String, SerializedParameter>) {
                #(#deserializers)*
            }
        }
    })
}
