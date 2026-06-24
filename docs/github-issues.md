# GitHub Issues

このファイルは GitHub に登録する Issue の原本である。

## Labels

- `type: foundation`
- `type: validation`
- `type: algorithm`
- `type: search`
- `type: integration`
- `type: architecture`
- `type: performance`
- `type: benchmark`
- `type: backend`
- `type: gpu`
- `type: docs`
- `type: research`
- `type: correctness`
- `type: ux`
- `priority: high`
- `priority: medium`
- `priority: low`
- `milestone: v0.1`
- `milestone: v0.2`
- `milestone: v0.3`
- `milestone: v0.4`
- `milestone: v0.5`
- `milestone: v0.6`

## Milestones

- `v0.1`: CPU single MVP
- `v0.2`: CPU multi backend
- `v0.3`: Benchmark framework
- `v0.4`: Backend selector and GPU stubs
- `v0.5`: CUDA search-only prototype
- `v0.6`: CUDA compute / AMD research branch

## Issues

### Issue 1: Create Rust CLI project skeleton

Labels: `type: foundation`, `milestone: v0.1`, `priority: high`

Milestone: `v0.1`

#### Goal

Create the initial Rust CLI project for `pi-birthday-bench`.

#### Requirements

- Create a Rust binary crate.
- Add CLI parsing.
- Add the following options:
  - `--target`
  - `--max-digits`
  - `--chunk`
  - `--backend`
  - `--threads`
  - `--json`
  - `--no-progress`
  - `--verify`
  - `--benchmark-only`
  - `--list-backends`

#### Acceptance Criteria

- `cargo build` succeeds.
- `cargo run -- --help` displays all options.
- Invalid options return a clear error.
- No pi calculation is required in this issue.

### Issue 2: Implement YYYYMMDD date validation

Labels: `type: validation`, `milestone: v0.1`, `priority: high`

Milestone: `v0.1`

#### Goal

Validate `--target` as an actual `YYYYMMDD` date.

#### Requirements

- Target must be exactly 8 digits.
- Validate year, month, and day.
- Leap years must be handled correctly.
- Invalid dates must return a clear error.

#### Examples

Valid:

- `20000229`
- `20240628`
- `19931203`

Invalid:

- `19930229`
- `19000229`
- `20241301`
- `20240631`
- `0628`
- `abc`

#### Acceptance Criteria

- Unit tests cover valid dates.
- Unit tests cover invalid dates.
- CLI rejects invalid dates before any pi calculation starts.

### Issue 3: Implement CPU single pi calculation

Labels: `type: algorithm`, `milestone: v0.1`, `priority: high`

Milestone: `v0.1`

#### Goal

Implement pi digit generation using CPU single-threaded calculation.

#### Requirements

- Use Chudnovsky algorithm.
- Use binary splitting where practical.
- Use `rug` or another GMP-backed multiprecision library.
- Compute the decimal digits of pi at runtime.
- Do not use bundled pi digit files.
- Do not fetch pi digits from the internet.
- Do not use a precomputed pi constant.

#### Precision

- Add guard digits internally.
- Return only the requested number of fractional digits.
- Exclude the integer part `3`.

#### Acceptance Criteria

- Can generate at least 10,000 fractional digits.
- The first 100 fractional digits match known pi prefix.
- Code path is single-threaded.
- `cargo test` passes.

### Issue 4: Implement pattern search with chunk overlap

Labels: `type: search`, `milestone: v0.1`, `priority: high`

Milestone: `v0.1`

#### Goal

Search for a numeric pattern in generated pi digits without missing matches across chunk boundaries.

#### Requirements

- Search target pattern in pi fractional digits.
- Position is 1-based from the first fractional digit.
- Keep `target.len() - 1` digits from the previous chunk.
- Detect matches that span chunk boundaries.

#### Important

The CLI target is `YYYYMMDD`, but this search function must accept arbitrary numeric strings for tests.

#### Acceptance Criteria

- `search_pattern("0628")` returns `71`.
- `search_pattern("0812")` returns `146`.
- `search_pattern("1027")` returns `163`.
- A synthetic chunk-boundary test passes.

### Issue 5: Connect CPU single backend to CLI search

Labels: `type: integration`, `milestone: v0.1`, `priority: high`

Milestone: `v0.1`

#### Goal

Wire the CPU single pi calculation and pattern search into the CLI.

#### Requirements

- `--backend cpu-single` must work.
- `--target YYYYMMDD` must be searched in pi fractional digits.
- Stop when found unless `--benchmark-only` is set.
- Stop at `--max-digits` if not found.
- Print a human-readable result.

#### Acceptance Criteria

- Command runs:

```bash
pi-birthday-bench --target 19930628 --max-digits 1000000 --backend cpu-single
```

