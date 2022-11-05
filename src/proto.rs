use std::{
    fs::File,
    io::Read,
    path::Path
};
use heck::ToSnakeCase;
use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;
use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize)]
pub struct Protocol {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
    #[serde(rename = "interface", default)]
    pub interfaces: Vec<Interface>
}
impl Protocol {
    pub fn from_str(string: &str) -> Result<Self> {
        Ok(toml::from_str(string)?)
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut protocol = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut protocol)?;
        Ok(Self::from_str(&protocol)?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Interface {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: u32,
    #[serde(rename = "enum", default)]
    pub enums: Vec<Enum>,
    #[serde(rename = "request", default)]
    pub requests: Vec<Request>,
    #[serde(rename = "event", default)]
    pub events: Vec<Event>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Enum {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub since: Option<u32>,
    #[serde(rename = "entry", default)]
    pub entries: Vec<Entry>
}
#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub name: String,
    pub since: Option<u32>,
    #[serde(default)]
    pub destructor: bool,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "arg", default)]
    pub args: Vec<Arg>
}
#[derive(Clone, Debug, Deserialize)]
pub struct Event {
    pub name: String,
    pub since: Option<u32>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "arg", default)]
    pub args: Vec<Arg>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    pub name: String,
    pub since: Option<u32>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: u32
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    Destructor
}

#[derive(Clone, Debug, Deserialize)]
pub struct Arg {
    pub name: String,
    #[serde(rename = "allow-null", default)]
    pub nullable: bool,
    #[serde(rename = "type")]
    pub ty: DataType,
    pub interface: Option<String>,
    #[serde(rename = "enum")]
    pub enumeration: Option<String>,
    pub summary: Option<String>
}
impl Arg {
    pub fn getter(&self, stream: &Ident) -> TokenStream {
        match self.ty {
            DataType::Int => quote!{#stream.i32()?},
            DataType::Uint => quote!{#stream.u32()?},
            DataType::Fixed => quote!{#stream.fixed()?},
            DataType::String => if self.nullable {
                quote!{#stream.string()?}
            } else {
                quote!{#stream.string()?.ok_or(::yutani::wire::WlError::NON_NULLABLE)?}
            },
            DataType::Array => quote!{#stream.bytes()?},
            DataType::Fd => quote!{#stream.file()?},
            DataType::Object => if self.nullable {
                quote!{#stream.object()?}
            } else {
                quote!{#stream.object()?.ok_or(::yutani::wire::WlError::NON_NULLABLE)?}
            },
            DataType::NewId => if let Some(_) = self.interface.as_ref() {
                quote!{#stream.object()?.ok_or(::yutani::wire::WlError::NON_NULLABLE)?}
            } else {
                quote!{#stream.new_id()?}
            }
        }
    }
    pub fn sender(&self, stream: &Ident) -> TokenStream {
        let ident = Ident::new_raw(&self.name.to_snake_case(), Span::call_site());
        match self.ty {
            DataType::Int => quote!{#stream.send_i32(#ident)?},
            DataType::Uint => quote!{#stream.send_u32(#ident)?},
            DataType::Fixed => quote!{#stream.send_fixed(#ident)?},
            DataType::String => if self.nullable {
                quote!{#stream.send_string(#ident)?}
            } else {
                quote!{#stream.send_string(::core::option::Option::Some(#ident))?}
            },
            DataType::Array => quote!{#stream.send_bytes(#ident)?},
            DataType::Fd => quote!{#stream.send_file(#ident)?},
            DataType::Object => if self.nullable {
                quote!{#stream.send_object(#ident)?}
            } else {
                quote!{#stream.send_object(Some(#ident))?}
            },
            DataType::NewId => if let Some(_) = self.interface.as_ref() {
                quote!{#stream.send_object(Some(#ident))?}
            } else {
                quote!{#stream.send_new_id(#ident)?}
            }
        }
    }
    pub fn ty(&self) -> TokenStream {
        match self.ty {
            DataType::Int => quote!{::core::primitive::i32},
            DataType::Uint => quote!{::core::primitive::u32},
            DataType::Fixed => quote!{::yutani::Fixed},
            DataType::String => if self.nullable {
                quote!{::core::option::Option<::std::string::String>}
            } else {
                quote!{::std::string::String}
            },
            DataType::Array => quote!{::std::vec::Vec<u8>},
            DataType::Fd => quote!{::yutani::File},
            DataType::Object => if self.nullable {
                quote!{::core::option::Option<::yutani::Id>}
            } else {
                quote!{::yutani::Id}
            },
            DataType::NewId => if let Some(_) = self.interface.as_ref() {
                quote!{::yutani::Id}
            } else {
                quote!{::yutani::NewId}
            }
        }
    }
    pub fn send_ty(&self) -> TokenStream {
        match self.ty {
            DataType::Int => quote!{::core::primitive::i32},
            DataType::Uint => quote!{::core::primitive::u32},
            DataType::Fixed => quote!{::yutani::Fixed},
            DataType::String => if self.nullable {
                quote!{::core::option::Option<&'_ ::core::primitive::str>}
            } else {
                quote!{&'_ ::core::primitive::str}
            },
            DataType::Array => quote!{&'_ [::core::primitive::u8]},
            DataType::Fd => quote!{::yutani::Fd<'static>},
            DataType::Object => if self.nullable {
                quote!{::core::option::Option<::yutani::Id>}
            } else {
                quote!{::yutani::Id}
            },
            DataType::NewId => if let Some(_) = self.interface.as_ref() {
                quote!{::yutani::Id}
            } else {
                quote!{&'_ ::yutani::NewId}
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Int,
    Uint,
    Fixed,
    String,
    Array,
    Fd,
    Object,
    NewId
}