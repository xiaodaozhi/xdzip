//! Real read-only archive benchmark. It never writes to the supplied source.
use std::path::Path;
use std::time::Instant;
use xdzip_codec_api::Cancellation;
use xdzip_runtime::{Archive, query::QueryRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("provide an owned ZIP fixture")?;
    let control = Cancellation::default();
    let start = Instant::now();
    let mut archive = Archive::open(Path::new(&path), &control)?;
    let open_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let page = archive
        .query_page(
            QueryRequest {
                directory: "",
                text: "",
                sort: 2,
                offset: 0,
                expected: None,
                generation: 1,
            },
            &control,
        )
        .map_err(|_| "query failed")?;
    let key = page.key;
    let total = page.total;
    let mut count = page.rows.len();
    let mut batches = 1;
    let sort_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    while count < total {
        let page = archive
            .query_page(
                QueryRequest {
                    directory: "",
                    text: "",
                    sort: 2,
                    offset: count as u64,
                    expected: Some(key),
                    generation: 1,
                },
                &control,
            )
            .map_err(|_| "continuation failed")?;
        if page.key != key || page.rows.is_empty() {
            return Err("inconsistent query result".into());
        }
        count += page.rows.len();
        batches += 1;
    }
    let batches_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let found = archive
        .query_page(
            QueryRequest {
                directory: "",
                text: "099999",
                sort: 1,
                offset: 0,
                expected: None,
                generation: 2,
            },
            &control,
        )
        .map_err(|_| "search failed")?
        .total;
    let search_ms = start.elapsed().as_secs_f64() * 1000.0;
    let peak_kib = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("VmHWM:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse::<u64>().ok())
        });
    println!(
        "{{\"physical_rows\":{},\"listing_rows\":{total},\"received_rows\":{count},\"batches\":{batches},\"open_ms\":{open_ms:.3},\"sort_ms\":{sort_ms:.3},\"continuations_ms\":{batches_ms:.3},\"search_ms\":{search_ms:.3},\"search_matches\":{found},\"peak_rss_kib\":{}}}",
        archive.len(),
        peak_kib.map_or("null".into(), |value| value.to_string())
    );
    Ok(())
}
