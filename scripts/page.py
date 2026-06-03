# /// script
# dependencies = [
#   "duckdb",
#   "requests",
#   "polars",
#   "connectorx",
#   "pyarrow",
#   "pymysql",
#   "sqlalchemy"
# ]
# ///
import subprocess
import time
from pathlib import Path

import duckdb
import polars as pl
import requests
from sqlalchemy import create_engine

data = Path("./data")
data.mkdir(exist_ok=True)
sql = data / "page.sql.gz"
if not sql.exists():
    print("Downloading to cache")
    sql.open("wb").write(
        requests.get(
            "https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-page.sql.gz"
        ).content
    )
MYSQL_PASSWORD = "root"
MYSQL_DB = "root"
CONTAINER = "tmp_page_mysql"
r = subprocess.run(
    [
        "docker",
        "run",
        "--name",
        CONTAINER,
        "-d",
        "--rm",
        "-e",
        f"MYSQL_ROOT_PASSWORD={MYSQL_PASSWORD}",
        "-e",
        f"MYSQL_DATABASE={MYSQL_DB}",
        "-p",
        "3308:3306",
        "mysql:latest",
    ]
)
subprocess.run(["docker", "cp", sql, f"{CONTAINER}:/main.sql.gz"])

while True:
    result = subprocess.run(
        [
            "docker",
            "exec",
            CONTAINER,
            "mysql",
            "-uroot",
            f"-p{MYSQL_PASSWORD}",
            "-e",
            "SELECT 1",
        ],
        capture_output=True,
    )

    if result.returncode == 0:
        break

    time.sleep(1)
print("mysql ok")
subprocess.run(
    [
        "docker",
        "exec",
        "-it",
        CONTAINER,
        "/bin/bash",
        "-c",
        f"gunzip -c /main.sql.gz | mysql -uroot -p{MYSQL_PASSWORD} {MYSQL_DB}",
    ],
)
print("execed")
engine = create_engine("mysql+pymysql://root:root@127.0.0.1:3308/root?charset=utf8mb4")
df = pl.read_database(
    "SELECT * FROM page",
    engine,
)
subprocess.run(["docker", "kill", CONTAINER])

con = duckdb.connect(data / "page.duckdb")

con.execute("CREATE TABLE t AS SELECT * FROM df")
sql.unlink(missing_ok=True)
