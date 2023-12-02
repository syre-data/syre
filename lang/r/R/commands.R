#' Send a command to the database.
#'
#' @param socket ZMQ socket.
#' @param cmd Command to send.
#' @param result Handle response as a Result.
#'
#' @returns Deserialzed result.
send_cmd <- function(socket, cmd, result = TRUE) {
  sent <- send.raw.string(socket, cmd)
  if (!sent) {
    stop("Could not send message.")
  }

  res <- receive.string(socket)
  res <- fromJSON(res, simplifyVector = FALSE)
  if (result) {
    err <- res$Err
    if (!is.null(err)) {
      stop(err)
    }

    res <- res$Ok
  }

  res
}

#' Checks if a Thot database is available.
#'
#' @returns Whether a Thot database is available.
database_available <- function() {
  server_up <- tryCatch({
    socketConnection(port = THOT_PORT)
    TRUE
  },
  error = function(cond) {
    # port not open, no chance for server
    FALSE
  },
  warning = function(cond) {
    FALSE
  })

  if (!server_up) {
    return(FALSE)
  }

  # check if database is responsive
  cmd <- '{"DatabaseCommand": "Id"}'
  id <- send_cmd(zmq_socket(), cmd, result = FALSE)
  id == "thot local database"
}

#' Loads a Thot project.
#'
#' @param socket ZMQ socket.
#' @param path Path to the Project.
#'
#' @returns List of Project properties.
load_project <- function(socket, path) {
  cmd <- sprintf('{"ProjectCommand": {"Load": "%s"}}', path)
  send_cmd(socket, cmd)
}

#' Loads a Thot Project's graph.
#'
#' @param socket ZMQ socket.
#' @param project Project's id.
#'
#' @returns List of graph components.
load_graph <- function(socket, project) {
  cmd <- sprintf('{"GraphCommand": {"Load": "%s"}}', project)
  send_cmd(socket, cmd)
}

#' Gets a Thot Container.
#'
#' @param socket ZMQ socket.
#' @param id Container id.
#'
#' @returns List of Container properties.
get_container <- function(socket, id) {
  cmd <- sprintf('{"ContainerCommand": {"Get": "%s"}}', id)
  send_cmd(socket, cmd, result = FALSE)
}

#' Gets a Thot Container's path.
#'
#' @param socket ZMQ socket.
#' @param id Container id.
#'
#' @returns Container's path.
container_path <- function(socket, id) {
  cmd <- sprintf('{"ContainerCommand": {"Path": "%s"}}', id)
  send_cmd(socket, cmd, result = FALSE)
}

#' Gets a Thot Container from its path.
#'
#' @param socket ZMQ socket.
#' @param path Path to the Container.
#'
#' @returns List of Container properties.
container_by_path <- function(socket, path) {
  cmd <- sprintf('{"ContainerCommand": {"ByPath": "%s"}}', path)
  send_cmd(socket, cmd, result = FALSE)
}
