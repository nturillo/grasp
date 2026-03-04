use proc_macro::{Span, TokenStream};

#[cfg(feature = "meta")]
use quote::{format_ident, quote};
#[cfg(feature = "meta")]
use syn::{Error, FnArg, ItemFn, LitStr, Meta, Token, parse_macro_input, punctuated::Punctuated, ExprTuple, ExprArray, Expr};

#[cfg(feature = "meta")]
fn new_error(string: String) -> TokenStream {
    Error::new(Span::call_site().into(), string).into_compile_error().into()
}

#[proc_macro_attribute]
pub fn register(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "meta")] {
        let mut input = parse_macro_input!(item as ItemFn);
        let attributes = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

        let args: Vec<Meta> = attributes.into_iter().collect();
        let mut name: Option<String> = None;
        let mut return_type: Option<String> = None;
        let mut tooltip = String::new();
        let mut simple = false;

        let mut param_data = Vec::new();

        for arg in &args {
            if let Meta::NameValue(nv) = arg {
                let val = &nv.value;
                let val_string = quote!(#val).to_string().replace('"', "");

                if nv.path.is_ident("name") {
                    name = Some(val_string);
                } else if nv.path.is_ident("ret") {
                    return_type = Some(val_string);
                } else if nv.path.is_ident("simple") {
                    simple = val_string.as_str() == "true";
                } else if nv.path.is_ident("desc") {
                    tooltip = val_string;
                } else if nv.path.is_ident("params") {
                    if let Expr::Array(ExprArray {elems, ..}) = val {
                        for e in elems {
                            if let Expr::Tuple(ExprTuple {elems: param, ..}) = e {
                                if param.len() != 2 {
                                    return new_error(format!("Expected 2 arguments, found {}", param.len()));
                                }

                                let p0 = &param[0];
                                let p1 = &param[1];

                                let disp_name = quote!(#p0).to_string().replace('"', "");
                                let basic_type_str = quote!(#p1).to_string();

                                param_data.push(quote! {
                                    &[#disp_name, #basic_type_str]
                                });
                            }
                        }
                    } else if let Expr::Tuple(ExprTuple {elems: param, ..}) = val {
                        if param.len() != 2 {
                            return new_error(format!("Expected 2 arguments, found {}", param.len()));
                        }

                        let p0 = &param[0];
                        let p1 = &param[1];

                        let disp_name = quote!(#p0).to_string().replace('"', "");
                        let basic_type_str = quote!(#p1).to_string();

                        param_data.push(quote! {
                            &[#disp_name, #basic_type_str]
                        });
                    }
                }
            }
        }

        let name = match name {
            Some(n) => n,
            None => { return new_error("Missing name attribute".to_string());},
        };

        let return_type = match return_type {
            Some(n) => n,
            None => { return new_error("Missing ret attribute".to_string());},
        };

        match return_type.as_str() {
            "String" | "Vertex" | "VertexList" | "Edge" | "EdgeList" | "None" => (),
            _ => { return new_error(format!("Expected one of [String, Vertex, VertexList, Edge, EdgeList, None], found {}", return_type)); }
        }

        let return_type_ident = format_ident!("{}", return_type);

        let fn_name = &input.sig.ident;
        let wrapped_name = quote::format_ident!("_wrap_{}", fn_name);
        let unique_id = quote::format_ident!("_ALGO_{}", fn_name.to_string().to_uppercase());

        let mut down_refs = Vec::new();
        let mut arg_names = Vec::new();

        for (i, param) in &mut input.sig.inputs.iter_mut().skip(1).enumerate() {
            if let FnArg::Typed(p_data) = param {
                let p_name = &p_data.pat;
                let p_type = &p_data.ty;

                down_refs.push(quote! {
                    let #p_name = <#p_type as crate::algorithms::registry::FromArgType>::from_arg(&args[#i]).expect(concat!("Parameter ", stringify!(#p_name), " cannot be parsed as ", stringify!(#p_type)));
                });
                arg_names.push(quote! { #p_name });
            }
        }

        if arg_names.len() != param_data.len() {
            return new_error(format!("Found {} extra parameters, but you registered {} parameters", arg_names.len() , param_data.len()));
        }

        let is_self = match input.sig.inputs.first() {
            Some(FnArg::Receiver(_)) => true,
            _ => false
        };

        let graph = if simple { quote! { &graph.underlying_graph() }} else { quote! {graph} };
        let call = if is_self { quote! {#graph.#fn_name(#(#arg_names),*)} } else { quote! {#fn_name(#graph, #(#arg_names),*)} };
        let ptr = if is_self { quote! {Self::#wrapped_name} } else { quote! {#wrapped_name} };

        let out = quote! {
            #input

            #[cfg(feature = "meta")]
            fn #wrapped_name(graph: &crate::graph::adjacency_list::SparseDiGraph, args: &[crate::algorithms::registry::ArgType]) -> crate::algorithms::registry::ReturnType {
                std::hint::black_box(&#unique_id);

                #(#down_refs)*

                crate::algorithms::registry::ReturnType::#return_type_ident(#call)
            }

            #[cfg(feature = "meta")]
            #[linkme::distributed_slice(crate::algorithms::registry::ALGORITHMS)]
            pub static #unique_id: crate::algorithms::registry::FunctionData = crate::algorithms::registry::FunctionData {
                name: #name,
                module: module_path!(),
                func: #ptr,
                return_type: #return_type,
                param_data: &[#(#param_data),*],
                desc: #tooltip,
            };
        };

        TokenStream::from(out)
    }

    #[cfg(not(feature = "meta"))]
    {
        item
    }
}