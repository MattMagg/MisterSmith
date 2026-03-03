# Data Integration Patterns

**Specification**: Unified message and database system integration patterns for the Mister Smith AI Agent Framework.

## Overview & Architecture

### Integration Philosophy

The Mister Smith framework employs a unified data integration approach that seamlessly connects messaging systems (NATS)
with database persistence layers. This integration ensures data consistency, reliability, and performance across all agent operations.

### Core Principles

- **Event-Driven Architecture**: All data changes propagate through events
- **Eventual Consistency**: Accept temporary inconsistencies for system availability
- **Idempotent Operations**: All data operations can be safely retried
- **Transactional Boundaries**: Clear separation between atomic operations
- **Audit Trail**: Complete traceability of all data changes

### System-Wide Data Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Agent Layer   │    │  Message Layer  │    │ Database Layer  │
│                 │    │                 │    │                 │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │   Agent   │◄─┼────┼─►│   NATS    │◄─┼────┼─►│PostgreSQL │  │
│  │  Actions  │  │    │  │ Subjects  │  │    │  │  Tables   │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │   Agent   │◄─┼────┼─►│  Event    │◄─┼────┼─►│   Redis   │  │
│  │   State   │  │    │  │  Store    │  │    │  │   Cache   │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Message-Database Integration Patterns

### 1. Event-Driven Synchronization

#### Publisher-Subscriber Pattern

```
// Event publishing from database changes
pub struct DatabaseEventPublisher {
    nats_client: NatsClient,
    event_store: EventStore,
}

impl DatabaseEventPublisher {
    pub async fn publish_entity_changed(&self, entity: &Entity, operation: Operation) -> Result<()> {
        let event = EntityChangeEvent {
            entity_id: entity.id,
            entity_type: entity.type_name(),
            operation,
            timestamp: Utc::now(),
            version: entity.version,
            data: entity.serialize()?,
        };

        // Publish to NATS
        self.nats_client.publish(
            &format!("entity.{}.{}", entity.type_name(), operation),
            &event.serialize()?
        ).await?;

        // Store in event store
        self.event_store.append(event).await?;
        Ok(())
    }
}
```

#### Event-Driven Database Updates

```
// Message consumption triggering database updates
pub struct DatabaseEventConsumer {
    nats_subscription: NatsSubscription,
    database_pool: DatabasePool,
}

impl DatabaseEventConsumer {
    pub async fn handle_entity_event(&self, event: EntityChangeEvent) -> Result<()> {
        let mut transaction = self.database_pool.begin().await?;
        
        match event.operation {
            Operation::Create => {
                self.create_entity(&mut transaction, &event).await?;
            },
            Operation::Update => {
                self.update_entity(&mut transaction, &event).await?;
            },
            Operation::Delete => {
                self.delete_entity(&mut transaction, &event).await?;
            },
        }
        
        transaction.commit().await?;
        Ok(())
    }
}
```

### 2. Bidirectional Data Flow

#### Message-to-Database Flow

```
pub struct MessageToDatabaseFlow {
    message_handler: MessageHandler,
    entity_repository: EntityRepository,
    transaction_manager: TransactionManager,
}

impl MessageToDatabaseFlow {
    pub async fn process_message(&self, message: Message) -> Result<()> {
        let transaction = self.transaction_manager.begin().await?;
        
        // Transform message to entity
        let entity = self.message_handler.transform_to_entity(message)?;
        
        // Validate entity
        entity.validate()?;
        
        // Save to database
        self.entity_repository.save(&entity, &transaction).await?;
        
        // Commit transaction
        transaction.commit().await?;
        
        Ok(())
    }
}
```

#### Database-to-Message Flow

```
pub struct DatabaseToMessageFlow {
    database_listener: DatabaseListener,
    message_publisher: MessagePublisher,
    event_transformer: EventTransformer,
}

impl DatabaseToMessageFlow {
    pub async fn handle_database_change(&self, change: DatabaseChange) -> Result<()> {
        // Transform database change to message
        let message = self.event_transformer.to_message(change)?;
        
        // Publish message
        self.message_publisher.publish(message).await?;
        
        Ok(())
    }
}
```

### 3. Message-to-Entity Mapping

#### Schema Mapping Configuration

```
pub struct MessageEntityMapping {
    mappings: HashMap<String, EntityMapping>,
}

#[derive(Debug, Clone)]
pub struct EntityMapping {
    pub message_type: String,
    pub entity_type: String,
    pub field_mappings: Vec<FieldMapping>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct FieldMapping {
    pub message_field: String,
    pub entity_field: String,
    pub transformer: Option<FieldTransformer>,
}
```

