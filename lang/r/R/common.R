# constants
THOT_LOCALHOST <- "127.0.0.1"
THOT_PORT <- 7047
THOT_TIMEOUT <- 1000L # in milliseconds

#' Create a ZMQ socket.
zmq_socket <- function() {
  socket <- init.socket(THOT_ZMQ_CONTEXT, "ZMQ_REQ")
  set.send.timeout(socket, THOT_TIMEOUT)
  set.linger(socket, THOT_TIMEOUT)
  connect.socket(socket, sprintf("tcp://%s:%d", THOT_LOCALHOST, THOT_PORT))

  socket
}

#' Convert an object into JSON.
#'
#' @param obj The object to convert.
to_json <- function(obj) {
  toJSON(obj, auto_unbox = TRUE, null = "null")
}

#' Get the path to the correct database server.
database_server_path <- function() {
  exe <- switch(Sys.info()["sysname"],
         "Linux" = "x86_64-unknown-linux-gnu",
         "Darwin" = "aarch64-apple-darwin",
         "Windows" = "x86_64-pc-windows-msvc.exe"
  )

  exe <- paste("thot-local-database-", exe)
  system.file({{ exe }}, package = "thot", mustWork = TRUE)
}