- Result contains:
  - target
  - found
  - first_position
  - backend
  - digits_computed
  - elapsed_seconds
  - digits_per_second

### Issue 6: Add README basic documentation

Labels: `type: docs`, `milestone: v0.1`, `priority: medium`

Milestone: `v0.1`

#### Goal

Write the initial README.

#### Requirements

README must explain:

- What this tool does.
- It computes pi at runtime.
- It does not use precomputed pi digit files.
- Fractional digit position definition.
- Basic usage.
- Why `YYYYMMDD` may require many digits.
- Why `--max-digits` is required.

#### Acceptance Criteria

- README includes at least one working command example.
- README defines "1st digit" clearly.
- README warns that 8-digit patterns may require very deep searches.

### Issue 7: Introduce backend abstraction

Labels: `type: architecture`, `milestone: v0.2`, `priority: high`

Milestone: `v0.2`

#### Goal

Introduce a backend abstraction so CPU and GPU modes can share the same CLI and result format.

#### Requirements

Create a trait similar to:

```rust
pub trait PiBackend {
    fn name(&self) -> &'static str;
    fn gpu_role(&self) -> GpuRole;
    fn is_available(&self) -> bool;
    fn compute_digits(&self, digits: usize) -> Result<String>;
}
```

Implement:

- `CpuSingleBackend`
- placeholder `CpuMultiBackend`

#### Acceptance Criteria

- Existing cpu-single behavior still works.
- Backend name is included in output.
- Adding new backends does not require rewriting CLI logic.

### Issue 8: Implement CPU multi backend

Labels: `type: performance`, `milestone: v0.2`, `priority: high`

Milestone: `v0.2`

#### Goal

Implement CPU multi-threaded backend.

#### Requirements

- Add `--backend cpu-multi`.
- Add `--threads N`.
- Use rayon or equivalent.
- Parallelize binary splitting or another meaningful CPU-heavy part.
- Preserve deterministic output.

#### Acceptance Criteria

- `cpu-multi` returns the same first_position as `cpu-single`.
- `--threads 1` behaves consistently with single-thread result.
- `--threads N` uses N threads where practical.
- Tests compare cpu-single and cpu-multi on known patterns.

### Issue 9: Add benchmark-only mode

Labels: `type: benchmark`, `milestone: v0.3`, `priority: high`

Milestone: `v0.3`

#### Goal

Add `--benchmark-only` mode.

#### Requirements

- In normal mode, stop when target is found.
- In benchmark-only mode, continue until `--max-digits` even if target is found.
- Still record the first position if found.
- Report total runtime and throughput for the full run.

#### Acceptance Criteria

- `--benchmark-only` does not stop early.
- Output includes first_position if found.
- Output includes digits_per_second for the full run.

### Issue 10: Add JSON output

Labels: `type: output`, `milestone: v0.3`, `priority: high`

Milestone: `v0.3`

#### Goal

Add machine-readable JSON output.

#### Requirements

When `--json` is specified, output a JSON object with:

- target
- found
- first_position
- backend
- algorithm
- digits_computed
- elapsed_seconds
- digits_per_second
- chunks_processed
- threads
- cpu_model
- gpu_name
- gpu_role
- memory_peak_mb
- verification_status

#### Acceptance Criteria

- JSON output is valid.
- JSON output has stable field names.
- Human-readable progress is suppressed or redirected when `--json` is used.

### Issue 11: Add progress reporting

Labels: `type: ux`, `milestone: v0.3`, `priority: medium`

Milestone: `v0.3`

#### Goal

Show progress during long runs.

#### Requirements

Display periodically:

- backend
- target
- current digit range
- digits_computed
- elapsed_seconds
- digits_per_second
- chunk
- threads

Add `--no-progress` to suppress progress.

#### Acceptance Criteria

- Long runs show progress.
- `--no-progress` suppresses progress.
- `--json` output is not polluted by progress text.

### Issue 12: Add system information collection

Labels: `type: benchmark`, `milestone: v0.3`, `priority: medium`

Milestone: `v0.3`

#### Goal

Collect system information for benchmark reports.

#### Requirements

Collect when possible:

- CPU model
- logical CPU count
- physical CPU count if available
- memory information if available
- peak memory usage if available

Do not fail if system info cannot be obtained.

#### Acceptance Criteria

- Output includes CPU model when available.
- Missing system info results in null or "unknown", not a crash.

### Issue 13: Implement --list-backends

Labels: `type: backend`, `milestone: v0.4`, `priority: high`

Milestone: `v0.4`

#### Goal

Add backend discovery output.

#### Requirements

`--list-backends` should print:

```text
available backends:
- cpu-single: available
- cpu-multi: available
- cuda-compute: unavailable, build with --features cuda
- cuda-search-only: unavailable, build with --features cuda
- hip: unavailable, not implemented
- opencl: unavailable, not implemented
- vulkan: unavailable, not implemented
```