#### Dynamic Mapping Implementation

```
impl MessageEntityMapping {
    pub fn transform_message_to_entity(&self, message: &Message) -> Result<Entity> {
        let mapping = self.mappings.get(&message.type_name())
            .ok_or_else(|| Error::MappingNotFound(message.type_name()))?;
        
        let mut entity = Entity::new(mapping.entity_type.clone());
        
        for field_mapping in &mapping.field_mappings {
            let message_value = message.get_field(&field_mapping.message_field)?;
            let entity_value = if let Some(transformer) = &field_mapping.transformer {
                transformer.transform(message_value)?
            } else {
                message_value
            };
            
            entity.set_field(&field_mapping.entity_field, entity_value)?;
        }
        
        // Apply validation rules
        for rule in &mapping.validation_rules {
            rule.validate(&entity)?;
        }
        
        Ok(entity)
    }
}
```

### 4. Schema Evolution Handling

#### Version Management

```
pub struct SchemaVersionManager {
    versions: HashMap<String, Vec<SchemaVersion>>,
}

#[derive(Debug, Clone)]
pub struct SchemaVersion {
    pub version: u32,
    pub schema: Schema,
    pub migration_rules: Vec<MigrationRule>,
}

impl SchemaVersionManager {
    pub fn migrate_message(&self, message: &Message, target_version: u32) -> Result<Message> {
        let current_version = message.schema_version;
        
        if current_version == target_version {
            return Ok(message.clone());
        }
        
        let migration_path = self.find_migration_path(current_version, target_version)?;
        
        let mut migrated_message = message.clone();
        for migration in migration_path {
            migrated_message = migration.apply(migrated_message)?;
        }
        
        Ok(migrated_message)
    }
}
```

## Consistency & Transaction Management

### 1. Consistency Models

#### Eventual Consistency Pattern

```
pub struct EventualConsistencyManager {
    event_store: EventStore,
    read_model_updater: ReadModelUpdater,
    consistency_checker: ConsistencyChecker,
}

impl EventualConsistencyManager {
    pub async fn ensure_eventual_consistency(&self) -> Result<()> {
        // Check for inconsistencies
        let inconsistencies = self.consistency_checker.find_inconsistencies().await?;
        
        for inconsistency in inconsistencies {
            // Replay events to fix inconsistency
            let events = self.event_store.get_events_since(inconsistency.last_known_good).await?;
            
            for event in events {
                self.read_model_updater.apply_event(event).await?;
            }
        }
        
        Ok(())
    }
}
```

#### Strong Consistency Pattern

```
pub struct StrongConsistencyManager {
    coordinator: TransactionCoordinator,
    participants: Vec<TransactionParticipant>,
}

impl StrongConsistencyManager {
    pub async fn execute_distributed_transaction(&self, transaction: DistributedTransaction) -> Result<()> {
        // Phase 1: Prepare
        let mut prepared_participants = Vec::new();
        
        for participant in &self.participants {
            match participant.prepare(&transaction).await {
                Ok(()) => prepared_participants.push(participant),
                Err(e) => {
                    // Abort all prepared participants
                    for p in prepared_participants {
                        p.abort(&transaction).await?;
                    }
                    return Err(e);
                }
            }
        }
        
        // Phase 2: Commit
        for participant in prepared_participants {
            participant.commit(&transaction).await?;
        }
        
        Ok(())
    }
}
```

### 2. Distributed Transaction Patterns

#### Saga Pattern Implementation

```
pub struct SagaOrchestrator {
    saga_store: SagaStore,
    compensations: HashMap<String, CompensationAction>,
}

#[derive(Debug, Clone)]
pub struct Saga {
    pub id: SagaId,
    pub steps: Vec<SagaStep>,
    pub current_step: usize,
    pub status: SagaStatus,
}

impl SagaOrchestrator {
    pub async fn execute_saga(&self, saga: &mut Saga) -> Result<()> {
        while saga.current_step < saga.steps.len() {
            let step = &saga.steps[saga.current_step];
            
            match step.execute().await {
                Ok(()) => {
                    saga.current_step += 1;
                    self.saga_store.save(saga).await?;
                },
                Err(e) => {
                    // Execute compensations in reverse order
                    for i in (0..saga.current_step).rev() {
                        if let Some(compensation) = self.compensations.get(&saga.steps[i].id) {
                            compensation.execute().await?;
                        }
                    }
                    
                    saga.status = SagaStatus::Failed;
                    self.saga_store.save(saga).await?;
                    return Err(e);
                }
            }
        }
        
        saga.status = SagaStatus::Completed;
        self.saga_store.save(saga).await?;
        Ok(())
    }
}
```

