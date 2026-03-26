use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
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
    version: u32,
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
    ty: Type,
    #[serde(rename = "@interface")]
    interface: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Type {
    Int,
    Uint,
    Fixed,
    Object,
    #[serde(rename = "new_id")]
    NewId,
    String,
    Array,
    FD,
    Enum,
}
impl Type {
    fn to_rust_type(&self) -> TokenStream2 {
        match self {
            Type::Int => quote!(i32),
            Type::Uint => quote!(u32),
            Type::Fixed => quote!(I24F8),
            Type::Object => quote!(u32),
            Type::NewId => quote!(u32),
            Type::String => quote!(String),
            // TODO: Handle more types
            Type::Array => quote!(()),
            Type::FD => quote!(()),
            Type::Enum => quote!(()),
        }
    }
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

fn generate_args<'a>(
    args: impl Iterator<Item = &'a Arg>,
) -> impl Iterator<Item = (proc_macro2::Ident, proc_macro2::TokenStream)> {
    args.flat_map(|arg| match (&arg.ty, &arg.interface) {
        (Type::NewId, None) => {
            let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
            let name_int = quote::format_ident!("r#{}_interface", ccase!(snake, &arg.name));
            let name_ver = quote::format_ident!("r#{}_version", ccase!(snake, &arg.name));
            vec![
                (name_int, quote!(String)),
                (name_ver, quote!(u32)),
                (name, quote!(u32)),
            ]
        }
        _ => {
            let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
            let ty = arg.ty.to_rust_type();
            vec![(name, ty)]
        }
    })
}

fn generate_protocols(protocols: Vec<Protocol>) -> TokenStream2 {
    protocols
        .into_iter()
        .map(|protocol| {
            let interfaces = protocol.interfaces.into_iter().map(|interface| {
                let (requests, events): (Vec<_>, Vec<_>) = interface
                    .messages
                    .into_iter()
                    .partition(|msg| matches!(msg, MessageWrapper::Request(_)));

                let struct_name = quote::format_ident!("{}", ccase!(pascal, &interface.name));
                let trait_name =
                    quote::format_ident!("{}Listener", ccase!(pascal, &interface.name));
                let xml_name_str = &interface.name;
                let xml_version = interface.version;
                let trait_methods = requests.iter().filter_map(|request| {
                    let MessageWrapper::Request(request) = request else {
                        return None;
                    };
                    // let args_name = request.args.iter().map(|arg| {
                    //     let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    //     quote!(#name,)
                    // });
                    // let (args_name, args_ty): (Vec<_>, Vec<_>) = request.args.iter().map(|arg| {
                    //     let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    //     let ty = arg.ty.to_rust_type();
                    //     // quote!(#name: #ty,)
                    //     (name, ty)
                    // }).collect();
                    let (args_name, args_ty): (Vec<_>, Vec<_>) = generate_args(request.args.iter()).collect();
                    let name = quote::format_ident!("r#{}", ccase!(snake, &request.name));
                    let name_raw =
                        quote::format_ident!("r#{}_from_raw", ccase!(snake, &request.name));
                    let raw_args = args_name.iter().map(|arg| {
                        quote!(let #arg = WaylandValue::from_raw(buf).unwrap();)
                    });
                    Some(quote! {
                        fn #name(&mut self, object_id: u32, #(#args_name: #args_ty,)* client: &mut WaylandClient);
                        fn #name_raw(&mut self, message: WaylandMessage, client: &mut WaylandClient) {
                            let mut buf = message.data.as_slice();
                            let buf = &mut buf;
                            #(#raw_args)*
                            self.#name(message.object_id, #(#args_name,)* client)
                        }
                    })
                });

                let struct_methods = events.iter().filter_map(|event| match event {
                    MessageWrapper::Event(event) => Some(event),
                    _ => None,
                }).enumerate().filter_map(|(i, event)| {
                    let opcode = i as u16;
                    // let (args_name, args_ty): (Vec<_>, Vec<_>) = event
                    //     .args
                    //     .iter()
                    //     .flat_map(|arg| {
                    //         match (&arg.ty, &arg.interface) {
                    //             (Type::NewId, None) => {
                    //                 let name_uid = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    //                 let name_int = quote::format_ident!("r#{}_interface", ccase!(snake, &arg.name));
                    //                 let name_ver = quote::format_ident!("r#{}_version", ccase!(snake, &arg.name));
                    //                 vec![(name_int, quote!(String)), (name_ver, quote!(u32)), (name_uid, quote!(u32))]
                    //             }
                    //             _ => {
                    //                 let name = quote::format_ident!("r#{}", ccase!(snake, &arg.name));
                    //                 let ty = arg.ty.to_rust_type();
                    //                 vec![(name, ty)]
                    //             }
                    //         }
                    //     })
                    //     .collect();
                    let (args_name, args_ty): (Vec<_>, Vec<_>) = generate_args(event.args.iter()).collect();
                    let name = quote::format_ident!("r#{}", ccase!(snake, &event.name));
                    Some(quote! {
                        pub fn #name(id_dont_collide_with_args: u32, #(#args_name: #args_ty),*) -> WaylandMessage {
                            let mut buf_dont_collide_with_args = Vec::new();
                            #(buf_dont_collide_with_args.extend(WaylandValue::to_raw(#args_name));)*
                            WaylandMessage::new(id_dont_collide_with_args, #opcode, buf_dont_collide_with_args)
                        }
                    })
                });

                let (trait_accessor_i, trait_accessor_name): (Vec<_>, Vec<_>) = requests
                    .iter()
                    .enumerate()
                    .filter_map(|(i, request)| {
                        let MessageWrapper::Request(request) = request else {
                            return None;
                        };
                        Some((
                            i as u16,
                            quote::format_ident!("r#{}_from_raw", ccase!(snake, &request.name)),
                        ))
                    })
                    .unzip();

                quote! {
                    #[derive(Copy, Clone)]
                    pub struct #struct_name;
                    impl #struct_name {
                        pub const NAME: &'static str = #xml_name_str;
                        pub const VERSION: u32 = #xml_version;
                        #(#struct_methods)*
                    }
                    impl<T: #trait_name> WaylandProtocol<T> for #struct_name {
                        fn call(&self, state: &mut T, message: WaylandMessage, client: &mut WaylandClient) {
                            state.call(message, client)
                        }
                        fn name(&self) -> &'static str {
                            Self::NAME
                        }
                        fn version(&self) -> u32 {
                            Self::VERSION
                        }
                    }
                    pub trait #trait_name {
                        #(#trait_methods)*
                        fn call(&mut self, message: WaylandMessage, client: &mut WaylandClient) {
                            match message.opcode {
                                #(#trait_accessor_i => self.#trait_accessor_name(message, client),)*
                                _ => {},
                            }
                        }
                    }
                }
            });
            let name = quote::format_ident!("{}", ccase!(snake, protocol.name));
            quote! {
                pub mod #name {
                    use fixed::types::I24F8;
                    // use tokio::net::UnixStream;
                    use pocowl_wlmessage::WaylandMessage;
                    use pocowl_wlvalue::WaylandValue;
                    use pocowl_protocols_base::WaylandProtocol;
                    use pocowl_wlclient::WaylandClient;
                    #(#interfaces)*
                }
            }
        })
        .collect()
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
    // println!("{}", tt);
    tt.into()
}
