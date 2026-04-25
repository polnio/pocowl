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
    Enum(Enum),
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
    #[serde(rename = "@allow-null", default)]
    allow_null: bool,
    #[serde(rename = "@enum", default)]
    enum_: Option<String>,
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
            Type::FD => quote!(i32),
            // TODO: Handle more types
            Type::Array => quote!(()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Enum {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "entry")]
    entries: Vec<EnumEntry>,
}
#[derive(Debug, Deserialize)]
struct EnumEntry {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
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

macro_rules! generate_ident {
    ($($name:tt)*) => {{
        let name = format!($($name)*);
        let name = if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            "_".to_owned() + &name
        } else {
            name
        };
        quote::format_ident!("r#{}", name)
    }};
}

fn generate_args<'a>(
    interface: &'a str,
    args: impl Iterator<Item = &'a Arg>,
) -> impl Iterator<Item = (proc_macro2::Ident, proc_macro2::TokenStream, bool)> {
    args.flat_map(|arg| match (&arg.ty, &arg.interface, &arg.enum_) {
        (Type::NewId, None, _) => {
            let name = generate_ident!("{}", ccase!(snake, &arg.name));
            let name_int = generate_ident!("{}_interface", ccase!(snake, &arg.name));
            let name_ver = generate_ident!("{}_version", ccase!(snake, &arg.name));
            vec![
                (name_int, quote!(String), false),
                (name_ver, quote!(u32), false),
                (name, quote!(u32), false),
            ]
        }
        (Type::NewId | Type::Object, Some(interface), _) => {
            let name = generate_ident!("{}", ccase!(snake, &arg.name));
            let ty = generate_ident!("{}", ccase!(pascal, &interface));
            if arg.allow_null {
                vec![(name, quote!(Option<#ty>), false)]
            } else {
                vec![(name, quote!(#ty), false)]
            }
        }
        (Type::Uint | Type::Int, _, Some(enum_)) => {
            let (interface, enum_) = enum_.split_once('.').unwrap_or((interface, enum_));
            let name = generate_ident!("{}", ccase!(snake, &arg.name));
            let ty = generate_ident!("{}{}", ccase!(pascal, &interface), ccase!(pascal, &enum_));
            if arg.allow_null {
                vec![(name, quote!(Option<#ty>), false)]
            } else {
                vec![(name, quote!(#ty), false)]
            }
        }
        (Type::FD, _, _) => {
            let name = generate_ident!("{}", ccase!(snake, &arg.name));
            vec![(name, quote!(std::os::fd::OwnedFd), true)]
        }
        _ => {
            let name = generate_ident!("{}", ccase!(snake, &arg.name));
            let ty = arg.ty.to_rust_type();
            if arg.allow_null {
                vec![(name, quote!(Option<#ty>), false)]
            } else {
                vec![(name, ty, false)]
            }
        }
    })
}

fn generate_protocols(protocols: Vec<Protocol>) -> TokenStream2 {
    protocols
        .into_iter()
        .map(|protocol| {
            let interfaces = protocol.interfaces.into_iter().map(|interface| {
                let struct_name = quote::format_ident!("{}", ccase!(pascal, &interface.name));
                let trait_name =
                    quote::format_ident!("{}Listener", ccase!(pascal, &interface.name));
                let xml_name_str = &interface.name;
                let xml_version = interface.version;
                let trait_methods = interface.messages.iter().filter_map(|request| {
                    let MessageWrapper::Request(request) = request else {
                        return None;
                    };
                    let (args_name, args_ty, is_fd): (Vec<_>, Vec<_>, Vec<_>) = generate_args(&interface.name, request.args.iter()).collect();
                    let name = generate_ident!("{}", ccase!(snake, &request.name));
                    let name_raw =
                        generate_ident!("{}_from_raw", ccase!(snake, &request.name));
                    let raw_args = Iterator::zip(args_name.iter(), is_fd.iter()).map(|(arg, is_fd)| {
                        if *is_fd {
                            quote!(let #arg = fds.pop_front().unwrap();)
                        } else {
                            quote!(let #arg = WaylandValue::from_raw(buf).unwrap();)
                        }
                    });
                    Some(quote! {
                        async fn #name(&mut self, object: #struct_name, #(#args_name: #args_ty,)*);
                        async fn #name_raw(&mut self, message: WaylandMessage, fds: &mut VecDeque<OwnedFd>) {
                            let mut buf = message.data.as_slice();
                            let buf = &mut buf;
                            #(#raw_args)*
                            let object = #struct_name { object_id: message.object_id };
                            self.#name(object, #(#args_name,)*).await
                        }
                    })
                });

                let struct_methods = interface.messages.iter().filter_map(|event| match event {
                    MessageWrapper::Event(event) => Some(event),
                    _ => None,
                }).enumerate().filter_map(|(i, event)| {
                    let opcode = i as u16;
                    let (args_name, args_ty, args_are_fds): (Vec<_>, Vec<_>, Vec<_>) = generate_args(&interface.name, event.args.iter()).collect();
                    // FIXME: Handle fds
                    let name = generate_ident!("{}", ccase!(snake, &event.name));
                    let raw_args = Iterator::zip(args_name.iter(), args_are_fds.iter()).filter_map(|(arg, is_fd)| {
                        (!is_fd).then(|| quote!(WaylandValue::to_raw(#arg)))
                    });
                    Some(quote! {
                        pub fn #name(self, #(#args_name: #args_ty),*) -> WaylandMessage {
                            let mut buf_dont_collide_with_args = Vec::new();
                            #(buf_dont_collide_with_args.extend(#raw_args);)*
                            WaylandMessage::new(self.object_id, #opcode, buf_dont_collide_with_args)
                        }
                    })
                });

                let enums = interface.messages.iter().filter_map(|enum_| {
                    let MessageWrapper::Enum(enum_) = enum_ else {
                        return None;
                    };
                    let whole_name = format!("{}_{}", interface.name, enum_.name);
                    let name = generate_ident!("{}", ccase!(pascal, &whole_name));
                    let entries = enum_.entries.iter().filter_map(|entry| {
                        let name = generate_ident!("{}", ccase!(pascal, &entry.name));
                        let value = if let Some(hex) = entry.value.strip_prefix("0x") {
                            u32::from_str_radix(hex, 16)
                        } else {
                            entry.value.parse()
                        };
                        let value = match value {
                            Ok(value) => value,
                            Err(_) => {
                                eprintln!("Invalid value for enum {}: {}", whole_name, entry.value);
                                return None;
                            },
                        };
                        Some(quote!(#name = #value,))
                    });
                    Some(quote! {
                        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, num_enum::TryFromPrimitive)]
                        #[repr(u32)]
                        pub enum #name {
                            #(#entries)*
                        }
                        impl WaylandValue for #name {
                            fn from_raw(buf: &mut &[u8]) -> anyhow::Result<Self> {
                                let value: u32 = WaylandValue::from_raw(buf)?;
                                anyhow::Context::context(#name::try_from(value), "invalid enum value")
                            }
                            fn to_raw(self) -> Vec<u8> {
                                let value = self as u32;
                                WaylandValue::to_raw(value)
                            }
                        }
                    })
                });

                let (trait_accessor_i, trait_accessor_name): (Vec<_>, Vec<_>) = interface.messages
                    .iter()
                    .filter_map(|message| match message {
                        MessageWrapper::Request(request) => Some(request),
                        _ => None,
                    })
                    .enumerate()
                    .map(|(i, request)| {
                        (
                            i as u16,
                            generate_ident!("{}_from_raw", ccase!(snake, &request.name)),
                        )
                    })
                    .unzip();

                quote! {
                    #(#enums)*
                    #[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
                    pub struct #struct_name {
                        pub object_id: u32,
                    }
                    impl #struct_name {
                        pub const NAME: &'static str = #xml_name_str;
                        pub const VERSION: u32 = #xml_version;
                        #(#struct_methods)*
                    }
                    impl WaylandValue for #struct_name {
                        fn from_raw(buf: &mut &[u8]) -> anyhow::Result<Self> {
                            let id = WaylandValue::from_raw(buf)?;
                            Ok(#struct_name { object_id: id })
                        }

                        fn to_raw(self) -> Vec<u8> {
                            WaylandValue::to_raw(self.object_id)
                        }
                    }
                    #[async_trait::async_trait(?Send)]
                    impl<T: #trait_name> WaylandProtocol<T> for #struct_name {
                        async fn call(&self, state: &mut T, message: WaylandMessage, fds: &mut VecDeque<OwnedFd>) {
                            state.call(message, fds).await
                        }
                        fn name(&self) -> &'static str {
                            Self::NAME
                        }
                        fn version(&self) -> u32 {
                            Self::VERSION
                        }
                        fn object_id(&self) -> u32 {
                            self.object_id
                        }
                    }
                    pub trait #trait_name {
                        #(#trait_methods)*
                        async fn call(&mut self, message: WaylandMessage, fds: &mut VecDeque<OwnedFd>) {
                            match message.opcode {
                                #(#trait_accessor_i => self.#trait_accessor_name(message, fds).await,)*
                                _ => {},
                            }
                        }
                    }
                }
            });
            let name = quote::format_ident!("{}", ccase!(snake, protocol.name));
            quote! {
                pub mod #name {
                    use super::*;
                    use fixed::types::I24F8;
                    use pocowl_wlmessage::WaylandMessage;
                    use pocowl_wlvalue::WaylandValue;
                    use pocowl_protocols_base::WaylandProtocol;
                    use pocowl_wlclient::WaylandClient;
                    use std::collections::VecDeque;
                    use std::os::fd::OwnedFd;
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
    tt.into()
}
