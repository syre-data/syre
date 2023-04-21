test_that("everything works", {
  dev_root <- "/home/brian/Downloads/thot-projects/0/data"
  db <- database(dev_root = dev_root)
  asset <- find_asset(db)
  assets <- find_assets(db)
  container <- find_container(db)
  containers <- find_containers(db)
})
