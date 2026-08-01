#!/usr/bin/env python3
"""从上游 TextParserTest.java 提取 testDoc/testDocError 用例为 JSON fixture。

逐字提取输入与期望输出（含转义还原），保证与 Java 源码 1:1：
- Java 字符串字面量（含 + 拼接）与转义（\\n \\t \\r \\" \\\\ \\uXXXX）还原；
- testDoc 的 8 种重载形态与 testDocError 的 2 种形态按参数形态区分；
- processComments 为 null 表示同时执行 true/false 两条路径。
"""
import json
import re
import sys

SOURCE = sys.argv[1]
OUTPUT = sys.argv[2]


def parse_string_literal(s):
    """把 Java 字符串字面量内容还原为实际字符序列。"""
    out = []
    i = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        i += 1
        if i >= n:
            raise ValueError("dangling backslash")
        e = s[i]
        if e == "n":
            out.append("\n")
        elif e == "t":
            out.append("\t")
        elif e == "r":
            out.append("\r")
        elif e == "b":
            out.append("\b")
        elif e == "f":
            out.append("\f")
        elif e == '"':
            out.append('"')
        elif e == "\\":
            out.append("\\")
        elif e == "u":
            hexpart = s[i + 1 : i + 5]
            out.append(chr(int(hexpart, 16)))
            i += 4
        else:
            raise ValueError(f"unknown escape \\{e}")
        i += 1
    return "".join(out)


TOKEN_RE = re.compile(
    r'"(?:[^"\\]|\\.)*"|null|Boolean\.(?:TRUE|FALSE)|-?\d+|[A-Za-z_][A-Za-z0-9_]*|[(),+]'
)


def tokenize(text):
    return TOKEN_RE.findall(text)


def parse_call_args(tokens, start):
    """从 start（'(' 之后）解析逗号分隔参数。返回 (args, end_index)。"""
    args = []
    current = []
    i = start
    depth = 0
    while i < len(tokens):
        tok = tokens[i]
        if tok == "(":
            depth += 1
            current.append(tok)
        elif tok == ")":
            if depth == 0:
                args.append(current)
                return args, i
            depth -= 1
            current.append(tok)
        elif tok == "," and depth == 0:
            args.append(current)
            current = []
        else:
            current.append(tok)
        i += 1
    raise ValueError("unbalanced call")


def decode_arg(tokens):
    """还原一个参数：字符串拼接 / null / Boolean / int。"""
    if not tokens:
        raise ValueError("empty arg")
    # 字符串拼接（可能交替 + 与字面量）
    if tokens[0].startswith('"'):
        parts = []
        j = 0
        while j < len(tokens):
            if tokens[j].startswith('"'):
                parts.append(parse_string_literal(tokens[j][1:-1]))
                j += 1
            elif tokens[j] == "+":
                j += 1
            else:
                raise ValueError(f"unexpected token in string concat: {tokens[j]}")
        return parts and "".join(parts) or ""
    if tokens[0] == "null":
        return None
    if tokens[0] == "Boolean.TRUE":
        return True
    if tokens[0] == "Boolean.FALSE":
        return False
    return int(tokens[0])


def main():
    with open(SOURCE, encoding="utf-8") as fh:
        source = fh.read()

    tokens = tokenize(source)
    cases = []

    i = 0
    while i < len(tokens):
        tok = tokens[i]
        if tok in ("testDoc", "testDocError") and tokens[i + 1] == "(":
            name = tok
            args, end = parse_call_args(tokens, i + 2)
            # 方法定义（static void testDoc(...)）首参数是类型名，跳过
            if not args or not args[0][0].startswith('"'):
                i = end
                continue
            decoded = [decode_arg(a) for a in args]
            if name == "testDoc":
                # 形态按解码后参数个数区分：
                #   2: (input, output)
                #   3: (input, outProc, outUnproc) | (input, output, Boolean)
                #   4: (input, output, offset, len) | (input, outProc, outUnproc, Boolean)
                #   5: (input, outProc, outUnproc, offset, len) | (input, output, offset, len, Boolean)
                if len(decoded) == 2:
                    cases.append(
                        {
                            "kind": "doc",
                            "input": decoded[0],
                            "outProc": decoded[1],
                            "outUnproc": decoded[1],
                            "offset": 0,
                            "len": len(decoded[0]),
                            "processComments": None,
                        }
                    )
                elif len(decoded) == 3:
                    if isinstance(decoded[2], bool):
                        cases.append(
                            {
                                "kind": "doc",
                                "input": decoded[0],
                                "outProc": decoded[1],
                                "outUnproc": decoded[1],
                                "offset": 0,
                                "len": len(decoded[0]),
                                "processComments": decoded[2],
                            }
                        )
                    else:
                        cases.append(
                            {
                                "kind": "doc",
                                "input": decoded[0],
                                "outProc": decoded[1],
                                "outUnproc": decoded[2],
                                "offset": 0,
                                "len": len(decoded[0]),
                                "processComments": None,
                            }
                        )
                elif len(decoded) == 4:
                    if isinstance(decoded[3], bool):
                        cases.append(
                            {
                                "kind": "doc",
                                "input": decoded[0],
                                "outProc": decoded[1],
                                "outUnproc": decoded[2],
                                "offset": 0,
                                "len": len(decoded[0]),
                                "processComments": decoded[3],
                            }
                        )
                    else:
                        cases.append(
                            {
                                "kind": "doc",
                                "input": decoded[0],
                                "outProc": decoded[1],
                                "outUnproc": decoded[1],
                                "offset": decoded[2],
                                "len": decoded[3],
                                "processComments": None,
                            }
                        )
                elif len(decoded) == 5:
                    if isinstance(decoded[4], bool):
                        cases.append(
                            {
                                "kind": "doc",
                                "input": decoded[0],
                                "outProc": decoded[1],
                                "outUnproc": decoded[1],
                                "offset": decoded[2],
                                "len": decoded[3],
                                "processComments": decoded[4],
                            }
                        )
                    else:
                        cases.append(
                            {
                                "kind": "doc",
                                "input": decoded[0],
                                "outProc": decoded[1],
                                "outUnproc": decoded[2],
                                "offset": decoded[3],
                                "len": decoded[4],
                                "processComments": None,
                            }
                        )
                else:
                    raise ValueError(f"unexpected arg count {len(decoded)}")
            else:
                # testDocError(input, null, line, col[, Boolean])
                if len(decoded) == 4:
                    cases.append(
                        {
                            "kind": "error",
                            "input": decoded[0],
                            "errorLine": decoded[2],
                            "errorCol": decoded[3],
                            "processComments": None,
                        }
                    )
                elif len(decoded) == 5:
                    cases.append(
                        {
                            "kind": "error",
                            "input": decoded[0],
                            "errorLine": decoded[2],
                            "errorCol": decoded[3],
                            "processComments": decoded[4],
                        }
                    )
                else:
                    raise ValueError(f"unexpected arg count {len(decoded)}")
            i = end
            continue
        i += 1

    with open(OUTPUT, "w", encoding="utf-8") as fh:
        json.dump({"baseline": "10f9dd2eb8cbd98515ce14b149d115e0287d0add", "cases": cases}, fh, ensure_ascii=False, indent=1)
    print(f"extracted {len(cases)} cases")


if __name__ == "__main__":
    main()
