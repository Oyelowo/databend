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

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use common_datablocks::DataBlock;
use common_datavalues::prelude::*;
use common_exception::Result;
use common_meta_types::{MetaId, TableIdent};
use common_meta_types::TableInfo;
use common_meta_types::TableMeta;
use common_planners::{Expression, Extras, TruncateTablePlan};
use common_planners::Part;
use common_planners::Partitions;
use common_planners::ReadDataSourcePlan;
use common_planners::Statistics;
use common_streams::DataBlockStream;
use common_streams::SendableDataBlockStream;
use crate::pipelines::new::{NewPipe, NewPipeline};
use crate::pipelines::new::processors::processor::ProcessorPtr;
use crate::pipelines::new::processors::{SyncSource, SyncSourcer};
use crate::pipelines::new::processors::port::{InputPort, OutputPort};

use crate::sessions::QueryContext;
use crate::storages::Table;

pub struct OneTable {
    table_info: TableInfo,
}

impl OneTable {
    pub fn create(table_id: u64) -> Self {
        let schema =
            DataSchemaRefExt::create(vec![DataField::new("dummy", DataType::UInt8, false)]);

        let table_info = TableInfo {
            desc: "'system'.'one'".to_string(),
            name: "one".to_string(),
            ident: TableIdent::new(table_id, 0),
            meta: TableMeta {
                schema,
                engine: "SystemOne".to_string(),
                ..Default::default()
            },
        };
        OneTable { table_info }
    }
}

#[async_trait::async_trait]
impl Table for OneTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_table_info(&self) -> &TableInfo {
        &self.table_info
    }

    async fn read_partitions(
        &self,
        _ctx: Arc<QueryContext>,
        _push_downs: Option<Extras>,
    ) -> Result<(Statistics, Partitions)> {
        Ok((Statistics::new_exact(1, 1, 1, 1), vec![Part {
            name: "".to_string(),
            version: 0,
        }]))
    }

    async fn read(
        &self,
        _ctx: Arc<QueryContext>,
        _plan: &ReadDataSourcePlan,
    ) -> Result<SendableDataBlockStream> {
        let block =
            DataBlock::create_by_array(self.table_info.schema(), vec![Series::new(vec![1u8])]);
        Ok(Box::pin(DataBlockStream::create(
            self.table_info.schema(),
            None,
            vec![block],
        )))
    }

    fn read2(&self, _: Arc<QueryContext>, _: &ReadDataSourcePlan, pipeline: &mut NewPipeline) -> Result<()> {
        let schema = self.table_info.schema();
        let output = OutputPort::create();
        pipeline.add_pipe(NewPipe::SimplePipe {
            processors: vec![OneSource::create(output.clone(), schema)?],
            inputs_port: vec![],
            outputs_port: vec![output],
        });

        Ok(())
    }
}

struct OneSource(Option<DataBlock>);

impl OneSource {
    pub fn create(output: Arc<OutputPort>, schema: DataSchemaRef) -> Result<ProcessorPtr> {
        let data_block = DataBlock::create_by_array(schema.clone(), vec![Series::new(vec![1u8])]);
        SyncSourcer::create(output, OneSource(Some(data_block)))
    }
}

impl SyncSource for OneSource {
    const NAME: &'static str = "OneSource";

    fn generate(&mut self) -> Result<Option<DataBlock>> {
        Ok(self.0.take())
    }
}

