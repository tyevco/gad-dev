use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use godot_asset_browser::asset_library::asset::{Asset, AssetCategory};
use std::sync::Arc;

/// Creates a sample asset for benchmarking
fn create_sample_asset(id: usize) -> Asset {
    Asset::new(
        format!("asset_{}", id),
        format!("Sample Asset {}", id),
        match id % 5 {
            0 => AssetCategory::TwoD,
            1 => AssetCategory::ThreeD,
            2 => AssetCategory::Shaders,
            3 => AssetCategory::Audio,
            _ => AssetCategory::Scripts,
        },
        String::new(),
        format!("Author {}", id % 10),
        "1.0.0".to_string(),
        format!("This is a description for asset {}", id),
        vec![
            format!("tag{}", id % 5),
            format!("category{}", id % 3),
            "common".to_string(),
        ],
        Some(format!("https://example.com/preview{}.png", id)),
        format!("https://example.com/download{}.zip", id),
        vec![],
        vec![],
    )
}

/// Creates a vector of sample assets
fn create_asset_list(count: usize) -> Vec<Asset> {
    (0..count).map(create_sample_asset).collect()
}

/// Benchmark for cloning asset lists (current implementation)
fn bench_asset_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_clone");

    for size in [10, 50, 100, 500, 1000].iter() {
        let assets = create_asset_list(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _cloned = black_box(assets.clone());
            });
        });
    }

    group.finish();
}

/// Benchmark for searching assets (linear scan)
fn bench_asset_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_search");

    for size in [10, 50, 100, 500, 1000].iter() {
        let assets = create_asset_list(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let query = "tag2";
                let query_lower = query.to_lowercase();
                let _results: Vec<Asset> = assets
                    .iter()
                    .filter(|asset| {
                        asset.name.to_lowercase().contains(&query_lower)
                            || asset.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                            || asset.description.to_lowercase().contains(&query_lower)
                    })
                    .cloned()
                    .collect();
                black_box(_results);
            });
        });
    }

    group.finish();
}

/// Benchmark for filtering by category
fn bench_category_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("category_filter");

    for size in [10, 50, 100, 500, 1000].iter() {
        let assets = create_asset_list(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _results: Vec<Asset> = assets
                    .iter()
                    .filter(|asset| asset.category == AssetCategory::TwoD)
                    .cloned()
                    .collect();
                black_box(_results);
            });
        });
    }

    group.finish();
}

/// Benchmark for Arc-based shared reference (optimization)
fn bench_asset_arc_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_arc_clone");

    for size in [10, 50, 100, 500, 1000].iter() {
        let assets = create_asset_list(*size);
        let arc_assets = Arc::new(assets);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _cloned = black_box(Arc::clone(&arc_assets));
            });
        });
    }

    group.finish();
}

/// Benchmark memory allocation for search results
fn bench_search_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_allocation");

    for size in [10, 50, 100, 500, 1000].iter() {
        let assets = create_asset_list(*size);

        // Pre-allocate lowercase versions (optimization strategy)
        group.bench_with_input(
            BenchmarkId::new("with_precomputed_lowercase", size),
            size,
            |b, _| {
                let precomputed: Vec<_> = assets
                    .iter()
                    .map(|asset| {
                        (
                            asset.clone(),
                            asset.name.to_lowercase(),
                            asset.tags.iter().map(|t| t.to_lowercase()).collect::<Vec<_>>(),
                            asset.description.to_lowercase(),
                        )
                    })
                    .collect();

                b.iter(|| {
                    let query = "tag2";
                    let query_lower = query.to_lowercase();
                    let _results: Vec<Asset> = precomputed
                        .iter()
                        .filter(|(_, name_lower, tags_lower, desc_lower)| {
                            name_lower.contains(&query_lower)
                                || tags_lower.iter().any(|tag| tag.contains(&query_lower))
                                || desc_lower.contains(&query_lower)
                        })
                        .map(|(asset, _, _, _)| asset.clone())
                        .collect();
                    black_box(_results);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_asset_clone,
    bench_asset_search,
    bench_category_filter,
    bench_asset_arc_clone,
    bench_search_allocation
);

criterion_main!(benches);