### 3. Outbox Pattern Implementation

#### Transactional Outbox

```
pub struct TransactionalOutbox {
    database_pool: DatabasePool,
    message_publisher: MessagePublisher,
}

impl TransactionalOutbox {
    pub async fn publish_with_transaction<T>(&self, 
        entity_operation: impl FnOnce(&mut Transaction) -> Result<T>,
        message: Message
    ) -> Result<T> {
        let mut transaction = self.database_pool.begin().await?;
        
        // Execute entity operation
        let result = entity_operation(&mut transaction)?;
        
        // Store message in outbox table
        sqlx::query!(
            "INSERT INTO outbox (id, message_type, payload, created_at) VALUES ($1, $2, $3, $4)",
            message.id,
            message.type_name(),
            message.serialize()?,
            Utc::now()
        ).execute(&mut transaction).await?;
        
        transaction.commit().await?;
        
        // Publish message asynchronously
        self.message_publisher.publish(message).await?;
        
        Ok(result)
    }
}
```

#### Inbox Pattern for Idempotency

```
pub struct InboxProcessor {
    database_pool: DatabasePool,
    processed_messages: HashSet<MessageId>,
}

impl InboxProcessor {
    pub async fn process_message_idempotently(&self, message: Message) -> Result<()> {
        let mut transaction = self.database_pool.begin().await?;
        
        // Check if message already processed
        let existing = sqlx::query!(
            "SELECT id FROM inbox WHERE message_id = $1",
            message.id
        ).fetch_optional(&mut transaction).await?;
        
        if existing.is_some() {
            return Ok(()); // Already processed
        }
        
        // Process message
        self.process_message(&message, &mut transaction).await?;
        
        // Record as processed
        sqlx::query!(
            "INSERT INTO inbox (message_id, processed_at) VALUES ($1, $2)",
            message.id,
            Utc::now()
        ).execute(&mut transaction).await?;
        
        transaction.commit().await?;
        Ok(())
    }
}
```

### 4. Conflict Resolution Strategies

#### Last-Write-Wins Strategy

```
pub struct LastWriteWinsResolver {
    clock: LogicalClock,
}

impl ConflictResolver for LastWriteWinsResolver {
    fn resolve_conflict(&self, local: &Entity, remote: &Entity) -> Result<Entity> {
        if remote.timestamp > local.timestamp {
            Ok(remote.clone())
        } else {
            Ok(local.clone())
        }
    }
}
```

#### Semantic Conflict Resolution

```
pub struct SemanticConflictResolver {
    merge_strategies: HashMap<String, MergeStrategy>,
}

impl ConflictResolver for SemanticConflictResolver {
    fn resolve_conflict(&self, local: &Entity, remote: &Entity) -> Result<Entity> {
        let strategy = self.merge_strategies.get(&local.type_name())
            .ok_or_else(|| Error::NoMergeStrategy(local.type_name()))?;
        
        strategy.merge(local, remote)
    }
}
```

## Event Sourcing & CQRS Implementation

### 1. Event Store Design

#### Event Store Interface

```
pub trait EventStore {
    async fn append(&self, event: Event) -> Result<()>;
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Event>>;
    async fn get_events_since(&self, timestamp: DateTime<Utc>) -> Result<Vec<Event>>;
    async fn create_snapshot(&self, aggregate_id: &str, snapshot: Snapshot) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EventId,
    pub aggregate_id: String,
    pub aggregate_version: u64,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

#### PostgreSQL Event Store Implementation

```
pub struct PostgresEventStore {
    pool: PgPool,
}

impl EventStore for PostgresEventStore {
    async fn append(&self, event: Event) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO events (id, aggregate_id, aggregate_version, event_type, event_data, timestamp, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            event.id,
            event.aggregate_id,
            event.aggregate_version as i64,
            event.event_type,
            event.event_data,
            event.timestamp,
            serde_json::to_value(&event.metadata)?
        ).execute(&self.pool).await?;
        
