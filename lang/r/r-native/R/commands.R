library(jsonlite)
library(rzmq)

#' Send a command to the database.
#'
#' @param socket ZMQ socket.
#' @param cmd Command to send.
#' @param result Handle response as a Result.
send_cmd <- function(socket, cmd, result = TRUE) {
  send.raw.string(socket, cmd)
  res <- receive.string(socket)
  res <- fromJSON(res, simplifyVector = FALSE)

  if (result) {
    res <- res$Ok
    err <- res$Err
    if (!is.null(err)) {
      stop(err)
    }
  }

  res
}

#' Checks if a Thot database is available.
#'
#' @param socket ZMQ socket.
database_available <- function(socket) {
  cmd <- '{"DatabaseCommand": "Id"}'
  id <- send_cmd(socket, cmd, result = FALSE)
  id == "thot local database"
}

#' Loads a Thot project.
#'
#' @param socket ZMQ socket.
#' @param path Path to the Project.
load_project <- function(socket, path) {
  cmd <- sprintf('{"ProjectCommand": {"Load": "%s"}}', path)
  send_cmd(socket, cmd)
}

#' Loads a Thot Project's graph.
#'
#' @param socket ZMQ socket.
#' @param project Project's id.
load_graph <- function(socket, project) {
  cmd <- sprintf('{"GraphCommand": {"Load": "%s"}}', project)
  send_cmd(socket, cmd)
}

#' Gets a Thot Container from its path.
#'
#' @param socket ZMQ socket.
#' @param path Path to the Container.
container_by_path <- function(socket, path) {
  cmd <- sprintf('{"ContainerCommand": {"ByPath": "%s"}}', path)
  send_cmd(socket, cmd, result = FALSE)
}

#' Convert an object into JSON.
#'
#' @param obj The object to convert.
to_json <- function(obj) {
  toJSON(obj, auto_unbox = TRUE, null = "null")
}
