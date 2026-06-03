from itertools import chain

all = []
for l in chain(
    open("./data/ok.txt", "r").read().split("\n"),
    open("./data/out.csv", "r").read().split("\n")[1:],
):
    if len(l) < 1:
        continue
    a, b = l.split(",")
    if "," in a or "," in b:
        print(f"Warning: {a}, {b}")
        continue
    all.append((b, a))

all = [(a.lower().replace(" ", ""), b) for (a, b) in all]
all = list(sorted(all, key=lambda a: -len(a[0])))
open("./data/all.csv", "w").write("\n".join([f"{a},{b}" for a, b in all]))