        Ok(())
    }
    
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<Event>> {
        let rows = sqlx::query!(
            "SELECT * FROM events WHERE aggregate_id = $1 ORDER BY aggregate_version",
            aggregate_id
        ).fetch_all(&self.pool).await?;
        
        let events = rows.into_iter()
            .map(|row| Event {
                id: row.id,
                aggregate_id: row.aggregate_id,
                aggregate_version: row.aggregate_version as u64,
                event_type: row.event_type,
                event_data: row.event_data,
                timestamp: row.timestamp,
                metadata: serde_json::from_value(row.metadata)?,
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
}
```

### 2. Command/Query Separation

#### Command Side Implementation

```
pub struct CommandHandler {
    event_store: Arc<dyn EventStore>,
    aggregate_factory: AggregateFactory,
}

impl CommandHandler {
    pub async fn handle_command(&self, command: Command) -> Result<()> {
        // Load aggregate from event store
        let events = self.event_store.get_events(&command.aggregate_id).await?;
        let mut aggregate = self.aggregate_factory.create_from_events(events)?;
        
        // Execute command
        let new_events = aggregate.handle_command(command)?;
        
        // Save new events
        for event in new_events {
            self.event_store.append(event).await?;
        }
        
        Ok(())
    }
}
```

#### Query Side Implementation

```
pub struct QueryHandler {
    read_model_store: Arc<dyn ReadModelStore>,
}

impl QueryHandler {
    pub async fn handle_query(&self, query: Query) -> Result<QueryResult> {
        match query {
            Query::GetEntity { id } => {
                let entity = self.read_model_store.get_entity(&id).await?;
                Ok(QueryResult::Entity(entity))
            },
            Query::ListEntities { filter } => {
                let entities = self.read_model_store.list_entities(filter).await?;
                Ok(QueryResult::EntityList(entities))
            },
            Query::GetAggregateData { aggregate_id } => {
                let data = self.read_model_store.get_aggregate_data(&aggregate_id).await?;
                Ok(QueryResult::AggregateData(data))
            }
        }
    }
}
```

### 3. Read Model Management

#### Read Model Updater

```
pub struct ReadModelUpdater {
    event_store: Arc<dyn EventStore>,
    read_model_store: Arc<dyn ReadModelStore>,
    projections: Vec<Arc<dyn Projection>>,
}

impl ReadModelUpdater {
    pub async fn update_read_models(&self) -> Result<()> {
        let last_processed = self.read_model_store.get_last_processed_event().await?;
        let events = self.event_store.get_events_since(last_processed).await?;
        
        for event in events {
            for projection in &self.projections {
                projection.apply_event(&event).await?;
            }
            
            self.read_model_store.set_last_processed_event(event.timestamp).await?;
        }
        
        Ok(())
    }
}
```

#### Projection Implementation

```
pub trait Projection {
    async fn apply_event(&self, event: &Event) -> Result<()>;
}

pub struct EntityProjection {
    read_model_store: Arc<dyn ReadModelStore>,
}

impl Projection for EntityProjection {
    async fn apply_event(&self, event: &Event) -> Result<()> {
        match event.event_type.as_str() {
            "EntityCreated" => {
                let entity_data: EntityData = serde_json::from_value(event.event_data.clone())?;
                self.read_model_store.create_entity(entity_data).await?;
            },
            "EntityUpdated" => {
                let update_data: EntityUpdateData = serde_json::from_value(event.event_data.clone())?;
                self.read_model_store.update_entity(&event.aggregate_id, update_data).await?;
            },
            "EntityDeleted" => {
                self.read_model_store.delete_entity(&event.aggregate_id).await?;
            },
            _ => {} // Ignore unknown event types
        }
        
        Ok(())
    }
}
```

### 4. Event Replay Mechanisms

#### Event Replay Manager

```
pub struct EventReplayManager {
    event_store: Arc<dyn EventStore>,
    read_model_store: Arc<dyn ReadModelStore>,
    projections: Vec<Arc<dyn Projection>>,
}

impl EventReplayManager {
    pub async fn replay_events(&self, from_timestamp: DateTime<Utc>) -> Result<()> {
        // Clear read models
        self.read_model_store.clear().await?;
        
        // Replay all events from the specified timestamp
        let events = self.event_store.get_events_since(from_timestamp).await?;
        
        for event in events {
            for projection in &self.projections {
                projection.apply_event(&event).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn replay_aggregate(&self, aggregate_id: &str) -> Result<()> {
        let events = self.event_store.get_events(aggregate_id).await?;
        
        for event in events {
            for projection in &self.projections {
                projection.apply_event(&event).await?;
            }
        }
        
        Ok(())
    }
}
```

## Performance & Optimization

### 1. Batch Processing Patterns

#### Message Batch Processor

```
pub struct MessageBatchProcessor {
    batch_size: usize,
    batch_timeout: Duration,
    processor: MessageProcessor,
}

impl MessageBatchProcessor {
    pub async fn process_messages(&self, messages: Vec<Message>) -> Result<()> {
        let chunks: Vec<_> = messages.chunks(self.batch_size).collect();
        
        for chunk in chunks {
            let batch_future = self.process_batch(chunk.to_vec());
            let timeout_future = tokio::time::sleep(self.batch_timeout);
            
            tokio::select! {
                result = batch_future => {
                    result?;
                },
                _ = timeout_future => {
                    return Err(Error::BatchTimeout);
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_batch(&self, messages: Vec<Message>) -> Result<()> {
        let mut transaction = self.processor.begin_transaction().await?;
        
        for message in messages {
            self.processor.process_message(message, &mut transaction).await?;
        }
        
        transaction.commit().await?;
        Ok(())
    }
}
```

### 2. Connection Pooling

#### Database Connection Pool Configuration

```
pub struct DatabaseConnectionPool {
    pool: PgPool,
    config: PoolConfig,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl DatabaseConnectionPool {
    pub async fn new(config: PoolConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&database_url()).await?;
        
        Ok(Self { pool, config })
    }
    
    pub async fn get_connection(&self) -> Result<PoolConnection<Postgres>> {
        self.pool.acquire().await.map_err(Into::into)
    }
}
```

### 3. Caching Strategies

#### Multi-Level Caching

```
pub struct MultiLevelCache {
    l1_cache: LruCache<String, CacheValue>, // In-memory
    l2_cache: RedisCache,                   // Redis
    l3_cache: DatabaseCache,                // Database
}

impl MultiLevelCache {
    pub async fn get(&self, key: &str) -> Result<Option<CacheValue>> {
        // Check L1 cache
        if let Some(value) = self.l1_cache.get(key) {
            return Ok(Some(value.clone()));
        }
        
        // Check L2 cache
        if let Some(value) = self.l2_cache.get(key).await? {
            self.l1_cache.put(key.to_string(), value.clone());
            return Ok(Some(value));
        }
        
        // Check L3 cache
        if let Some(value) = self.l3_cache.get(key).await? {
            self.l2_cache.set(key, &value).await?;
            self.l1_cache.put(key.to_string(), value.clone());
            return Ok(Some(value));
        }
        
        Ok(None)
    }
}
```

### 4. Throughput Optimization

#### Parallel Processing Pipeline

```
pub struct ParallelProcessingPipeline {
    workers: usize,
    queue: Arc<Mutex<VecDeque<Message>>>,
    processors: Vec<MessageProcessor>,
}

impl ParallelProcessingPipeline {
    pub async fn process_messages(&self, messages: Vec<Message>) -> Result<()> {
        let (tx, rx) = mpsc::channel(messages.len());
        
        // Spawn worker tasks
        let mut handles = Vec::new();
        for i in 0..self.workers {
            let processor = self.processors[i].clone();
            let rx = rx.clone();
            
            let handle = tokio::spawn(async move {
                while let Ok(message) = rx.recv().await {
                    if let Err(e) = processor.process(message).await {
                        eprintln!("Worker {} error: {}", i, e);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Send messages to workers
        for message in messages {
            tx.send(message).await?;
        }
        
        // Wait for all workers to complete
        for handle in handles {
            handle.await?;
        }
        
        Ok(())
    }
}
```

## Error Handling & Recovery

### 1. Retry Mechanisms

#### Exponential Backoff Retry

```
pub struct RetryManager {
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_factor: f64,
}

impl RetryManager {
    pub async fn retry_with_backoff<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug,
    {
        let mut delay = self.initial_delay;
        
        for attempt in 1..=self.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == self.max_attempts {
                        return Err(e);
                    }
                    
                    println!("Attempt {} failed: {:?}. Retrying in {:?}", attempt, e, delay);
                    tokio::time::sleep(delay).await;
                    
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * self.backoff_factor) as u64),
                        self.max_delay
                    );
                }
            }
        }
        
        unreachable!()
    }
}
```

### 2. Circuit Breaker Pattern

#### Circuit Breaker Implementation

```
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    failure_threshold: usize,
    timeout: Duration,
    reset_timeout: Duration,
}

#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed { failure_count: usize },
    Open { last_failure: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug,
    {
        let state = {
            let guard = self.state.lock().await;
            guard.clone()
        };
        
        match state {
            CircuitBreakerState::Open { last_failure } => {
                if last_failure.elapsed() > self.reset_timeout {
                    // Try to reset to half-open
                    let mut guard = self.state.lock().await;
                    *guard = CircuitBreakerState::HalfOpen;
                } else {
                    return Err(/* Circuit breaker open error */);
                }
            },
            CircuitBreakerState::HalfOpen => {
                // Allow one request through
            },
            CircuitBreakerState::Closed { .. } => {
                // Normal operation
            }
        }
        
        match operation().await {
            Ok(result) => {
                // Reset on success
                let mut guard = self.state.lock().await;
                *guard = CircuitBreakerState::Closed { failure_count: 0 };
                Ok(result)
            },
            Err(e) => {
                // Handle failure
                let mut guard = self.state.lock().await;
                match *guard {
                    CircuitBreakerState::Closed { failure_count } => {
                        let new_count = failure_count + 1;
                        if new_count >= self.failure_threshold {
                            *guard = CircuitBreakerState::Open { last_failure: Instant::now() };
                        } else {
                            *guard = CircuitBreakerState::Closed { failure_count: new_count };
                        }
                    },
                    CircuitBreakerState::HalfOpen => {
                        *guard = CircuitBreakerState::Open { last_failure: Instant::now() };
                    },
                    _ => {}
                }
                
                Err(e)
            }
        }
    }
}
```

### 3. Data Reconciliation

#### Reconciliation Service

```
pub struct DataReconciliationService {
    source_store: Arc<dyn DataStore>,
    target_store: Arc<dyn DataStore>,
    reconciliation_rules: Vec<ReconciliationRule>,
}

impl DataReconciliationService {
    pub async fn reconcile_data(&self) -> Result<ReconciliationReport> {
        let mut report = ReconciliationReport::new();
        
        // Get all entities from both stores
        let source_entities = self.source_store.get_all_entities().await?;
        let target_entities = self.target_store.get_all_entities().await?;
        
        // Find differences
        for source_entity in source_entities {
            if let Some(target_entity) = target_entities.get(&source_entity.id) {
                if source_entity != *target_entity {
                    // Entities differ - resolve conflict
                    let resolved = self.resolve_conflict(&source_entity, target_entity)?;
                    
                    // Update target
                    self.target_store.update_entity(resolved).await?;
                    report.add_resolved_conflict(source_entity.id);
                }
            } else {
                // Entity missing in target
                self.target_store.create_entity(source_entity).await?;
                report.add_missing_entity(source_entity.id);
            }
        }
        
        Ok(report)
    }
    
    fn resolve_conflict(&self, source: &Entity, target: &Entity) -> Result<Entity> {
        for rule in &self.reconciliation_rules {
            if rule.applies_to(source, target) {
                return rule.resolve(source, target);
            }
        }
        
        // Default: use latest timestamp
        if source.updated_at > target.updated_at {
            Ok(source.clone())
        } else {
            Ok(target.clone())
        }
    }
}
```

### 4. Monitoring & Alerting

#### Health Check System

```
pub struct HealthCheckSystem {
    checks: Vec<Box<dyn HealthCheck>>,
    alert_manager: AlertManager,
}

pub trait HealthCheck {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthCheckResult;
}

pub struct DataIntegrationHealthCheck {
    database_pool: DatabasePool,
    message_client: MessageClient,
}

impl HealthCheck for DataIntegrationHealthCheck {
    fn name(&self) -> &str {
        "data_integration"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let mut result = HealthCheckResult::healthy();
        
        // Check database connectivity
        if let Err(e) = self.database_pool.get_connection().await {
            result.add_error(format!("Database connection failed: {}", e));
        }
        
        // Check message system connectivity
        if let Err(e) = self.message_client.ping().await {
            result.add_error(format!("Message system failed: {}", e));
        }
        
        // Check data consistency
        if let Err(e) = self.check_data_consistency().await {
            result.add_warning(format!("Data consistency issue: {}", e));
        }
        
        result
    }
}
```

## Testing & Validation

### 1. Integration Testing Patterns

#### Test Containers for Integration Tests

```
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_message_database_integration() {
        let docker = clients::Cli::default();
        let postgres = docker.run(images::postgres::Postgres::default());
        let redis = docker.run(images::redis::Redis::default());
        
        // Set up test environment
        let database_url = format!("postgres://postgres:postgres@localhost:{}/test", 
                                  postgres.get_host_port_ipv4(5432));
        let redis_url = format!("redis://localhost:{}", redis.get_host_port_ipv4(6379));
        
        let integration_service = DataIntegrationService::new(
            database_url, 
            redis_url, 
            test_config()
        ).await.unwrap();
        
        // Test message to database flow
        let test_message = create_test_message();
        integration_service.process_message(test_message.clone()).await.unwrap();
        
        // Verify database state
        let entity = integration_service.get_entity(&test_message.entity_id).await.unwrap();
        assert_eq!(entity.data, test_message.data);
        
        // Test database to message flow
        integration_service.update_entity(&entity.id, "new_data").await.unwrap();
        
        // Verify message was published
        let published_messages = integration_service.get_published_messages().await.unwrap();
        assert_eq!(published_messages.len(), 1);
        assert_eq!(published_messages[0].entity_id, entity.id);
    }
}
```

### 2. Contract Testing

#### Message Contract Tests

```
#[cfg(test)]
mod contract_tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_message_schema_compatibility() {
        let schema_v1 = json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"},
                "version": {"type": "integer"}
            },
            "required": ["id", "name"]
        });
        
        let schema_v2 = json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"},
                "description": {"type": "string"},
                "version": {"type": "integer"}
            },
            "required": ["id", "name"]
        });
        
        let compatibility_checker = SchemaCompatibilityChecker::new();
        
        // Test backward compatibility
        assert!(compatibility_checker.is_backward_compatible(&schema_v1, &schema_v2));
        
        // Test forward compatibility
        assert!(compatibility_checker.is_forward_compatible(&schema_v2, &schema_v1));
    }
}
```

### 3. Data Consistency Validation

#### Consistency Validation Framework

```
pub struct ConsistencyValidator {
    database_store: Arc<dyn DataStore>,
    message_store: Arc<dyn MessageStore>,
    validation_rules: Vec<ConsistencyRule>,
}

