// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ops::AddAssign;

use chrono::Duration;
use chrono::NaiveDate;
use common_exception::Result;
use common_io::prelude::FormatSettings;
use lexical_core::ToLexical;
use num::cast::AsPrimitive;
use serde_json::Value;

use crate::serializations::write_csv_string;
use crate::serializations::write_escaped_string;
use crate::serializations::write_json_string;
use crate::serializations::TypeSerializer;
use crate::ColumnRef;
use crate::PrimitiveColumn;
use crate::PrimitiveType;
use crate::Series;

const DATE_FMT: &str = "%Y-%m-%d";

#[derive(Debug, Clone)]
pub struct DateSerializer<'a, T: PrimitiveType + AsPrimitive<i64> + ToLexical> {
    pub(crate) values: &'a [T],
}

fn v_to_string(v: &i64) -> String {
    let mut date = NaiveDate::from_ymd(1970, 1, 1);
    let d = Duration::days(*v);
    date.add_assign(d);
    date.format(DATE_FMT).to_string()
}

impl<'a, T: PrimitiveType + AsPrimitive<i64> + ToLexical> DateSerializer<'a, T> {
    pub fn try_create(col: &'a ColumnRef) -> Result<Self> {
        let col: &PrimitiveColumn<T> = Series::check_get(col)?;
        Ok(Self {
            values: col.values(),
        })
    }

    pub fn fmt(&self, row_index: usize) -> String {
        v_to_string(&self.values[row_index].as_i64())
    }
}

impl<'a, T: PrimitiveType + AsPrimitive<i64> + ToLexical> TypeSerializer<'a>
    for DateSerializer<'a, T>
{
    fn write_field_values(
        &self,
        row_index: usize,
        buf: &mut Vec<u8>,
        format: &FormatSettings,
        in_nested: bool,
    ) {
        let s = self.fmt(row_index);
        if in_nested {
            buf.push(format.nested.quote_char);
        }
        write_escaped_string(s.as_bytes(), buf, format.nested.quote_char);
        if in_nested {
            buf.push(format.nested.quote_char);
        }
    }

    fn write_field_tsv(
        &self,
        row_index: usize,
        buf: &mut Vec<u8>,
        format: &FormatSettings,
        in_nested: bool,
    ) {
        if in_nested {
            buf.push(format.quote_char);
        }
        let s = self.fmt(row_index);
        write_escaped_string(s.as_bytes(), buf, format.quote_char);
        if in_nested {
            buf.push(format.quote_char);
        }
    }

    fn write_field_csv(&self, row_index: usize, buf: &mut Vec<u8>, format: &FormatSettings) {
        let s = self.fmt(row_index);
        write_csv_string(s.as_bytes(), buf, format.quote_char);
    }

    fn write_field_json(
        &self,
        row_index: usize,
        buf: &mut Vec<u8>,
        format: &FormatSettings,
        quote: bool,
    ) {
        let s = self.fmt(row_index);
        if quote {
            buf.push(b'\"');
        }
        write_json_string(s.as_bytes(), buf, format);
        if quote {
            buf.push(b'\"');
        }
    }

    fn serialize_json_values(&self, _format: &FormatSettings) -> Result<Vec<Value>> {
        let result: Vec<Value> = (0..self.values.len())
            .map(|row_index| {
                let s = v_to_string(&self.values[row_index].as_i64());
                serde_json::to_value(s).unwrap()
            })
            .collect();
        Ok(result)
    }
}
