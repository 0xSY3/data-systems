// use std::{collections::HashMap, sync::Arc};

// use fuel_core::{
//     combined_database::CombinedDatabase,
//     service::{Config, FuelService},
//     ShutdownListener,
// };
// use fuel_core_importer::ImporterResult;
// use fuel_core_types::blockchain::SealedBlock;
// use fuel_streams_core::prelude::*;
// use tokio::sync::broadcast::{self, Receiver, Sender};

// // TODO - Re-implement with `mockall` and `mock` macros
// struct TestFuelCore {
//     fuel_service: FuelService,
//     chain_id: FuelCoreChainId,
//     base_asset_id: FuelCoreAssetId,
//     database: CombinedDatabase,
//     blocks_broadcaster: Sender<fuel_core_importer::ImporterResult>,
//     receipts: Option<Vec<FuelCoreReceipt>>,
// }

// impl TestFuelCore {
//     fn default(
//         blocks_broadcaster: Sender<fuel_core_importer::ImporterResult>,
//     ) -> Self {
//         let mut shutdown = ShutdownListener::spawn();
//         let service = FuelService::new(
//             Default::default(),
//             Config::local_node(),
//             &mut shutdown,
//         )
//         .unwrap();
//         Self {
//             fuel_service: service,
//             chain_id: FuelCoreChainId::default(),
//             base_asset_id: FuelCoreAssetId::zeroed(),
//             database: CombinedDatabase::default(),
//             blocks_broadcaster,
//             receipts: None,
//         }
//     }
//     fn with_receipts(mut self, receipts: Vec<FuelCoreReceipt>) -> Self {
//         self.receipts = Some(receipts);
//         self
//     }
//     fn arc(self) -> Arc<Self> {
//         Arc::new(self)
//     }
// }

// #[async_trait::async_trait]
// impl FuelCoreLike for TestFuelCore {
//     async fn start(&self) -> anyhow::Result<()> {
//         Ok(())
//     }
//     fn is_started(&self) -> bool {
//         true
//     }
//     fn fuel_service(&self) -> &FuelService {
//         &self.fuel_service
//     }
//     async fn await_synced_at_least_once(
//         &self,
//         _historical: bool,
//     ) -> anyhow::Result<()> {
//         Ok(())
//     }
//     async fn stop(&self) {}

//     async fn await_offchain_db_sync(
//         &self,
//         _block_id: &FuelCoreBlockId,
//     ) -> anyhow::Result<()> {
//         Ok(())
//     }

//     fn base_asset_id(&self) -> &FuelCoreAssetId {
//         &self.base_asset_id
//     }
//     fn chain_id(&self) -> &FuelCoreChainId {
//         &self.chain_id
//     }

//     fn database(&self) -> &CombinedDatabase {
//         &self.database
//     }

//     fn blocks_subscription(
//         &self,
//     ) -> Receiver<fuel_core_importer::ImporterResult> {
//         self.blocks_broadcaster.subscribe()
//     }

//     fn get_receipts(
//         &self,
//         _tx_id: &FuelCoreBytes32,
//     ) -> anyhow::Result<Option<Vec<FuelCoreReceipt>>> {
//         Ok(self.receipts.clone())
//     }

//     fn get_tx_status(
//         &self,
//         _tx_id: &FuelCoreBytes32,
//     ) -> anyhow::Result<Option<FuelCoreTransactionStatus>> {
//         Ok(Some(FuelCoreTransactionStatus::Success {
//             receipts: self.receipts.clone().unwrap_or_default(),
//             block_height: 0.into(),
//             result: None,
//             time: FuelCoreTai64::now(),
//             total_gas: 0,
//             total_fee: 0,
//         }))
//     }
// }

// #[tokio::test(flavor = "multi_thread")]
// async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
//     let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
//     let storage = Arc::new(S3Storage::new_for_testing().await);
//     let publisher = new_publisher(blocks_broadcaster.clone(), &storage).await;

//     let shutdown_controller = start_publisher(&publisher).await;
//     stop_publisher(shutdown_controller).await;

//     assert!(publisher.get_fuel_streams().is_empty().await);
// }

// #[tokio::test(flavor = "multi_thread")]
// async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
//     let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
//     let storage = Arc::new(S3Storage::new_for_testing().await);
//     let publisher = new_publisher(blocks_broadcaster.clone(), &storage).await;

