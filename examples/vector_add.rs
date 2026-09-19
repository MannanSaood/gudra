use std::{env, error::Error as StdError, process::ExitCode, time::Instant};

use cutile::prelude::*;

const ELEMENTS: usize = 1_024;
const TILE_SIZE: usize = 128;

#[cutile::module]
mod kernels {
    use cutile::core::*;

    #[cutile::entry()]
    fn add<const TILE: i32>(
        output: &mut Tensor<f32, { [TILE] }>,
        left: &Tensor<f32, { [-1] }>,
        right: &Tensor<f32, { [-1] }>,
    ) {
        let left_tile = left.load_like(output);
        let right_tile = right.load_like(output);
        output.store(left_tile + right_tile);
    }
}

fn run_once() -> Result<(Vec<f32>, std::time::Duration), Box<dyn StdError>> {
    let started = Instant::now();
    let result = kernels::add(
        api::zeros::<f32>(&[ELEMENTS]).partition([TILE_SIZE]),
        api::ones::<f32>(&[ELEMENTS]),
        api::ones::<f32>(&[ELEMENTS]),
    )
    .first()
    .unpartition()
    .to_host_vec()
    .sync()?;
    Ok((result, started.elapsed()))
}

fn cache_location() -> String {
    env::var_os("XDG_CACHE_HOME").map_or_else(
        || {
            env::var_os("HOME").map_or_else(
                || "<unresolved: set XDG_CACHE_HOME or HOME>".to_owned(),
                |home| format!("{}/.cache/cutile/kernels", home.to_string_lossy()),
            )
        },
        |root| format!("{}/cutile/kernels", root.to_string_lossy()),
    )
}

fn run() -> Result<(), Box<dyn StdError>> {
    let persistent_cache = env::args().any(|argument| argument == "--disk-cache");
    if persistent_cache {
        cutile::jit_cache::enable_default()?;
        println!("persistent_cache=enabled location={}", cache_location());
    } else {
        println!("persistent_cache=disabled (cuTile default)");
    }

    let compiles_before = cutile::tile_kernel::jit_compile_count();
    let backend_before = cutile::jit_cache::jit_backend_compile_count();
    let disk_hits_before = cutile::jit_cache::jit_disk_hit_count();

    let (first, first_elapsed) = run_once()?;
    let compiles_after_first = cutile::tile_kernel::jit_compile_count();
    let backend_after_first = cutile::jit_cache::jit_backend_compile_count();
    let disk_hits_after_first = cutile::jit_cache::jit_disk_hit_count();

    let (second, second_elapsed) = run_once()?;
    let compiles_after_second = cutile::tile_kernel::jit_compile_count();

    let expected = vec![2.0_f32; ELEMENTS];
    if first != expected || second != expected {
        return Err("GPU result did not match the CPU reference".into());
    }

    println!("result=PASS elements={ELEMENTS} expected_value=2");
    println!(
        "first_sync_ms={:.3} second_sync_ms={:.3}",
        first_elapsed.as_secs_f64() * 1_000.0,
        second_elapsed.as_secs_f64() * 1_000.0
    );
    println!(
        "jit_after_first={} backend_after_first={} disk_hits_after_first={} jit_after_second={}",
        compiles_after_first - compiles_before,
        backend_after_first - backend_before,
        disk_hits_after_first - disk_hits_before,
        compiles_after_second - compiles_after_first
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("GPU smoke test failed: {error}");
            eprintln!("Run ./scripts/check-gpu-env.sh, then follow docs/setup.md.");
            ExitCode::from(2)
        }
    }
}
