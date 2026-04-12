# spill — Intelligent GPU Memory Tiering

A llama.cpp plugin that makes local LLM inference faster by learning what data belongs in which memory tier.

## What it does

LLM inference on consumer hardware is memory-bound. A 26B model needs more VRAM than most GPUs have. Current solutions offload to system RAM and hope for the best — every cache miss stalls the GPU.

spill learns your access patterns in real time and prefetches data before the GPU needs it. First prompt: normal speed. By the tenth prompt: near-VRAM speed for everything. No parameters lost. No quality reduction. Just smarter memory.

## Architecture

spill is a shared library (.so/.dylib/.dll) that plugs into llama.cpp's backend system. It intercepts memory allocation and data access, adding a three-tier intelligent cache:

```
Tier 0: VRAM     (fastest — active weights, hot KV cache)
Tier 1: Host RAM (fast — warm experts, recent KV pages, prefetch buffer)
Tier 2: NVMe     (acceptable — cold experts, overflow KV, model archive)
```

### Components

```
spill/
├── src/
│   ├── lib.rs              # Plugin entry point, llama.cpp FFI interface
│   ├── tier/               # Memory tier management
│   │   ├── mod.rs          # TierManager: allocate, evict, promote, demote
│   │   ├── vram.rs         # VRAM tier via vyre::runtime::buffer
│   │   ├── host.rs         # Host RAM tier via vyre::runtime::transfer::host
│   │   └── storage.rs      # NVMe tier via vyre::runtime::transfer::storage
│   ├── tracker/            # Access pattern tracking
│   │   ├── mod.rs          # AccessTracker trait
│   │   ├── frequency.rs    # Frequency-based tracking (which buffers accessed most)
│   │   ├── recency.rs      # LRU tracking (most recently accessed)
│   │   └── combined.rs     # Weighted frequency + recency (ARC-like policy)
│   ├── predictor/          # CPU-side prediction (runs on idle CPU)
│   │   ├── mod.rs          # Predictor trait
│   │   ├── markov.rs       # Markov chain on access sequences
│   │   ├── locality.rs     # "Keep last N accessed" heuristic
│   │   └── moe_router.rs   # MoE-specific: shadow router (~2K params on CPU)
│   ├── prefetch/           # Async DMA orchestration
│   │   ├── mod.rs          # PrefetchController
│   │   └── pipeline.rs     # Overlap compute with transfer using tenshift backpressure
│   ├── profile/            # Workload profiling
│   │   ├── mod.rs          # ProfileManager
│   │   ├── online.rs       # Profile during inference (first N tokens)
│   │   └── format.rs       # Save/load profiles to disk
│   └── ffi/                # llama.cpp plugin interface
│       ├── mod.rs          # C FFI exports
│       └── ggml_backend.rs # ggml backend implementation
├── Cargo.toml
└── profiles/               # Saved workload profiles
```

### Dependencies (Santh only)

```toml
[dependencies]
vyre = { version = "0.4", features = ["transfer-dma", "transfer-gds", "cache"] }
tenshift-core = { version = "0.1", features = ["backpressure"] }
```

spill depends on exactly two Santh crates:
- **vyre** for GPU memory primitives (pinned DMA, GDS, buffer management)
- **tenshift-core** for backpressure-aware data pipeline (overlap compute with transfer)

Everything else (predictor, tracker, profiler) is self-contained. Zero external deps beyond libc for FFI.

## How it works

### Cold start (first prompt)
1. llama.cpp loads model weights via spill's ggml backend
2. spill places weights in the best available tier (VRAM first, overflow to RAM, then NVMe)
3. Inference runs. Every buffer access is tracked.
4. After the first layer completes: spill has enough data to start predicting.

### Warm-up (prompts 2-10)
1. The access tracker has frequency + recency data for every buffer
2. The predictor starts issuing prefetch commands based on observed patterns
3. Hot buffers get promoted to VRAM. Cold buffers get demoted to RAM.
4. Hit rate climbs from ~60% to ~90% over 10 prompts.

### Steady state (prompt 10+)
1. The predictor is accurate. 90-95% of accesses hit VRAM.
2. The 5-10% misses are served from host RAM via DMA (~6ms each).
3. Effective throughput: within 5% of all-in-VRAM speed.
4. The profile can be saved to disk for instant warm start next time.

### MoE-specific optimization
For MoE models (Gemma 4 26B-A4B, Mixtral, etc.):
1. The router output is intercepted before expert computation
2. The shadow router on CPU predicts the NEXT token's expert selection
3. Predicted experts are prefetched from host RAM to VRAM during current token's compute
4. Hit rate: 90-95% because expert selection has high locality in code workloads

### Dense model optimization  
For dense models:
1. Layer-granularity tracking: which layers are accessed in which order (always sequential, but attention patterns vary)
2. KV cache page tracking: which context positions are attended to most
3. Weight-group tracking: which attention heads contribute most (via activation magnitude)
4. Hot weight groups stay in VRAM at full precision. Cold groups can be in RAM at lower precision.

## Plugin interface

spill exports a standard ggml backend:

```c
// llama.cpp loads this via dlopen
GGML_BACKEND_API ggml_backend_t spill_backend_init(void);
GGML_BACKEND_API const char * spill_backend_name(void);
GGML_BACKEND_API bool spill_backend_supports_op(ggml_backend_t backend, const struct ggml_tensor * op);
```

Users enable it by setting an environment variable or llama.cpp flag:
```bash
# With Ollama
OLLAMA_HELIX=1 ollama serve

# With llama.cpp directly
./llama-server --backend spill --model gemma4-26b.gguf

# With saved profile (instant warm start)
HELIX_PROFILE=~/.spill/code-review.json ollama serve
```

## What spill is NOT

- Not a model format (uses standard GGUF)
- Not an inference engine (llama.cpp does inference)
- Not a model server (Ollama does serving)
- Not a consumer app (it's a plugin — invisible infrastructure)

## Quality standards

Same Santh standards as every other crate:
- #![forbid(unsafe_code)] except ffi/ module (C FFI requires unsafe)
- #![warn(missing_docs, clippy::pedantic)]
- All tracker/predictor/profiler code has CPU reference + tests
- Conformance: tracker must produce identical promotion/demotion decisions on CPU replay
- Every file under 500 lines
