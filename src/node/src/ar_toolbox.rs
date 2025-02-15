use arrow::array::{
    ArrayRef, BinaryArray, BinaryBuilder, StringArray, StringBuilder, UInt32Array, UInt32Builder,
    UInt64Array, UInt64Builder,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use db3_error::{DB3Error, Result};
use db3_proto::db3_mutation_v2_proto::{MutationBody, MutationHeader};
use db3_storage::ar_fs::{ArFileSystem, ArFileSystemConfig};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, GzipLevel};
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tempdir::TempDir;
use tracing::info;
pub struct ArToolBox {
    pub network_id: Arc<AtomicU64>,
    pub schema: SchemaRef,
    pub ar_filesystem: ArFileSystem,
    pub temp_data_path: String,
}

impl ArToolBox {
    pub fn new(
        key_root_path: String,
        arweave_url: String,
        temp_data_path: String,
        network_id: Arc<AtomicU64>,
    ) -> Result<Self> {
        let ar_fs_config = ArFileSystemConfig {
            key_root_path,
            arweave_url,
        };
        let ar_filesystem = ArFileSystem::new(ar_fs_config)?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("payload", DataType::Binary, true),
            Field::new("signature", DataType::Utf8, true),
            Field::new("block", DataType::UInt64, true),
            Field::new("order", DataType::UInt32, true),
        ]));

        Ok(Self {
            network_id,
            schema,
            ar_filesystem,
            temp_data_path,
        })
    }
    pub async fn get_ar_account(&self) -> Result<(String, String)> {
        let addr = self.ar_filesystem.get_address();
        let balance = self.ar_filesystem.get_balance().await?;
        Ok((addr, balance.to_string()))
    }

    pub async fn download_and_parse_record_batch(&self, tx: &str) -> Result<Vec<RecordBatch>> {
        let tmp_dir = TempDir::new_in(&self.temp_data_path, "download")
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        let file_path = tmp_dir.path().join(format!("{}.gz.parquet", tx));
        self.ar_filesystem
            .download_file(file_path.as_path(), tx)
            .await?;
        Self::parse_gzip_file(file_path.as_path())
    }
    pub async fn get_prev_arware_tx(&self, tx_id: &str) -> Result<Option<String>> {
        self.ar_filesystem.get_last_rollup_tag(tx_id).await
    }
    pub async fn compress_and_upload_record_batch(
        &self,
        tx: String,
        last_end_block: u64,
        current_block: u64,
        recordbatch: &RecordBatch,
    ) -> Result<(String, u64, u64, u64)> {
        let tmp_dir = TempDir::new_in(&self.temp_data_path, "compression")
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        let file_path = tmp_dir.path().join("rollup.gz.parquet");
        let (num_rows, size) = Self::dump_recordbatch(&file_path, recordbatch)?;
        let filename = format!("{}_{}.gz.parquet", last_end_block, current_block);
        //TODO add tx status confirmation
        let (id, reward) = self
            .ar_filesystem
            .upload_file(
                &file_path,
                tx.as_str(),
                last_end_block,
                current_block,
                self.network_id.load(Ordering::Relaxed),
                filename.as_str(),
            )
            .await?;
        Ok((id, reward, num_rows, size))
    }

    /// Compress recordbatch to parquet file
    pub fn dump_recordbatch(path: &Path, recordbatch: &RecordBatch) -> Result<(u64, u64)> {
        let properties = WriterProperties::builder()
            .set_compression(Compression::GZIP(GzipLevel::default()))
            .build();
        let fd = File::create(path).map_err(|e| DB3Error::RollupError(format!("{e}")))?;

        let mut writer = ArrowWriter::try_new(fd, recordbatch.schema(), Some(properties))
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        writer
            .write(recordbatch)
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        let meta = writer
            .close()
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        let metadata =
            std::fs::metadata(path).map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        Ok((meta.num_rows as u64, metadata.len()))
    }

    /// Parse recordbatch from parquet file
    pub fn parse_gzip_file(path: &Path) -> Result<Vec<RecordBatch>> {
        let fd = File::open(path).map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        // Create a sync parquet reader with batch_size.
        // batch_size is the number of rows to read up to buffer once from pages, defaults to 1024
        let parquet_reader = ParquetRecordBatchReaderBuilder::try_new(fd)
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?
            .with_batch_size(8192)
            .build()
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;

        let mut batches = Vec::new();

        for batch in parquet_reader {
            let each = batch.map_err(|e| DB3Error::RollupError(format!("{e}")))?;
            batches.push(each);
        }
        Ok(batches)
    }

    /// Parse mutation body, block and order from recordbatch
    pub fn convert_recordbatch_to_mutation(
        record_batch: &RecordBatch,
    ) -> Result<Vec<(MutationBody, u64, u32)>> {
        let mut mutations = Vec::new();
        let payloads = record_batch
            .column_by_name("payload")
            .unwrap()
            .as_any()
            .downcast_ref::<BinaryArray>()
            .unwrap();
        let signatures = record_batch
            .column_by_name("signature")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let blocks = record_batch
            .column_by_name("block")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        let orders = record_batch
            .column_by_name("order")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();

        for i in 0..record_batch.num_rows() {
            let payload = payloads.value(i);
            let signature = signatures.value(i);
            let block = blocks.value(i);
            let order = orders.value(i);
            let mutation = MutationBody {
                payload: payload.to_vec(),
                signature: signature.to_string(),
            };
            mutations.push((mutation, block, order));
        }
        Ok(mutations)
    }

    /// convert mutation to recordbatch
    /// encode mutation body, block and order to recordbatch
    pub fn convert_mutations_to_recordbatch(
        &self,
        mutations: &[(MutationHeader, MutationBody)],
    ) -> Result<RecordBatch> {
        //TODO limit the memory usage
        let mut payload_builder = BinaryBuilder::new();
        let mut signature_builder = StringBuilder::new();
        let mut block_builder = UInt64Builder::new();
        let mut order_builder = UInt32Builder::new();
        for (header, body) in mutations {
            let body_ref: &[u8] = &body.payload;
            payload_builder.append_value(body_ref);
            signature_builder.append_value(body.signature.as_str());
            block_builder.append_value(header.block_id);
            order_builder.append_value(header.order_id);
        }
        let array_refs: Vec<ArrayRef> = vec![
            Arc::new(payload_builder.finish()),
            Arc::new(signature_builder.finish()),
            Arc::new(block_builder.finish()),
            Arc::new(order_builder.finish()),
        ];
        let record_batch = RecordBatch::try_new(self.schema.clone(), array_refs)
            .map_err(|e| DB3Error::RollupError(format!("{e}")))?;
        info!(
            "convert {} into recordbatch with memory {}",
            mutations.len(),
            record_batch.get_array_memory_size()
        );
        Ok(record_batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, AsArray, BinaryArray, StringArray, UInt32Array, UInt64Array};
    use arrow::compute::or;
    use arrow::datatypes::{BinaryType, DataType, Field, Schema};
    use std::env;
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn it_works() {}

    fn mock_batch_record() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("payload", DataType::Binary, true),
            Field::new("signature", DataType::Utf8, true),
            Field::new("block", DataType::UInt64, true),
            Field::new("order", DataType::UInt32, true),
        ]));
        let mut payload_builder = BinaryBuilder::new();
        let mut signature_builder = StringBuilder::new();
        let mut block_builder = UInt64Builder::new();
        let mut order_builder = UInt32Builder::new();
        for block in 0..10 {
            let body_ref: &[u8] = "this is a payload sample".as_bytes();
            payload_builder.append_value(body_ref);
            signature_builder.append_value("0x1234567890");
            block_builder.append_value(block);
            order_builder.append_value((block * 10) as u32);
        }
        let array_refs: Vec<ArrayRef> = vec![
            Arc::new(payload_builder.finish()),
            Arc::new(signature_builder.finish()),
            Arc::new(block_builder.finish()),
            Arc::new(order_builder.finish()),
        ];
        RecordBatch::try_new(schema.clone(), array_refs).unwrap()
    }
    #[test]
    fn dump_recordbatch_ut() {
        let tmp_dir_path = TempDir::new("dump_recordbatch_ut").expect("create temp dir");

        let record_batch = mock_batch_record();
        let (num_rows, size) = ArToolBox::dump_recordbatch(
            Path::new(tmp_dir_path.path().join("test.parquet").to_str().unwrap()),
            &record_batch,
        )
        .unwrap();
        assert_eq!(num_rows, 10);
        assert_eq!(size, 1862);
    }
    #[test]
    fn parse_gzip_file_ut() {
        let tmp_dir_path = TempDir::new("dump_recordbatch_ut").expect("create temp dir");

        let parquet_file = tmp_dir_path.path().join("test.parquet");
        let record_batch = mock_batch_record();
        let (num_rows, size) = ArToolBox::dump_recordbatch(&parquet_file, &record_batch).unwrap();
        assert_eq!(num_rows, 10);
        assert_eq!(size, 1862);
        let res = ArToolBox::parse_gzip_file(parquet_file.as_path()).unwrap();
        assert_eq!(res.len(), 1);
        let rec = res[0].clone();
        assert!(rec.num_columns() == 4);
        assert_eq!(rec.num_rows(), 10);
        let payloads = rec
            .column_by_name("payload")
            .unwrap()
            .as_any()
            .downcast_ref::<BinaryArray>()
            .unwrap();
        assert_eq!(payloads.len(), 10);
        assert_eq!(payloads.value(5), "this is a payload sample".as_bytes());

        let signatures = rec
            .column_by_name("signature")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(signatures.len(), 10);
        assert_eq!(signatures.value(5), "0x1234567890");

        let blocks = rec
            .column_by_name("block")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        assert_eq!(blocks.len(), 10);
        assert_eq!(blocks.value(5), 5);

        let orders = rec
            .column_by_name("order")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(orders.len(), 10);
        assert_eq!(orders.value(5), 50);
    }

    #[test]
    fn parse_sample_ar_parquet_ut() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/37829_37968.gz.parquet");

        let res = ArToolBox::parse_gzip_file(path.as_path()).unwrap();
        assert_eq!(res.len(), 1);
        let rec = res[0].clone();
        assert_eq!(rec.num_columns(), 4);
        assert_eq!(rec.num_rows(), 204);

        let mutations = ArToolBox::convert_recordbatch_to_mutation(&rec).unwrap();
        assert_eq!(mutations.len(), 204);
        let (mutation, block, order) = mutations[0].clone();
        assert_eq!(block, 37829);
        assert_eq!(order, 1);
        assert_eq!(mutation.signature, "0xf6afe1165ae87fa09375eabccdedc61f3e5af4ed1e5c6456f1b63d397862252667e1f13f0f076f30609754f787c80135c52f7c249e95c9b8fab1b9ed27846c1b1c");
    }

    #[tokio::test]
    async fn download_arware_tx_ut() {
        let temp_dir = TempDir::new("download_arware_tx_ut").expect("create temp dir");
        let arweave_url = "https://arweave.net";
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let key_root_path = path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tools/keys")
            .to_str()
            .unwrap()
            .to_string();

        let network_id = Arc::new(AtomicU64::new(1687961160));
        let ar_toolbox = ArToolBox::new(
            key_root_path.to_string(),
            arweave_url.to_string(),
            temp_dir.path().to_str().unwrap().to_string(),
            network_id,
        )
        .unwrap();
        let tx_id = "TY5SMaPPRk_TMvSDROaQWyc_WHyJrEL760-UhiNnHG4";
        let res = ar_toolbox
            .download_and_parse_record_batch(tx_id)
            .await
            .unwrap();
        let rec1 = res[0].clone();
        let mutations = ArToolBox::convert_recordbatch_to_mutation(&rec1).unwrap();
        assert_eq!(mutations.len(), 8192);
        let (mutation, block, order) = mutations[0].clone();
        assert_eq!(block, 3712);
        assert_eq!(order, 1);
    }
    #[tokio::test]
    async fn get_prev_arware_tx_ut() {
        let temp_dir = TempDir::new("download_arware_tx_ut").expect("create temp dir");
        let arweave_url = "https://arweave.net";
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let key_root_path = path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tools/keys")
            .to_str()
            .unwrap()
            .to_string();

        let network_id = Arc::new(AtomicU64::new(1687961160));
        let ar_toolbox = ArToolBox::new(
            key_root_path.to_string(),
            arweave_url.to_string(),
            temp_dir.path().to_str().unwrap().to_string(),
            network_id,
        )
        .unwrap();
        let tx_id = "TY5SMaPPRk_TMvSDROaQWyc_WHyJrEL760-UhiNnHG4";
        let res = ar_toolbox.get_prev_arware_tx(tx_id).await.unwrap();
        assert!(res.is_some());
        assert_eq!(res.unwrap(), "ld2W-KnmHhmgYcSgc_DcqjjoU_ke9gkwrQEWk0A2Fpg");
    }
}
