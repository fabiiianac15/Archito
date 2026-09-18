"""
Calculadora segura de Archito — evalúa expresiones aritméticas
usando AST (sin eval/exec). Solo acepta números, operadores,
paréntesis, funciones matemáticas y las constantes pi / e.
"""

import ast
import math
import operator as op
import re

_BINOPS = {
    ast.Add: op.add, ast.Sub: op.sub, ast.Mult: op.mul,
    ast.Div: op.truediv, ast.FloorDiv: op.floordiv,
    ast.Mod: op.mod, ast.Pow: op.pow,
}
_UNARY = {ast.UAdd: op.pos, ast.USub: op.neg}
_FUNCS = {
    "sqrt": math.sqrt, "abs": abs, "round": round,
    "sin": math.sin, "cos": math.cos, "tan": math.tan,
    "log": math.log, "log2": math.log2, "log10": math.log10,
    "floor": math.floor, "ceil": math.ceil, "factorial": math.factorial,
}
_CONSTS = {"pi": math.pi, "e": math.e}


def _eval(node):
    if isinstance(node, ast.Expression):
        return _eval(node.body)
    if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
        return node.value
    if isinstance(node, ast.BinOp) and type(node.op) in _BINOPS:
        left, right = _eval(node.left), _eval(node.right)
        if isinstance(node.op, ast.Pow) and abs(right) > 1000:
            raise ValueError("exponente demasiado grande")
        return _BINOPS[type(node.op)](left, right)
    if isinstance(node, ast.UnaryOp) and type(node.op) in _UNARY:
        return _UNARY[type(node.op)](_eval(node.operand))
    if isinstance(node, ast.Name) and node.id.lower() in _CONSTS:
        return _CONSTS[node.id.lower()]
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
        fname = node.func.id.lower()
        if fname in _FUNCS and len(node.args) <= 2 and not node.keywords:
            args = [_eval(a) for a in node.args]
            return _FUNCS[fname](*args)
    raise ValueError("expresión no permitida")


def calc(expr: str) -> str:
    """Evalúa y formatea el resultado como texto."""
    expr = expr.replace("^", "**")
    expr = re.sub(r"(\d),(\d)", r"\1.\2", expr)
    result = _eval(ast.parse(expr.strip(), mode="eval"))
    return fmt_num(result)


def fmt_num(x) -> str:
    if isinstance(x, float):
        if abs(x) >= 1e15 or (x != 0 and abs(x) < 1e-9):
            return f"{x:.6g}"
        if x == int(x):
            x = int(x)
        else:
            s = f"{x:,.6f}".rstrip("0").rstrip(".")
            return s.replace(",", "§").replace(".", ",").replace("§", ".")
    return f"{x:,}".replace(",", ".")
