library(rzmq)
library(purrr)

#' Create a new Thot database connection.
#'
#' @param dev_root Path to the root Container for use in developement mode.
#' @export
database <- function(dev_root = NULL) {
  context <- init.context()
  socket <- init.socket(context, "ZMQ_REQ")
  connect.socket(socket, "tcp://127.0.0.1:7047")

  root_id <- Sys.getenv("THOT_CONTAINER_ID", unset = NA)
  if (is.na(root_id)) {
    # dev mode
    stopifnot(!is.null(dev_root))
    root_path <- dev_root
  } else {
    stop("TODO: not in dev mode")
  }

  project_path <- project_resource_root_path(root_path)
  project <- load_project(socket, project_path)
  graph <- load_graph(socket, project$rid)
  root <- container_by_path(socket, root_path)

  db <- new("Database", root = root$rid, root_path = root_path, socket = socket)
}
#' Find Containers matching the given filter criteria.
#'
#' @param db Thot database connection.
#' @param name Name of the Container to match.
#' @param type Type of the Container to match.
#' @param tags List of tags the Container has to match.
#' @param metadata Named list of metadata the Container has to match.
#' @export
find_containers <- function(db, name = NULL, type = NULL, tags = NULL, metadata = NULL) {
  args <- to_json(list(
    db@root,
    list("name" = name, "type" = type, "tags" = tags, "metadata" = metadata)
  ))

  cmd <- sprintf('{"ContainerCommand": {"FindWithMetadata": %s}}', args)
  containers <- send_cmd(db@socket, cmd, result = FALSE)
  containers |> map(function(container) {
    # convert assets
    assets <- container$assets |> map(function(asset) {
      properties <- asset$properties
      Asset(
        file = asset$path[[1]],
        name = properties$name,
        type = properties$kind,
        tags = properties$tags,
        metadata = properties$metadata
      )
    })

    # convert container
    properties <- container$properties
    Container(
      name = properties$name,
      type = properties$type,
      tags = properties$tag,
      metadata = properties$metadata,
      assets = assets
    )
  })
}

#' Finds a single Container matching the given filter criteria.
#' If multiple matching Containers are found, a random one is returned.
#'
#' @param db Thot database connection.
#' @param name Name of the Container to match.
#' @param type Type of the Container to match.
#' @param tags List of tags the Container has to match.
#' @param metadata Named list of metadata the Container has to match.
#' @export
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
#' @export
find_assets <- function(db, name = NULL, type = NULL, tags = NULL, metadata = NULL) {
  args <- to_json(list(
    db@root,
    list("name" = name, "type" = type, "tags" = tags, "metadata" = metadata)
  ))

  cmd <- sprintf('{"AssetCommand": {"FindWithMetadata": %s}}', args)
  assets <- send_cmd(db@socket, cmd, result = FALSE)
  assets |> map(function(asset) {
    properties <- asset$properties
    Asset(
      file = asset$path[[1]],
      name = properties$name,
      type = properties$kind,
      tags = properties$tags,
      metadata = properties$metadata
    )
  })
}

#' Finds a single Asset matching the given filter criteria.
#' If multiple matching Assets are found, a random one is returned.
#'
#' @param db Thot database connection.
#' @param name Name of the Asset to match.
#' @param type Type of the Asset to match.
#' @param tags List of tags the Asset has to match.
#' @param metadata Named list of metadata the Asset has to match.
#' @export
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
#' @export
add_asset <- function(db, file, name = NULL, type = NULL, tags = list(), metadata = list()) {
  asset <- fromJSON(new_asset(file))
  asset$properties$name = name
  asset$properties$type = type
  asset$properties$tags = tags
  asset$properties$metadata = metadata
  args <- to_json(list(asset, db@root))
  if (length(metadata) == 0) {
    # coerce metadata to an object
    args <- gsub('"metadata":\\[\\]', '"metadata":{}', args)
  }

  cmd <- sprintf('{"AssetCommand": {"Add": %s}}', args)
  path <- send_cmd(db@socket, cmd, result = FALSE)
  path
}