impl ConsistencyValidator {
    pub async fn validate_consistency(&self) -> Result<ConsistencyReport> {
        let mut report = ConsistencyReport::new();
        
        for rule in &self.validation_rules {
            let violations = rule.validate(&self.database_store, &self.message_store).await?;
            report.add_violations(violations);
        }
        
        Ok(report)
    }
}

pub trait ConsistencyRule {
    async fn validate(&self, 
        database_store: &dyn DataStore, 
        message_store: &dyn MessageStore
    ) -> Result<Vec<ConsistencyViolation>>;
}

pub struct EventualConsistencyRule {
    tolerance_window: Duration,
}

impl ConsistencyRule for EventualConsistencyRule {
    async fn validate(&self, 
        database_store: &dyn DataStore, 
        message_store: &dyn MessageStore
    ) -> Result<Vec<ConsistencyViolation>> {
        let mut violations = Vec::new();
        
        // Check if all recent messages have been processed
        let recent_messages = message_store.get_messages_since(
            Utc::now() - self.tolerance_window
        ).await?;
        
        for message in recent_messages {
            if !database_store.has_processed_message(&message.id).await? {
                violations.push(ConsistencyViolation {
                    rule_name: "eventual_consistency".to_string(),
                    message: format!("Message {} not processed within tolerance window", message.id),
                    severity: Severity::Warning,
                });
            }
        }
        
        Ok(violations)
    }
}
```

### 4. Performance Benchmarking

#### Performance Test Suite

```
#[cfg(test)]
mod performance_tests {
    use super::*;
    use criterion::*;
    
