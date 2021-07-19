use crate::errors::{AlreadyDefined, TypeError};
use std::convert::TryFrom;
use std::fmt;

use crate::context::FunctionAttributes;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use strum::IntoStaticStr;
use vec1::Vec1;

pub fn u256_min() -> BigInt {
    BigInt::from(0)
}

pub fn u256_max() -> BigInt {
    BigInt::from(2).pow(256) - 1
}

pub fn i256_max() -> BigInt {
    BigInt::from(2).pow(255) - 1
}

pub fn i256_min() -> BigInt {
    BigInt::from(-2).pow(255)
}

/// Names that can be used to build identifiers without collision.
pub trait SafeNames {
    /// Name in the lower snake format (e.g. lower_snake_case).
    fn lower_snake(&self) -> String;
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Type {
    Base(Base),
    Array(Array),
    Map(Map),
    Tuple(Tuple),
    String(FeString),
    Contract(Contract),
    Struct(Struct),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum FixedSize {
    Base(Base),
    Array(Array),
    Tuple(Tuple),
    String(FeString),
    Contract(Contract),
    Struct(Struct),
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Base {
    Numeric(Integer),
    Bool,
    Address,
    Unit,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, IntoStaticStr)]
pub enum Integer {
    U256,
    U128,
    U64,
    U32,
    U16,
    U8,
    I256,
    I128,
    I64,
    I32,
    I16,
    I8,
}

pub const U256: Base = Base::Numeric(Integer::U256);

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Array {
    pub size: usize,
    pub inner: Base,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Map {
    pub key: Base,
    pub value: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Tuple {
    pub items: Vec1<FixedSize>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, FixedSize)>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FeString {
    pub max_size: usize,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Contract {
    pub name: String,
    pub functions: Vec<FunctionAttributes>,
}

impl Struct {
    pub fn new(name: &str) -> Struct {
        Struct {
            name: name.to_string(),
            fields: vec![],
        }
    }

    /// Return `true` if the struct does not have any fields, otherwise return `false`
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Add a field to the struct
    pub fn add_field(&mut self, name: &str, value: &FixedSize) -> Result<(), AlreadyDefined> {
        if self.fields.iter().any(|(fname, _)| fname == name) {
            Err(AlreadyDefined)
        } else {
            self.fields.push((name.to_string(), value.clone()));
            Ok(())
        }
    }

    /// Return the type of the given field name
    pub fn get_field_type(&self, name: &str) -> Option<&FixedSize> {
        self.fields
            .iter()
            .find(|(nm, _)| nm == name)
            .map(|(_, typ)| typ)
    }

    /// Return the types of all fields
    pub fn get_field_types(&self) -> Vec<FixedSize> {
        self.fields.iter().cloned().map(|(_, typ)| typ).collect()
    }

    /// Return the index of the given field name
    pub fn get_field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|(field, _)| field == name)
    }
}

impl Integer {
    /// Returns `true` if the integer is signed, otherwise `false`
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Integer::I256
                | Integer::I128
                | Integer::I64
                | Integer::I32
                | Integer::I16
                | Integer::I8
        )
    }

    pub fn size(&self) -> usize {
        match self {
            Integer::U256 => 32,
            Integer::U128 => 16,
            Integer::U64 => 8,
            Integer::U32 => 4,
            Integer::U16 => 2,
            Integer::U8 => 1,
            Integer::I256 => 32,
            Integer::I128 => 16,
            Integer::I64 => 8,
            Integer::I32 => 4,
            Integer::I16 => 2,
            Integer::I8 => 1,
        }
    }

    /// Returns `true` if the integer is at least the same size (or larger) than
    /// `other`
    pub fn can_hold(&self, other: &Integer) -> bool {
        self.size() >= other.size()
    }

    /// Returns `true` if `num` represents a number that fits the type
    pub fn fits(&self, num: BigInt) -> bool {
        match self {
            Integer::U8 => num.to_u8().is_some(),
            Integer::U16 => num.to_u16().is_some(),
            Integer::U32 => num.to_u32().is_some(),
            Integer::U64 => num.to_u64().is_some(),
            Integer::U128 => num.to_u128().is_some(),
            Integer::I8 => num.to_i8().is_some(),
            Integer::I16 => num.to_i16().is_some(),
            Integer::I32 => num.to_i32().is_some(),
            Integer::I64 => num.to_i64().is_some(),
            Integer::I128 => num.to_i128().is_some(),
            Integer::U256 => num >= u256_min() && num <= u256_max(),
            Integer::I256 => num >= i256_min() && num <= i256_max(),
        }
    }
}

impl Type {
    pub fn is_signed_integer(&self) -> bool {
        if let Type::Base(Base::Numeric(integer)) = &self {
            return integer.is_signed();
        }
        false
    }

