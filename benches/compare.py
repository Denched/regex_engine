import re, time, subprocess

for n in range(10, 31, 2):
    inp = "a" * n
    start = time.perf_counter()
    re.search(r"(a+)+b", inp)
    elapsed = time.perf_counter() - start
    print(f"n={n:2d}  python={elapsed:.6f}s")
    if elapsed > 30:
        print("  (aborting — exponential growth confirmed)")
        break