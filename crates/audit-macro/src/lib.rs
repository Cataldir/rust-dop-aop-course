use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input,ItemFn};
/// Teaching aspect for Result-returning functions. Logs outcome/duration, never inputs.
/// It preserves ordinary return/? behavior; panic and cancellation are not completion.
#[proc_macro_attribute]
pub fn audit(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut function=parse_macro_input!(input as ItemFn);
    if !args.is_empty() { return syn::Error::new(function.sig.ident.span(),"audit takes no arguments").to_compile_error().into(); }
    let body=&function.block;let name=function.sig.ident.to_string();
    let invocation=if function.sig.asyncness.is_some() { quote! { (async #body).await } } else { quote! { (|| #body)() } };
    function.block=Box::new(syn::parse_quote!({
        let audit_started=::std::time::Instant::now();
        ::metrics::counter!("course_audit_started_total","operation"=>#name).increment(1);
        let audit_result=#invocation;
        let audit_ok=audit_result.is_ok();
        ::tracing::info!(operation=#name,success=audit_ok,elapsed_us=audit_started.elapsed().as_micros() as u64,"audit completion");
        ::metrics::counter!("course_audit_completed_total","operation"=>#name,"success"=>audit_ok.to_string()).increment(1);
        audit_result
    }));
    quote!(#function).into()
}
