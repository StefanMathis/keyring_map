/*!
This module contains implementations of the [`Serialize`] and [`Deserialize`]
trait for [`KeyringMap`] (only available if the feature `serde` is enabled).

They are adapted from the `Vec<T>` implementation in
<https://github.com/serde-rs/serde/blob/master/serde/src/de/impls.rs> to enable
the simple serialized representation of [`KeyringMap`].
 */
use std::marker::PhantomData;

use super::*;
use serde::de::{Error, MapAccess, SeqAccess};
use serde::ser::SerializeSeq;
use serde::{
    de::{self, Deserialize, Deserializer, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};
use std::fmt;

struct ElementSer<'a, K, V> {
    keys: Vec<&'a K>,
    value: &'a V,
}

impl<'a, K, V> Serialize for ElementSer<'a, K, V>
where
    K: Serialize + Eq + Hash + Clone,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ElementSer", 2)?;
        state.serialize_field("keys", &self.keys)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

// Serialization
impl<K, V> Serialize for KeyringMap<K, V>
where
    K: Serialize + Eq + Hash + Clone,
    V: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.num_values()))?;
        for (key, value) in self.iter_unique() {
            // Collect all keys into the hashmap and serialize it afterwards
            let keys: Vec<&K> = self.aliases(&key).collect();
            let element = ElementSer { keys, value };
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

// =====================================================================================================================

struct ElementDe<K, V> {
    keys: Vec<K>,
    value: V,
}

impl<'de, K, V> Deserialize<'de> for ElementDe<K, V>
where
    K: Deserialize<'de> + Eq + Hash + Clone,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Keys,
            Value,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`keys` or `value`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "keys" => Ok(Field::Keys),
                            "value" => Ok(Field::Value),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ElementVisitor<K, V> {
            phantom_keys: PhantomData<K>,
            phantom_value: PhantomData<V>,
        }

        impl<'de, K, V> Visitor<'de> for ElementVisitor<K, V>
        where
            K: Deserialize<'de> + Eq + Hash + Clone,
            V: Deserialize<'de>,
        {
            type Value = ElementDe<K, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<ElementDe<K, V>, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let keys = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let value = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(ElementDe { keys, value })
            }

            fn visit_map<M>(self, mut map: M) -> Result<ElementDe<K, V>, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut keys = None;
                let mut value = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Keys => {
                            if keys.is_some() {
                                return Err(de::Error::duplicate_field("keys"));
                            }
                            keys = Some(map.next_value()?);
                        }
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value()?);
                        }
                    }
                }
                let keys = keys.ok_or_else(|| de::Error::missing_field("keys"))?;
                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                Ok(ElementDe { keys, value })
            }
        }

        const FIELDS: &[&str] = &["keys", "value"];
        deserializer.deserialize_struct(
            "ElementDe",
            FIELDS,
            ElementVisitor {
                phantom_keys: PhantomData,
                phantom_value: PhantomData,
            },
        )
    }
}

// Deserialization
impl<'de, K, V> Deserialize<'de> for KeyringMap<K, V>
where
    K: Deserialize<'de> + Eq + Hash + Clone,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyringMapVisitor<K, V> {
            marker_key: PhantomData<K>,
            marker_value: PhantomData<V>,
        }

        impl<'de, K, V> serde_impl::de::Visitor<'de> for KeyringMapVisitor<K, V>
        where
            K: Deserialize<'de> + Eq + Hash + Clone,
            V: Deserialize<'de>,
        {
            type Value = KeyringMap<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde_impl::de::SeqAccess<'de>,
            {
                let mut map: KeyringMap<K, V> = match seq.size_hint() {
                    Some(capacity) => KeyringMap::with_capacity(capacity, capacity),
                    None => KeyringMap::new(),
                };

                // Populate the KeyringMap from the sequence
                while let Some(deserializer) = seq.next_element::<ElementDe<K, V>>()? {
                    match map.insert_many(deserializer.keys, deserializer.value) {
                        Ok(_) => (),
                        Err(err) => {
                            // Create the error message
                            return Err(A::Error::custom(err.to_string()));
                        }
                    }
                }

                Ok(map)
            }
        }

        let visitor = KeyringMapVisitor {
            marker_key: PhantomData,
            marker_value: PhantomData,
        };
        deserializer.deserialize_seq(visitor)
    }
}
