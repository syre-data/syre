.onLoad <- function(libname, pkgname) {
  ns <- topenv()
  ns$SYRE_ZMQ_CONTEXT <- init.context()
  ns$SYSNAME <- Sys.info()["sysname"]
}
