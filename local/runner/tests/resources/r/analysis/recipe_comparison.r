suppressPackageStartupMessages(library(tidyverse))
library(ggplot2)
library(syre)

# initialize project
db <- database()

# get `recipe-stats` data
recipe_stats <- db |> find_assets(type = "recipe-stats")

# import data into tibble
df <- list()
for (stat in recipe_stats) {
    # read data for each recipe
    tdf <- read_csv(stat@file)
    orig_colnames <- colnames(tdf)
    tdf <- tdf |>
        t() |>
        as.data.frame()

    colnames(tdf) <- stat@metadata$recipe
    rownames(tdf) <- orig_colnames
    df[[length(df) + 1]] <- tdf
}

# combine all the data frames into one
df <- bind_cols(df)

# create new asset for the statistics
comparison_path <- db |> add_asset(
    "recipe_comparison.csv",
    name = "Recipe Comparison",
    type = "recipe-comparison"
)

# save the statistics to the new asset
df |> write.csv(comparison_path)

# prepare data for bar chart
df_long <- df |>
    rownames_to_column("statistic") |>
    gather(key = "recipe", value = "value", -statistic) |>
    spread(key = "statistic", value = "value") |>
    rename(mean = mean, std = std)

# create the bar chart with error bars
p <- ggplot(data = df_long, aes(x = recipe, y = mean, fill = recipe)) +
    geom_bar(stat = "identity", position = "dodge") +
    geom_errorbar(
        aes(
            x = recipe,
            ymin = mean - std,
            ymax = mean + std
        ),
        width = 0.2,
        position = position_dodge(width = 0.9)
    ) +
    theme_minimal() +
    theme(
        axis.title.x = element_blank(),
        axis.title.y = element_blank(),
        legend.position = "none"
    )

bar_path <- db |> add_asset(
    "recipe_comparison.png",
    name = "Recipe Comparison",
    type = "recipe-bar-chart"
)

# save the plot to file
bar_path |> ggsave(plot = p, width = 10, height = 6, dpi = 300)
