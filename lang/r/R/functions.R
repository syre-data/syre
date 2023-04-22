#' Create a new Thot database connection.
#'
#' @param dev_root Path to the root Container for use in development mode.
#'
#' @returns A Thot Database connection.
#' @export
#'
#' @examples
#' db <- database(dev_root = "/path/to/my/thot/project/container")
database <- function(dev_root = NULL) {
  if (!database_available()) {
    exe_path <- database_server_path()
    system(exe_path, wait = FALSE)
  }

  socket <- zmq_socket()
  root_id <- thot_container_id()
  if (is.null(root_id)) {
    # dev mode
    stopifnot(!is.null(dev_root))
    root_path <- dev_root
  } else {
    root_path <- container_path(socket, root_id)
  }

  stopifnot(!is.null(root_path))
  project_path <- project_resource_root_path(root_path)

  stopifnot(!is.null(project_path))
  project <- load_project(socket, escape_str(project_path))

  stopifnot(!is.null(project))
  graph <- load_graph(socket, project$rid)
  root <- container_by_path(socket, escape_str(root_path))

  stopifnot(!is.null(root))
  db <- new("Database", root = root$rid, root_path = root_path, socket = socket)
}

#' Gets the root Container of the database.
#'
#' @param db Thot database connection.
#'
#' @returns Root Container.
#' @export
#'
#' @examples
#' db <- database()
#' root <- root(db)
root <- function(db) {
  root <- get_container(db@socket, db@root)
  stopifnot(!is.null(root))
  container_from_json(root)
}

#' Find Containers matching the given filter criteria.
#'
#' @param db Thot database connection.
#' @param name Name of the Container to match.
#' @param type Type of the Container to match.
#' @param tags List of tags the Container has to match.
#' @param metadata Named list of metadata the Container has to match.
#'
#' @returns List of Containers matching the filter.
#' @export
#'
#' @examples
#' db <- database()
#' containers <- find_containers(db, type = "my_container")
find_containers <- function(db, name = NULL, type = NULL, tags = NULL, metadata = NULL) {
  args <- to_json(list(
    db@root,
    list(name = name, kind = type, tags = tags, metadata = metadata)
  ))

  cmd <- sprintf('{"ContainerCommand": {"FindWithMetadata": %s}}', args)
  containers <- send_cmd(db@socket, cmd, result = FALSE)
  containers |> map(container_from_json)
}

#' Finds a single Container matching the given filter criteria.
#' If multiple matching Containers are found, a random one is returned.
#'
#' @param db Thot database connection.
#' @param name Name of the Container to match.
#' @param type Type of the Container to match.
#' @param tags List of tags the Container has to match.
#' @param metadata Named list of metadata the Container has to match.
#'
#' @returns Single Container matched by the filter or `NULL` if none exist.
#' @export
#'
#' @examples
#' db <- database()
#' container <- find_container(db, name = "My Container")
find_container <- function(db, name = NULL, type = NULL, tags = NULL, metadata = NULL) {
  containers <- find_containers(db, name = name, type = type, tags = tags, metadata = metadata)
  if (length(containers) > 0) {
    return(containers[[1]])
  }

  NULL
}

#' Find Assets matching the given filter criteria.
#'
#' @param db Thot database connection.
#' @param name Name of the Asset to match.
#' @param type Type of the Asset to match.
#' @param tags List of tags the Asset has to match.
#' @param metadata Named list of metadata the Asset has to match.
#'
#' @returns List of Assets matching the filter.
#' @export
#'
#' @examples
#' db <- database()
#' assets <- find_assets(db, type = "my_asset")
find_assets <- function(db, name = NULL, type = NULL, tags = NULL, metadata = NULL) {
  args <- to_json(list(
    db@root,
    list(name = name, kind = type, tags = tags, metadata = metadata)
  ))

  cmd <- sprintf('{"AssetCommand": {"FindWithMetadata": %s}}', args)
  assets <- send_cmd(db@socket, cmd, result = FALSE)
  assets |> map(asset_from_json)
}

#' Finds a single Asset matching the given filter criteria.
#' If multiple matching Assets are found, a random one is returned.
#'
#' @param db Thot database connection.
#' @param name Name of the Asset to match.
#' @param type Type of the Asset to match.
#' @param tags List of tags the Asset has to match.
#' @param metadata Named list of metadata the Asset has to match.
#'
#' @returns A single Asset, or `NULL` if none exist.
#' @export
#'
#' @examples
#' db <- database()
#' asset <- find_asset(db, name = "My Asset")
find_asset <- function(db, name = NULL, type = NULL, tags = NULL, metadata = NULL) {
  assets <- find_assets(db, name = name, type = type, tags = tags, metadata = metadata)
  if (length(assets) > 0) {
    return(assets[[1]])
  }

  NULL
}

#' Adds an Asset to the Thot project.
#' The associated data should be saved at the return path.
#'
#' @param db Thot database connection.
#' @param file File name of the associated data. Use relative paths to place the Asset in a bucket.
#' @param name Name of the Asset to match.
#' @param type Type of the Asset to match.
#' @param tags List of tags the Asset has to match.
#' @param metadata Named list of metadata the Asset has to match.
#'
#' @returns Path to save the Asset's related data to.
#' @export
#'
#' @examples
#' db <- database()
#' path <- add_asset(db, "my_file.txt", name = "My Text File")
#' cat("Hello!", path)
add_asset <- function(db, file, name = NULL, type = NULL, tags = list(), metadata = list()) {
  asset <- fromJSON(new_asset(file))
  asset$properties$name <- name
  asset$properties$kind <- type
  asset$properties$tags <- tags
  asset$properties$metadata <- metadata
  args <- to_json(list(asset, db@root))
  if (length(metadata) == 0) {
    # coerce metadata to an object
    args <- gsub('"metadata":\\[\\]', '"metadata":{}', args)
  }

  cmd <- sprintf('{"AssetCommand": {"Add": %s}}', args)
  send_cmd(db@socket, cmd)
  if (SYSNAME == "Windows") {
    path <- join_path_windows(db@root_path, asset$path[[1]])
  } else {
    path <- normalizePath(file.path(db@root_path, asset$path))
  }

  # ensure bucket is created
  dir.create(dirname(path), recursive = TRUE, showWarnings = FALSE)
  path
}
