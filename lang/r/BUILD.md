# Build R package

## Building R bindings

`devtools::document()`

To install Syre globally (default path is ./)

`devtools::install()`

## Creating a zipped R library for sharing
### R Terminal
1. Move to the `R` folder: `setwd(<path/to/syre>/lang/r)`
2. `devtools::build()`

### RStudio
1. On the top menu, click on `File/Open project ...`
2. Select `<path/to/syre>/lang/r/syre.Rproj`

Based on this [guide](https://support.posit.co/hc/en-us/articles/115000239587-Sharing-Internal-R-Packages).

- Click on the `build` menu on the right hand side menu.
- Click on `More`
- Click on `Build source package` 
