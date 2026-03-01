use anyhow::anyhow;
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Value;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

use arrow_array::{Array, FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};

// --- Models & State ---

#[derive(Deserialize)]
struct ProductInput {
    name: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[derive(Serialize)]
struct ProductResponse {
    name: String,
    score: f32,
}

struct AppState {
    model: Mutex<Session>,
    tokenizer: Tokenizer,
    db_table: lancedb::Table,
}

#[derive(Deserialize)]
struct UpdateProduct {
    old_name: String,
    new_name: String,
}

// --- Handlers ---

async fn clear_products(State(state): State<Arc<AppState>>) -> &'static str {
    state.db_table.delete("1=1").await.unwrap();
    "All products cleared"
}

async fn update_product(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateProduct>,
) -> &'static str {
    // Delete old
    state
        .db_table
        .delete(&format!("name = '{}'", payload.old_name))
        .await
        .unwrap();

    // Re-embed and insert new
    let vector = {
        let mut model = state.model.lock().await;
        embed_text(&payload.new_name, &mut model, &state.tokenizer, false)
            .expect("Embedding failed")
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 384),
            false,
        ),
    ]));

    let name_array = StringArray::from(vec![payload.new_name]);
    let vector_array =
        FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
            vec![Some(vector.into_iter().map(Some).collect::<Vec<_>>())],
            384,
        );

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(name_array), Arc::new(vector_array)],
    )
    .unwrap();

    let batches = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());
    state.db_table.add(batches).execute().await.unwrap();

    "Product updated"
}

async fn add_product(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProductInput>,
) -> &'static str {
    let vector = {
        let mut model = state.model.lock().await;
        embed_text(&payload.name, &mut model, &state.tokenizer, false).expect("Embedding failed")
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 384),
            false,
        ),
    ]));

    let name_array = StringArray::from(vec![payload.name]);
    let vector_array =
        FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
            vec![Some(
                vector.into_iter().map(Some).collect::<Vec<Option<f32>>>(),
            )],
            384,
        );

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(name_array), Arc::new(vector_array)],
    )
    .unwrap();

    let batches = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());
    state.db_table.add(batches).execute().await.unwrap();

    "Product added successfully"
}

async fn list_products(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let mut stream = state.db_table.query().limit(50).execute().await.unwrap();
    let mut names = Vec::new();

    while let Some(Ok(batch)) = futures::StreamExt::next(&mut stream).await {
        let name_col = batch
            .column_by_name("name")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        for i in 0..name_col.len() {
            names.push(name_col.value(i).to_string());
        }
    }
    Json(names)
}

async fn get_similar(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchQuery>,
) -> Json<Vec<ProductResponse>> {
    let query_vec = {
        let mut model = state.model.lock().await;
        embed_text(&payload.query, &mut model, &state.tokenizer, true).expect("Embedding failed")
    };

    let mut results = state
        .db_table
        .query()
        .nearest_to(query_vec)
        .unwrap()
        .distance_type(lancedb::DistanceType::Cosine)
        .limit(5)
        .execute()
        .await
        .unwrap();

    let mut products = Vec::new();

    while let Some(Ok(batch)) = futures::StreamExt::next(&mut results).await {
        let name_col = batch
            .column_by_name("name")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();

        let distance_col = batch
            .column_by_name("_distance")
            .unwrap()
            .as_any()
            .downcast_ref::<arrow_array::Float32Array>()
            .unwrap();

        for i in 0..name_col.len() {
            products.push(ProductResponse {
                name: name_col.value(i).to_string(),
                score: 1.0 - distance_col.value(i), // cosine distance -> similarity
            });
        }
    }

    Json(products)
}

async fn delete_product(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProductInput>,
) -> &'static str {
    state
        .db_table
        .delete(&format!("name = '{}'", payload.name))
        .await
        .unwrap();
    "Product deleted"
}

// --- AI Logic ---

fn embed_text(
    text: &str,
    model: &mut Session,
    tokenizer: &Tokenizer,
    is_query: bool,
) -> anyhow::Result<Vec<f32>> {
    let input = if is_query {
        format!(
            "Represent this sentence for searching relevant passages: {}",
            text
        )
    } else {
        text.to_string()
    };

    let tokens = tokenizer
        .encode(input.as_str(), true)
        .map_err(|e| anyhow!(e))?;

    let input_ids = tokens
        .get_ids()
        .iter()
        .map(|&i| i as i64)
        .collect::<Vec<_>>();
    let attention_mask_values = tokens
        .get_attention_mask()
        .iter()
        .map(|&i| i as i64)
        .collect::<Vec<_>>();

    let seq_len = input_ids.len();
    let token_type_ids = vec![0i64; seq_len];

    // Save mask as f32 BEFORE moving into inputs
    let mask_f32: Vec<f32> = attention_mask_values.iter().map(|&x| x as f32).collect();
    let real_token_count: f32 = mask_f32.iter().sum();

    let outputs = model.run(ort::inputs![
        "input_ids" => Value::from_array(
            (vec![1usize, seq_len], input_ids.into_boxed_slice())
        )?,
        "attention_mask" => Value::from_array(
            (vec![1usize, seq_len], attention_mask_values.into_boxed_slice())
        )?,
        "token_type_ids" => Value::from_array(
            (vec![1usize, seq_len], token_type_ids.into_boxed_slice())
        )?,
    ])?;

    let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;

    // Mean pooling using attention mask to ignore padding
    let mut embedding = vec![0.0f32; 384];
    for token_idx in 0..seq_len {
        let mask_val = mask_f32[token_idx];
        for dim in 0..384 {
            embedding[dim] += data[token_idx * 384 + dim] * mask_val;
        }
    }
    for dim in 0..384 {
        embedding[dim] /= real_token_count;
    }

    // L2 normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    for dim in 0..384 {
        embedding[dim] /= norm;
    }

    Ok(embedding)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tokenizer = Tokenizer::from_file("model/tokenizer.json").map_err(|e| anyhow!(e))?;
    let model = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(2)?
        .commit_from_file("model/model.onnx")?;

    let db = connect("data/grocery_db").execute().await?;

    let table = if db
        .table_names()
        .execute()
        .await?
        .contains(&"products".to_string())
    {
        db.open_table("products").execute().await?
    } else {
        println!("Initializing new LanceDB table...");
        let schema = Arc::new(Schema::new(vec![
            Field::new("name", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 384),
                false,
            ),
        ]));
        let empty_batches = RecordBatchIterator::new(
            vec![] as Vec<Result<RecordBatch, arrow_schema::ArrowError>>,
            schema.clone(),
        );
        db.create_table("products", empty_batches).execute().await?
    };

    let shared_state = Arc::new(AppState {
        model: Mutex::new(model),
        tokenizer,
        db_table: table,
    });

    let app = Router::new()
        .route(
            "/products",
            post(add_product)
                .get(list_products)
                .delete(delete_product)
                .put(update_product),
        )
        .route("/products/clear", post(clear_products))
        .route("/getSimilar", post(get_similar))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Pentium AI Server running on port 3000...");
    axum::serve(listener, app).await?;

    Ok(())
}
