//! FFI layout for llama.cpp backend integration.
//!
//! The concrete C ABI is intentionally not exported yet because this crate is
//! currently compiled with `forbid(unsafe_code)` while the Rust tiering core is
//! validated. Expected signatures:
//!
//! ```c
//! GGML_BACKEND_API ggml_backend_t helix_backend_init(void);
//! GGML_BACKEND_API const char * helix_backend_name(void);
//! GGML_BACKEND_API bool helix_backend_supports_op(
//!     ggml_backend_t backend,
//!     const struct ggml_tensor * op
//! );
//! ```
