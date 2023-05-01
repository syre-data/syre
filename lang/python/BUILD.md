# Build module

Go to the root of thot, and then

`cd lang/python`
`python3 -m venv .venv`
`source .venv/bin/activate.fish`
`pip install -U pip maturin`
`maturin develop`

To build the build the python bindings run:

`maturin build`

After the build remember to `pip install` the wheel

> if the version is still the same do `pip install --force-reinstall`
