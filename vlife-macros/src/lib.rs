mod genome;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BuildGenome, attributes(build_genome))]
pub fn derive_build_genome(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    genome::derive_build_genome(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
