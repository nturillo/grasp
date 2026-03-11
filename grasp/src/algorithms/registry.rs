use crate::graph::{EdgeID, VertexID, prelude::{SparseDiGraph}};

#[linkme::distributed_slice]
pub static ALGORITHMS: [FunctionData];

macro_rules! arg_types {
    ($($arg:ident($arg_type:ty)),*$(,)?) => {
        pub enum ArgType {
            $($arg($arg_type)),*
        }

        pub const ARG_LIST: &[&str] = &[
            $(stringify!($arg)),*
        ];
    }
}

macro_rules! safeish_supported_casts {
    ($($data_type:ty [$($arg_type:ident),*]),*) => {
        $(impl FromArgType for $data_type {
            fn from_arg(arg: &ArgType) -> Option<Self> {
                match arg {
                    $(ArgType::$arg_type(num) => (*num).try_into().ok()),*,
                    _ => None,
                }
            }
        })*
    };
}

arg_types! {
    Integer(i64),
    Float(f64),
    UnsignedInteger(u64),
    Vertex(VertexID),
    Edge(EdgeID),
    String(String),
    Boolean(bool),
    VertexList(Vec<VertexID>),
    EdgeList(Vec<EdgeID>),
}

pub trait FromArgType: Sized {
    fn from_arg(arg: &ArgType) -> Option<Self>;
}

impl FromArgType for String {
    fn from_arg(arg: &ArgType) -> Option<Self> {
        match arg {
            ArgType::String(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

impl FromArgType for bool {
    fn from_arg(arg: &ArgType) -> Option<Self> {
        match arg {
            ArgType::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl FromArgType for EdgeID {
    fn from_arg(arg: &ArgType) -> Option<Self> {
        match arg {
            ArgType::Edge(e) => Some(e.clone()),
            _ => None,
        }
    }
}

impl FromArgType for Vec<VertexID> {
    fn from_arg(arg: &ArgType) -> Option<Self> {
        match arg {
            ArgType::VertexList(b) => Some(b.clone()),
            _ => None,
        }
    }
}

impl FromArgType for Vec<EdgeID> {
    fn from_arg(arg: &ArgType) -> Option<Self> {
        match arg {
            ArgType::EdgeList(b) => Some(b.clone()),
            _ => None,
        }
    }
}

safeish_supported_casts!(
    i8 [Integer, UnsignedInteger, Vertex],
    i16 [Integer, UnsignedInteger, Vertex],
    i32 [Integer, UnsignedInteger, Vertex],
    i64 [Integer, UnsignedInteger, Vertex],
    u8 [Integer, UnsignedInteger, Vertex],
    u16 [Integer, UnsignedInteger, Vertex],
    u32 [Integer, UnsignedInteger, Vertex],
    u64 [Integer, UnsignedInteger, Vertex],
    usize [Integer, UnsignedInteger, Vertex]);

pub enum ReturnType {
    None(()),
    String(String),
    Vertex(VertexID),
    Edge(EdgeID),
    VertexList(Vec<VertexID>),
    EdgeList(Vec<EdgeID>),
}

pub struct FunctionData {
    pub name: &'static str,
    pub module: &'static str,
    pub func: fn(&SparseDiGraph, &[ArgType]) -> ReturnType,
    pub return_type: &'static str,
    pub param_data: &'static [ &'static [ &'static str; 2 ]],
    pub desc: &'static str,
}