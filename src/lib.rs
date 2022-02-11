use proc_macro::TokenStream;
use quote::*;
use syn::{
    parse::Parse, parse_macro_input, Attribute, GenericArgument, Ident, ItemImpl, Path,
    PathArguments, Type, TypePath,
};

struct MonomoInput {
    attrs: Vec<Attribute>,
    path: Path,
}

impl Parse for MonomoInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let TypePath { qself: _, path } = input.parse::<TypePath>()?;

        Ok(MonomoInput { attrs, path })
    }
}

#[proc_macro]
pub fn rphize(tokens: TokenStream) -> TokenStream {
    let parse_input = parse_macro_input!(tokens as MonomoInput);

    let monomorphized = monomorphize_path(&parse_input.path);
    let attrs = parse_input.attrs;
    let full_path = parse_input.path;

    quote! {
        #[allow(non_camel_case_types)]
        #(#attrs)*
        trait #monomorphized : #full_path {}
    }
    .into()
}

#[proc_macro]
pub fn rph(tokens: TokenStream) -> TokenStream {
    let parse_input = parse_macro_input!(tokens as MonomoInput);
    let monomorphized = format_ident!("__{}", flatten_type_path(&parse_input.path));

    quote! {
        dyn #monomorphized
    }
    .into()
}

#[proc_macro_attribute]
pub fn rphize_impl(_input: TokenStream, tokens: TokenStream) -> TokenStream {
    let mut parse_input = parse_macro_input!(tokens as ItemImpl);
    let ty = &parse_input.self_ty;

    let monomorphized = monomorphize_path(
        &parse_input
            .trait_
            .as_ref()
            .expect("No trait to monomorphize!")
            .1,
    );

    let attrs = parse_input.attrs.clone();
    parse_input.attrs = vec![];

    quote! {
        #parse_input
        #(#attrs)*
        impl #monomorphized for #ty {}
    }
    .into()
}

fn monomorphize_path(path: &Path) -> Ident {
    format_ident!("__{}", flatten_type_path(path))
}

fn flatten_type_path(path: &Path) -> Ident {
    let ty = path.segments.last().expect("Incorrect identifier!");
    let mut flattened_ident = ty.ident.clone();

    if let PathArguments::AngleBracketed(gen_args) = &ty.arguments {
        gen_args.args.iter().for_each(|gen_arg| match gen_arg {
            GenericArgument::Type(Type::Path(t)) => {
                flattened_ident =
                    format_ident!("{}_{}", flattened_ident, flatten_type_path(&t.path));
            }
            _ => panic!("Lifetimes cannot be monomorphized!"),
        })
    }
    flattened_ident
}
