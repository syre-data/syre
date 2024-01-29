# import packages
import pandas as pd
import syre

# intialize database
db = syre.Database()

# get recipe data
recipe_stats = db.find_assets(type='recipe-stats')

df = []
for stat in recipe_stats:
    tdf = pd.read_pickle(stat.file)
    tdf.rename({0: stat.metadata['recipe']}, axis = 1, inplace = True)
    
    df.append(tdf)

# combine into one dataframe
df = pd.concat(df, axis = 1)

# export data as csv for reading
comparison_path = db.add_asset(
    'recipe_comparison.csv',
	name='Recipe Comparison',
	type='recipe-comparison',
)

df.to_csv(comparison_path)

# create bar char
means = df.loc['mean']
errs = df.loc['std']

ax = means.plot(kind = 'bar', yerr = errs)

# add to chart project
bar_path = db.add_asset(
    'recipe_comparison.png',
	name='Recipe Comparison',
	type='recipe-bar-chart',
	tags=[ 'chart', 'image' ]
)

ax.get_figure().savefig(bar_path, format = 'png')
