# Build R package

## Building R bindings

Based on this [guide](https://extendr.github.io/rextendr/articles/package.html).

- `rustup toolchain install nightly` (If not yet installed)
- `r` (Run r)
- `install.packages("rzmq")`
- `rextendr::document()` (Compile rust code)

To install Thot globally (default path is ./)

`devtools::install()`

## Creating a zipped R library for sharing

- Open th `RStudio` application
- On the top menu, click on `File/Open project ...`
- Select `PATH-TO-THOT/lang/r/thot.Rproj`

Based on this [guide](https://support.posit.co/hc/en-us/articles/115000239587-Sharing-Internal-R-Packages).

- Click on the `build` menu on the right hand side menu.
- Click on `More`
- Click on `Build Source Package`
