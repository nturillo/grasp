// Boilerplate provided by serde

use std::collections::BTreeMap;
use std::ops::Add;

use crate::serialization::error::{SerializationError, Result};
use serde::{
    Deserialize, Deserializer, Serialize, de::{DeserializeOwned, EnumAccess, IntoDeserializer, VariantAccess, value::{MapDeserializer, SeqDeserializer}}, ser
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Value {
    Null,
    Object(BTreeMap<String, Value>),
    Array(Vec<Value>),
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
    Unsigned(u64),
}

impl Value {
    pub(crate) fn is_primative(&self) -> bool {
        match self {
            Value::Array(_) | Value::Object(_) => false,
            _ => true,
        }
    }
}

impl<'a> IntoDeserializer<'a, SerializationError> for Value {
    type Deserializer = Value;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'a> VariantAccess<'a> for Value {
    type Error = SerializationError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) ->Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'a> {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a> {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a> {
        self.deserialize_map(visitor)
    }
}

impl<'a> EnumAccess<'a> for Value {
    type Error = SerializationError;
    type Variant = Value;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'a> {
        match self {
            Value::Object(map) => {
                let (key, value) = map.into_iter().next().ok_or(SerializationError::Message("Invalid enum".to_string()))?;
                Ok((seed.deserialize(Value::String(key).into_deserializer())?, value))
            },
            Value::String(s) => Ok((seed.deserialize(Value::String(s).into_deserializer())?, Value::Null)),
            _ => Err(SerializationError::Message("Invalid enum".to_string()))
        }
    }
}

impl<'a> serde::Deserializer<'a> for Value {
    type Error = SerializationError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Object(map) => visitor.visit_map(MapDeserializer::new(map.into_iter())),
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a.into_iter())),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::String(s) => visitor.visit_string(s),
            Value::Int(i) => visitor.visit_i64(i),
            Value::Float(f) => visitor.visit_f64(f),
            Value::Unsigned(u) => {if let Ok(i) = i64::try_from(u) {
                    visitor.visit_i64(i)
                } else {
                    visitor.visit_u64(u)
                }},
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Bool(b) => visitor.visit_bool(b),
            _ => Err(SerializationError::Message("Not bool type".to_string())),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Int(i) => visitor.visit_i8(
                i.try_into()
                    .map_err(|_| SerializationError::Message("Int too large".to_string()))?,
            ),
            Value::Unsigned(i) => visitor.visit_i8(
                i.try_into()
                    .map_err(|_| SerializationError::Message("Int too large".to_string()))?,
            ),
            _ => Err(SerializationError::Message("Not int type".to_string())),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Int(i) => visitor.visit_i16(
                i.try_into()
                    .map_err(|_| SerializationError::Message("Int too large".to_string()))?,
            ),
            Value::Unsigned(i) => visitor.visit_i16(
                i.try_into()
                    .map_err(|_| SerializationError::Message("Int too large".to_string()))?,
            ),
            _ => Err(SerializationError::Message("Not int type".to_string())),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Int(i) => visitor.visit_i32(
                i.try_into()
                    .map_err(|_| SerializationError::Message("Int too large".to_string()))?,
            ),
            Value::Unsigned(i) => visitor.visit_i32(
                i.try_into()
                    .map_err(|_| SerializationError::Message("Int too large".to_string()))?,
            ),
            _ => Err(SerializationError::Message("Not int type".to_string())),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Int(i) => visitor.visit_i64(i),
            Value::Unsigned(i) => visitor.visit_i64(i as i64),
            _ => Err(SerializationError::Message("Not int type".to_string())),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Unsigned(u) => visitor.visit_u8(
                u.try_into()
                    .map_err(|_| SerializationError::Message("Unsigned int too large".to_string()))?,
            ),
            _ => Err(SerializationError::Message("Not unsigned int type".to_string())),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Unsigned(u) => visitor.visit_u16(
                u.try_into()
                    .map_err(|_| SerializationError::Message("Unsigned int too large".to_string()))?,
            ),
            _ => Err(SerializationError::Message("Not unsigned int type".to_string())),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Unsigned(u) => visitor.visit_u32(
                u.try_into()
                    .map_err(|_| SerializationError::Message("Unsigned int too large".to_string()))?,
            ),
            _ => Err(SerializationError::Message("Not unsigned int type".to_string())),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Unsigned(u) => visitor.visit_u64(u),
            _ => Err(SerializationError::Message("Not unsigned int type".to_string())),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Float(f) => visitor.visit_f32(f as f32),
            _ => Err(SerializationError::Message("Not float type".to_string())),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Float(f) => visitor.visit_f64(f),
            _ => Err(SerializationError::Message("Not float type".to_string())),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::String(s) => visitor.visit_char(s.parse::<char>().map_err(|_| {
                SerializationError::Message("Cannot convert multi-char string into char".to_string())
            })?),
            _ => Err(SerializationError::Message("Not char type".to_string())),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::String(s) => visitor.visit_str(s.as_str()),
            _ => Err(SerializationError::Message("Not str type".to_string())),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::String(s) => visitor.visit_string(s),
            _ => Err(SerializationError::Message("Not string type".to_string())),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Array(a) => visitor.visit_bytes(
                a.iter()
                    .map(|b| from_value::<u8>(b.clone()))
                    .collect::<Result<Vec<u8>>>()?
                    .as_slice(),
            ),
            _ => Err(SerializationError::Message("Not byte list type".to_string())),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Array(a) => visitor.visit_bytes(
                a.iter()
                    .map(|b| from_value::<u8>(b.clone()))
                    .collect::<Result<Vec<u8>>>()?
                    .as_slice(),
            ),
            _ => Err(SerializationError::Message("Not byte list type".to_string())),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            _ => Err(SerializationError::Message("Not unit/null type".to_string())),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a.into_iter())),
            _ => Err(SerializationError::Message("Not list type".to_string())),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Object(map) => visitor.visit_map(MapDeserializer::new(map.into_iter())),
            _ => Err(SerializationError::Message("Not map type".to_string())),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        match self {
            Value::Int(i) => visitor.visit_i64(i),
            _ => self.deserialize_string(visitor),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'a>,
    {
        visitor.visit_unit()
    }
}

