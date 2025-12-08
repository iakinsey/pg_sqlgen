use crate::types::errors::SqlgenError;
use simsimd::SpatialSimilarity;

pub fn convert_vectors_to_pgvector() -> Result<(), SqlgenError> {
    unimplemented!()
}

pub fn convert_vectors_to_in_memory() -> Result<(), SqlgenError> {
    unimplemented!()
}

pub fn guess_vectors_and_convert() -> Result<(), SqlgenError> {
    unimplemented!()
}

pub fn cosine_similarity(query: Vec<f32>, vectors: Vec<Vec<f32>>, limit: usize) -> Vec<f32> {
    let mut sims: Vec<f32> = vectors
        .iter()
        .map(|v| {
            let dist = f32::cosine(&query, v).unwrap();
            (1.0 - dist) as f32
        })
        .collect();

    sims.sort_by(|a, b| b.partial_cmp(a).unwrap());
    sims.truncate(limit);
    sims
}
