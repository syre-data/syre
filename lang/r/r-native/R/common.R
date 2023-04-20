THOT_LOCALHOST <- "127.0.0.1"
THOT_PORT <- 7047
THOT_TIMEOUT <- 1000 # in milliseconds
THOT_RCV_ATTEMPTS <- 100
THOT_RCV_SLEEP <- THOT_TIMEOUT / THOT_RCV_ATTEMPTS / 1000 # account for milliseconds

#' Create a ZMQ socket.
zmq_socket <- function() {
  context <- init.context()
  socket <- init.socket(context, "ZMQ_REQ")
  # set.send.timeout(socket, as.integer(THOT_TIMEOUT))
  # set.linger(socket, as.integer(THOT_TIMEOUT))
  # set.reconnect.ivl(socket, as.integer(-1))
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
  R.version
  system.file({{ exe }}, package = "thot", mustWork = TRUE)
}