    fn benchmark_message_processing(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let integration_service = rt.block_on(async {
            DataIntegrationService::new(test_config()).await.unwrap()
        });
        
        c.bench_function("message_processing_throughput", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let messages = create_test_messages(1000);
                    integration_service.process_messages(messages).await.unwrap();
                })
            })
        });
    }
    
    fn benchmark_data_consistency_check(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let consistency_validator = rt.block_on(async {
            ConsistencyValidator::new(test_config()).await.unwrap()
        });
        
        c.bench_function("consistency_validation", |b| {
            b.iter(|| {
                rt.block_on(async {
                    consistency_validator.validate_consistency().await.unwrap();
                })
            })
        });
    }
    
    criterion_group!(benches, benchmark_message_processing, benchmark_data_consistency_check);
    criterion_main!(benches);
}
```

## Implementation Guidelines

### 1. Technology-Specific Patterns

#### Rust-Specific Implementation

```
// Use of async/await for non-blocking operations
pub struct AsyncDataIntegrationService {
    database_pool: Arc<PgPool>,
    message_client: Arc<NatsClient>,
    config: IntegrationConfig,
}

impl AsyncDataIntegrationService {
    pub async fn new(config: IntegrationConfig) -> Result<Self> {
        let database_pool = Arc::new(
            PgPoolOptions::new()
                .max_connections(config.max_db_connections)
                .connect(&config.database_url)
                .await?
        );
        
        let message_client = Arc::new(
            NatsClient::connect(&config.nats_url).await?
        );
        
        Ok(Self {
            database_pool,
            message_client,
            config,
        })
    }
    
