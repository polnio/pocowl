use convert_case::ccase;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use serde::{Deserialize, Deserializer};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Protocol {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "interface", default)]
    interfaces: Vec<Interface>,
}
#[derive(Debug, Deserialize)]
struct Interface {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@version")]
    version: String,
    #[serde(rename = "#content", default)]
    messages: Vec<MessageWrapper>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageWrapper {
    Request(Message),
    Event(Message),
    #[serde(other, deserialize_with = "deserialize_ignore_any")]
    Other,
}
#[derive(Debug, Deserialize)]
struct Message {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "arg", default)]
    args: Vec<Arg>,
}
#[derive(Debug, Deserialize)]
struct Arg {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    ty: String,
}

fn deserialize_ignore_any<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    serde::de::IgnoredAny::deserialize(deserializer)?;
    Ok(())
}

fn get_protocols(path: impl AsRef<Path>) -> Result<Vec<Protocol>, String> {
    let reader = std::fs::File::open(path).map_err(|err| err.to_string())?;
    let protocol = serde_xml_rs::from_reader(reader).map_err(|err| err.to_string())?;
    return Ok(vec![protocol]);
}

fn generate_protocols(protocols: Vec<Protocol>) -> TokenStream {
    let protocols = protocols.into_iter().map(|protocol| {
        let interfaces = protocol.interfaces.into_iter().map(|interface| {
            let (requests, events): (Vec<_>, Vec<_>) = interface
                .messages
                .into_iter()
                .partition(|msg| matches!(msg, MessageWrapper::Request(_)));

            let struct_name = quote::format_ident!("{}", ccase!(pascal, &interface.name));
            let trait_name = quote::format_ident!("{}Listener", ccase!(pascal, &interface.name));
            // let trait_accessor = requests.iter().enumerate().map(|(i, request)| {
            //     let MessageWrapper::Request(request) = request else {
            //         unreachable!()
            //     };
            //     let name = quote::format_ident!("r#{}_from_raw", ccase!(snake, &request.name));
            //     quote!(#i => self.#name(buf),)
            // });
            let trait_methods = requests.iter().map(|request| {
                let MessageWrapper::Request(request) = request else {
                    unreachable!()
                };
                let args_name = request.args.iter().map(|arg| {
                    let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    quote!(#name,)
                });
                let args = request.args.iter().map(|arg| {
                    let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    quote!(#name: u32,)
                });
                let name = quote::format_ident!("r#{}", ccase!(snake, &request.name));
                let name_raw = quote::format_ident!("r#{}_from_raw", ccase!(snake, &request.name));
                let raw_args = request.args.iter().map(|arg| {
                    let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    quote!(let #name = WaylandValue::from_raw(buf).unwrap();)
                });
                quote! {
                    fn #name(&mut self, #(#args)*) -> u32;
                    fn #name_raw(&mut self, buf: &mut &[u8]) -> u32 {
                        #(#raw_args)*
                        self.#name(#(#args_name)*)
                    }
                }
            });

            let (trait_accessor_i, trait_accessor_name): (Vec<_>, Vec<_>) = requests
                .iter()
                .enumerate()
                .map(|(i, request)| {
                    let MessageWrapper::Request(request) = request else {
                        unreachable!()
                    };
                    (
                        i as u16,
                        quote::format_ident!("r#{}_from_raw", ccase!(snake, &request.name)),
                    )
                })
                .unzip();

            quote! {
                #[derive(Copy, Clone)]
                pub struct #struct_name;
                impl<T: #trait_name> WaylandProtocol<T> for #struct_name {
                    fn call(&self, state: &mut T, opcode: u16, buf: &mut &[u8]) -> u32 {
                        state.call(opcode, buf)
                    }
                }
                pub trait #trait_name {
                    #(#trait_methods)*
                    fn call(&mut self, opcode: u16, buf: &mut &[u8]) -> u32 {
                        match opcode {
                            // #(#trait_accessor)*
                            #(#trait_accessor_i => self.#trait_accessor_name(buf),)*
                            _ => 0,
                        }
                    }
                }
            }
        });
        let name = quote::format_ident!("{}", ccase!(snake, protocol.name));
        quote! {
            pub mod #name {
                use super::{WaylandValue, WaylandProtocol};
                #(#interfaces)*
            }
        }
    });
    let tt: TokenStream = quote! {
        #(#protocols)*
    }
    .into();
    tt
}

#[proc_macro]
pub fn scan_protocol(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr);
    let path = PathBuf::from(input.value());
    let protocols = match get_protocols(&path) {
        Ok(protocols) => protocols,
        Err(err) => {
            return quote! {compile_error!(#err);}.into();
        }
    };
    let tt = generate_protocols(protocols);
    println!("{}", tt);
    tt
}