pub(crate) fn from_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    T::deserialize(value)
}
struct Serializer;

impl<'a> ser::Serializer for Serializer {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::Int(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(Value::Unsigned(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Seq {
            sequence: Default::default(),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(NamedSeq {
            sequence: Default::default(),
            name: variant.to_string(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Map {
            map: Default::default(),
            key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructObj {
            map: Default::default(),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(NamedStruct {
            map: Default::default(),
            name: variant.to_string(),
        })
    }

    fn serialize_i128(self, v: i128) -> Result<Value> {
        let _ = v;
        Err(ser::Error::custom("i128 is not supported"))
    }

    fn serialize_u128(self, v: u128) -> Result<Value> {
        let _ = v;
        Err(ser::Error::custom("u128 is not supported"))
    }

    type SerializeSeq = Seq;
    type SerializeTuple = Seq;
    type SerializeTupleStruct = Seq;
    type SerializeTupleVariant = NamedSeq;
    type SerializeMap = Map;
    type SerializeStruct = StructObj;
    type SerializeStructVariant = NamedStruct;

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        let mut map = BTreeMap::new();
        map.insert(variant.to_string(), value.serialize(Serializer)?);
        Ok(Value::Object(map))
    }
}

pub(crate) struct Seq {
    sequence: Vec<Value>,
}

impl<'a> ser::SerializeSeq for Seq {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let s_val = value.serialize(Serializer)?;
        self.sequence.push(s_val);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.sequence))
    }
}

impl<'a> ser::SerializeTuple for Seq {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let s_val = value.serialize(Serializer)?;
        self.sequence.push(s_val);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.sequence))
    }
}

impl<'a> ser::SerializeTupleStruct for Seq {
    type Ok = Value;
    type Error = SerializationError;

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.sequence))
    }

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let s_val = value.serialize(Serializer)?;
        self.sequence.push(s_val);
        Ok(())
    }
}

