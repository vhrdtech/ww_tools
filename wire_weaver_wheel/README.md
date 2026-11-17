# wire_weaver_wheel

The `wire_weaver` Python wheel: a `cdylib` whose one function hands the `wire_weaver._wire_weaver` module to
`wire_weaver_py::init` and its `logring` submodule to `logring_py::init`, both taken by git. The Python package
(`python/wire_weaver`), the tests, the console script and `pyproject.toml` live here; the classes and functions of the
module live in those library crates. It embeds the API snapshots of the whole standard library.

```sh
uv run --with maturin maturin build --release   # wheel in target/wheels/ (`just wheel` from the repository root)
uv run --group dev pytest                       # the tests (`just test-py` from the repository root)
```

The tests read WireWeaver's fixtures from Cargo's checkout of the wire_weaver commit `Cargo.lock` pins
(`tests/wire_weaver_checkout.py`). Documentation: WireWeaver's
[Python page](https://github.com/vhrdtech/wire_weaver/blob/master/docs/python.md).
