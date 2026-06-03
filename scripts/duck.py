# /// script
# dependencies = [
#   "duckdb",
#   "polars",
#   "numpy",
#   "pandas",
#   "pyarrow",
#   "datasets"
# ]
# ///
import duckdb
import polars as pl
from datasets import load_dataset

con = duckdb.connect("./data/page.duckdb")
con2 = duckdb.connect("./data/langlinks.duckdb")

r = con.execute(
    "SELECT page_id, page_title FROM t WHERE page_is_redirect = 0 AND page_namespace = 0"
).fetchdf()
r2 = con2.execute("SELECT ll_from, ll_title FROM t WHERE ll_lang = 'en'").fetchdf()
r = pl.from_pandas(r)
r2 = pl.from_pandas(r2)
r = (
    r.join(r2, left_on="page_id", right_on="ll_from", how="inner")
    .drop("page_id")
    .rename({"page_title": "ja", "ll_title": "en"})
    .with_columns(pl.all().cast(pl.Utf8))
)
r2 = (
    r.clone()
    .with_columns(pl.col("ja").str.to_lowercase(), pl.col("en").str.to_lowercase())
    .filter(
        pl.col("ja").str.contains(r"^[a-z ]+$")
        & pl.col("en").str.contains(r"^[a-z ]+$")
        & (pl.col("ja") == pl.col("en"))
    )
    .with_columns(pl.col("ja"))["ja"]
)
r = r.filter(
    (pl.col("ja") != pl.col("en"))
    & ~pl.col("ja").str.contains("_")
    & ~pl.col("en").str.contains("_")
    & ~pl.col("ja").str.contains(r"^[a-zA-Z ]+$")
    & pl.col("en").str.contains(r"^[a-zA-Z ]+$")
    & (pl.col("en").str.contains(" ") | pl.col("en").str.len_chars() > 3)
)
r = r.filter(pl.col("ja").str.contains(r"^[\u30a0-\u30ff\uff66-\uff9d]+$"))
r = r.sort("en")
r.write_csv("./data/out.csv", separator=",")
r2 = list(r2)
r2.extend(load_dataset("VOICEVOX/kanalizer-dataset", split="train")["word"])
r2 = list(set(r2))
open("./data/ng.txt", "w").write(",".join(r2))
