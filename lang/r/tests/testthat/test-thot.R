test_that("api works", {
  dev_root <- "/path/to/test/container"
  db <- database(dev_root = dev_root)
  root_container <- root(db)
  asset <- find_asset(db)
  assets <- find_assets(db)
  container <- find_container(db)
  containers <- find_containers(db)
})
