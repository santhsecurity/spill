# spill — Vision

## Identity

spill makes models larger than VRAM run fast. Your model spills past VRAM, spill makes it fast.

spill is a consumer product under the Santh brand. It lives in its own repository, not in the Santh monorepo. It consumes vyre GPU primitives as a downstream dependency, the same way any external developer would.

## The Problem

A 70B model at Q4 is ~35GB. A 4090 has 24GB VRAM. ~11GB must live in RAM. Every token generated reads the entire model. The 11GB in RAM crosses PCIe at 25 GB/s — that's 440ms per token just for the offloaded portion. Unusable.

Current solutions (llama.cpp, ollama) do dumb layer-level offloading. All layers in VRAM or all in RAM. No intelligence. No compression. No pipelining. No parallelism.

## The Thesis

The AI community is transferring raw data over a narrow pipe and blaming the pipe. The right response is:
1. Compress the data before transfer
2. Decompress on the massively parallel device at the other end
3. Predict what's needed next and start transferring before it's requested
4. Use ALL available compute (GPU + CPU + iGPU) in parallel, not serially

Software intelligence sidesteps hardware bottlenecks. TurboQuant proved this for KV cache (6x compression, zero accuracy loss). APEX proved this for CPU-GPU parallelism (84-96% throughput improvement). Nobody has composed all of these.

## Architecture

spill is maximally modular. Each component is independently useful.

### Components

**spill-predict** — Access pattern prediction. Markov chains on layer activation sequences. Locality tracking. Pure CPU logic, no GPU dependency. Useful for any tiered memory system.

**spill-profile** — Workload profiling and persistence. Learn which layers/experts are hot for a specific model. Persist across runs. First inference is cold, subsequent runs are warm. Pure CPU + filesystem.

**spill** (orchestrator) — Wires prediction + vyre primitives + backend integration. Feature-gated per backend (llama.cpp, ollama, vllm).

### vyre primitives consumed

- `vyre::runtime::transfer` — async DMA, pinned memory pools
- `vyre::runtime::cache` — tiered cache with pluggable eviction
- `vyre::runtime::alloc` — GPU buffer suballocator (when available)
- `vyre::runtime::pipeline` — multi-stage overlapped execution (when available)

spill does NOT depend on vyre's security-specific code (ops, engine, rules). Only the runtime layer.

### Interfaces

- **C FFI** (cdylib) — llama.cpp loads spill as a shared library
- **Rust API** — `cargo add spill` for Rust inference engines
- **CLI** — `spill serve <model>` wraps ollama/llama.cpp with intelligent tiering
- **Environment hook** — `OLLAMA_SPILL=1 ollama serve` for drop-in acceleration

## Research Primitives to Implement

These are the techniques from current research (2025-2026) that spill should compose:

### 1. Compressed Transfer (TurboQuant-inspired)
Compress weights/KV cache before PCIe transfer. Decompress on GPU. Effective PCIe bandwidth = raw bandwidth x compression ratio. Even lossless compression (LZ4) gives 2-3x. TurboQuant-style vector quantization for KV cache gives 6x.

### 2. Heterogeneous Parallel Execution (APEX/TwinPilots-inspired)  
CPU and GPU compute different layers simultaneously. Not serial offload — true parallelism. Profile each device, assign layers to maximize overlap. Pin hot layers to GPU, schedule cold layers on CPU.

### 3. Sparsity-Aware Scheduling (Q-Infer-inspired)
MoE models activate ~10% of parameters per token. Don't transfer 100% of a layer when only 10% is active. Dynamic scheduling based on expert selection.

### 4. Hybrid Attention (HGCA-inspired)
Split attention across CPU and GPU. Recent KV entries on GPU (full attention). Older KV entries on CPU (sparse attention). Heterogeneous attention without full KV cache in VRAM.

### 5. Predictive Prefetch
Layer access in autoregressive generation is deterministic (layer 0, 1, 2, ..., N, repeat). Start transferring layer N+1 while computing layer N. For MoE, predict which experts will be active based on Markov model of past activations.

## The wgpu Moat

Every research paper above is CUDA-only. spill + vyre runs on wgpu: NVIDIA (Vulkan), AMD (Vulkan), Intel Arc (Vulkan/DX12), Apple Silicon (Metal), browsers (WebGPU).

spill is the first inference optimizer that works on every GPU. Half the market has no CUDA.

## Demo Target

- Model: Qwen3-Coder-Next 80B MoE (3B active parameters, 256K context)
- Hardware: single 4090 (24GB VRAM), 64GB RAM, NVMe Gen4
- Machine: axiomexec
- Metric: tok/s with and without spill on the same hardware
- Goal: 3-5x improvement over vanilla llama.cpp offloading

## Extraction Plan

1. Move from `libs/performance/memory/spill/` to standalone repo `santhsecurity/spill`
2. Add vyre as a crates.io dependency (not path dep)
3. Own CI, own tests, own release cycle
4. First release: spill-predict + spill-profile + basic tier manager (no GPU primitives yet)
5. Second release: wire vyre transfer primitives for compressed PCIe transfer
6. Third release: full pipeline (compress + prefetch + heterogeneous scheduling)
