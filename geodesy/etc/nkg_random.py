import random

# Generate input data for NKG transformation timing and consistency checks
with open("untracked/timing/nkg_test.pts", "w") as f:
    for _ in range(10_000_000):
        coord = [
            str(round(60 + random.uniform(-10, 10), 12)),
            str(round(25 + random.uniform(-20, 20), 12)),
            "0  2026"
        ]
        f.write(" ".join(coord) + "\n")
