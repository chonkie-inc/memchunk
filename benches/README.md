# Benchmarks

## Setup

Download enwik8 (100MB Wikipedia extract):

```bash
cd benches/data
curl -O https://mattmahoney.net/dc/enwik8.zip
unzip enwik8.zip
rm enwik8.zip
```

Download enwik9 (1GB Wikipedia extract):

```bash
cd benches/data
curl -O https://mattmahoney.net/dc/enwik9.zip
unzip enwik9.zip
rm enwik9.zip
```

## Run

```bash
cargo bench
```

## Results (enwik8 — 100MB)

| Chunk Size | Time | Throughput |
|------------|------|------------|
| 4KB | 526 µs | 190 GB/s |
| 16KB | 147 µs | 680 GB/s |
| 32KB | 59 µs | 1.7 TB/s |

## Results (enwik9 — 1GB)

| Chunk Size | Time | Throughput |
|------------|------|------------|
| 4KB | 20.1 ms | 50 GB/s |
| 16KB | 4.7 ms | 215 GB/s |
| 32KB | 1.0 ms | 1 TB/s |

All benchmarks were taken on a Apple M3 MacBook Air.
