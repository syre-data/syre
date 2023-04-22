.onLoad <- function(libname, pkgname) {
  ns <- topenv()
  ns$THOT_ZMQ_CONTEXT <- init.context()
}
