# constants
LOCALHOST <- "127.0.0.1"
THOT_PORT <- 7047
THOT_TIMEOUT <- 1000L # in milliseconds

#' Create a ZMQ socket.
#'
#' @returns ZMQ socket.
zmq_socket <- function() {
  socket <- init.socket(THOT_ZMQ_CONTEXT, "ZMQ_REQ")
  set.send.timeout(socket, THOT_TIMEOUT)
  set.linger(socket, THOT_TIMEOUT)
  connect.socket(socket, sprintf("tcp://%s:%d", LOCALHOST, THOT_PORT))

  socket
}

#' Convert an object into JSON.
#'
#' @param obj Object to convert.
#'
#' @returns JSON encoding of the object.
to_json <- function(obj) {
  toJSON(obj, auto_unbox = TRUE, null = "null")
}

#' Get the path to the correct database server.
#'
#' @returns Path to the local database executable for the current system.
database_server_path <- function() {
  exe <- switch(Sys.info()["sysname"],
    "Linux" = "x86_64-unknown-linux-gnu",
    "Darwin" = "aarch64-apple-darwin",
    "Windows" = "x86_64-pc-windows-msvc.exe"
  )

  exe <- paste("thot-local-database-", exe, sep = "")
  system.file(exe, package = "thot", mustWork = TRUE)
}

#' Escape special string characters.
#' Characters escaped:
#' + Backslash (\)
#'
#' @param val Sring to escape.
#'
#' @returns Escaped string.
escape_str <- function(val) {
  gsub("\\\\", "\\\\\\\\", val)
}

#' Normalize Windows paths.
#' If total path length is less than 256, returns the normalized path,
#' otherwise returns the device (UNC) path.
#'
#' @param p1 Base path.
#' @param p2 Extension path.
#'
#' @returns Normalized path
join_path_windows <- function(p1, p2) {
  len <- nchar(p1) + nchar(p2)
  if (len < 256) {
    return(normalizePath(file.path(p1, p2), mustWork = FALSE))
  }

  PATH_PREFIX <- "\\\\?\\"
  if (!startsWith(p1, PATH_PREFIX)) {
    p1 <- paste(PATH_PREFIX, p1, sep = "")
  }

  if (!endsWith(p1, "\\")) {
    # no slashes
    p1 <- paste(p1, "\\\\", sep = "")
  } else if (!endsWith("\\\\")) {
    # one slash
    p1 <- paste(p1, "\\", sep = "")
  }

  if (startsWith(p2, "\\\\")) {
    p2 <- substring(p2, 2)
  } else if (startsWith(p2, "\\")) {
    p2 <- substring(p2, 1)
  }

  paste(p1, p2)
}

#' Datetime in ISO 8601 format.
utc_now <- function() {
  now <- as.POSIXlt(Sys.time(), tz = "UTC")
  strftime(now, "%Y-%m-%dT%H:%M:%SZ", tz = "UTC")
}

#' Converts an empty object (`[]`) in JSON to an empty object (`{}`)
#'
#' @param keys List of or singular object key(s) to convert.
#' @param json JSON in which to convert.
#' @returns JSON with key replaced as empty object if it was an empty list.
#'
#' @examples
#' my_json <- '{"my_obj": [], "my_num": 4}'
#' json_empty_list_to_obj("my_obj", my_json)
json_empty_list_to_obj <- function(keys, json) {
  if (!is.list(keys)) {
    keys <- list(keys)
  }

  for (key in keys) {
    search <- sprintf('"%s":\\s*\\[\\]', key)
    replace <- sprintf('"%s":{}', key)
    json <- gsub(search, replace, json)
  }

  json
}