    pub async fn process_message_stream(&self) -> Result<()> {
        let mut subscription = self.message_client
            .subscribe(&self.config.message_subject)
            .await?;
        
        while let Some(message) = subscription.next().await {
            let service = self.clone();
            
            tokio::spawn(async move {
                if let Err(e) = service.process_single_message(message).await {
                    eprintln!("Error processing message: {}", e);
                }
            });
        }
        
        Ok(())
    }
}
```

### 2. Agent Coordination Patterns

#### Multi-Agent Data Coordination

```
pub struct MultiAgentDataCoordinator {
    agents: HashMap<AgentId, AgentDataHandler>,
    coordination_protocol: CoordinationProtocol,
    conflict_resolver: ConflictResolver,
}

impl MultiAgentDataCoordinator {
    pub async fn coordinate_data_operation(&self, operation: DataOperation) -> Result<()> {
        let affected_agents = self.find_affected_agents(&operation).await?;
        
        // Phase 1: Prepare
        let mut prepared_agents = Vec::new();
        for agent_id in affected_agents {
            let agent = self.agents.get(&agent_id).unwrap();
            
            match agent.prepare_operation(&operation).await {
                Ok(()) => prepared_agents.push(agent_id),
                Err(e) => {
                    // Abort all prepared agents
                    for prepared_agent_id in prepared_agents {
                        let prepared_agent = self.agents.get(&prepared_agent_id).unwrap();
                        prepared_agent.abort_operation(&operation).await?;
                    }
                    return Err(e);
                }
            }
        }
        
        // Phase 2: Commit
        for agent_id in prepared_agents {
            let agent = self.agents.get(&agent_id).unwrap();
            agent.commit_operation(&operation).await?;
        }
        
        Ok(())
    }
}
```

### 3. Security Considerations

#### Data Security Implementation

```
pub struct SecureDataIntegrationService {
    encryption_service: EncryptionService,
    access_control: AccessControl,
    audit_logger: AuditLogger,
}