//     publish_block(&publisher, &blocks_broadcaster).await;

//     assert!(publisher
//         .get_fuel_streams()
//         .blocks()
//         .get_last_published(BlocksSubject::WILDCARD)
//         .await
//         .is_ok_and(|result| result.is_some()));
//     storage.cleanup_after_testing().await;
// }

// #[tokio::test(flavor = "multi_thread")]
// async fn publishes_transaction_for_each_published_block() {
//     let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
//     let storage = Arc::new(S3Storage::new_for_testing().await);
//     let publisher = new_publisher(blocks_broadcaster.clone(), &storage).await;

//     publish_block(&publisher, &blocks_broadcaster).await;

//     assert!(publisher
//         .get_fuel_streams()
//         .transactions()
//         .get_last_published(TransactionsSubject::WILDCARD)
//         .await
//         .is_ok_and(|result| result.is_some()));
//     storage.cleanup_after_testing().await;
// }

// #[tokio::test(flavor = "multi_thread")]
// async fn publishes_receipts() {
//     let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);

//     let receipts = [
//         FuelCoreReceipt::Call {
//             id: FuelCoreContractId::default(),
//             to: Default::default(),
//             amount: 0,
//             asset_id: Default::default(),
//             gas: 0,
//             param1: 0,
//             param2: 0,
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::Return {
//             id: FuelCoreContractId::default(),
//             val: 0,
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::ReturnData {
//             id: FuelCoreContractId::default(),
//             ptr: 0,
//             len: 0,
//             digest: FuelCoreBytes32::default(),
//             pc: 0,
//             is: 0,
//             data: None,
//         },
//         FuelCoreReceipt::Revert {
//             id: FuelCoreContractId::default(),
//             ra: 0,
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::Log {
//             id: FuelCoreContractId::default(),
//             ra: 0,
//             rb: 0,
//             rc: 0,
//             rd: 0,
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::LogData {
//             id: FuelCoreContractId::default(),
//             ra: 0,
//             rb: 0,
//             ptr: 0,
//             len: 0,
//             digest: FuelCoreBytes32::default(),
//             pc: 0,
//             is: 0,
//             data: None,
//         },
//         FuelCoreReceipt::Transfer {
//             id: FuelCoreContractId::default(),
//             to: FuelCoreContractId::default(),
//             amount: 0,
//             asset_id: FuelCoreAssetId::default(),
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::TransferOut {
//             id: FuelCoreContractId::default(),
//             to: FuelCoreAddress::default(),
//             amount: 0,
//             asset_id: FuelCoreAssetId::default(),
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::Mint {
//             sub_id: FuelCoreBytes32::default(),
//             contract_id: FuelCoreContractId::default(),
//             val: 0,
//             pc: 0,
//             is: 0,
//         },
//         FuelCoreReceipt::Burn {
//             sub_id: FuelCoreBytes32::default(),
//             contract_id: FuelCoreContractId::default(),
//             val: 0,
//             pc: 0,
//             is: 0,
//         },
//     ];

//     let fuel_core = TestFuelCore::default(blocks_broadcaster.clone())
//         .with_receipts(receipts.to_vec())
//         .arc();

//     let storage = Arc::new(S3Storage::new_for_testing().await);
//     let publisher =
//         Publisher::new_for_testing(&nats_client().await, &storage, fuel_core)
//             .await
//             .unwrap();

//     publish_block(&publisher, &blocks_broadcaster).await;

//     let mut receipts_stream = publisher
//         .get_fuel_streams()
//         .receipts()
//         .catchup(10)
//         .await
//         .unwrap();

//     let expected_receipts: Vec<Receipt> =
//         receipts.iter().map(Into::into).collect();
//     let mut found_receipts = Vec::new();

//     while let Some(Some(receipt)) = receipts_stream.next().await {
//         found_receipts.push(receipt);
//     }
//     assert_eq!(
//         found_receipts.len(),
//         expected_receipts.len(),
//         "Number of receipts doesn't match"
//     );

//     // Create sets of receipt identifiers
//     let found_ids: std::collections::HashSet<_> = found_receipts
//         .into_iter()
//         .map(|r| match r {
//             Receipt::Call(r) => r.id,
//             Receipt::Return(r) => r.id,
//             Receipt::ReturnData(r) => r.id,
//             Receipt::Revert(r) => r.id,
//             Receipt::Log(r) => r.id,
//             Receipt::LogData(r) => r.id,
//             Receipt::Transfer(r) => r.id,
//             Receipt::TransferOut(r) => r.id,
//             Receipt::Mint(r) => r.contract_id,
//             Receipt::Burn(r) => r.contract_id,
//             Receipt::Panic(r) => r.id,
//             _ => unreachable!(),
//         })
//         .collect();

//     let expected_ids: std::collections::HashSet<_> = expected_receipts
//         .into_iter()
//         .map(|r| match r {
//             Receipt::Call(r) => r.id,
//             Receipt::Return(r) => r.id,
//             Receipt::ReturnData(r) => r.id,
//             Receipt::Revert(r) => r.id,
//             Receipt::Log(r) => r.id,
//             Receipt::LogData(r) => r.id,
//             Receipt::Transfer(r) => r.id,
//             Receipt::TransferOut(r) => r.id,
//             Receipt::Mint(r) => r.contract_id,
//             Receipt::Burn(r) => r.contract_id,
//             Receipt::Panic(r) => r.id,
//             _ => unreachable!(),
//         })
//         .collect();

//     assert_eq!(
//         found_ids, expected_ids,
//         "Published receipt IDs don't match expected IDs"
//     );

//     storage.cleanup_after_testing().await;
// }

// #[tokio::test(flavor = "multi_thread")]
// async fn publishes_inputs() {
//     let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
//     let storage = Arc::new(S3Storage::new_for_testing().await);
//     let publisher = new_publisher(blocks_broadcaster.clone(), &storage).await;

//     publish_block(&publisher, &blocks_broadcaster).await;

//     assert!(publisher
//         .get_fuel_streams()
//         .inputs()
//         .get_last_published(InputsByIdSubject::WILDCARD)
//         .await
//         .is_ok_and(|result| result.is_some()));
//     storage.cleanup_after_testing().await;
// }

// async fn new_publisher(
//     broadcaster: Sender<ImporterResult>,
//     storage: &Arc<S3Storage>,
// ) -> Publisher {
//     let fuel_core = TestFuelCore::default(broadcaster).arc();
//     Publisher::new_for_testing(&nats_client().await, storage, fuel_core)
//         .await
//         .unwrap()
// }

// async fn publish_block(
//     publisher: &Publisher,
//     blocks_broadcaster: &Sender<ImporterResult>,
// ) {
//     let shutdown_controller = start_publisher(publisher).await;
//     send_block(blocks_broadcaster);
//     stop_publisher(shutdown_controller).await;
// }

// async fn start_publisher(publisher: &Publisher) -> ShutdownController {
//     let (shutdown_controller, shutdown_token) = get_controller_and_token();
//     tokio::spawn({
//         let publisher = publisher.clone();
//         async move {
//             publisher.run(shutdown_token, true).await.unwrap();
//         }
//     });
//     wait_for_publisher_to_start().await;
//     shutdown_controller
// }
// async fn stop_publisher(shutdown_controller: ShutdownController) {
//     wait_for_publisher_to_process_block().await;

//     assert!(shutdown_controller.initiate_shutdown().is_ok());
// }

// async fn wait_for_publisher_to_start() {
//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
// }
// async fn wait_for_publisher_to_process_block() {
//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
// }

// fn send_block(broadcaster: &Sender<ImporterResult>) {
//     let block = create_test_block();
//     assert!(broadcaster.send(block).is_ok());
// }
// fn create_test_block() -> ImporterResult {
//     let mut block_entity = FuelCoreBlock::default();
//     let tx = FuelCoreTransaction::default_test_tx();

//     *block_entity.transactions_mut() = vec![tx];

//     ImporterResult {
//         shared_result: Arc::new(FuelCoreImportResult {
//             sealed_block: SealedBlock {
//                 entity: block_entity,
//                 ..Default::default()
//             },
//             ..Default::default()
//         }),
//         changes: Arc::new(HashMap::new()),
//     }
// }

// async fn nats_client() -> NatsClient {
//     let opts = NatsClientOpts::admin_opts().with_rdn_namespace();
//     NatsClient::connect(&opts)
//         .await
//         .expect("NATS connection failed")
// }
