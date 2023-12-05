//! Strategies management backend module.

use std::collections::{BTreeMap, BTreeSet};

use dapi_grpc::platform::v0::{ResponseMetadata, GetDocumentsResponse};
use dash_platform_sdk::{Sdk, Error, platform::{transition::broadcast::BroadcastStateTransition, DocumentQuery, FetchMany}};
use dpp::{
    data_contract::{created_data_contract::CreatedDataContract, document_type::accessors::DocumentTypeV0Getters}, platform_value::{Bytes32, Identifier, string_encoding::Encoding},
    version::PlatformVersion, block::{block_info::BlockInfo, epoch::Epoch}, identity::{Identity, PartialIdentity}, document::Document,
};
use drive::drive::identity::key::fetch::IdentityKeysRequest;
use rand::{rngs::StdRng, SeedableRng};
use simple_signer::signer::SimpleSigner;
use strategy_tests::{
    frequency::Frequency, operations::Operation, transitions::create_identities_state_transitions,
    Strategy, LocalDocumentQuery,
};
use tokio::{sync::{Mutex, MutexGuard}, runtime::Handle};

use super::{
    state::{KnownContractsMap, StrategiesMap},
    AppStateUpdate, BackendEvent, StrategyContractNames, Task, identities::fetch_identity_by_b58_id,
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum StrategyTask {
    CreateStrategy(String),
    SelectStrategy(String),
    DeleteStrategy(String),
    CloneStrategy(String),
    SetContractsWithUpdates(String, Vec<String>),
    SetIdentityInserts {
        strategy_name: String,
        identity_inserts_frequency: Frequency,
    },
    SetStartIdentities {
        strategy_name: String,
        count: u16,
        key_count: u32,
    },
    AddOperation {
        strategy_name: String,
        operation: Operation,
    },
    RunStrategy(String),
    RemoveLastContract(String),
    RemoveIdentityInserts(String),
    RemoveStartIdentities(String),
    RemoveLastOperation(String),
}

pub(crate) async fn run_strategy_task<'s>(
    sdk: &mut Sdk,
    available_strategies: &'s Mutex<StrategiesMap>,
    available_strategies_contract_names: &'s Mutex<BTreeMap<String, StrategyContractNames>>,
    selected_strategy: &'s Mutex<Option<String>>,
    known_contracts: &'s Mutex<KnownContractsMap>,
    task: StrategyTask,
) -> BackendEvent<'s> {
    match task {
        StrategyTask::CreateStrategy(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            let mut contract_names_lock = available_strategies_contract_names.lock().await;

            strategies_lock.insert(
                strategy_name.clone(),
                Strategy {
                    contracts_with_updates: Default::default(),
                    operations: Default::default(),
                    start_identities: Default::default(),
                    identities_inserts: Default::default(),
                    signer: Default::default(),
                },
            );
            contract_names_lock.insert(strategy_name, Default::default());
            BackendEvent::AppStateUpdated(AppStateUpdate::Strategies(
                strategies_lock,
                contract_names_lock,
            ))
        }
        StrategyTask::SelectStrategy(ref strategy_name) => {
            let mut selected_strategy_lock = selected_strategy.lock().await;
            let strategies_lock = available_strategies.lock().await;

            if strategies_lock.contains_key(strategy_name) {
                *selected_strategy_lock = Some(strategy_name.clone());
                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(available_strategies_contract_names.lock().await, |names| {
                        names.get_mut(strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::DeleteStrategy(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            let mut contract_names_lock = available_strategies_contract_names.lock().await;
            let mut selected_strategy_lock = selected_strategy.lock().await;

            // Check if the strategy exists and remove it
            if strategies_lock.contains_key(&strategy_name) {
                strategies_lock.remove(&strategy_name);
                contract_names_lock.remove(&strategy_name);

                // If the deleted strategy was the selected one, unset the selected strategy
                if let Some(selected) = selected_strategy_lock.as_ref() {
                    if selected == &strategy_name {
                        *selected_strategy_lock = None;
                    }
                }

                BackendEvent::AppStateUpdated(AppStateUpdate::Strategies(
                    strategies_lock,
                    contract_names_lock,
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::CloneStrategy(new_strategy_name) => {
            let strategies_lock = available_strategies.lock().await;
            let mut contract_names_lock = available_strategies_contract_names.lock().await;
            let selected_strategy_lock = selected_strategy.lock().await;

            if let Some(selected_strategy_name) = &*selected_strategy_lock {
                if let Some(strategy_to_clone) = strategies_lock.get(selected_strategy_name) {
                    let cloned_strategy = strategy_to_clone.clone();
                    drop(strategies_lock); // Release the lock before re-acquiring it as mutable

                    // Clone the display data for the new strategy
                    let cloned_display_data = contract_names_lock
                        .get(selected_strategy_name)
                        .cloned()
                        .unwrap_or_default();

                    let mut strategies_lock = available_strategies.lock().await;
                    strategies_lock.insert(new_strategy_name.clone(), cloned_strategy);
                    contract_names_lock.insert(new_strategy_name.clone(), cloned_display_data);

                    BackendEvent::AppStateUpdated(AppStateUpdate::Strategies(
                        strategies_lock,
                        contract_names_lock,
                    ))
                } else {
                    BackendEvent::None // Selected strategy does not exist
                }
            } else {
                BackendEvent::None // No strategy selected to clone
            }
        }
        StrategyTask::SetContractsWithUpdates(strategy_name, selected_contract_names) => {
            let mut strategies_lock = available_strategies.lock().await;
            let known_contracts_lock = known_contracts.lock().await;
            let mut contract_names_lock = available_strategies_contract_names.lock().await;

            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                let mut rng = StdRng::from_entropy();
                let platform_version = PlatformVersion::latest();

                if let Some(first_contract_name) = selected_contract_names.first() {
                    if let Some(data_contract) = known_contracts_lock.get(first_contract_name) {
                        let entropy = Bytes32::random_with_rng(&mut rng);
                        match CreatedDataContract::from_contract_and_entropy(
                            data_contract.clone(),
                            entropy,
                            platform_version,
                        ) {
                            Ok(initial_contract) => {
                                // Create a map for updates
                                let mut updates = BTreeMap::new();

                                // Process the subsequent contracts as updates
                                for (order, contract_name) in
                                    selected_contract_names.iter().enumerate().skip(1)
                                {
                                    if let Some(update_contract) =
                                        known_contracts_lock.get(contract_name)
                                    {
                                        let update_entropy = Bytes32::random_with_rng(&mut rng);
                                        match CreatedDataContract::from_contract_and_entropy(
                                            update_contract.clone(),
                                            update_entropy,
                                            platform_version,
                                        ) {
                                            Ok(created_update_contract) => {
                                                updates
                                                    .insert(order as u64, created_update_contract);
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Error converting DataContract to \
                                                     CreatedDataContract for update: {:?}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }

                                // Add the initial contract and its updates as a new entry
                                strategy.contracts_with_updates.push((
                                    initial_contract,
                                    if updates.is_empty() {
                                        None
                                    } else {
                                        Some(updates)
                                    },
                                ));
                            }
                            Err(e) => {
                                eprintln!(
                                    "Error converting DataContract to CreatedDataContract: {:?}",
                                    e
                                );
                            }
                        }
                    }
                }

                // Transform the selected_contract_names into the expected format for display
                let mut transformed_contract_names = Vec::new();
                if let Some(first_contract_name) = selected_contract_names.first() {
                    let updates: BTreeMap<u64, String> = selected_contract_names
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(order, name)| (order as u64, name.clone()))
                        .collect();
                    transformed_contract_names.push((first_contract_name.clone(), Some(updates)));
                }

                // Check if there is an existing entry for the strategy
                if let Some(existing_contracts) = contract_names_lock.get_mut(&strategy_name) {
                    // Append the new transformed contracts to the existing list
                    existing_contracts.extend(transformed_contract_names);
                } else {
                    // If there is no existing entry, create a new one
                    contract_names_lock.insert(strategy_name.clone(), transformed_contract_names);
                }

                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(&strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(contract_names_lock, |names| {
                        names.get_mut(&strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::AddOperation {
            ref strategy_name,
            ref operation,
        } => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(strategy_name) {
                strategy.operations.push(operation.clone());
                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(available_strategies_contract_names.lock().await, |names| {
                        names.get_mut(strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::SetIdentityInserts {
            strategy_name,
            identity_inserts_frequency,
        } => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                strategy.identities_inserts = identity_inserts_frequency;
                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(&strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(available_strategies_contract_names.lock().await, |names| {
                        names.get_mut(&strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::SetStartIdentities {
            ref strategy_name,
            count,
            key_count,
        } => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(strategy_name) {
                tokio::task::block_in_place(|| set_start_identities(strategy, count, key_count));
                BackendEvent::TaskCompletedStateChange {
                    task: Task::Strategy(task.clone()),
                    execution_result: Ok("Start identities set".into()),
                    app_state_update: AppStateUpdate::SelectedStrategy(
                        strategy_name.clone(),
                        MutexGuard::map(strategies_lock, |strategies| {
                            strategies.get_mut(strategy_name).expect("strategy exists")
                        }),
                        MutexGuard::map(
                            available_strategies_contract_names.lock().await,
                            |names| names.get_mut(strategy_name).expect("inconsistent data"),
                        ),
                    ),
                }
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::RunStrategy(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                let mut block_info = BlockInfo::default();

                // 1. send initial state transitions
                // a. get block info and call state_transitions_for_block_with_new_identities
                let mut document_query_callback = |query: LocalDocumentQuery| {
                    let handle = Handle::current();
                    handle.block_on(async {
                        match fetch_documents_with_block_info(sdk, query.clone()).await {
                            Ok((documents, metadata)) => { 
                                // Update block_info with the metadata
                                block_info = BlockInfo {
                                    time_ms: metadata.time_ms,
                                    height: metadata.height as u64,
                                    core_height: metadata.core_chain_locked_height,
                                    epoch: Epoch::new(metadata.epoch as u16).unwrap_or_default(),
                                };
                                documents
                            },
                            Err(e) => {
                                eprintln!("Error fetching documents or block info: {:?}", e);
                                vec![]
                            }
                        }
                    })
                };
                let mut identity_fetch_callback = |identifier: Identifier, _keys_request: Option<IdentityKeysRequest>| {
                    let handle = Handle::current();
                    handle.block_on(async {
                        let base58_id = identifier.to_string(Encoding::Base58);
                        match fetch_identity_by_b58_id(sdk, &base58_id).await {
                            Ok((Some(identity), _)) => identity.into_partial_identity_info(),
                            Ok((None, _)) | Err(_) => {
                                eprintln!("Error fetching identity or identity not found for ID: {}", base58_id);
                                PartialIdentity {
                                    id: identifier,
                                    loaded_public_keys: BTreeMap::new(),
                                    balance: None,
                                    revision: None,
                                    not_found_public_keys: BTreeSet::new(),
                                }
                            }
                        }
                    })
                };
                // block info is apparently in metadata when you query Platform
                let initial_block_info = BlockInfo::default(); // this is wrong, need to actually get info
                let mut current_identities: Vec<Identity> = vec![];
                let mut signer = SimpleSigner::default();
                let mut rng = StdRng::from_entropy();
                // maybe don't need this add_strategy_contracts_into_drive part?
                // strategy.add_strategy_contracts_into_drive(drive, PlatformVersion::latest());
                let state_transitions = strategy.state_transitions_for_block_with_new_identities(
                    &mut document_query_callback, 
                    &mut identity_fetch_callback,
                    &initial_block_info, 
                    &mut current_identities, 
                    &mut signer, 
                    &mut rng, 
                    PlatformVersion::latest()
                );
                // b. send state transitions
                for state_transition in state_transitions.0 {
                    let result = state_transition.broadcast(sdk);
                }

                // 2. wait for next block

                // 3. get next block info, send to state_transitions_for_block_with_new_identities
                // and get back state transitions
                let next_block_info = BlockInfo::default();
                // maybe don't need this add_strategy_contracts_into_drive part?
                // strategy.add_strategy_contracts_into_drive(drive, PlatformVersion::latest());
                let state_transitions = strategy.state_transitions_for_block_with_new_identities(
                    &mut document_query_callback,
                    &mut identity_fetch_callback, 
                    &next_block_info, 
                    &mut current_identities, 
                    &mut signer, 
                    &mut rng, 
                    PlatformVersion::latest()
                );

                // 4. send next batch of state transitions
                for state_transition in state_transitions.0 {
                    let result = state_transition.broadcast(sdk);
                }

                // repeat 2-4 until finished
                // when is the strategy finished? In rs-drive-abci tests it specifies
                // a number of blocks...

                BackendEvent::None // probably want something else here
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::RemoveLastContract(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            let mut contract_names_lock = available_strategies_contract_names.lock().await;

            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                // Remove the last contract_with_update entry from the strategy
                strategy.contracts_with_updates.pop();

                // Also remove the corresponding entry from the displayed contracts
                if let Some(contract_names) = contract_names_lock.get_mut(&strategy_name) {
                    // Assuming each entry in contract_names corresponds to an entry in
                    // contracts_with_updates
                    contract_names.pop();
                }

                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(&strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(contract_names_lock, |names| {
                        names.get_mut(&strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::RemoveIdentityInserts(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                strategy.identities_inserts = Frequency {
                    times_per_block_range: Default::default(),
                    chance_per_block: None,
                };
                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(&strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(available_strategies_contract_names.lock().await, |names| {
                        names.get_mut(&strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::RemoveStartIdentities(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                strategy.start_identities = vec![];
                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(&strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(available_strategies_contract_names.lock().await, |names| {
                        names.get_mut(&strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
        StrategyTask::RemoveLastOperation(strategy_name) => {
            let mut strategies_lock = available_strategies.lock().await;
            if let Some(strategy) = strategies_lock.get_mut(&strategy_name) {
                strategy.operations.pop();
                BackendEvent::AppStateUpdated(AppStateUpdate::SelectedStrategy(
                    strategy_name.clone(),
                    MutexGuard::map(strategies_lock, |strategies| {
                        strategies.get_mut(&strategy_name).expect("strategy exists")
                    }),
                    MutexGuard::map(available_strategies_contract_names.lock().await, |names| {
                        names.get_mut(&strategy_name).expect("inconsistent data")
                    }),
                ))
            } else {
                BackendEvent::None
            }
        }
    }
}

fn set_start_identities(strategy: &mut Strategy, count: u16, key_count: u32) {
    let identities = create_identities_state_transitions(
        count,
        key_count,
        &mut SimpleSigner::default(),
        &mut StdRng::seed_from_u64(567),
        PlatformVersion::latest(),
    );

    strategy.start_identities = identities;
}

async fn fetch_documents_with_block_info<'a>(
    sdk: &mut Sdk,
    query: LocalDocumentQuery<'a>
) -> Result<(Vec<Document>, ResponseMetadata), Error> {
    let document_query = match query {
        LocalDocumentQuery::RandomDocumentQuery(random_query) => {
            let data_contract = random_query.data_contract;
            let document_type_name = random_query.document_type.name();
            DocumentQuery::new(data_contract.clone(), document_type_name)?
        },
    };

    let response: GetDocumentsResponse = Document::fetch_many(sdk, document_query).await?;

    let documents = response.documents.into_iter()
        .filter_map(|(_, doc)| doc)
        .collect();

    let metadata = response.metadata.unwrap_or_default();

    Ok((documents, metadata))
}
