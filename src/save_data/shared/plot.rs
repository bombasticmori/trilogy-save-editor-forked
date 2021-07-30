use anyhow::Result;
use bitvec::prelude::*;
use derive_more::{Deref, DerefMut};
use indexmap::IndexMap;
use serde::{de, Deserialize, Serialize};
use std::fmt;

pub type BoolSlice = BitSlice<Lsb0, u32>;

#[derive(Deref, DerefMut, Clone)]
pub struct BoolVec(BitVec<Lsb0, u32>);

impl<'de> serde::Deserialize<'de> for BoolVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoolVecVisitor;
        impl<'de> de::Visitor<'de> for BoolVecVisitor {
            type Value = BoolVec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a BoolVec")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut bitfields: Vec<u32> = Vec::new();
                while let Some(element) = seq.next_element()? {
                    bitfields.push(element);
                }

                let variables = BitVec::from_vec(bitfields);
                Ok(BoolVec(variables))
            }
        }
        deserializer.deserialize_seq(BoolVecVisitor)
    }
}

impl serde::Serialize for BoolVec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bitfields = self.0.clone().into_vec();
        serializer.collect_seq(bitfields)
    }
}

#[rc_ize_fields_derive(RawUi)]
#[derive(Deserialize, Serialize, Clone)]
pub struct Me1PlotTable {
    pub booleans: BoolVec,
    pub integers: Vec<i32>,
    pub floats: Vec<f32>,
}

#[rc_ize_fields_derive(RawUi)]
#[derive(Deserialize, Serialize, Clone)]
pub struct PlotTable {
    pub booleans: BoolVec,
    pub integers: Vec<i32>,
    pub floats: Vec<f32>,
    quest_progress_counter: i32,
    quest_progress: Vec<PlotQuest>,
    quest_ids: Vec<i32>,
    codex_entries: Vec<PlotCodex>,
    codex_ids: Vec<i32>,
}

#[rc_ize_fields_derive(RawUi)]
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct PlotQuest {
    quest_counter: i32,
    quest_updated: bool,
    history: Vec<i32>,
}

#[rc_ize_fields_derive(RawUi)]
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct PlotCodex {
    pages: IndexMap<i32, bool>,
}

#[derive(Deserialize)]
pub struct PlotCategory {
    pub booleans: IndexMap<usize, String>,
    pub integers: IndexMap<usize, String>,
}

#[derive(Deserialize, Clone)]
pub struct RawPlotDb {
    pub booleans: IndexMap<usize, String>,
    pub integers: IndexMap<usize, String>,
    pub floats: IndexMap<usize, String>,
}