    pub fn unit() -> Self {
        Type::Base(Base::Unit)
    }

    pub fn int(int_type: Integer) -> Self {
        Type::Base(Base::Numeric(int_type))
    }
}

pub trait TypeDowncast {
    fn as_array(&self) -> Option<&Array>;
    fn as_tuple(&self) -> Option<&Tuple>;
    fn as_string(&self) -> Option<&FeString>;
    fn as_map(&self) -> Option<&Map>;
    fn as_int(&self) -> Option<Integer>;
}

impl TypeDowncast for Type {
    fn as_array(&self) -> Option<&Array> {
        match self {
            Type::Array(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_tuple(&self) -> Option<&Tuple> {
        match self {
            Type::Tuple(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_string(&self) -> Option<&FeString> {
        match self {
            Type::String(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_map(&self) -> Option<&Map> {
        match self {
            Type::Map(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_int(&self) -> Option<Integer> {
        match self {
            Type::Base(Base::Numeric(int)) => Some(*int),
            _ => None,
        }
    }
}

impl TypeDowncast for Option<&Type> {
    fn as_array(&self) -> Option<&Array> {
        self.and_then(|t| t.as_array())
    }
    fn as_tuple(&self) -> Option<&Tuple> {
        self.and_then(|t| t.as_tuple())
    }
    fn as_string(&self) -> Option<&FeString> {
        self.and_then(|t| t.as_string())
    }
    fn as_map(&self) -> Option<&Map> {
        self.and_then(|t| t.as_map())
    }
    fn as_int(&self) -> Option<Integer> {
        self.and_then(|t| t.as_int())
    }
}

impl From<FixedSize> for Type {
    fn from(value: FixedSize) -> Self {
        match value {
            FixedSize::Array(array) => Type::Array(array),
            FixedSize::Base(base) => Type::Base(base),
            FixedSize::Tuple(tuple) => Type::Tuple(tuple),
            FixedSize::String(string) => Type::String(string),
            FixedSize::Contract(contract) => Type::Contract(contract),
            FixedSize::Struct(val) => Type::Struct(val),
        }
    }
}

impl From<Base> for Type {
    fn from(value: Base) -> Self {
        Type::Base(value)
    }
}

impl FixedSize {
    /// Returns true if the type is `()`.
    pub fn is_unit(&self) -> bool {
        self == &Self::Base(Base::Unit)
    }

    /// Creates an instance of bool.
    pub fn bool() -> Self {
        FixedSize::Base(Base::Bool)
    }

    /// Creates an instance of address.
    pub fn address() -> Self {
        FixedSize::Base(Base::Address)
    }

    /// Creates an instance of u256.
    pub fn u256() -> Self {
        FixedSize::Base(Base::Numeric(Integer::U256))
    }

    /// Creates an instance of u8.
    pub fn u8() -> Self {
        FixedSize::Base(Base::Numeric(Integer::U8))
    }

    /// Creates an instance of `()`.
    pub fn unit() -> Self {
        FixedSize::Base(Base::Unit)
    }
}

impl PartialEq<Type> for FixedSize {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (FixedSize::Array(in1), Type::Array(in2)) => in1 == in2,
            (FixedSize::Base(in1), Type::Base(in2)) => in1 == in2,
            (FixedSize::Tuple(in1), Type::Tuple(in2)) => in1 == in2,
            (FixedSize::String(in1), Type::String(in2)) => in1 == in2,
            (FixedSize::Contract(in1), Type::Contract(in2)) => in1 == in2,
            (FixedSize::Struct(in1), Type::Struct(in2)) => in1 == in2,
            _ => false,
        }
    }
}

impl From<Base> for FixedSize {
    fn from(value: Base) -> Self {
        FixedSize::Base(value)
    }
}

impl From<FeString> for FixedSize {
    fn from(value: FeString) -> Self {
        FixedSize::String(value)
    }
}

impl TryFrom<Type> for FixedSize {
    type Error = TypeError;

    fn try_from(value: Type) -> Result<Self, TypeError> {
        match value {
            Type::Array(array) => Ok(FixedSize::Array(array)),
            Type::Base(base) => Ok(FixedSize::Base(base)),
            Type::Tuple(tuple) => Ok(FixedSize::Tuple(tuple)),
            Type::String(string) => Ok(FixedSize::String(string)),
            Type::Struct(val) => Ok(FixedSize::Struct(val)),
            Type::Map(_) => Err(TypeError),
            Type::Contract(contract) => Ok(FixedSize::Contract(contract)),
        }
    }
}

impl From<Tuple> for FixedSize {
    fn from(value: Tuple) -> Self {
        FixedSize::Tuple(value)
    }
}

impl SafeNames for FixedSize {
    fn lower_snake(&self) -> String {
        match self {
            FixedSize::Array(array) => array.lower_snake(),
            FixedSize::Base(base) => base.lower_snake(),
            FixedSize::Tuple(tuple) => tuple.lower_snake(),
            FixedSize::String(string) => string.lower_snake(),
            FixedSize::Contract(contract) => contract.lower_snake(),
            FixedSize::Struct(val) => val.lower_snake(),
        }
    }
}

impl SafeNames for Base {
    fn lower_snake(&self) -> String {
        match self {
            Base::Numeric(Integer::U256) => "u256".to_string(),
            Base::Numeric(Integer::U128) => "u128".to_string(),
            Base::Numeric(Integer::U64) => "u64".to_string(),
            Base::Numeric(Integer::U32) => "u32".to_string(),
            Base::Numeric(Integer::U16) => "u16".to_string(),
            Base::Numeric(Integer::U8) => "u8".to_string(),
            Base::Numeric(Integer::I256) => "i256".to_string(),
            Base::Numeric(Integer::I128) => "i128".to_string(),
            Base::Numeric(Integer::I64) => "i64".to_string(),
            Base::Numeric(Integer::I32) => "i32".to_string(),
            Base::Numeric(Integer::I16) => "i16".to_string(),
            Base::Numeric(Integer::I8) => "i8".to_string(),
            Base::Address => "address".to_string(),
            Base::Bool => "bool".to_string(),
            Base::Unit => "unit".to_string(),
        }
    }
}

impl SafeNames for Array {
    fn lower_snake(&self) -> String {
        format!("array_{}_{}", self.inner.lower_snake(), self.size)
    }
}

impl SafeNames for Struct {
    fn lower_snake(&self) -> String {
        format!("struct_{}", self.name)
    }
}

impl SafeNames for Tuple {
    fn lower_snake(&self) -> String {
        let field_names = self
            .items
            .iter()
            .map(|typ| typ.lower_snake())
            .collect::<Vec<String>>();
        let joined_names = field_names.join("_");

        // The trailing `_` denotes the end of the tuple, to differentiate between
        // different tuple nestings. Eg
        // (A, (B, C), D) => tuple_A_tuple_B_C__D_
        // (A, (B, C, D)) => tuple_A_tuple_B_C_D__
        // Conceptually, each paren and comma is replaced with an underscore.
        format!("tuple_{}_", joined_names)
    }
}

impl SafeNames for Contract {
    fn lower_snake(&self) -> String {
        unimplemented!();
    }
}

impl SafeNames for FeString {
    fn lower_snake(&self) -> String {
        format!("string_{}", self.max_size)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Base(inner) => inner.fmt(f),
            Type::Array(inner) => inner.fmt(f),
            Type::Map(inner) => inner.fmt(f),
            Type::Tuple(inner) => inner.fmt(f),
            Type::String(inner) => inner.fmt(f),
            Type::Contract(inner) => inner.fmt(f),
            Type::Struct(inner) => inner.fmt(f),
        }
    }
}

impl fmt::Display for FixedSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedSize::Base(inner) => inner.fmt(f),
            FixedSize::Array(inner) => inner.fmt(f),
            FixedSize::Tuple(inner) => inner.fmt(f),
            FixedSize::String(inner) => inner.fmt(f),
            FixedSize::Contract(inner) => inner.fmt(f),
            FixedSize::Struct(inner) => inner.fmt(f),
        }
    }
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Base::Numeric(int) => return int.fmt(f),
            Base::Bool => "bool",
            Base::Address => "address",
            Base::Unit => "()",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Integer::U256 => "u256",
            Integer::U128 => "u128",
            Integer::U64 => "u64",
            Integer::U32 => "u32",
            Integer::U16 => "u16",
            Integer::U8 => "u8",
            Integer::I256 => "i256",
            Integer::I128 => "i128",
            Integer::I64 => "i64",
            Integer::I32 => "i32",
            Integer::I16 => "i16",
            Integer::I8 => "i8",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.inner, self.size)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map<{}, {}>", self.key, self.value)
    }
}

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut delim = "";
        for item in &self.items {
            write!(f, "{}{}", delim, item)?;
            delim = ", ";
        }
        write!(f, ")")
    }
}

impl fmt::Display for FeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String<{}>", self.max_size)
    }
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
