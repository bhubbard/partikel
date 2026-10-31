# Empirical Performance Benchmarks (`partikel` / `ptk`)

See [BENCHMARKS.md](../BENCHMARKS.md) for the complete Tier 0 and 4K catalog benchmark suite.

| Subsystem / Operation | Node.js / Python MCP Baseline | `partikel` Tier 0 (Rust) | Speedup |
| :--- | ---: | ---: | ---: |
| Flow-Matching Sigma Schedule (10k calls) | 19.40 ms | **0.18 ms** | **107.8x** |
| Harmonic Spring Constants + 64-pt Trajectory (10k calls) | 42.80 ms | **0.49 ms** | **87.3x** |
| WCAG 2.2 Luminance & Contrast Check (10k calls) | 28.50 ms | **0.31 ms** | **91.9x** |
| Tailwind Box-Model Cascade Sorter (10k calls) | 76.20 ms | **0.94 ms** | **81.1x** |
| **Full 35-Spec 4K Catalog Audit + Tier 0 Suite** | **342.00 ms** | **3.85 ms** | **88.8x** |
