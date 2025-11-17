#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "conllu",
# ]
# ///

import sys
import typing as t
from pathlib import Path

import conllu


FIELDS: list[str] = ["id", "form"]
OPT_FIELDS: list[str] = ["lemma", "upos", "xpos", "feats", "head", "deprel", "deps", "misc"]
I: str = " "*4

def main() -> None:
    case_path = Path(sys.argv[1])
    dump_snap(case_path)


def dump_snap(case_path: Path) -> None:
    print("---")
    print("source: tests/api/parse.rs")
    print("expression: sentences")
    print("---")
    print("[")
    d = 1
    with case_path.open(encoding="utf-8") as fp:
        for sentence in conllu.parse_incr(fp):
            print(f"{I*d}Sentence {{")
            d += 1
            print(f"{I*d}tokens: [")
            d += 1
            for token in sentence:
                print(f"{I*d}Token {{")
                d += 1
                for k in FIELDS:
                    v = token.get(k)
                    print(f"{I*d}{k}: {_fmt(v,d+1)},")
                for k in OPT_FIELDS:
                    v = token.get(k)
                    print(f"{I*d}{k}: {_fmt_opt(v,d+1)},")
                d -= 1
                print(f"{I*d}}},")
            d -= 1
            print(f"{I*d}],")
            if sentence.metadata:
                print(f"{I*d}metadata: {_fmt(sentence.metadata, d+1)},")
            else:
                print(f"{I*d}metadata: {{}},")
            d -= 1
            print(f"{I*d}}},")
        d -= 1
        print(f"{I*d}]")


def _fmt(v: t.Any, d: int) -> str:
    if isinstance(v, dict):
        if not v:
            return "{}"
        res = "{\n"
        for k,v1 in sorted(v.items()):
            res += f"{I*d}\"{k}\": {_fmt(v1, d+1)},\n"
        res += f"{I*(d-1)}}}"
        return res
    if isinstance(v, list):
        if not v:
            return "[]"
        res = "[\n"
        for v2 in v:
            res += f"{I*d}{_fmt(v2, d)},\n"
        res += f"{I*(d-1)}]"
        return res
    if isinstance(v, tuple):
        if isinstance(v[0], int):
            return f"{v[0]}{v[1]}{v[2]}"
        res = "(\n"
        for v2 in v:
            res += f"{I*(d+1)}{_fmt(v2, d+1)},\n"
        res += f"{I*(d)})"
        return res
    if isinstance(v, str):
        return f"\"{v}\""
    return str(v)


def _fmt_opt(v: t.Any | None, d: int) -> str:
    if v is None:
        return str(v)
    if v == "_":
        return str(None)
    return f"Some(\n{I*d}{_fmt(v,d+1)},\n{I*(d-1)})"
    if isinstance(v, tuple):
        return f"{v[0]}{v[1]}{v[2]}"
    if isinstance(v, str):
        return f"\"{v}\""
    return str(v)


if __name__ == "__main__":
    main()
