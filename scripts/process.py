# /// script
# dependencies = [
#   "duckdb",
#   "openai"
# ]
# ///
import os
import re
import time
from asyncio import Semaphore

from openai import AsyncOpenAI

client = AsyncOpenAI(
    base_url=os.environ.get("OPENAI_API_BASE"),
    api_key="dummy",
)

kata = re.compile(r"^[\u30a0-\u30ff\uff66-\uff9d]+$")
alphabet = re.compile(r"^[a-zA-Z ]+$")


class Exp(BaseException):
    pass


async def conv(word: str):
    response = await client.chat.completions.create(
        model="model",
        messages=[
            {
                "role": "user",
                "content": f"""Convert the following word to Japanesse katakana which describes how to pronounce the english word, output converted word only in <output> xml tag.
Word: {word}""",
            },
        ],
        extra_body={
            "grammar": r"""
root ::= "<output>" katakana "</output>"

katakana ::= kana | kana katakana
kana ::= [ァ-ヿ] | [ｦ-ﾟ]
"""
        },
        max_tokens=256,
    )
    content = response.choices[0].message.content.strip()
    if not content.startswith("<output>"):
        raise Exp(f"invalid output: {content}")
    content = content[len("<output>") :]
    if not content.endswith("</output>"):
        raise Exp(f"invalid output: {content}")
    content = content[: -len("</output>")]
    if not kata.match(content):
        raise Exp(f"invalid match: {content}")
    return content


async def reconv(word: str):
    response = await client.chat.completions.create(
        model="model",
        messages=[
            {
                "role": "user",
                "content": f"""Convert the following word to English, output converted word only in <output> xml tag.
Word: {word}""",
            },
        ],
        extra_body={
            "grammar": r"""root ::= "<output>" text "</output>"

text ::= char | char text
char ::= [a-zA-Z] | " "
"""
        },
        max_tokens=256,
    )
    content = response.choices[0].message.content.strip()
    if not content.startswith("<output>"):
        raise Exp(f"reconv invalid output: {content}")
    content = content[len("<output>") :]
    if not content.endswith("</output>"):
        raise Exp(f"reconv invalid output: {content}")
    content = content[: -len("</output>")]
    if not alphabet.match(content):
        raise Exp(f"reconv invalid match: {content}")
    return content


async def do_conv_one(word: str):
    c = await conv(word)
    cc = await reconv(c)
    if word.lower().replace(" ", "") != cc.lower().replace(" ", ""):
        raise Exp(f"mismatch: {word} - {cc} ({c})")
    return c, cc


async def do_conv(word: str, retry=3):
    err = None
    for _ in range(retry):
        try:
            return await do_conv_one(word)
        except Exp as e:
            err = e
    if err is None:
        raise Exp("ha?")
    raise err


async def do_job(count: int):
    w = list(open("./data/ng.txt", "r").read().split(","))
    w_total = len(w)

    semaphore = Semaphore(16)
    errors = []
    results = []

    async def wrapped(item):
        async with semaphore:
            try:
                result = await do_conv(item)
                results.append(result)
            except Exp:
                errors.append(item)

    tasks = [asyncio.create_task(wrapped(item)) for item in w[:count]]

    await asyncio.gather(*tasks)
    open("./data/ok.txt", "a").write("\n".join([f"{a},{b}" for a, b in results]) + "\n")
    w = w[count:]
    w.extend(errors)
    open("./data/ng.txt", "w").write(",".join(w))
    return w_total, len(results), len(errors)


async def main():
    j = len(list(open("./data/ng.txt", "r").read().split(",")))
    while j > 0:
        now = time.time()
        remain, ok, err = await do_job(min(512, j))
        print(f"remain: {remain - ok}, ok: {ok}, err: {err}, time: {time.time() - now}")
        j = remain - ok


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