pub(crate) struct NamedSeq {
    sequence: Vec<Value>,
    name: String,
}

impl<'a> ser::SerializeTupleVariant for NamedSeq {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let s_val = value.serialize(Serializer)?;
        self.sequence.push(s_val);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut map = BTreeMap::new();
        map.insert(self.name, Value::Array(self.sequence));
        Ok(Value::Object(map))
    }
}

pub(crate) struct Map {
    map: BTreeMap<String, Value>,
    key: Option<String>,
}

impl<'a> ser::SerializeMap for Map {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if let Value::String(s) = key.serialize(Serializer)? {
            self.key = Some(s);
            Ok(())
        } else {
            Err(SerializationError::Message("Expected string type for keys".to_string()))
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let val = value.serialize(Serializer)?;
        self.map.insert(
            self.key
                .as_deref()
                .expect("Unexpected error: Setting value before key")
                .to_string(),
            val,
        );
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}

pub(crate) struct StructObj {
    map: BTreeMap<String, Value>,
}

impl<'a> ser::SerializeStruct for StructObj {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map
            .insert(key.to_string(), value.serialize(Serializer)?);

        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}

pub(crate) struct NamedStruct {
    map: BTreeMap<String, Value>,
    name: String,
}

impl<'a> ser::SerializeStructVariant for NamedStruct {
    type Ok = Value;
    type Error = SerializationError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map
            .insert(key.to_string(), value.serialize(Serializer)?);

        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut nmap = BTreeMap::new();
        nmap.insert(self.name, Value::Object(self.map));
        Ok(Value::Object(nmap))
    }
}

pub(crate) fn to_value<T: Serialize>(data: T) -> Result<Value> {
    data.serialize(Serializer)
}

#[cfg(feature = "serde")]
pub fn serialize<V: Serialize>(data: &V) -> Value {
    to_value(data).unwrap_or(Value::Null)
}

#[cfg(feature = "serde")]
pub fn wrap_value(val: Value) -> BTreeMap<String, Value> {
    match val {
        Value::Object(map) => map,
        Value::Array(vec) => {
            let mut map = BTreeMap::new();
            map.insert("data".to_string(), Value::Array(vec));
            return map;
        }
        Value::Null => BTreeMap::new(),
        Value::Bool(_) => {
            let mut map = BTreeMap::new();
            map.insert("data".to_string(), val);
            return map;
        }
        Value::Int(_) | Value::Float(_) | Value::Unsigned(_) => {
            let mut map = BTreeMap::new();
            map.insert("data".to_string(), val);
            return map;
        }
        Value::String(_) => {
            let mut map = BTreeMap::new();
            map.insert("label".to_string(), val);
            return map;
        }
    }
}

#[cfg(feature = "serde")]
pub fn get_flat_map(key: String, val: Value, map: &mut BTreeMap<String, (String, String)>) {
    match val {
        Value::Object(obj) => {
            let prefix = if key.is_empty() {String::new()} else {key.add(".")};

            for (name, data) in obj {
                get_flat_map(prefix.clone().add(name.as_str()), data, map);
            }
        }
        Value::Array(arr) => {
            let prefix = if key.is_empty() {String::new()} else {key.add(".")};

            for (index, data) in arr.iter().enumerate() {
                get_flat_map(prefix.clone().add(index.to_string().as_str()), data.clone(), map);
            }
        }
        Value::Unsigned(n) => {
            map.insert(key, (n.to_string(), "long".to_string()));
        }
        Value::Int(n) => {
            map.insert(key, (n.to_string(), "long".to_string()));
        }
        Value::Float(n) => {
            map.insert(key, (n.to_string(), "double".to_string()));
        }
        Value::Bool(b) => {
            map.insert(key, (b.to_string(), "boolean".to_string()));
        }
        Value::String(s) => {
            map.insert(key, (s, "string".to_string()));
        }
        Value::Null => {
            map.insert(key, (String::new(), String::new()));
        }
    }
}