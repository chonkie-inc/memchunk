#!/usr/bin/env python3
"""Benchmark Python text chunking libraries."""

import time

RUNS = 5

def benchmark(name, func, text_len):
    """Run benchmark multiple times and return average."""
    times = []
    for _ in range(RUNS):
        start = time.perf_counter()
        result = func()
        elapsed = time.perf_counter() - start
        times.append(elapsed)
    avg = sum(times) / len(times)
    throughput = text_len / avg / 1e6
    print(f"{name:14} {avg*1000:>10.2f} ms  ({throughput:.1f} MB/s)  [avg of {RUNS} runs]")
    return avg, throughput

def main():
    # Load enwik8
    with open("benches/data/enwik8", "r", encoding="utf-8", errors="replace") as f:
        text = f.read()

    size_mb = len(text) / 1_000_000
    print(f"Benchmarking enwik8 ({size_mb:.0f}MB), 4KB chunks\n")

    # LangChain RecursiveCharacterTextSplitter
    try:
        from langchain_text_splitters import RecursiveCharacterTextSplitter
        splitter = RecursiveCharacterTextSplitter(chunk_size=4096, chunk_overlap=0)
        benchmark("langchain:", lambda: splitter.split_text(text), len(text))
    except ImportError:
        print("langchain:     (not installed)")

    # Chonkie RecursiveChunker with ByteTokenizer
    try:
        from chonkie import RecursiveChunker
        from chonkie.tokenizer import ByteTokenizer
        tokenizer = ByteTokenizer()
        chunker = RecursiveChunker(tokenizer=tokenizer, chunk_size=4096)
        benchmark("chonkie:", lambda: chunker.chunk(text), len(text))
    except Exception as e:
        print(f"chonkie:       (error: {e})")

    # semchunk (Python)
    try:
        import semchunk
        benchmark("semchunk:", lambda: semchunk.chunk(text, 4096, len), len(text))
    except Exception as e:
        print(f"semchunk:      (error: {e})")

    # LlamaIndex SentenceSplitter (only 1 run - too slow)
    try:
        from llama_index.core.node_parser import SentenceSplitter
        from llama_index.core import Document
        splitter = SentenceSplitter(chunk_size=4096, chunk_overlap=0)
        doc = Document(text=text)
        print("llama-index:   (running 1 iteration - slow)")
        start = time.perf_counter()
        nodes = splitter.get_nodes_from_documents([doc])
        elapsed = time.perf_counter() - start
        throughput = len(text) / elapsed / 1e6
        print(f"llama-index:   {elapsed*1000:>10.2f} ms  ({throughput:.1f} MB/s)")
    except ImportError:
        print("llama-index:   (not installed)")

    # Simple Python baseline
    def naive_chunk():
        chunks = []
        pos = 0
        while pos < len(text):
            end = min(pos + 4096, len(text))
            if end < len(text):
                window = text[pos:end]
                for delim in ['\n', '.', '?']:
                    idx = window.rfind(delim)
                    if idx != -1:
                        end = pos + idx + 1
                        break
            chunks.append(text[pos:end])
            pos = end
        return chunks
    benchmark("python-naive:", naive_chunk, len(text))

if __name__ == "__main__":
    main()
