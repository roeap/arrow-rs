// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::sync::Arc;

use arrow_array::{RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use once_cell::sync::Lazy;

use crate::error::Result;

mod db_schemas;

/// Returns the list of catalogs in the DataFusion catalog
pub fn get_catalogs_batch(mut catalog_names: Vec<String>) -> Result<RecordBatch> {
    catalog_names.sort_unstable();

    let batch = RecordBatch::try_new(
        Arc::clone(&GET_CATALOG_SCHEMA),
        vec![Arc::new(StringArray::from_iter_values(catalog_names)) as _],
    )?;

    Ok(batch)
}

/// Returns the schema that will result from [`get_catalogs`]
pub fn get_catalogs_schema() -> &'static Schema {
    &GET_CATALOG_SCHEMA
}

/// The schema for GetCatalogs
static GET_CATALOG_SCHEMA: Lazy<SchemaRef> = Lazy::new(|| {
    Arc::new(Schema::new(vec![Field::new(
        "catalog_name",
        DataType::Utf8,
        false,
    )]))
});