#### Acceptance Criteria

- Command does not require target.
- Command does not start pi calculation.
- Unavailable GPU backends do not cause panic.

### Issue 14: Add GPU backend stubs and feature flags

Labels: `type: gpu`, `milestone: v0.4`, `priority: high`

Milestone: `v0.4`

#### Goal

Add GPU backend modes as selectable stubs.

#### Requirements

Add backends:

- `cuda-compute`
- `cuda-search-only`
- `hip`
- `opencl`
- `vulkan`

Add feature flags:

- `cuda`
- `hip`
- `opencl`
- `vulkan`

Default build must not require CUDA, HIP, OpenCL, or Vulkan SDK.

#### Acceptance Criteria

- `cargo build --release` succeeds on CPU-only machine.
- GPU backends return clear unavailable errors.
- `--list-backends` shows GPU backend status.
- No GPU SDK is required unless the corresponding feature is enabled.

### Issue 15: Implement CUDA search-only prototype

Labels: `type: gpu`, `milestone: v0.5`, `priority: medium`

Milestone: `v0.5`

#### Goal

Implement an experimental CUDA search-only backend.

#### Requirements

- CPU computes pi digits.
- CUDA searches for target pattern inside digit chunks.
- Output must set:
  - backend: cuda-search-only
  - gpu_role: search_only
- Must not claim that pi calculation itself is GPU-accelerated.
- Must compare result against CPU search when `--verify` is set.

#### Acceptance Criteria

- Builds with `--features cuda`.
- Runs on CUDA-capable NVIDIA GPU.
- Returns the same first_position as CPU search.
- Prints GPU device name.
- JSON output identifies `gpu_role: search_only`.

### Issue 16: Document GPU compute limitations

Labels: `type: docs`, `milestone: v0.5`, `priority: medium`

Milestone: `v0.5`

#### Goal

Document what GPU acceleration means in this project.

#### Requirements

README must distinguish:

- CPU pi compute + CPU search
- CPU pi compute + GPU search
- GPU pi compute + GPU/CPU search

Explain that `cuda-search-only` is not a pi calculation benchmark.

#### Acceptance Criteria

- README contains a GPU role table.
- README warns against comparing `cuda-search-only` directly with `cpu-multi` as if both accelerate the same workload.

### Issue 17: Research CUDA compute backend

Labels: `type: research`, `milestone: v0.6`, `priority: low`

Milestone: `v0.6`

#### Goal

Research feasibility of using CUDA for pi calculation itself.

#### Requirements

Investigate:

- Chudnovsky binary splitting on GPU
- multiprecision integer arithmetic on GPU
- memory transfer overhead
- whether GPU compute is actually faster than optimized CPU/GMP
- implementation complexity

#### Deliverable

Create `docs/gpu-compute-research.md`.

#### Acceptance Criteria

Document includes:

- recommended approach
- rejected approaches
- expected performance risks
- implementation estimate
- whether to proceed

### Issue 18: Research AMD GPU support

Labels: `type: research`, `milestone: v0.6`, `priority: low`

Milestone: `v0.6`

#### Goal

Research AMD GPU support options.

#### Candidates

- HIP
- OpenCL
- Vulkan Compute

#### Requirements

Compare:

- Rust ecosystem support
- Windows support
- Linux support
- installation burden
- expected maintainability
- suitability for search-only mode
- suitability for pi compute mode

#### Deliverable

Create `docs/amd-gpu-support.md`.

#### Acceptance Criteria

Document recommends one of:

- implement HIP
- implement OpenCL
- implement Vulkan Compute
- keep AMD unsupported for now

The decision must include reasons.

### Issue 19: Add verification mode

Labels: `type: correctness`, `milestone: v0.3`, `priority: high`

Milestone: `v0.3`

#### Goal

Add `--verify` mode to reduce false benchmark results.

#### Requirements

When `--verify` is set:

- Validate generated pi prefix against known prefix.
- If using cpu-multi, compare a small range against cpu-single.
- If using GPU search-only, compare found result against CPU search.
- Return verification_status.

#### Acceptance Criteria

- `verification_status` can be:
  - passed
  - failed
  - skipped
- Failed verification exits with non-zero status.
- JSON output includes verification_status.

### Issue 20: Add result schema and benchmark examples

Labels: `type: docs`, `milestone: v0.3`, `priority: medium`

Milestone: `v0.3`

#### Goal

Document result fields and benchmark usage.

#### Requirements

README or docs must include:

- result field definitions
- JSON example
- CPU single example
- CPU multi example
- benchmark-only example
- list-backends example

#### Acceptance Criteria

- A user can run a benchmark from README alone.
- The meaning of digits_per_second is documented.
- The meaning of first_position is documented.