impl SecureDataIntegrationService {
    pub async fn process_secure_message(&self, message: EncryptedMessage) -> Result<()> {
        // Decrypt message
        let decrypted_message = self.encryption_service.decrypt(message)?;
        
        // Validate access permissions
        self.access_control.validate_access(&decrypted_message.sender, &decrypted_message.resource)?;
        
        // Log access
        self.audit_logger.log_access(&decrypted_message.sender, &decrypted_message.resource).await?;
        
        // Process message
        self.process_message(decrypted_message).await?;
        
        Ok(())
    }
}
```

### 4. Configuration Management

#### Configuration Structure

```
#[derive(Debug, Clone, Deserialize)]
pub struct DataIntegrationConfig {
    pub database: DatabaseConfig,
    pub messaging: MessagingConfig,
    pub consistency: ConsistencyConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: Duration,
    pub query_timeout: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagingConfig {
    pub nats_url: String,
    pub subjects: Vec<String>,
    pub consumer_groups: Vec<String>,
    pub message_timeout: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsistencyConfig {
    pub consistency_level: ConsistencyLevel,
    pub reconciliation_interval: Duration,
    pub conflict_resolution_strategy: ConflictResolutionStrategy,
}

impl DataIntegrationConfig {
    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("DATA_INTEGRATION"))
            .add_source(config::File::with_name("config/data_integration"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

## Summary

This comprehensive data integration patterns specification provides:

1. **Unified Architecture**: Seamless integration between messaging and database systems
2. **Consistency Guarantees**: Multiple consistency models with appropriate trade-offs
3. **Event-Driven Design**: Complete event sourcing and CQRS implementation
4. **Performance Optimization**: Batching, caching, and parallel processing patterns
5. **Reliability**: Robust error handling, retry mechanisms, and recovery procedures
6. **Testing Framework**: Comprehensive testing strategies for data flows
7. **Security**: End-to-end security considerations for data integration
8. **Monitoring**: Health checks and observability for integration components

These patterns ensure that the Mister Smith AI Agent Framework can handle complex data integration scenarios
while maintaining data consistency, performance, and reliability across all agent operations.

---

**Status**: Complete data integration patterns specification
**Next Steps**: Implementation of specific patterns based on agent requirements
**Dependencies**: Core architecture, transport layer, and security framework
**Validation**: Integration testing and performance benchmarking required
