mod context;
mod eml;
mod ess;
mod ext;
mod run;
mod widgets;

extern crate proc_macro;
use quote::*;
use run::Run;
use syn::parse_macro_input;
use syn_rsx::parse;

#[proc_macro]
pub fn eml(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ctx = context::Context::new();
    match parse(tree.into()) {
        Err(err) => err.to_compile_error().into(),
        Ok(root) => proc_macro::TokenStream::from(match eml::construct(&ctx, &root[0]) {
            Ok(stream) => stream,
            Err(e) => e.to_compile_error(),
        }),
    }
}

#[proc_macro_attribute]
pub fn widget(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::ItemFn);
    proc_macro::TokenStream::from(match widgets::widget(ast) {
        Err(e) => e.to_compile_error(),
        Ok(stream) => stream,
    })
}

#[proc_macro]
pub fn ess_define(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let def = parse_macro_input!(input as ess::EssDefinition);
    let ident = &def.ident;
    let value = format!("{:#}", def.stylesheet);
    proc_macro::TokenStream::from(quote! {
        const #ident: &'static str =
        #value;
    })
}

#[proc_macro]
pub fn ess(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stylesheet = parse_macro_input!(input as ess::StyleSheet);
    let ctx = context::Context::new();
    let core = ctx.core_path();
    let repr = format!("{stylesheet:#}");
    proc_macro::TokenStream::from(quote! {
        #core::ess::StyleSheet::parse(#repr)
    })
}

#[proc_macro]
pub fn run(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let run = parse_macro_input!(tree as Run);
    proc_macro::TokenStream::from(run.build())
}
