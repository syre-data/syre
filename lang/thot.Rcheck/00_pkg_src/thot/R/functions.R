CONTAINER_ID_KEY <- "THOT_CONTAINER_ID"

#' Gets the active user id or `NULL`.
#'
#' @returns Active user or `NULL`
active_user <- function() {
  cmd <- '{"UserCommand": "GetActive"}'
  user <- send_cmd(zmq_socket(), cmd)
  user
}

#' Gets the `THOT_CONTAINER_ID` environment variable.
#'
#' @returns Active Container id or `NA`.
thot_container_id <- function() {
  Sys.getenv(CONTAINER_ID_KEY, unset = NA)
}

#' Gets the Project path given a path.
#' Returns `NULL` if the path is not in a project.
#'
#' @param path Path to get the Project root of.
#'
#' @returns Project path of the resource, or `NULL`.
project_resource_root_path <- function(path) {
  cmd <- sprintf('{"ProjectCommand": {"ResourceRootPath": "%s"}}', escape_str(path))
  path <- send_cmd(zmq_socket(), cmd)
  path
}

#' Creates a new core Asset.
#' @param file File name of the associated data. Use relative paths to place the Asset in a bucket.
#' @param name Name of the Asset to match.
#' @param type Type of the Asset to match.
#' @param tags List of tags the Asset has to match.
#' @param metadata Named list of metadata the Asset has to match.
#'
#' @returns New core Asset as JSON.
json_asset <- function(file, name = NULL, type = NULL, tags = list(), metadata = list()) {
  path_type <- if(isAbsolutePath(file)) "Absolute" else "Relative"
  path_val <- list()
  path_val[[path_type]] = file
  asset <- list(
    rid = uuid::UUIDgenerate(),
    properties = StandardProperties(),
    path = path_val
  )

  asset <- to_json(asset)
  asset <- json_empty_list_to_obj("metadata", asset)
  asset
}


#' Create a new `StandardProperties` list.
#'
#' @returns Default standard properties as a named list.
StandardProperties <- function() {
  user <- active_user()
  if (!is.null(user)) {
    user <- user$rid
  }

  list(
    created = utc_now(),
    creator = list(User = user),
    name = NULL,
    kind = NULL,
    description = NULL,
    tags = list(),
    metadata = list()
  )
}
